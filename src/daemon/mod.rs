mod cmd;

pub use cmd::DaemonArgs;

use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    time::Duration,
};

use anyhow::{Context, Result};
use daemonize_me::Daemon;

use crate::{
    datetime::{get_now, human_datetime},
    engine::start_engine,
    error_anyhow, info,
    paths::Paths,
    save::read_tasks,
    success,
    task::Tasks,
    warn,
};

pub fn start_daemon(paths: &Paths, args: &DaemonArgs) -> Result<()> {
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
        .pid_file(d_paths.pid_file(), Some(false))
        .stdout(stdout_file)
        .stderr(stderr_file)
        .setup_post_fork_parent_hook(fork_exit)
        .start()
        .context("Failed to start the daemon")?;

    info!("Successfully started the daemon on {}", get_now());
    info!("Starting the engine...");

    match start_engine(paths, &args.engine_args) {
        Ok(()) => warn!("Engine's loop broke, exiting the daemon."),
        Err(err) => error_anyhow!(err.context("Engine returned an error")),
    }

    std::process::exit(0);
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

pub(super) fn treat_reload_request(paths: &Paths) -> Result<Option<Tasks>> {
    if paths.reload_request_file.is_file() {
        fs::remove_file(&paths.reload_request_file)
            .context("Failed to remove the reload request file")?;
        Ok(Some(read_tasks(paths)?))
    } else {
        Ok(None)
    }
}

pub fn ask_daemon_reload(paths: &Paths) -> Result<()> {
    if paths.reload_request_file.is_file() {
        warn!("A reload request is already pending!");
        return Ok(());
    }

    fs::write(&paths.reload_request_file, human_datetime(get_now()))
        .context("Failed to write the reload request file")?;

    info!("Reload request created, waiting for the daemon to treat it...");

    let mut treated = false;

    // Short timeout as the daemon is supposed to detect changes in ~1s max.
    for _ in 0..15 {
        std::thread::sleep(Duration::from_millis(100));

        if !paths.reload_request_file.is_file() {
            treated = true;
            break;
        }
    }

    if !treated {
        warn!("Timeout reached, is the daemon started?");
    } else {
        success!("Daemon successfully treated the reload request.");
    }

    Ok(())
}
