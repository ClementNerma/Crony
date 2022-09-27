mod cmd;
mod runner;
mod scheduler;
mod upcoming;

pub use cmd::*;
pub use runner::runner;
pub use scheduler::SharedSchedulerQueue;
pub use upcoming::get_upcoming_moment;

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
    marker: impl Fn(&Task, bool) + Send + Sync + 'static,
    stop_on: impl Fn(SharedSchedulerQueue) -> bool,
) {
    let paths = paths.clone();

    let direct_output = args.direct_output;

    run_tasks(
        tasks,
        move |task| {
            (marker)(task, true);

            let result = runner(task, &paths, !direct_output);

            (marker)(task, false);

            if let Err(err) = result {
                error_anyhow!(err.context("Runner failed to run (from Scheduler)"));
                error!("Now sleeping for 5 seconds...");
                sleep_ms(5000);
            }
        },
        stop_on,
    )
}
