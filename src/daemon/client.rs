use std::{io::ErrorKind, os::unix::net::UnixStream, path::Path};

use anyhow::{bail, Result};

use crate::ipc::{ServiceClient, SocketClient};

use super::service::daemon::{Client, RequestContent as Req, ResponseContent as Res};

pub struct DaemonClient {
    inner: SocketClient<Req, Res>,
}

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

impl ServiceClient<Req, Res> for DaemonClient {
    fn send_unchecked(&mut self, req: Req) -> Result<Res> {
        self.inner.send_unchecked(req)
    }
}

impl Client for DaemonClient {
    type Client = Self;

    fn retrieve_client(&mut self) -> &mut Self::Client {
        self
    }
}
