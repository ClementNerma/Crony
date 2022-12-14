use std::fmt::{Display, Formatter};

use anyhow::{bail, Context, Result};
use time::{format_description::well_known::Iso8601, OffsetDateTime};

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

    pub fn entries(&self) -> &[HistoryEntry] {
        &self.entries
    }

    pub fn find_last_for(&self, task_id: u64) -> Option<&HistoryEntry> {
        self.entries
            .iter()
            .rev()
            .find(|entry| entry.task_id == task_id)
    }
}

pub struct HistoryEntry {
    pub task_id: u64,
    pub task_name: String,
    pub started_at: OffsetDateTime,
    pub ended_at: OffsetDateTime,
    pub result: TaskResult,
}

impl HistoryEntry {
    pub fn parse(input: &str) -> Result<Self> {
        let mut segments = input.split(';');

        let task_id = segments.next().context("Missing task ID")?;
        let task_name = segments.next().context("Missing task name")?;
        let started_at = segments.next().context("Missing start date")?;
        let ended_at = segments.next().context("Missing end date")?;
        let result = segments.next().context("Missing failure code")?;

        Ok(Self {
            task_id: task_id.parse().context("Failed to parse task ID")?,
            task_name: task_name.to_string(),
            started_at: OffsetDateTime::parse(started_at, &Iso8601::DEFAULT)
                .context("Failed to parse start date")?,
            ended_at: OffsetDateTime::parse(ended_at, &Iso8601::DEFAULT)
                .context("Failed to parse end date")?,
            result: TaskResult::parse(result).context("Failed to parse task result")?,
        })
    }

    pub fn encode(&self) -> String {
        format!(
            "{};{};{};{};{}",
            self.task_id,
            self.task_name,
            self.started_at.format(&Iso8601::DEFAULT).unwrap(),
            self.ended_at.format(&Iso8601::DEFAULT).unwrap(),
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
                Some(code) => format!("{TASK_RESULT_ERR}{}", code),
                None => format!("{TASK_RESULT_ERR}{}", TASK_RESULT_NO_CODE),
            },
        }
    }
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

const TASK_RESULT_OK: &str = "OK";
const TASK_RESULT_ERR: &str = "FAILED:";
const TASK_RESULT_NO_CODE: &str = "-";
