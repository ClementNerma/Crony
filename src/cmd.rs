use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::{daemon::DaemonArgs, engine::cmd::EngineArgs};

#[derive(Parser)]
pub struct Cmd {
    #[clap(short, long, help = "Path to the data directory")]
    pub data_dir: Option<PathBuf>,

    #[clap(subcommand)]
    pub action: Action,
}

#[derive(Subcommand)]
pub enum Action {
    #[clap(about = "List registered tasks")]
    List(ListArgs),

    #[clap(about = "Register a task (if not registered yet)")]
    Register(RegisterArgs),

    #[clap(about = "Unregister a task")]
    Unregister(UnregisterArgs),

    #[clap(about = "Run a task immediatly")]
    Run(RunArgs),

    #[clap(about = "Run the engine in foreground")]
    Foreground(EngineArgs),

    #[clap(about = "Start the daemon")]
    Daemon(DaemonArgs),
}

#[derive(Args)]
pub struct ListArgs {}

#[derive(Args)]
pub struct RegisterArgs {
    #[clap(help = "Name of the task")]
    pub name: String,

    #[clap(help = "The command to run")]
    pub cmd: String,

    #[clap(help = "Times to run at (pattern like 'D=10,20 h=*")]
    pub run_at: String,

    #[clap(short, long, help = "The shell to use")]
    pub shell: String,

    #[clap(short, long, help = "Display name of the task")]
    pub display_name: Option<String>,

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
