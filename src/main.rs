#![forbid(unsafe_code)]
#![forbid(unused_must_use)]

mod cmd;
mod daemon;
mod data;
mod engine;
mod ipc;
mod utils;

use daemon::DaemonStatusArgs;
pub use data::*;
pub use engine::*;
pub use utils::*;

use std::fs;

use anyhow::{bail, Context, Result};
use clap::Parser;
use colored::Colorize;
use rand::random;
use tabular::{row, Table};

use crate::{
    at::At,
    cmd::{Action, Cmd, ListArgs, RegisterArgs, RunArgs, UnregisterArgs},
    daemon::{is_daemon_running, start_daemon, Client, DaemonClient},
    datetime::human_datetime,
    engine::{runner::runner, start_engine},
    save::{construct_data_dir_paths, read_history_if_exists, read_tasks, write_tasks},
    task::Task,
};

fn main() {
    if let Err(err) = inner_main() {
        error_anyhow!(err);
        std::process::exit(1);
    }
}

fn inner_main() -> Result<()> {
    let cmd = Cmd::parse();

    let paths = construct_data_dir_paths(cmd.data_dir)?;

    let mut tasks = read_tasks(&paths)?;

    match cmd.action {
        Action::List(ListArgs {}) => {
            if tasks.is_empty() {
                info!("No task found.");
                return Ok(());
            }

            info!("Found {} tasks:", tasks.len().to_string().bright_yellow());
            info!("");

            let mut table = Table::new("{:>} {:<} {:<} {:<} {:<} {:<} {:<}");

            for task in tasks.values() {
                let history = read_history_if_exists(&paths.task_paths(&task.name))?;

                let last_run = match history.find_last_for(&task.name) {
                    None => "Never run".bright_black().italic(),
                    Some(entry) => {
                        let time = human_datetime(entry.started_at);

                        if entry.succeeded() {
                            time.bright_green()
                        } else {
                            time.bright_red()
                        }
                    }
                };

                let display_name = match &task.display_name {
                    Some(display_name) => display_name.bright_cyan(),
                    None => "".white(),
                };

                table.add_row(row!(
                    "*".bright_blue(),
                    task.name.bright_yellow(),
                    display_name,
                    last_run,
                    task.shell.bright_magenta(),
                    task.cmd.bright_cyan(),
                    task.run_at.encode().bright_black(),
                ));
            }

            println!("{}", table);
        }

        Action::Register(RegisterArgs {
            name,
            run_at,
            shell,
            cmd,
            display_name,
            silent,
        }) => {
            if !Task::is_valid_name(&name) {
                bail!("The provided name is invalid, only letters, digits, dashes and underscores are allowed.");
            }

            if tasks.contains_key(&name) && !silent {
                warn!("WARNING: Going to override the existing task!");

                fs::rename(
                    paths.task_paths(&name).dir(),
                    paths.generate_old_task_dir_name(&name),
                )
                .context("Failed to move the previous task's directory")?;
            }

            fs::create_dir(paths.task_paths(&name).dir())
                .context("Failed to create the task's directory")?;

            let run_at = At::parse(&run_at)?;

            tasks.insert(
                name.clone(),
                Task {
                    id: random(),
                    name: name.clone(),
                    display_name: display_name.clone(),
                    run_at,
                    cmd,
                    shell,
                },
            );

            write_tasks(&paths, &tasks)?;

            if !silent {
                success!(
                    "Successfully registered task {}{}.",
                    name.bright_yellow(),
                    if let Some(dp) = display_name {
                        format!("({})", dp.bright_cyan())
                    } else {
                        String::new()
                    }
                )
            }

            // TODO: ask the daemon to reload
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

            // TODO: ask the daemon to reload
        }

        Action::Run(RunArgs {
            name,
            use_log_files,
        }) => {
            let task = tasks
                .get(&name)
                .with_context(|| format!("Task '{}' does not exist.", name.bright_yellow()))?;

            runner(task, &paths.task_paths(&task.name), use_log_files)?;
        }

        Action::Foreground(args) => {
            info!("Starting the engine (foreground)...");
            start_engine(&paths, &read_tasks(&paths)?, &args, || false);
        }

        Action::DaemonStart(args) => {
            info!("Starting the daemon...");
            start_daemon(&paths, &args)?;
        }

        Action::DaemonStatus(DaemonStatusArgs {}) => {
            info!("Checking daemon's status...");

            let socket_file = paths.daemon_paths().socket_file();

            if !is_daemon_running(&socket_file)? {
                warn!("Daemon is not running.");
                return Ok(());
            }

            success!("Daemon is running, sending a test request...");

            let mut client = DaemonClient::connect(&socket_file)?;
            let res = client.hello()?;

            if res == "Hello" {
                success!("Daemon responsed successfully to a test request.");
            } else {
                error!("Daemon responsed unsuccessfully to a test request.");
            }
        }
    }

    Ok(())
}
