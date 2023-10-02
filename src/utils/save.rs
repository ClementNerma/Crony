use std::{
    fs::{self},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use crate::{
    history::{History, HistoryEntry},
    paths::Paths,
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

pub fn read_history_file(history_file: &Path) -> Result<Option<History>> {
    if !history_file.is_file() {
        return Ok(None);
    }

    let raw = fs::read_to_string(history_file).context("Failed to read history file")?;

    let history = serde_json::from_str(&raw).context("Failed to parse the history file")?;

    Ok(Some(history))
}

pub fn append_to_history(history_file: &Path, entry: HistoryEntry) -> Result<()> {
    let mut history = if history_file.exists() {
        let content = fs::read_to_string(history_file).context("Failed to read history file")?;

        serde_json::from_str::<History>(&content).context("Failed to parse history file")?
    } else {
        History::empty()
    };

    history.append(entry);

    fs::write(history_file, serde_json::to_string(&history).unwrap())
        .context("Failed to update history file")?;

    Ok(())
}
