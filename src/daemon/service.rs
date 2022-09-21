use crate::service;

service!(
    daemon (functions) {
        fn hello() -> String;
        fn reload_tasks() -> ();
    }
);

mod functions {
    use std::{
        sync::{Arc, RwLock},
        time::Duration,
    };

    pub type State = RwLock<super::State>;

    pub fn hello(_: Arc<State>) -> String {
        "Hello".to_string()
    }

    pub fn reload_tasks(state: Arc<State>) {
        state.write().unwrap().must_reload_tasks = true;

        while state.read().unwrap().must_reload_tasks {
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}

pub struct State {
    pub must_reload_tasks: bool,
}

impl State {
    pub fn new() -> Self {
        Self {
            must_reload_tasks: false,
        }
    }
}
