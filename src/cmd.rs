use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::daemon::DaemonStartArgs;

#[derive(Parser)]
#[clap(author, version, help = "Replacement program for 'cron' and 'crontab'")]
pub struct Cmd {
    #[clap(short, long, help = "Path to the data directory")]
    pub data_dir: Option<PathBuf>,

    #[clap(short, long, help = "Display debug messages")]
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
    DaemonStart(DaemonStartArgs),

    #[clap(about = "Check the daemon's status")]
    DaemonStatus,

    #[clap(about = "Stop the daemon")]
    DaemonStop,
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
