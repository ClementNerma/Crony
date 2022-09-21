use std::{
    io::{BufRead, BufReader, Write},
    marker::PhantomData,
    os::unix::net::UnixStream,
};

use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};

use super::{Request, Response};

pub struct SocketClient<A: Serialize, B: DeserializeOwned> {
    stream: UnixStream,
    _req: PhantomData<A>,
    _res: PhantomData<B>,
}

impl<A: Serialize, B: DeserializeOwned> SocketClient<A, B> {
    pub fn new(stream: UnixStream) -> Self {
        Self {
            stream,
            _req: PhantomData,
            _res: PhantomData,
        }
    }

    // pub fn connect(socket_path: &Path) -> Result<Self> {
    //     let stream =
    //         UnixStream::connect(socket_path).context("Failed to connect to the provided socket")?;

    //     Ok(Self {
    //         stream,
    //         _req: PhantomData,
    //         _res: PhantomData,
    //     })
    // }

    pub fn send_unchecked(&mut self, req: A) -> Result<B> {
        let req = Request {
            id: rand::random(),
            content: req,
        };

        let mut req_str =
            serde_json::to_string(&req).context("Failed to stringify request for server")?;

        // Message separator
        req_str.push('\n');

        self.stream
            .write_all(req_str.as_bytes())
            .context("Failed to transmit request to server")?;

        self.stream
            .flush()
            .context("Failed to flush the server's stream")?;

        let response = BufReader::new(&self.stream)
            .lines()
            .next()
            .context("Failed to get a response from the server")?
            .context("Failed to retrieve the server's response")?;

        let response = serde_json::from_str::<Response<B>>(&response)
            .context("Failed to parse server's response")?;

        // TODO: queue system with untreated responses
        assert_eq!(req.id, response.for_id);

        Ok(response.result)
    }
}
