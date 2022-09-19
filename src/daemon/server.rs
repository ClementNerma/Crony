use std::sync::RwLock;

use crate::service;

service!(
    daemon (WrappedState) {
        fn hello(state, __: ()) -> Result<String> {
            Ok("Hello".to_string())
        }
    }
);

pub struct State {}

impl State {
    pub fn new() -> Self {
        Self {}
    }
}

type WrappedState = RwLock<State>;
