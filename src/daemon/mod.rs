mod cmd;
mod notify;
mod scheduler;
mod upcoming;

use std::time::Duration;

pub use cmd::DaemonArgs;
pub use notify::ask_daemon_reload;

use anyhow::Result;

use crate::{error, error_anyhow, paths::Paths, runner::runner, save::read_tasks};

use self::{notify::treat_reload_request, scheduler::run_tasks};

pub fn start_scheduler(paths: &Paths, args: &DaemonArgs) -> Result<()> {
    let tasks = read_tasks(paths)?;

    treat_reload_request(paths)?;

    run_tasks(
        &tasks,
        |task| {
            let result = runner(task, &paths.task_paths(&task.name), !args.direct_output);

            if let Err(err) = result {
                error_anyhow!(err.context("Runner failed to run (from Scheduler)"));
                error!("Now sleeping for 5 seconds...");
                std::thread::sleep(Duration::from_secs(5));
            }
        },
        || match treat_reload_request(paths) {
            Ok(val) => val,
            Err(err) => {
                error_anyhow!(err);
                false
            }
        },
    )?;

    Ok(())
}
