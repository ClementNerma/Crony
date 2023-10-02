use std::{
    fs::OpenOptions,
    io::{BufRead, BufReader, Write},
    process::Command,
};

use crate::{
    datetime::{get_now, get_now_second_precision},
    history::{HistoryEntry, TaskResult},
    info,
    paths::Paths,
    save::append_to_history,
    task::Task,
};
use anyhow::{Context, Result};

pub static DEFAULT_SHELL_CMD: &str = "/bin/sh -c";

pub fn runner(task: &Task, paths: &Paths, use_log_files: bool) -> Result<HistoryEntry> {
    let global_history_file = &paths.history_file;

    let started_at = get_now();

    info!(
        "Starting task '{}' on {}...",
        task.name.bright_yellow(),
        started_at.to_string().bright_magenta()
    );

    let shell_cmd = task
        .shell
        .clone()
        .unwrap_or_else(|| DEFAULT_SHELL_CMD.to_string());

    let mut shell_cmd_parts = shell_cmd.split(' ');

    let mut cmd = Command::new(shell_cmd_parts.next().unwrap());

    for part in shell_cmd_parts {
        cmd.arg(part);
    }

    cmd.arg(&task.cmd);

    let mut log_file = if use_log_files {
        Some(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(paths.task_log_file(&task.name))
                .context("Failed to open the task's log file")?,
        )
    } else {
        None
    };

    let (reader, writer) = os_pipe::pipe().context("Failed to obtain a pipe")?;

    cmd.stdout(writer.try_clone().context("Failed to clone the writer")?);
    cmd.stderr(writer);

    let mut handle = cmd.spawn().context("Failed to spawn the command")?;

    drop(cmd);

    if let Some(log_file) = &mut log_file {
        log_file
            .write_all(format!("=======> Started on {}\n\n", get_now_second_precision()).as_bytes())
            .unwrap();
    }

    let reader = BufReader::new(reader);

    for line in reader.lines() {
        let line = line.unwrap();
        let mut line = format!("[{}] {}", get_now(), line);

        if let Some(log_file) = &mut log_file {
            line.push('\n');
            log_file.write_all(line.as_bytes()).unwrap();
        } else {
            println!("{line}");
        }
    }

    if let Some(log_file) = &mut log_file {
        log_file
            .write_all(
                format!("\n=======> Ended on {}\n\n\n", get_now_second_precision()).as_bytes(),
            )
            .unwrap();
    }

    let status = handle.wait().context("Failed to run the task's command")?;

    let ended_at = get_now();

    let result = if status.success() {
        TaskResult::Success
    } else {
        TaskResult::Failed {
            code: status.code(),
        }
    };

    info!(
        "Task '{}' finished running on {} ({})",
        task.name.bright_yellow(),
        ended_at.to_string().bright_magenta(),
        match result {
            TaskResult::Success => format!("{}", result).bright_green(),
            TaskResult::Failed { code: _ } => format!("{}", result).bright_red(),
        }
    );

    let entry = HistoryEntry {
        task_id: task.id,
        task_name: task.name.clone(),
        started_at,
        ended_at,
        result,
    };

    append_to_history(global_history_file, entry.clone()).with_context(|| {
        format!(
            "Failed to append an entry to history file at path: {}",
            global_history_file.display()
        )
    })?;

    Ok(entry)
}
