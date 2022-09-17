use clap::Args;

use crate::engine::cmd::EngineArgs;

#[derive(Args)]
pub struct DaemonArgs {
    #[clap(flatten)]
    pub engine_args: EngineArgs,
}
