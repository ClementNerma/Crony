use clap::Args;

use crate::engine::EngineArgs;

#[derive(Args)]
pub struct DaemonStartArgs {
    #[clap(flatten)]
    pub engine_args: EngineArgs,

    #[clap(long, help = "Do nothing if the daemon is already started")]
    pub ignore_started: bool,
}
