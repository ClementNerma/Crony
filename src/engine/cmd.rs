use clap::Args;

#[derive(Args)]
pub struct EngineArgs {
    #[clap(
        short,
        long,
        help = "Display tasks's STDOUT and STDERR directly (bypasses log files)"
    )]
    pub(super) direct_output: bool,
}
