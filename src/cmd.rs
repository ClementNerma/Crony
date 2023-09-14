use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::daemon::DaemonStartArgs;

#[derive(Parser)]
#[clap(version, about, author)]
pub struct Cmd {
    #[clap(short, long, help = "Path to the data directory")]
    pub data_dir: Option<PathBuf>,

    #[clap(short, long, global = true, help = "Display debug messages")]
    pub verbose: bool,

    #[clap(subcommand)]
    pub action: Action,
}

#[derive(Subcommand)]
pub enum Action {
    #[clap(about = "List registered tasks")]
    List,

    #[clap(about = "Check if any task recently failed")]
    Check,

    #[clap(about = "Register a task (if not registered yet)")]
    Register(RegisterArgs),

    #[clap(about = "Unregister a task")]
    Unregister(UnregisterArgs),

    #[clap(about = "Run a task immediatly")]
    Run(RunArgs),

    #[clap(about = "Start the daemon")]
    Start(DaemonStartArgs),

    #[clap(about = "Check the daemon's status")]
    Status,

    #[clap(about = "Check the scheduled and running tasks")]
    Scheduled,

    #[clap(about = "Stop the daemon")]
    Stop,

    #[clap(about = "Display the logs")]
    Logs(LogsArgs),

    #[clap(about = "Display the execution history")]
    History(HistoryArgs),
}

#[derive(Args)]
pub struct RegisterArgs {
    #[clap(help = "Name of the task")]
    pub name: String,

    #[clap(short, long, help = "The command to run")]
    pub run: String,

    #[clap(long, help = "Times to run at (pattern like 'D=10,20 h=*")]
    pub at: String,

    #[clap(long, help = "The shell to use")]
    pub using: Option<String>,

    #[clap(long, help = "Override any task with the provided name")]
    pub force_override: bool,

    #[clap(
        long,
        help = "Do nothing if a task with the same name and parameters already exist"
    )]
    pub ignore_identical: bool,

    #[clap(long, help = "Don't display messages outside of errors")]
    pub silent: bool,
}

#[derive(Args)]
pub struct UnregisterArgs {
    #[clap(help = "Name of the task to unregister")]
    pub name: String,
}

#[derive(Args)]
pub struct RunArgs {
    #[clap(help = "Name of the task to run")]
    pub name: String,

    #[clap(
        long,
        help = "Redirect output to the log files instead of STDOUT/STDERR"
    )]
    pub use_log_files: bool,
}

#[derive(Args)]
pub struct LogsArgs {
    #[clap(help = "Show the logs of a task")]
    pub task_name: Option<String>,

    #[clap(
        long,
        help = "Use an alternative pager (default: PAGER env var, or 'less')"
    )]
    pub pager: Option<String>,

    #[clap(
        long,
        help = "Don't provide recommanded arguments when pager is 'less'"
    )]
    pub no_less_options: bool,
}

#[derive(Args)]
pub struct HistoryArgs {
    #[clap(help = "Show the history of a task")]
    pub task_name: Option<String>,

    #[clap(help = "Show the N last entries")]
    pub last_entries: Option<usize>,
}
