use std::{io::ErrorKind, os::unix::net::UnixStream, path::Path};

use anyhow::{bail, Result};

use crate::ipc::SocketClient;

pub use super::service::daemon::Client as DaemonClient;

impl DaemonClient {
    pub fn connect(socket_path: &Path) -> Result<Self> {
        if !socket_path.exists() {
            bail!("Daemon is not running.");
        }

        match UnixStream::connect(socket_path) {
            Ok(stream) => Ok(Self {
                inner: SocketClient::new(stream),
            }),

            Err(err) => match err.kind() {
                ErrorKind::ConnectionRefused => bail!("Daemon is not running."),
                err => bail!("Failed to handle the socket file: {}", err),
            },
        }
    }
}
