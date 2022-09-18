pub mod cmd;
pub mod runner;
pub mod scheduler;
pub mod upcoming;

use std::time::Duration;

use crate::{error, error_anyhow, paths::Paths, task::Tasks};

use self::{cmd::EngineArgs, runner::runner, scheduler::run_tasks};

pub fn start_engine(paths: &Paths, tasks: &Tasks, args: &EngineArgs, stop_on: impl Fn() -> bool) {
    let paths = paths.clone();

    let direct_output = args.direct_output;

    run_tasks(
        tasks,
        move |task| {
            let result = runner(task, &paths.task_paths(&task.name), !direct_output);

            if let Err(err) = result {
                error_anyhow!(err.context("Runner failed to run (from Scheduler)"));
                error!("Now sleeping for 5 seconds...");
                std::thread::sleep(Duration::from_secs(5));
            }
        },
        stop_on,
    )
}
