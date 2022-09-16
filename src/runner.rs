use std::{
    fs::OpenOptions,
    process::{Command, Stdio},
};

use crate::{
    history::{HistoryEntry, TaskResult},
    info,
    paths::TaskPaths,
    save::append_to_history,
    task::Task,
};
use anyhow::{Context, Result};
use chrono::Local;

pub fn runner(task: &Task, paths: &TaskPaths, use_log_files: bool) -> Result<HistoryEntry> {
    let started_at = Local::now();

    info!(
        "Starting task '{}' at {}...",
        task.name.bright_yellow(),
        started_at.to_rfc2822().bright_magenta()
    );

    let mut shell_cmd_parts = task.shell.split(' ');

    let mut cmd = Command::new(shell_cmd_parts.next().unwrap());

    for part in shell_cmd_parts {
        cmd.arg(part);
    }

    cmd.arg(&task.cmd);

    if use_log_files {
        let stdout_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(paths.stdout_log_file())
            .context("Failed to open the task's STDOUT log file")?;

        let stderr_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(paths.stderr_log_file())
            .context("Failed to open the task's STDERR log file")?;

        cmd.stdout(Stdio::from(stdout_file));
        cmd.stderr(Stdio::from(stderr_file));
    }

    let status = cmd.status().context("Failed to run the task's command")?;

    let ended_at = Local::now();

    let result = if status.success() {
        TaskResult::Success
    } else {
        TaskResult::Failed {
            code: status.code(),
        }
    };

    info!(
        "Task finished running at {} ; exit status: {}",
        ended_at.to_rfc2822().bright_magenta(),
        result
    );

    let entry = HistoryEntry {
        task_name: task.name.clone(),
        started_at,
        ended_at,
        result,
    };

    append_to_history(paths, &entry)?;

    Ok(entry)
}
