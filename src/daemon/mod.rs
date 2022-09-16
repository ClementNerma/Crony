mod notify;
mod scheduler;

use anyhow::Result;

use crate::paths::Paths;

use self::scheduler::Scheduler;

pub fn start_scheduler(paths: &Paths) -> Result<()> {
    Scheduler::new(paths)?.run();
    Ok(())
}
