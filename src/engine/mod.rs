mod cmd;
mod runner;
mod scheduler;
mod upcoming;

pub use cmd::*;
pub use runner::runner;
pub use upcoming::get_upcoming_moment;

use std::sync::Arc;

use crate::{
    error, error_anyhow,
    paths::Paths,
    sleep::sleep_ms,
    task::{Task, Tasks},
};

use self::scheduler::run_tasks;

pub fn start_engine(
    paths: &Paths,
    tasks: &Tasks,
    args: &EngineArgs,
    interface: Arc<RunningTasksInterface>,
    stop_on: impl Fn() -> bool,
) {
    let paths = paths.clone();

    let direct_output = args.direct_output;

    run_tasks(
        tasks,
        move |task| {
            (interface.mark_task_as_running)(task);

            let result = runner(task, &paths.task_paths(&task.name), !direct_output);

            (interface.mark_task_as_done)(task.id);

            if let Err(err) = result {
                error_anyhow!(err.context("Runner failed to run (from Scheduler)"));
                error!("Now sleeping for 5 seconds...");
                sleep_ms(5000);
            }
        },
        stop_on,
    )
}

pub struct RunningTasksInterface {
    pub mark_task_as_running: Box<dyn Fn(&Task) + Send + Sync>,
    pub mark_task_as_done: Box<dyn Fn(u64) + Send + Sync>,
}
