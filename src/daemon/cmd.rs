use clap::Args;

use crate::engine::EngineArgs;

#[derive(Args)]
pub struct DaemonStartArgs {
    #[clap(flatten)]
    pub engine_args: EngineArgs,
}
