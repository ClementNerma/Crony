use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::Duration,
};

use time::OffsetDateTime;

use crate::{
    datetime::{get_now, get_now_second_precision},
    info, notice,
    task::{Task, Tasks},
};

use super::upcoming::{get_new_upcoming_moment, get_upcoming_moment};

pub fn run_tasks(
    tasks: &Tasks,
    task_runner: impl Fn(&Task) + Send + Sync + 'static,
    stop_on: impl Fn() -> bool,
) {
    let task_runner = Arc::new(RwLock::new(task_runner));

    let now = get_now();

    let queue = tasks
        .values()
        .map(|task| {
            (
                task.name.clone(),
                get_upcoming_moment(now, &task.run_at).unwrap(),
            )
        })
        .collect::<HashMap<_, _>>();

    let queue = Arc::new(RwLock::new(queue));

    let mut last_displayed_planned = None;

    let mut short_sleep = |next: Option<OffsetDateTime>| {
        if let Some(next) = next {
            if last_displayed_planned != Some(next) {
                last_displayed_planned.replace(next);
                notice!("Next task is planned to run on: {}", next);
            }
        }

        // Sleep until the next second
        let remaining = 1_000_000_000 - get_now().nanosecond();
        std::thread::sleep(Duration::from_nanos(remaining.into()));
    };

    info!("Scheduler is running.");

    while !stop_on() {
        let now = get_now_second_precision();

        let nearest = queue
            .read()
            .unwrap()
            .iter()
            .min_by_key(|(_, moment)| **moment)
            .map(|(a, b)| (a.clone(), *b));

        let (task_name, planned_for) = match nearest {
            None => {
                short_sleep(None);
                continue;
            }
            Some((_, planned_for)) if planned_for > now => {
                short_sleep(Some(planned_for));
                continue;
            }
            Some(nearest) => nearest,
        };

        queue.write().unwrap().remove(&task_name).unwrap();

        let queue = Arc::clone(&queue);
        let task = tasks.get(&task_name).unwrap().clone();
        let task_runner = Arc::clone(&task_runner);

        notice!(
            "Running task '{}' late of {} second(s).",
            task.name,
            (now - planned_for).whole_seconds()
        );

        std::thread::spawn(move || {
            task_runner.read().unwrap()(&task);

            let mut queue = queue.write().unwrap();

            let planned = get_new_upcoming_moment(get_now(), &task.run_at, planned_for).unwrap();

            queue.insert(task.name.clone(), planned);
        });
    }
}
