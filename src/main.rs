#![forbid(unsafe_code)]
#![forbid(unused_must_use)]

mod cmd;
mod daemon;
mod data;
mod engine;
mod ipc;
mod utils;

pub use data::*;
pub use engine::*;
use utils::logging::PRINT_DEBUG_MESSAGES;
pub use utils::*;

use std::{fs, sync::atomic::Ordering};

use anyhow::{bail, Context, Result};
use clap::Parser;
use colored::Colorize;
use rand::random;
use tabular::{row, Table};

use crate::{
    at::At,
    cmd::{Action, Cmd, HistoryArgs, LogsArgs, RegisterArgs, RunArgs, UnregisterArgs},
    daemon::{is_daemon_running, start_daemon, DaemonClient, RunningTask},
    datetime::get_now,
    history::History,
    paging::run_pager,
    save::{construct_data_dir_paths, read_history_if_exists, read_tasks, write_tasks},
    sleep::sleep_ms,
    task::Task,
};

fn main() {
    if let Err(err) = inner_main() {
        error_anyhow!(err);
        std::process::exit(1);
    }
}

fn inner_main() -> Result<()> {
    debug!("Entered inner main.");

    let cmd = Cmd::parse();

    if cmd.verbose {
        PRINT_DEBUG_MESSAGES.store(true, Ordering::SeqCst);
    }

    let paths = construct_data_dir_paths(cmd.data_dir)?;

    let mut tasks = read_tasks(&paths)?;

    match cmd.action {
        Action::List => {
            if tasks.is_empty() {
                info!("No task found.");
                return Ok(());
            }

            info!("Found {} tasks:", tasks.len().to_string().bright_yellow());
            info!("");

            let mut table = Table::new("{:>} {:<} {:<} {:<} {:<} {:<}");

            for task in tasks.values() {
                let history = read_history_if_exists(&paths.task_paths(&task.name))?;

                let last_run = match history.find_last_for(task.id) {
                    None => "Never run".bright_black(),
                    Some(entry) => {
                        let time = entry.started_at.replace_nanosecond(0).unwrap().to_string();

                        if entry.succeeded() {
                            time.bright_green()
                        } else {
                            time.bright_red()
                        }
                    }
                };

                table.add_row(row!(
                    "*".bright_blue(),
                    task.name.bright_yellow(),
                    last_run,
                    match &task.shell {
                        Some(shell) => shell.bright_magenta(),
                        None => "-".bright_black(),
                    },
                    task.cmd.bright_cyan(),
                    task.at.encode().bright_black(),
                ));
            }

            println!("{}", table);
        }

        Action::Check => {
            let mut errors = vec![];

            for task in tasks.values() {
                let history = read_history_if_exists(&paths.task_paths(&task.name))?;

                if let Some(last_run) = history.find_last_for(task.id) {
                    if !last_run.succeeded() {
                        errors.push(format!(
                            "Task '{}' failed on {}.",
                            task.name.bright_yellow(),
                            last_run.ended_at.to_string().bright_magenta()
                        ));
                    }
                }
            }

            if !errors.is_empty() {
                bail!("{}", errors.join("\n"));
            }
        }

        Action::Register(RegisterArgs {
            name,
            at,
            using,
            run,
            force_override,
            ignore_identical,
            silent,
        }) => {
            if !Task::is_valid_name(&name) {
                bail!("The provided name is invalid, only letters, digits, dashes and underscores are allowed.");
            }

            let at = At::parse(&at)?;

            let task = Task {
                id: random(),
                name: name.clone(),
                at,
                cmd: run,
                shell: using,
            };

            let next = task.at.next_occurrence().context(
                "Failed to find a valid next occurrence for the provided repetition pattern",
            )?;

            if let Some(existing) = tasks.get(&name) {
                let mut simili = existing.clone();
                simili.id = task.id;

                let identical = simili == task;

                if identical && ignore_identical {
                    return Ok(());
                }

                if !force_override {
                    bail!("A task with this name already exists!");
                }

                warn!("WARNING: Going to override the existing task!");

                fs::rename(
                    paths.task_paths(&name).dir(),
                    paths.generate_old_task_dir_name(&name),
                )
                .context("Failed to move the previous task's directory")?;
            }

            fs::create_dir(paths.task_paths(&name).dir())
                .context("Failed to create the task's directory")?;

            tasks.insert(name.clone(), task);

            write_tasks(&paths, &tasks)?;

            if !silent {
                success!("Successfully registered task {}.", name.bright_yellow());
                success!(
                    "If the daemon is running, the task will run on {}",
                    next.to_string().bright_magenta()
                );
            }

            let socket_file = &paths.daemon_socket_file;

            if is_daemon_running(socket_file)? {
                debug!("Asking the daemon to reload the tasks...");

                let mut client = DaemonClient::connect(socket_file)?;
                client.reload_tasks()?;

                success!("Daemon successfully reloaded the tasks!");
            } else {
                warn!("Warning: the daemon is not running.")
            }
        }

        Action::Unregister(UnregisterArgs { name }) => {
            if !tasks.contains_key(&name) {
                bail!("Task '{}' does not exist.", name.bright_yellow());
            }

            fs::rename(
                paths.task_paths(&name).dir(),
                paths.generate_old_task_dir_name(&name),
            )
            .context("Failed to move the previous task's directory")?;

            tasks.remove(&name);

            write_tasks(&paths, &tasks)?;

            success!("Successfully removed task {}.", name.bright_yellow());

            let socket_file = &paths.daemon_socket_file;

            if is_daemon_running(socket_file)? {
                debug!("Asking the daemon to reload the tasks...");

                let mut client = DaemonClient::connect(socket_file)?;
                client.reload_tasks()?;

                success!("Daemon successfully reloaded the tasks!");
            } else {
                warn!("Warning: the daemon is not running.")
            }
        }

        Action::Run(RunArgs {
            name,
            use_log_files,
        }) => {
            let task = tasks
                .get(&name)
                .with_context(|| format!("Task '{}' does not exist.", name.bright_yellow()))?;

            runner(task, &paths, use_log_files)?;
        }

        Action::Start(args) => {
            start_daemon(&paths, &args)?;
        }

        Action::Status => {
            debug!("Checking daemon's status...");

            let socket_file = paths.daemon_socket_file;

            if !is_daemon_running(&socket_file)? {
                warn!("Daemon is not running.");
                return Ok(());
            }

            debug!("Daemon is running, sending a test request...");

            let mut client = DaemonClient::connect(&socket_file)?;
            let pid = client.hello()?;

            success!("Daemon is running and responding to requests.");
            debug!("Daemon PID: {pid}");
        }

        Action::Scheduled => {
            let mut client = DaemonClient::connect(&paths.daemon_socket_file)?;
            let scheduled = client.scheduled()?;

            info!("List of upcoming / running tasks:");
            info!("");

            let mut table = Table::new("{:>} {:<} {:<} {:<}");

            let now = get_now();

            for RunningTask { task, started } in scheduled.running {
                table.add_row(row!(
                    task.name.bright_cyan(),
                    "Running".bright_green(),
                    format!(
                        "since {}",
                        now.replace_nanosecond(0).unwrap() - started.replace_nanosecond(0).unwrap()
                    )
                    .bright_blue(),
                    started.to_string().bright_magenta(),
                ));
            }

            for (task, time) in scheduled.upcoming {
                table.add_row(row!(
                    task.name.bright_cyan(),
                    "Scheduled".bright_yellow(),
                    format!(
                        "in {}",
                        time.replace_nanosecond(0).unwrap() - now.replace_nanosecond(0).unwrap()
                    )
                    .bright_blue(),
                    time.to_string().bright_magenta(),
                ));
            }

            println!("{}", table);
        }

        Action::Stop => {
            debug!("Asking the daemon to stop...");

            let mut client = DaemonClient::connect(&paths.daemon_socket_file)?;

            match client.stop() {
                Ok(()) => {}
                Err(err) => {
                    if let Ok(false) = is_daemon_running(&paths.daemon_socket_file) {
                        success!("Daemon was successfully stopped!");
                        return Ok(());
                    }

                    return Err(err);
                }
            }

            debug!("Request succesfully transmitted, waiting for the daemon to actually stop...");

            let mut last_running = 0;
            let mut had_error = false;

            loop {
                match is_daemon_running(&paths.daemon_socket_file) {
                    Ok(true) => {}
                    Ok(false) => break,
                    Err(err) => {
                        if had_error {
                            return Err(err);
                        }

                        had_error = true;
                        sleep_ms(20);
                        continue;
                    }
                }

                let running = match client.running_tasks() {
                    Ok(running) => running,
                    Err(err) => {
                        if had_error {
                            return Err(err);
                        }

                        had_error = true;
                        sleep_ms(20);
                        continue;
                    }
                };

                if running != last_running {
                    info!("Waiting for {} task(s) to complete...", running);
                    last_running = running;
                }

                sleep_ms(100);
            }

            success!("Daemon was successfully stopped!");
        }

        Action::Logs(LogsArgs {
            task_name,
            pager,
            no_less_options,
        }) => {
            let log_file = match task_name {
                Some(task_name) => {
                    if !tasks.contains_key(&task_name) {
                        bail!("Provided task does not exist.");
                    }

                    paths.task_paths(&task_name).log_file()
                }
                None => paths.daemon_log_file,
            };

            if !log_file.exists() {
                info!("No log file found.");
                return Ok(());
            }

            let logs =
                fs::read_to_string(&log_file).context("Failed to read the daemon's log file")?;

            let pager = pager
                .or_else(|| std::env::var("PAGER").ok())
                .unwrap_or_else(|| "less".to_owned());

            run_pager(&logs, &pager, no_less_options)?;
        }

        Action::History(HistoryArgs {
            task_name,
            last_entries,
        }) => {
            let log_file = match task_name {
                Some(task_name) => {
                    if !tasks.contains_key(&task_name) {
                        bail!("Provided task does not exist.");
                    }

                    paths.task_paths(&task_name).history_file()
                }
                None => paths.global_history_file,
            };

            let history = fs::read_to_string(&log_file).context("Failed to read history file")?;
            let history = History::parse(&history).context("Failed to parse history file")?;

            let entries = history.entries();

            if entries.is_empty() {
                info!("History is empty.");
                return Ok(());
            }

            info!(
                "Found {} entries in history",
                entries.len().to_string().bright_yellow()
            );
            info!("");

            let last_entries = match last_entries {
                Some(count) => {
                    if count >= entries.len() {
                        &entries[entries.len() - 1 - count..entries.len() - 1]
                    } else {
                        entries
                    }
                }
                None => entries,
            };

            let mut table = Table::new("{:>} {:>} {:<} {:<} {:<}");

            for entry in last_entries.iter().rev() {
                let exists = tasks.values().any(|task| task.id == entry.task_id);

                let display_name = match exists {
                    true => entry.task_name.bright_yellow(),
                    false => format!("{} (deleted)", entry.task_name).bright_red(),
                };

                let result = entry.result.encode();
                let result = match entry.succeeded() {
                    true => result.bright_green(),
                    false => result.bright_red(),
                };

                table.add_row(row!(
                    "*".bright_cyan(),
                    display_name,
                    entry
                        .started_at
                        .replace_nanosecond(0)
                        .unwrap()
                        .to_string()
                        .bright_blue(),
                    (entry.ended_at.replace_nanosecond(0).unwrap()
                        - entry.started_at.replace_nanosecond(0).unwrap())
                    .to_string()
                    .bright_magenta(),
                    result
                ));
            }

            println!("{table}");
        }
    }

    Ok(())
}
