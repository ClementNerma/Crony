use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Serialize, Deserialize)]
pub struct History {
    entries: Vec<HistoryEntry>,
}

impl History {
    pub fn empty() -> Self {
        Self { entries: vec![] }
    }

    pub fn entries(&self) -> &[HistoryEntry] {
        &self.entries
    }

    pub fn append(&mut self, entry: HistoryEntry) {
        self.entries.push(entry);
    }

    pub fn for_task(&self, task_id: u64) -> impl Iterator<Item = &HistoryEntry> {
        self.entries
            .iter()
            .filter(move |entry| entry.task_id == task_id)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HistoryEntry {
    pub task_id: u64,
    pub task_name: String,
    pub started_at: OffsetDateTime,
    pub ended_at: OffsetDateTime,
    pub result: TaskResult,
}

impl HistoryEntry {
    pub fn succeeded(&self) -> bool {
        matches!(self.result, TaskResult::Success)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum TaskResult {
    Success,
    Failed { code: Option<i32> },
}

impl Display for TaskResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskResult::Success => write!(f, "success"),
            TaskResult::Failed { code } => match code {
                None => write!(f, "failed (no exit code)"),
                Some(code) => write!(f, "failed with code {}", code),
            },
        }
    }
}
