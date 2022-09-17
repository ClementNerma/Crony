use std::{
    io::{Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::Path,
    sync::Arc,
    time::Duration,
};

use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};

use crate::{error, info};

use super::{Processed, Request, Response};

pub fn create_socket(socket_path: &Path) -> Result<UnixListener> {
    UnixListener::bind(&socket_path).context("Failed to create socket with the provided path")
}

pub fn serve_on_socket<A: DeserializeOwned, B: Serialize>(
    listener: UnixListener,
    process: impl Fn(A) -> Processed<B> + Send + Sync + 'static,
) -> ! {
    let process = Arc::new(process);

    for client in listener.incoming() {
        let client = match client {
            Ok(client) => client,
            Err(err) => {
                error!("Failed to retrieve client: {err}");
                continue;
            }
        };

        if let Err(err) = client.set_nonblocking(true) {
            error!("Failed to set client in non-blocking mode: {err}");
            continue;
        }

        let process = Arc::clone(&process);
        std::thread::spawn(move || serve_client(client, process));
    }

    unreachable!()
}

fn serve_client<A: DeserializeOwned, B: Serialize>(
    mut client: UnixStream,
    process: Arc<impl Fn(A) -> Processed<B>>,
) -> ! {
    loop {
        let mut message = String::new();

        if let Err(err) = client.read_to_string(&mut message) {
            error!("Failed to read from client: {err}");
            short_sleep();
            continue;
        }

        if message.is_empty() {
            short_sleep();
            continue;
        }

        let Request { id, content } = match serde_json::from_str::<Request<A>>(&message) {
            Ok(req) => req,
            Err(err) => {
                error!("Failed to parse request from client: {err}");
                short_sleep();
                continue;
            }
        };

        info!("Treating message from client (message ID = {})...", id);

        let res = Response {
            for_id: id,
            result: process(content),
        };

        info!(
            "Finished treating message from client (message ID = {}).",
            id
        );

        let res = match serde_json::to_string(&res) {
            Ok(res) => res,
            Err(err) => {
                error!("Failed to stringify response for client: {err}");
                short_sleep();
                continue;
            }
        };

        if let Err(err) = client.write_all(res.as_bytes()) {
            error!("Failed to transmit response to client: {err}");
            short_sleep();
            continue;
        }
    }
}

fn short_sleep() {
    std::thread::sleep(Duration::from_millis(100))
}
