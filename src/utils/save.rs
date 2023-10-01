use std::{
    fs::{self, OpenOptions},
    io::Read,
    os::unix::prelude::FileExt,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use crate::{
    history::{History, HistoryEntry},
    paths::{Paths, TaskPaths},
    task::Tasks,
};

pub fn construct_data_dir_paths(custom_data_dir: Option<PathBuf>) -> Result<Paths> {
    let default_data_dir = dirs::data_dir()
        .context("Failed to determine path to the user's data directory")?
        .join("crony");

    let data_dir = custom_data_dir.unwrap_or(default_data_dir);

    if !data_dir.is_dir() {
        fs::create_dir_all(&data_dir).context("Failed to create the data directory")?;
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

pub fn read_history_file(task_paths: &TaskPaths) -> Result<Option<History>> {
    let history_file = task_paths.history_file();

    if !history_file.is_file() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&history_file).context("Failed to read history file")?;

    let history = serde_json::from_str(&raw).context("Failed to parse the history file")?;

    Ok(Some(history))
}

pub fn append_to_history(history_file: &Path, entry: HistoryEntry) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(history_file)
        .context("Failed to open the history file")?;

    let mut content = String::new();
    file.read_to_string(&mut content)
        .context("Failed to read history file")?;

    let mut history = if content.is_empty() {
        History::empty()
    }  else {    
        serde_json::from_str::<History>(&content).context("Failed to parse history file")?
    };

    history.append(entry);

    let content = serde_json::to_string(&history).unwrap();

    file.write_all_at(content.as_bytes(), 0)
        .context("Failed to update history file")?;

    Ok(())
}
