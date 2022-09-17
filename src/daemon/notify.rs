use std::{fs, time::Duration};

use anyhow::{Context, Result};

use crate::{
    datetime::{get_now, human_datetime},
    info,
    paths::Paths,
    warn,
};

pub fn treat_reload_request(paths: &Paths) -> Result<bool> {
    if paths.reload_request_file.is_file() {
        fs::remove_file(&paths.reload_request_file)
            .context("Failed to remove the reload request file")?;
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn ask_daemon_reload(paths: &Paths, timeout_s: u8) -> Result<()> {
    if paths.reload_request_file.is_file() {
        warn!("A reload request is already pending!");
        return Ok(());
    }

    fs::write(&paths.reload_request_file, human_datetime(get_now()))
        .context("Failed to write the reload request file")?;

    info!("Reload request created, waiting for the daemon to treat it...");

    let mut treated = false;

    for _ in 0..(u64::from(timeout_s) * 1000 / ASK_DAEMON_RELOAD_MS_BETWEEN_CHECKS) {
        std::thread::sleep(Duration::from_millis(ASK_DAEMON_RELOAD_MS_BETWEEN_CHECKS));

        if !paths.reload_request_file.is_file() {
            treated = true;
            break;
        }
    }

    if treated {
        warn!("Timeout reached, is the daemon started?");
    }

    Ok(())
}

static ASK_DAEMON_RELOAD_MS_BETWEEN_CHECKS: u64 = 200;
