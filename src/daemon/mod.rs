mod client;
mod cmd;
mod service;
mod start;

pub use client::DaemonClient;
pub use cmd::*;
pub use service::daemon::Client;
pub use start::start_daemon;

use std::{io::ErrorKind, os::unix::net::UnixStream, path::Path};

use anyhow::{bail, Result};

pub fn is_daemon_running(socket_path: &Path) -> Result<bool> {
    if !socket_path.exists() {
        return Ok(false);
    }

    match UnixStream::connect(socket_path) {
        // TODO: communicate with the daemon to be *sure* that it is, indeed, running?
        Ok(_) => Ok(true),
        Err(err) => match err.kind() {
            ErrorKind::ConnectionRefused => Ok(false),
            err => bail!("Failed to handle the socket file: {}", err),
        },
    }
}
