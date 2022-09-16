mod cmd;
mod notify;
mod scheduler;

pub use cmd::DaemonArgs;

use anyhow::Result;

use crate::paths::Paths;

use self::scheduler::Scheduler;

pub fn start_scheduler(paths: &Paths, args: &DaemonArgs) -> Result<()> {
    Scheduler::new(paths, args)?.run();
    Ok(())
}
