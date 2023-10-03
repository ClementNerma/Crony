use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::Duration,
};

use time::OffsetDateTime;

use crate::{
    datetime::{get_now, get_now_second_precision},
    info, notice,
    sleep::sleep_ms,
    task::{Task, Tasks},
};

use super::upcoming::{get_new_upcoming_moment, get_upcoming_moment};

pub fn run_tasks(
    tasks: &Tasks,
    task_runner: impl Fn(&Task) + Send + Sync + 'static,
    stop_on: impl Fn(SharedSchedulerQueue) -> bool,
) {
    let task_runner = Arc::new(RwLock::new(task_runner));

    let now = get_now();

    let queue = tasks
        .values()
        .map(|task| (task.id, get_upcoming_moment(now, &task.at).unwrap()))
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

    while !stop_on(Arc::clone(&queue)) {
        let now = get_now_second_precision();

        let nearest = queue
            .read()
            .unwrap()
            .iter()
            .min_by_key(|(_, moment)| **moment)
            .map(|(a, b)| (*a, *b));

        let (task_id, planned_for) = match nearest {
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

        queue.write().unwrap().remove(&task_id).unwrap();

        let task = tasks
            .values()
            .find(|task| task.id == task_id)
            .unwrap()
            .clone();

        let task_runner = Arc::clone(&task_runner);

        let late = (now - planned_for).whole_seconds();

        notice!("Running task '{}' late of {} second(s).", task.name, late);

        if late > 60 {
            notice!("More than 60 seconds late ; the computer may have been to sleep.");
            notice!("Waiting 30 more seconds to ensure all capabilities (e.g. internet access) are available again.");

            sleep_ms(30);
        }

        let queue = Arc::clone(&queue);

        std::thread::spawn(move || {
            task_runner.read().unwrap()(&task);

            let planned = get_new_upcoming_moment(get_now(), &task.at, planned_for).unwrap();

            queue.write().unwrap().insert(task.id, planned);
        });
    }
}

pub type SharedSchedulerQueue = Arc<RwLock<HashMap<u64, OffsetDateTime>>>;
