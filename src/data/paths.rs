use std::path::{Path, PathBuf};

use time::OffsetDateTime;

#[derive(Clone)]
pub struct Paths {
    pub data_dir: PathBuf,
    pub daemon_dir: PathBuf,
    pub tasks_dir: PathBuf,
    pub old_tasks_dir: PathBuf,

    pub tasks_file: PathBuf,

    pub daemon_socket_file: PathBuf,
    pub daemon_log_file: PathBuf,
}

impl Paths {
    pub fn new(data_dir: PathBuf) -> Self {
        let daemon_dir = data_dir.join("daemon");

        Self {
            tasks_file: data_dir.join("tasks.json"),

            tasks_dir: data_dir.join("tasks"),
            old_tasks_dir: data_dir.join("tasks.old"),

            daemon_socket_file: daemon_dir.join("daemon.sock"),
            daemon_log_file: daemon_dir.join("output.log"),

            daemon_dir,
            data_dir,
        }
    }

    pub fn task_paths(&self, task_name: &str) -> TaskPaths {
        TaskPaths {
            task_dir: self.tasks_dir.join(task_name),
        }
    }

    pub fn generate_old_task_dir_name(&self, task_name: &str) -> PathBuf {
        self.old_tasks_dir.join(format!(
            "{}-{}",
            OffsetDateTime::now_local()
                .expect("Failed to get current date/time")
                .unix_timestamp(),
            task_name
        ))
    }
}

pub struct TaskPaths {
    task_dir: PathBuf,
}

impl TaskPaths {
    pub fn dir(&self) -> &Path {
        &self.task_dir
    }

    pub fn history_file(&self) -> PathBuf {
        self.task_dir.join("history")
    }

    pub fn log_file(&self) -> PathBuf {
        self.task_dir.join("output.log")
    }
}
