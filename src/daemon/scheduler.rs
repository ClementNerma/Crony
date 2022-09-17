use std::time::Duration;

use anyhow::{Context, Result};

use crate::{
    daemon::upcoming::get_upcoming_moment,
    datetime::get_now,
    info, notice,
    task::{Task, Tasks},
};

// TODO: this version relies on the `crono`'s crate scheduler
// This unfortunately requires to hackily convert the occurrence into
// a cron-formatted string, parse it, and then get the upcoming occurrence
// Which is obviously far from ideal.

pub fn run_tasks(
    tasks: &Tasks,
    task_runner: impl Fn(&Task),
    stop_on: impl Fn() -> bool,
) -> Result<()> {
    let now = get_now();

    let mut queue = tasks
        .values()
        .map(|task| (task, get_upcoming_moment(now, &task.run_at).unwrap()))
        .collect::<Vec<_>>();

    info!("Scheduler is running.");

    loop {
        if tasks.is_empty() {
            notice!("No task registered, sleeping for 1 second.");
            std::thread::sleep(Duration::from_secs(1));
            continue;
        }

        let now = get_now();

        let (task_index, (task, planned_for)) = queue
            .iter()
            .enumerate()
            .min_by_key(|(_, (_, moment))| moment)
            .unwrap();

        if planned_for > &now {
            notice!("No task to run, checking free time before next task...");

            let can_sleep_for = queue
                .iter()
                .map(|(_, moment)| (*moment - now).whole_seconds())
                .min()
                .context("No future task found in queue, should not be empty")
                .unwrap();

            notice!(
                "Nearest task scheduled to run in {} second(s), sleeping until then.",
                can_sleep_for
            );

            // NOTE: Waiting for one more second is required as it can otherwise lead
            // to a very tricky bug: the clock may get to the task's planned time, minus
            // a few milliseconds or even microseconds. In which case, this will run thousands of times.
            std::thread::sleep(Duration::from_secs(
                u64::try_from(can_sleep_for + 1)
                    .context("Found negative elapsed time for planned task")
                    .unwrap(),
            ));
            continue;
        }

        notice!(
            "Running task '{}' late of {} second(s).",
            task.name,
            (now - *planned_for).whole_seconds()
        );

        task_runner(task);

        queue.push((task, get_upcoming_moment(get_now(), &task.run_at).unwrap()));
        queue.remove(task_index);

        if stop_on() {
            return Ok(());
        }
    }
}
