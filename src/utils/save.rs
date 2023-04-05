use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use dirs::config_dir;

use crate::{
    history::{History, HistoryEntry},
    paths::{Paths, TaskPaths},
    task::Tasks,
};

pub fn construct_data_dir_paths(custom_data_dir: Option<PathBuf>) -> Result<Paths> {
    let default_data_dir = config_dir()
        .context("Failed to find user's config directory")?
        .join("crony");

    let data_dir = custom_data_dir.unwrap_or(default_data_dir);

    if !data_dir.is_dir() {
        fs::create_dir(&data_dir).context("Failed to create the data directory")?;
    }

    let paths = Paths::new(data_dir);

    ensure_data_dirs_exist(&paths)?;

    Ok(paths)
}

pub fn ensure_data_dirs_exist(paths: &Paths) -> Result<()> {
    if !paths.tasks_dir.exists() {
        fs::create_dir(&paths.tasks_dir).context("Failed to create the tasks directory")?;
    }

    if !paths.old_tasks_dir.exists() {
        fs::create_dir(&paths.old_tasks_dir).context("Failed to create the old tasks directory")?;
    }

    Ok(())
}

pub fn read_tasks(paths: &Paths) -> Result<Tasks> {
    if paths.tasks_file.is_file() {
        read_tasks_no_default(paths)
    } else {
        Ok(Tasks::default())
    }
}

pub fn read_tasks_no_default(paths: &Paths) -> Result<Tasks> {
    let raw = fs::read_to_string(&paths.tasks_file).context("Failed to read the tasks file")?;
    serde_json::from_str(&raw).context("Failed to parse the tasks file")
}

pub fn write_tasks(paths: &Paths, tasks: &Tasks) -> Result<()> {
    let raw =
        serde_json::to_string_pretty(tasks).context("Failed to stringify the provided tasks")?;

    fs::write(&paths.tasks_file, raw).context("Failed to write the tasks file")
}

pub fn read_history_if_exists(task_paths: &TaskPaths) -> Result<History> {
    let history_file = task_paths.history_file();

    if !history_file.is_file() {
        return Ok(History::empty());
    }

    let raw = fs::read_to_string(&history_file).context("Failed to read history file")?;

    History::parse(&raw).context("Failed to parse the history file")
}

pub fn append_to_history(history_file: &Path, entry: &HistoryEntry) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(history_file)
        .context("Failed to open the history file")?;

    writeln!(file, "{}", entry.encode())?;

    Ok(())
}
