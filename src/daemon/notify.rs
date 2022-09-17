use std::{fs, time::Duration};

use anyhow::{Context, Result};

use crate::{
    datetime::{get_now, human_datetime},
    info,
    paths::Paths,
    save::read_tasks,
    success,
    task::Tasks,
    warn,
};

pub fn treat_reload_request(paths: &Paths) -> Result<Option<Tasks>> {
    if paths.reload_request_file.is_file() {
        fs::remove_file(&paths.reload_request_file)
            .context("Failed to remove the reload request file")?;
        Ok(Some(read_tasks(paths)?))
    } else {
        Ok(None)
    }
}

pub fn ask_daemon_reload(paths: &Paths) -> Result<()> {
    if paths.reload_request_file.is_file() {
        warn!("A reload request is already pending!");
        return Ok(());
    }

    fs::write(&paths.reload_request_file, human_datetime(get_now()))
        .context("Failed to write the reload request file")?;

    info!("Reload request created, waiting for the daemon to treat it...");

    let mut treated = false;

    // Short timeout as the daemon is supposed to detect changes in ~1s max.
    for _ in 0..15 {
        std::thread::sleep(Duration::from_millis(100));

        if !paths.reload_request_file.is_file() {
            treated = true;
            break;
        }
    }

    if !treated {
        warn!("Timeout reached, is the daemon started?");
    } else {
        success!("Daemon successfully treated the reload request.");
    }

    Ok(())
}
