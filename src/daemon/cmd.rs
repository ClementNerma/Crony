use clap::Args;

use crate::engine::cmd::EngineArgs;

#[derive(Args)]
pub struct DaemonStartArgs {
    #[clap(flatten)]
    pub engine_args: EngineArgs,
}

#[derive(Args)]
pub struct DaemonStatusArgs {}
