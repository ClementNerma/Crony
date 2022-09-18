use std::{
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::Path,
    sync::Arc,
    time::Duration,
};

use anyhow::{bail, Context, Result};
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    daemon::{check_daemon_status, DaemonStatus},
    error, info,
};

use super::{Request, Response};

pub fn create_socket(socket_path: &Path) -> Result<UnixListener> {
    match check_daemon_status(socket_path)? {
        DaemonStatus::NoSocketFile => {}
        DaemonStatus::NotRunning => {
            fs::remove_file(socket_path).context("Failed to remove the existing socket file")?
        }
        DaemonStatus::Running => bail!("Daemon is already running!"),
    }

    UnixListener::bind(&socket_path).context("Failed to create socket with the provided path")
}

pub fn serve_on_socket<A: DeserializeOwned, B: Serialize>(
    listener: UnixListener,
    process: impl Fn(A) -> Result<B, String> + Send + Sync + 'static,
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

        // if let Err(err) = client.set_nonblocking(true) {
        //     error!("Failed to set client in non-blocking mode: {err}");
        //     continue;
        // }

        let process = Arc::clone(&process);
        std::thread::spawn(move || serve_client(client, process));
    }

    unreachable!()
}

fn serve_client<A: DeserializeOwned, B: Serialize>(
    mut client: UnixStream,
    process: Arc<impl Fn(A) -> Result<B, String>>,
) -> ! {
    loop {
        let mut message = String::new();

        if let Err(err) = BufReader::new(&client).read_line(&mut message) {
            error!(
                "Failed to read message from the client (waiting before retrying): {:?}",
                err
            );
            std::thread::sleep(Duration::from_secs(5));
        }

        if message.is_empty() {
            std::thread::sleep(Duration::from_millis(100));
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

        let mut res = match serde_json::to_string(&res) {
            Ok(res) => res,
            Err(err) => {
                error!("Failed to stringify response for client: {err}");
                short_sleep();
                continue;
            }
        };

        // Message separator
        res.push('\n');

        if let Err(err) = client.write_all(res.as_bytes()) {
            error!("Failed to transmit response to client: {err}");
            short_sleep();
            continue;
        }

        if let Err(err) = client.flush() {
            error!("Failed to flush the client's stream: {err}");
            short_sleep();
            continue;
        }
    }
}

fn short_sleep() {
    std::thread::sleep(Duration::from_millis(100))
}
