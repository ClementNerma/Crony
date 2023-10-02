use std::path::PathBuf;

#[derive(Clone)]
pub struct Paths {
    pub data_dir: PathBuf,
    pub daemon_dir: PathBuf,
    pub tasks_dir: PathBuf,

    pub tasks_file: PathBuf,
    pub history_file: PathBuf,

    pub daemon_socket_file: PathBuf,
    pub daemon_log_file: PathBuf,
}

impl Paths {
    pub fn new(data_dir: PathBuf) -> Self {
        let daemon_dir = data_dir.join("daemon");

        Self {
            tasks_file: data_dir.join("tasks.json"),
            history_file: data_dir.join("history.json"),

            tasks_dir: data_dir.join("tasks"),

            daemon_socket_file: daemon_dir.join("daemon.sock"),
            daemon_log_file: daemon_dir.join("daemon.log"),

            daemon_dir,
            data_dir,
        }
    }

    pub fn task_log_file(&self, task_name: &str) -> PathBuf {
        self.tasks_dir.join(format!("{task_name}.log"))
    }
}
