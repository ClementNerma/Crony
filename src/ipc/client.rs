use std::{
    io::{Read, Write},
    marker::PhantomData,
    os::unix::net::UnixStream,
    path::Path,
};

use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};

use crate::error;

pub struct SocketClient<A: Serialize, B: DeserializeOwned> {
    stream: UnixStream,
    _req: PhantomData<A>,
    _res: PhantomData<B>,
}

impl<A: Serialize, B: DeserializeOwned> SocketClient<A, B> {
    pub fn connect(socket_path: &Path) -> Result<Self> {
        let stream =
            UnixStream::connect(socket_path).context("Failed to context to the provided socket")?;

        Ok(Self {
            stream,
            _req: PhantomData,
            _res: PhantomData,
        })
    }

    pub fn send_unchecked_base(&mut self, req: A) -> Result<B> {
        let req = serde_json::to_string(&req).context("Failed to stringify request for server")?;
        self.stream
            .write_all(req.as_bytes())
            .context("Failed to transmit request to server")?;

        // TODO: queue system with untreated responses

        let mut message = String::new();

        while message.is_empty() {
            self.stream
                .read_to_string(&mut message)
                .context("Failed to read response from server")?;
        }

        let response =
            serde_json::from_str::<B>(&message).context("Failed to parse server's response")?;

        Ok(response)
    }
}
