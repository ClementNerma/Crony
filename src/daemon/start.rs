use std::{
    fs::{self, OpenOptions},
    os::unix::net::UnixListener,
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
    time::Duration,
};

use anyhow::{bail, Context, Result};
use daemonize_me::Daemon;
use once_cell::sync::Lazy;

use crate::{
    daemon::{
        is_daemon_running,
        service::{daemon::process, RunningTask, State},
        Client, DaemonClient, DaemonStartArgs,
    },
    datetime::get_now,
    engine::start_engine,
    error, error_anyhow, info,
    ipc::serve_on_socket,
    paths::Paths,
    save::read_tasks,
    success, RunningTasksInterface,
};

static SOCKET_FILE_PATH: Lazy<Mutex<Option<PathBuf>>> = Lazy::new(|| Mutex::new(None));

pub fn start_daemon(paths: &Paths, args: &DaemonStartArgs) -> Result<()> {
    if !paths.daemon_dir.exists() {
        fs::create_dir(&paths.daemon_dir)
            .context("Failed to create the daemon's data directory")?;
    }

    if is_daemon_running(&paths.daemon_socket_file)? {
        bail!("Daemon is already running.");
    }

    if paths.daemon_socket_file.exists() {
        fs::remove_file(&paths.daemon_socket_file)
            .context("Failed to remove the existing socket file")?;
    }

    let stdout_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&paths.daemon_stdout_logfile)
        .context("Failed to open the daemon's STDOUT log file")?;

    let stderr_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&paths.daemon_stderr_logfile)
        .context("Failed to open the daemon's STDOUT log file")?;

    *SOCKET_FILE_PATH.lock().unwrap() = Some(paths.daemon_socket_file.clone());

    Daemon::new()
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

    let socket = UnixListener::bind(&paths.daemon_socket_file)
        .context("Failed to create socket with the provided path")?;

    info!("Launching a separate thread for the socket listener...");

    let state = Arc::new(RwLock::new(State::new()));
    let state_server = Arc::clone(&state);

    std::thread::spawn(|| serve_on_socket(socket, process, state_server));

    daemon_core_loop(paths, args, state)
}

fn daemon_core_loop(paths: &Paths, args: &DaemonStartArgs, state: Arc<RwLock<State>>) -> ! {
    info!("Starting the engine...");

    let state_for_interface_1 = Arc::clone(&state);
    let state_for_interface_2 = Arc::clone(&state);
    let state_for_interface_3 = Arc::clone(&state);

    let interface = Arc::new(RunningTasksInterface {
        is_task_running: Box::new(move |task_id| {
            state_for_interface_1
                .read()
                .unwrap()
                .running_tasks
                .contains_key(&task_id)
        }),

        mark_task_as_done: Box::new(move |task_id| {
            state_for_interface_2
                .write()
                .unwrap()
                .running_tasks
                .remove(&task_id)
                .unwrap();
        }),

        mark_task_as_running: Box::new(move |task| {
            state_for_interface_3.write().unwrap().running_tasks.insert(
                task.id,
                RunningTask {
                    task: task.clone(),
                    started: get_now(),
                },
            );
        }),
    });

    loop {
        if state.read().unwrap().exit {
            info!("Exiting safely as requested...");

            state.write().unwrap().exit = false;

            let mut last_running = 0;

            loop {
                let len = state.read().unwrap().running_tasks.len();

                if len == 0 {
                    break;
                }

                if len != last_running {
                    info!("[Exiting] Waiting for {} tasks to complete...", len);
                    last_running = len;
                }

                std::thread::sleep(Duration::from_millis(100));
            }

            info!("[Exiting] Now exiting.");

            if let Err(err) = fs::remove_file(&paths.daemon_socket_file) {
                error!("Failed to remove the socket file, this might cause problem during the next start: {err}");
            }

            std::process::exit(0);
        }

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

        start_engine(
            paths,
            &tasks,
            &args.engine_args,
            Arc::clone(&interface),
            || {
                let state = state.read().unwrap();
                state.must_reload_tasks || state.exit
            },
        );
    }

    #[allow(unreachable_code)]
    {
        unreachable!()
    }
}

fn fork_exit(_parent_pid: i32, child_pid: i32) -> ! {
    info!("Started the daemon, waiting for response...");

    let guard = SOCKET_FILE_PATH.lock().unwrap();
    let socket_path = guard.as_ref().unwrap();

    while !socket_path.exists() {
        std::thread::sleep(Duration::from_millis(50));
    }

    let mut client = DaemonClient::connect(socket_path).unwrap();
    client.hello().unwrap();

    success!(
        "Successfully setup daemon with PID {}!",
        child_pid.to_string().bright_yellow()
    );

    std::process::exit(0);
}
