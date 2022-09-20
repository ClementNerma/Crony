use std::sync::{Condvar, RwLock};

use crate::service;

service!(
    daemon (WrappedState) from (functions) {
        fn hello() -> String;
        fn reload_tasks(__: ()) -> ();
    }
);

mod functions {
    use std::sync::{Arc, RwLock};

    use super::State;

    pub fn hello(state: Arc<RwLock<State>>) -> String {
        "Hello".to_string()
    }

    pub fn reload_tasks(state: Arc<RwLock<State>>, __: ()) {
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

type WrappedState = RwLock<State>;
