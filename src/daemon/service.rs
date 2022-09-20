use std::sync::{Condvar, RwLock};

use crate::service;

service!(
    daemon (WrappedState) {
        fn hello(state, __: ()) -> Result<String> {
            Ok("Hello".to_string())
        }

        fn reload_tasks(state, __: ()) -> Result<()> {
            let cvar = std::sync::Condvar::new();
            state.write().unwrap().must_reload_tasks = Some(cvar);

            Ok(())
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
