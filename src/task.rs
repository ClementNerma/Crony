use std::collections::BTreeMap;

use once_cell::sync::Lazy;
use pomsky_macro::pomsky;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::at::At;

static NAME_VALIDATOR: Lazy<Regex> =
    Lazy::new(|| Regex::new(pomsky!(Start ['a'-'z' 'A'-'Z' '0'-'9' '-' '_']+ End)).unwrap());

pub type Tasks = BTreeMap<String, Task>;

#[derive(Serialize, Deserialize)]
pub struct Task {
    pub name: String,
    pub display_name: Option<String>,
    pub run_at: At,
    pub shell: String,
    pub cmd: String,
}

impl Task {
    pub fn is_valid_name(name: &str) -> bool {
        NAME_VALIDATOR.is_match(name)
    }
}
