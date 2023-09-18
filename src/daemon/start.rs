use std::{
    fs::{self, OpenOptions},
    io::ErrorKind,
    os::unix::net::UnixListener,
    path::{Path, PathBuf},
    sync::{atomic::Ordering, Arc, Mutex, RwLock},
};

use anyhow::{bail, Context, Result};
use daemonize_me::Daemon;
use once_cell::sync::Lazy;

use crate::{
    daemon::{
        is_daemon_running,
        service::{daemon::process, RunningTask, State},
        DaemonClient, DaemonStartArgs,
    },
    datetime::get_now,
    debug,
    engine::start_engine,
    error, error_anyhow, info,
    ipc::serve_on_socket,
    logging::PRINT_MESSAGES_DATETIME,
    paths::Paths,
    save::read_tasks,
    sleep::sleep_ms,
    success,
    task::Task,
};

static SOCKET_FILE_PATH: Lazy<Mutex<Option<PathBuf>>> = Lazy::new(|| Mutex::new(None));

pub fn start_daemon(paths: &Paths, args: &DaemonStartArgs) -> Result<()> {
    if !paths.daemon_dir.exists() {
        fs::create_dir(&paths.daemon_dir)
            .context("Failed to create the daemon's data directory")?;
    }

    if is_daemon_running(&paths.daemon_socket_file)? {
        if args.ignore_started {
            return Ok(());
        }

        bail!("Daemon is already running.");
    }

    let socket = create_socket(&paths.daemon_socket_file)?;

    *SOCKET_FILE_PATH.lock().unwrap() = Some(paths.daemon_socket_file.clone());

    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&paths.daemon_log_file)
        .context("Failed to open the daemon's log file")?;

    Daemon::new()
        .stdout(log_file.try_clone().unwrap())
        .stderr(log_file)
        .setup_post_fork_parent_hook(fork_exit)
        .start()
        .context("Failed to start the daemon")?;

    PRINT_MESSAGES_DATETIME.store(true, Ordering::SeqCst);

    if let Err(err) = daemon_core(paths, args, socket) {
        error!("Daemon exited with an error: {:?}", err);
        std::process::exit(1);
    }

    // We should never reach this part
    unreachable!()
}

fn create_socket(socket_path: &Path) -> Result<UnixListener> {
    match UnixListener::bind(socket_path) {
        Ok(socket) => Ok(socket),
        Err(err) => match err.kind() {
            ErrorKind::AddrInUse => {
                debug!("Socket file exists but daemon is not running, restarting...");

                if let Err(err) = fs::remove_file(socket_path) {
                    match err.kind() {
                        // Sometimes the file will vanish just after the existence check, so we ignore "not found" errors
                        ErrorKind::NotFound => {},
                        // Handle other errors
                        _ => bail!("Failed to remove socket file: {err:?}"),
                    }
                }

                create_socket(socket_path)
            }
            _ => bail!("Failed to connect socket: {err}"),
        },
    }
}

fn daemon_core(paths: &Paths, args: &DaemonStartArgs, socket: UnixListener) -> Result<()> {
    info!("Successfully started the daemon on {}", get_now());
    info!("Launching a separate thread for the socket listener...");

    let state = Arc::new(RwLock::new(State::new()));
    let state_server = Arc::clone(&state);

    std::thread::spawn(|| serve_on_socket(socket, process, state_server));

    daemon_core_loop(paths, args, state)
}

fn daemon_core_loop(paths: &Paths, args: &DaemonStartArgs, state: Arc<RwLock<State>>) -> ! {
    info!("Starting the engine...");

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

                sleep_ms(100);
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
                sleep_ms(5000);
                continue;
            }
        };

        if state.read().unwrap().must_reload_tasks {
            state.write().unwrap().must_reload_tasks = false;
        }

        let state_for_marker = Arc::clone(&state);

        let interface = move |task: &Task, running| {
            let running_tasks = &mut state_for_marker.write().unwrap().running_tasks;

            if running {
                running_tasks.insert(
                    task.id,
                    RunningTask {
                        task: task.clone(),
                        started: get_now(),
                    },
                );
            } else {
                running_tasks.remove(&task.id).unwrap();
            }
        };

        start_engine(paths, &tasks, &args.engine_args, interface, |scheduled| {
            if state.read().unwrap().scheduled_request == Some(None) {
                let mut state = state.write().unwrap();

                let scheduled = scheduled
                    .read()
                    .unwrap()
                    .iter()
                    .map(|(a, b)| {
                        (
                            tasks.values().find(|task| task.id == *a).unwrap().clone(),
                            *b,
                        )
                    })
                    .collect();

                state.scheduled_request = Some(Some(scheduled));

                drop(state);
            }

            let state = state.read().unwrap();
            state.must_reload_tasks || state.exit
        });
    }

    #[allow(unreachable_code)]
    {
        unreachable!()
    }
}

fn fork_exit(_parent_pid: i32, _child_pid: i32) -> ! {
    let socket_path = SOCKET_FILE_PATH.lock().unwrap().as_ref().unwrap().clone();

    while !is_daemon_running(&socket_path).unwrap() {
        sleep_ms(50);
    }

    let mut client = DaemonClient::connect(&socket_path).unwrap();
    let daemon_pid = client.hello().unwrap();

    success!("Successfully started Crony daemon!");
    debug!("Daemon PID: {daemon_pid}");

    std::process::exit(0);
}
