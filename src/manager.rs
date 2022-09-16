use std::collections::BTreeMap;

use crate::task::Task;

pub type Tasks = BTreeMap<String, Task>;

// TODO: TaskManager which contains an inner Tasks
