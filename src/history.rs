use std::fmt::{Display, Formatter};

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Local};

pub struct History {
    entries: Vec<HistoryEntry>,
}

impl History {
    pub fn empty() -> Self {
        Self { entries: vec![] }
    }

    pub fn parse(history: &str) -> Result<Self> {
        let entries = history
            .lines()
            .enumerate()
            .map(|(i, line)| {
                HistoryEntry::parse(line).with_context(|| format!("Failed to parse line {}", i + 1))
            })
            .collect::<Result<_>>()?;

        Ok(Self { entries })
    }

    pub fn find_last_for(&self, task_name: &str) -> Option<&HistoryEntry> {
        self.entries
            .iter()
            .rev()
            .find(|entry| entry.task_name == task_name)
    }
}

pub struct HistoryEntry {
    pub task_name: String,
    pub started_at: DateTime<Local>,
    pub ended_at: DateTime<Local>,
    pub result: TaskResult,
}

impl HistoryEntry {
    pub fn success(
        task_name: String,
        started_at: DateTime<Local>,
        ended_at: DateTime<Local>,
    ) -> Self {
        Self {
            task_name,
            started_at,
            ended_at,
            result: TaskResult::Success,
        }
    }

    pub fn failure(
        task_name: String,
        started_at: DateTime<Local>,
        ended_at: DateTime<Local>,
        exit_code: Option<i32>,
    ) -> Self {
        Self {
            task_name,
            started_at,
            ended_at,
            result: TaskResult::Failed { code: exit_code },
        }
    }

    pub fn parse(input: &str) -> Result<Self> {
        let mut segments = input.split(';');

        let task_name = segments.next().context("Missing task name")?;
        let started_at = segments.next().context("Missing start date")?;
        let ended_at = segments.next().context("Missing end date")?;
        let result = segments.next().context("Missing failure code")?;

        Ok(Self {
            task_name: task_name.to_string(),
            started_at: str::parse(started_at).context("Failed to parse start date")?,
            ended_at: str::parse(ended_at).context("Failed to parse end date")?,
            result: TaskResult::parse(result).context("Failed to parse task result")?,
        })
    }

    pub fn encode(&self) -> String {
        format!(
            "{};{};{};{}",
            self.task_name,
            self.started_at,
            self.ended_at,
            self.result.encode()
        )
    }

    pub fn succeeded(&self) -> bool {
        matches!(self.result, TaskResult::Success)
    }
}

pub enum TaskResult {
    Success,
    Failed { code: Option<i32> },
}

impl TaskResult {
    pub fn parse(input: &str) -> Result<Self> {
        if input == TASK_RESULT_OK {
            Ok(Self::Success)
        } else if let Some(code) = input.strip_prefix(TASK_RESULT_ERR) {
            Ok(Self::Failed {
                code: if code == TASK_RESULT_NO_CODE {
                    None
                } else {
                    Some(str::parse::<i32>(code).context("Invalid task result code")?)
                },
            })
        } else {
            bail!("Invalid task result provided");
        }
    }

    pub fn encode(&self) -> String {
        match self {
            TaskResult::Success => TASK_RESULT_OK.to_string(),
            TaskResult::Failed { code } => match code {
                Some(code) => format!("{TASK_RESULT_ERR}{}", code.to_string()),
                None => format!("{TASK_RESULT_ERR}{}", TASK_RESULT_NO_CODE),
            },
        }
    }
}

impl Display for TaskResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskResult::Success => writeln!(f, "Success"),
            TaskResult::Failed { code } => match code {
                None => writeln!(f, "Failed (no exit code)"),
                Some(code) => writeln!(f, "Failed with code {}", code),
            },
        }
    }
}

const TASK_RESULT_OK: &str = "OK";
const TASK_RESULT_ERR: &str = "FAILED:";
const TASK_RESULT_NO_CODE: &str = "-";
