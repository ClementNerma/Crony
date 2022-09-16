#![forbid(unsafe_code)]
#![forbid(unused_must_use)]

mod at;
mod cmd;
mod daemon;
mod history;
mod logging;
mod paths;
mod runner;
mod save;
mod task;

use std::fs;

use anyhow::{bail, Context, Result};
use clap::Parser;
use colored::Colorize;
use tabular::{row, Table};

use crate::{
    at::At,
    cmd::{Action, Cmd, ListArgs, RegisterArgs, RunArgs, SchedulerArgs, UnregisterArgs},
    daemon::start_scheduler,
    runner::runner,
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
                        let time = entry.started_at.format("%Y-%m-%d %H:%M:%S").to_string();

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

            // TODO: notify daemon
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

            // TODO: notify daemon
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

        Action::Scheduler(SchedulerArgs { args }) => {
            info!("Starting the scheduler...");
            start_scheduler(&paths, &args)?;
        }
    }

    Ok(())
}
