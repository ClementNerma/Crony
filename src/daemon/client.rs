use std::path::Path;

use anyhow::Result;

use crate::ipc::{Processed, ServiceClient, SocketClient};

use super::server::daemon::{Client, RequestContent as Req, ResponseContent as Res};

pub struct DaemonClient {
    inner: SocketClient<Req, Res>,
}

impl DaemonClient {
    pub fn connect(socket_path: &Path) -> Result<Self> {
        Ok(Self {
            inner: SocketClient::connect(socket_path)?,
        })
    }
}

impl ServiceClient<Req, Res> for DaemonClient {
    fn send_unchecked(&mut self, req: Req) -> Processed<Res> {
        self.inner.send_unchecked(req)
    }
}

impl Client for DaemonClient {
    type Client = Self;

    fn retrieve_client(&mut self) -> &mut Self::Client {
        self
    }
}
