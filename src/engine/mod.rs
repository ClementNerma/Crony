pub mod cmd;
pub mod runner;
pub mod scheduler;
pub mod upcoming;

use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

use anyhow::Result;

use crate::{
    datetime::get_now, error, error_anyhow, paths::Paths, runner::runner, save::read_tasks,
    success, task::Tasks, treat_reload_request,
};

use self::{cmd::EngineArgs, scheduler::run_tasks};

pub fn start_engine(paths: &Paths, args: &EngineArgs) -> Result<()> {
    treat_reload_request(paths)?;

    loop {
        engine_core_path(paths, args)?;
    }
}

fn engine_core_path(paths: &Paths, args: &EngineArgs) -> Result<()> {
    let paths_1 = paths.clone();
    let paths_2 = paths.clone();

    let reloaded = Arc::new(RwLock::new(Option::<Tasks>::None));
    let reloaded_writer = Arc::clone(&reloaded);

    let direct_output = args.direct_output;

    run_tasks(
        read_tasks(paths)?,
        move |task| {
            let result = runner(
                task,
                &paths_1.task_paths(&task.name),
                !direct_output,
                || match &*reloaded.read().unwrap() {
                    None => false,
                    Some(tasks) => !tasks.values().any(|c| c.id == task.id),
                },
            );

            if let Err(err) = result {
                error_anyhow!(err.context("Runner failed to run (from Scheduler)"));
                error!("Now sleeping for 5 seconds...");
                std::thread::sleep(Duration::from_secs(5));
            }
        },
        || match treat_reload_request(&paths_2) {
            Ok(Some(tasks)) => {
                reloaded_writer.write().unwrap().replace(tasks);
                success!(
                    "Reloading request was successfully treated on {}",
                    get_now().to_string().bright_magenta()
                );
                true
            }
            Ok(None) => false,
            Err(err) => {
                error_anyhow!(err);
                false
            }
        },
    )
}
