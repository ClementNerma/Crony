mod cmd;
mod notify;
mod scheduler;

use std::time::Duration;

pub use cmd::DaemonArgs;

use anyhow::Result;

use crate::{error, error_anyhow, paths::Paths, runner::runner, save::read_tasks};

use self::scheduler::Scheduler;

pub fn start_scheduler(paths: &Paths, args: &DaemonArgs) -> Result<()> {
    let tasks = read_tasks(paths)?;

    Scheduler::new(&tasks, &|task| {
        let result = runner(task, &paths.task_paths(&task.name), !args.direct_output);

        if let Err(err) = result {
            error_anyhow!(err.context("Runner failed to run (from Scheduler)"));
            error!("Now sleeping for 5 seconds...");
            std::thread::sleep(Duration::from_secs(5));
        }
    })?
    .run();

    Ok(())
}
