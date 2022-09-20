use std::sync::Condvar;

use crate::service;

service!(
    daemon (functions) {
        fn hello() -> String;
        fn reload_tasks() -> ();
    }
);

mod functions {
    use std::sync::{Arc, RwLock};

    pub type State = RwLock<super::State>;

    pub fn hello(_: Arc<State>) -> String {
        "Hello".to_string()
    }

    pub fn reload_tasks(state: Arc<State>) {
        let cvar = std::sync::Condvar::new();
        state.write().unwrap().must_reload_tasks = Some(cvar);
    }
}

pub struct State {
    must_reload_tasks: Option<Condvar>,
}

impl State {
    pub fn new() -> Self {
        Self {
            must_reload_tasks: None,
        }
    }
}
