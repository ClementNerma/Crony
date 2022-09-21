use std::collections::HashMap;

use time::OffsetDateTime;

use crate::{service, task::Task};

service!(
    daemon (functions) {
        fn hello() -> String;
        fn stop() -> ();
        fn reload_tasks() -> ();
        fn running_tasks() -> usize;
    }
);

mod functions {
    use std::sync::{Arc, RwLock};

    use crate::sleep::sleep_ms;

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
}

pub struct State {
    pub must_reload_tasks: bool,
    pub exit: bool,
    pub running_tasks: HashMap<u64, RunningTask>,
}

impl State {
    pub fn new() -> Self {
        Self {
            must_reload_tasks: false,
            exit: false,
            running_tasks: HashMap::new(),
        }
    }
}

pub struct RunningTask {
    pub task: Task,
    pub started: OffsetDateTime,
}
