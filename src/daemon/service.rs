use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::{service, task::Task};

service!(
    daemon (functions) {
        fn hello() -> String;
        fn stop();
        fn reload_tasks();
        fn running_tasks() -> usize;
        fn scheduled() -> super::super::Scheduled;
    }
);

mod functions {
    use std::sync::{Arc, RwLock};

    use crate::sleep::sleep_ms;

    use super::Scheduled;

    pub type State = RwLock<super::State>;

    pub fn hello(_: Arc<State>) -> String {
        "Hello".to_string()
    }

    pub fn stop(state: Arc<State>) {
        state.write().unwrap().exit = true;

        while state.read().unwrap().exit {
            sleep_ms(20);
        }
    }

    pub fn reload_tasks(state: Arc<State>) {
        state.write().unwrap().must_reload_tasks = true;

        while state.read().unwrap().must_reload_tasks {
            sleep_ms(20);
        }
    }

    pub fn running_tasks(state: Arc<State>) -> usize {
        state.read().unwrap().running_tasks.len()
    }

    pub fn scheduled(state: Arc<State>) -> Scheduled {
        {
            state.write().unwrap().scheduled_request = Some(None);
        }

        let upcoming = loop {
            let state = state.read().unwrap();

            if let Some(Some(ref scheduled)) = state.scheduled_request {
                break scheduled.clone();
            }

            drop(state);

            sleep_ms(50);
        };

        Scheduled {
            upcoming,
            running: state
                .read()
                .unwrap()
                .running_tasks
                .values()
                .cloned()
                .collect(),
        }
    }
}

pub struct State {
    pub must_reload_tasks: bool,
    pub exit: bool,
    pub running_tasks: HashMap<u64, RunningTask>,
    pub scheduled_request: Option<Option<Vec<(Task, OffsetDateTime)>>>,
}

impl State {
    pub fn new() -> Self {
        Self {
            must_reload_tasks: false,
            exit: false,
            running_tasks: HashMap::new(),
            scheduled_request: None,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RunningTask {
    pub task: Task,
    pub started: OffsetDateTime,
}

#[derive(Serialize, Deserialize)]
pub struct Scheduled {
    pub upcoming: Vec<(Task, OffsetDateTime)>,
    pub running: Vec<RunningTask>,
}
