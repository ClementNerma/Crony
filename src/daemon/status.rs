use std::{io::ErrorKind, os::unix::net::UnixStream, path::Path};

use anyhow::{bail, Result};

pub fn check_daemon_status(socket_path: &Path) -> Result<DaemonStatus> {
    if !socket_path.exists() {
        return Ok(DaemonStatus::NoSocketFile);
    }

    match UnixStream::connect(socket_path) {
        // TODO: communicate with the daemon to be *sure* that it is, indeed, running?
        Ok(_) => Ok(DaemonStatus::Running),
        Err(err) => match err.kind() {
            ErrorKind::ConnectionRefused => Ok(DaemonStatus::NotRunning),
            err => bail!("Failed to handle the socket file: {}", err),
        },
    }
}

#[derive(PartialEq, Eq)]
pub enum DaemonStatus {
    NoSocketFile,
    NotRunning,
    Running,
}
