use std::sync::{Condvar, RwLock};

use crate::service;

service!(
    daemon (WrappedState) {
        fn hello(state, __: ()) -> String {
            "Hello".to_string()
        }

        fn reload_tasks(state, __: ()) -> () {
            let cvar = std::sync::Condvar::new();
            state.write().unwrap().must_reload_tasks = Some(cvar);
        }
    }
);

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
