use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    os::unix::net::UnixListener,
    sync::{Arc, RwLock},
    time::Duration,
};

use anyhow::{bail, Context, Result};
use daemonize_me::Daemon;

use crate::{
    daemon::{
        is_daemon_running,
        service::{daemon::process, State},
    },
    datetime::get_now,
    engine::start_engine,
    error, error_anyhow, info,
    ipc::serve_on_socket,
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

    if let Err(err) = daemon_core(paths, args) {
        error!("Daemon exited with an error: {:?}", err);
        std::process::exit(1);
    }

    #[allow(unreachable_code)]
    {
        unreachable!()
    }
}

fn daemon_core(paths: &Paths, args: &DaemonStartArgs) -> Result<()> {
    info!("Successfully started the daemon on {}", get_now());
    info!("Setting up the socket...");

    let socket_path = &paths.daemon_paths().socket_file();

    if is_daemon_running(socket_path)? {
        bail!("Daemon is already running!");
    }

    if socket_path.exists() {
        fs::remove_file(socket_path).context("Failed to remove the existing socket file")?;
    }

    let socket = UnixListener::bind(&socket_path)
        .context("Failed to create socket with the provided path")?;

    info!("Launching a separate thread for the socket listener...");

    let state = Arc::new(RwLock::new(State::new()));
    let state_server = Arc::clone(&state);

    std::thread::spawn(|| serve_on_socket(socket, process, state_server));

    daemon_core_loop(paths, args, state)
}

fn daemon_core_loop(paths: &Paths, args: &DaemonStartArgs, state: Arc<RwLock<State>>) -> ! {
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

        if state.read().unwrap().must_reload_tasks {
            state.write().unwrap().must_reload_tasks = false;
        }

        start_engine(paths, &tasks, &args.engine_args, || {
            state.read().unwrap().must_reload_tasks
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
