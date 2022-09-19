use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    sync::{Arc, RwLock},
    time::Duration,
};

use anyhow::{Context, Result};
use daemonize_me::Daemon;

use crate::{
    daemon::server::{daemon::process, State},
    datetime::get_now,
    engine::start_engine,
    error_anyhow, info,
    ipc::{create_socket, serve_on_socket},
    paths::Paths,
    save::read_tasks,
    success,
};

use super::DaemonStartArgs;

pub fn start_daemon(paths: &Paths, args: &DaemonStartArgs) -> Result<()> {
    let d_paths = paths.daemon_paths();

    if !d_paths.dir().exists() {
        fs::create_dir(d_paths.dir()).context("Failed to create the daemon's data directory")?;
    }

    let stdout_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(d_paths.stdout_log_file())
        .context("Failed to open the daemon's STDOUT log file")?;

    let stderr_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(d_paths.stderr_log_file())
        .context("Failed to open the daemon's STDOUT log file")?;

    Daemon::new()
        // .pid_file(d_paths.pid_file(), Some(false))
        .stdout(stdout_file)
        .stderr(stderr_file)
        .setup_post_fork_parent_hook(fork_exit)
        .start()
        .context("Failed to start the daemon")?;

    info!("Successfully started the daemon on {}", get_now());
    info!("Setting up the socket...");

    let socket = create_socket(&d_paths.socket_file()).unwrap();

    info!("Launching a separate thread for the socket listener...");
    let state = Arc::new(RwLock::new(State::new()));
    let state_server = Arc::clone(&state);

    std::thread::spawn(|| serve_on_socket(socket, process, state_server));

    info!("Starting the engine...");

    loop {
        let tasks = match read_tasks(paths) {
            Ok(tasks) => tasks,
            Err(err) => {
                error_anyhow!(
                    err.context("Failed to load tasks, waiting for 5 seconds before retrying...")
                );
                std::thread::sleep(Duration::from_secs(5));
                continue;
            }
        };

        start_engine(paths, &tasks, &args.engine_args, || {
            // TODO
            false
        });
    }

    #[allow(unreachable_code)]
    {
        unreachable!()
    }
}

fn fork_exit(_parent_pid: i32, child_pid: i32) -> ! {
    success!(
        "Successfully setup daemon with PID {}!",
        child_pid.to_string().bright_yellow()
    );

    io::stdout()
        .flush()
        .context("Failed to flush STDOUT")
        .unwrap();

    std::process::exit(0);
}
