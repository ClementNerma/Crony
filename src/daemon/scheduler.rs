use std::{collections::HashMap, str::FromStr, time::Duration};

use anyhow::{Context, Result};
use chrono::{DateTime, Local};

use crate::{
    at::Occurrences,
    error, error_anyhow, info, notice,
    paths::Paths,
    runner::runner,
    save::read_tasks,
    task::{Task, Tasks},
};

use super::DaemonArgs;

// TODO: this version relies on the `crono`'s crate scheduler
// This unfortunately requires to hackily convert the occurrence into
// a cron-formatted string, parse it, and then get the upcoming occurrence
// Which is obviously far from ideal.
pub struct Scheduler<'a, 'b> {
    paths: &'a Paths,
    args: &'b DaemonArgs,
    tasks: Tasks,
    cron_schedulers: HashMap<String, cron::Schedule>,
}

impl<'a, 'b> Scheduler<'a, 'b> {
    pub fn new(paths: &'a Paths, args: &'b DaemonArgs) -> Result<Self> {
        let tasks = read_tasks(paths)?;

        let cron_schedulers = tasks
            .values()
            .map(|task| {
                let cronify = |occ: &Occurrences, fallback: u8| match occ {
                    Occurrences::First => fallback.to_string(),
                    Occurrences::Every => "*".to_string(),
                    Occurrences::Once(at) => at.to_string(),
                    Occurrences::Multiple(at) => {
                        at.iter().map(u8::to_string).collect::<Vec<_>>().join(",")
                    }
                };

                let expr = format!(
                    "{} {} {} {} {} * *",
                    cronify(&task.run_at.seconds, 0),
                    cronify(&task.run_at.minutes, 0),
                    cronify(&task.run_at.hours, 0),
                    cronify(&task.run_at.days, 1),
                    cronify(&task.run_at.months, 1)
                );

                let schedule = cron::Schedule::from_str(&expr)
                    .context("Failed to parse CRONified expression")?;

                Ok((task.name.clone(), schedule))
            })
            .collect::<Result<HashMap<_, _>>>()?;

        Ok(Self {
            paths,
            args,
            tasks,
            cron_schedulers,
        })
    }

    fn upcoming(&self, task_name: &str) -> DateTime<Local> {
        self.cron_schedulers
            .get(task_name)
            .context("Cached CRON scheduler not found for task")
            .unwrap()
            .upcoming(Local)
            .next()
            .context("Failed to determine upcoming occurrence")
            .unwrap()
    }

    pub fn run(&self) {
        let mut queue = self
            .tasks
            .values()
            .map(|task| (task, self.upcoming(&task.name)))
            .collect::<Vec<_>>();

        info!("Scheduler is running.");

        let mut refresh_scheduling_for: Option<&Task> = None;

        loop {
            if let Some(task) = refresh_scheduling_for {
                let index = queue
                    .iter()
                    .position(|(c, _)| c.name == task.name)
                    .context("Scheduled task to remove was not found in queue")
                    .unwrap();

                queue.remove(index);
                queue.push((task, self.upcoming(&task.name)));

                refresh_scheduling_for = None;
            }

            if self.tasks.is_empty() {
                notice!("No task registered, sleeping for 1 second.");
                std::thread::sleep(Duration::from_secs(1));
                continue;
            }

            let now = Local::now();

            let (task, planned_for) = queue.iter().min_by_key(|(_, moment)| moment).unwrap();

            if planned_for > &now {
                notice!("No task to run, checking free time before next task...");

                let can_sleep_for = queue
                    .iter()
                    .map(|(_, moment)| {
                        moment
                            .signed_duration_since(now)
                            .to_std()
                            .context("Found negative moment after scheduler comparison")
                            .unwrap()
                            .as_secs()
                    })
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
                std::thread::sleep(Duration::from_secs(can_sleep_for + 1));
                continue;
            }

            let late_of = now
                .signed_duration_since(*planned_for)
                .to_std()
                .context("Found negative moment after scheduler comparison")
                .unwrap()
                .as_secs();

            notice!(
                "Running task '{}' late of {} second(s).",
                task.name,
                late_of
            );

            refresh_scheduling_for = Some(task);

            let result = runner(
                task,
                &self.paths.task_paths(&task.name),
                !self.args.direct_output,
            );

            if let Err(err) = result {
                error_anyhow!(err.context("Runner failed to run (from Scheduler)"));
                error!("Now sleeping for 5 seconds...");
                std::thread::sleep(Duration::from_secs(5));
            }
        }
    }
}
