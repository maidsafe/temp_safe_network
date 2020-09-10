// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{
    authd::run as authd_run,
    errors::{Error, Result},
};
use cluFlock::ExclusiveFlock;
use directories::BaseDirs;
use flexi_logger::{DeferredNow, Logger};
use log::{self, debug, info, Record};
use std::{
    fs::{create_dir_all, File, OpenOptions},
    io::{self, prelude::*},
    path::PathBuf,
    process::{self, Command, Stdio},
    str, thread,
    time::Duration,
};

const SAFE_AUTHD_PID_FILE: &str = "sn_authd.pid";
const DEFAULT_LOG_LEVEL: &str = "info";

pub async fn start_authd(listen: &str, log_dir: Option<PathBuf>, foreground: bool) -> Result<()> {
    if foreground {
        // Let's run it as a normal process in the foreground
        run_in_foreground(listen, log_dir).await
    } else {
        // Run it as a daemon, i.e. a detached process in the background
        launch_detached_process(listen, log_dir)
    }
}

pub fn stop_authd(log_dir: Option<PathBuf>) -> Result<()> {
    println!("Stopping SAFE Authenticator daemon (sn_authd)...");

    if cfg!(windows) {
        // Since in Windows we cannot read the locked PID file,
        // we kill authd by using the binary name
        let binary_file_name = "safe-authd.exe";
        let current_pid = process::id();
        let output = Command::new("taskkill")
            .args(&[
                "/F",
                "/IM",
                binary_file_name,
                "/FI",
                &format!("PID ne {}", current_pid),
            ])
            .output()?;

        if output.status.success() {
            io::stdout().write_all(&output.stdout).map_err(|err| {
                Error::GeneralError(format!("Failed to output success info: {}", err))
            })?;
            Ok(())
        } else {
            let msg = format!(
                "Error when attempting to stop authd: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            println!("{}", msg);
            Err(Error::GeneralError(msg))
        }
    } else {
        // For Linux and Mac we can read the locked PID file,
        // and then kill authd using its PID
        let mut pid_file_path: PathBuf = get_authd_log_path(log_dir)?;
        pid_file_path.push(SAFE_AUTHD_PID_FILE);

        debug!("Retrieving authd PID from: {:?}", &pid_file_path);
        let mut file = File::open(&pid_file_path).map_err(|err| {
            Error::GeneralError(format!(
                "Failed to open sn_authd daemon PID file ('{}') to stop daemon: {}",
                pid_file_path.display(),
                err
            ))
        })?;
        let mut pid = String::new();
        file.read_to_string(&mut pid)?;

        let output = Command::new("kill").arg("-9").arg(&pid).output()?;
        if output.status.success() {
            println!("Success, sn_authd (PID: {}) stopped!", pid);
        } else {
            println!("No running sn_authd process (with PID {}) was found", pid);
        }
        Ok(())
    }
}

pub async fn restart_authd(listen: &str, log_dir: Option<PathBuf>, foreground: bool) -> Result<()> {
    match stop_authd(log_dir.clone()) {
        Ok(()) => {
            // Let's give it a sec so it's properlly stopped
            thread::sleep(Duration::from_millis(1000));
        }
        Err(err) => println!("{}", err),
    }
    start_authd(listen, log_dir, foreground).await?;
    println!("Success, sn_authd restarted!");
    Ok(())
}

// Private functions

async fn run_in_foreground(listen: &str, log_dir: Option<PathBuf>) -> Result<()> {
    let log_path = get_authd_log_path(log_dir.clone())?;
    let authd_exec = std::env::current_exe()?;

    // Custom formatter for logs
    let do_format = move |writer: &mut dyn Write, clock: &mut DeferredNow, record: &Record| {
        write!(
            writer,
            "{} {} [{}:{}] {}",
            record.level(),
            clock.now().to_rfc3339(),
            record.file().unwrap_or_default(),
            record.line().unwrap_or_default(),
            record.args()
        )
    };

    // Depending on log_dir arg received we output logs to stdout or to a file
    let logger = Logger::with_env_or_str(DEFAULT_LOG_LEVEL)
        .format(do_format)
        .suppress_timestamp();
    if let Some(log_file_path) = log_dir {
        logger
            .log_to_file()
            .directory(log_file_path)
            .append()
            .start()
    } else {
        logger.start()
    }
    .map_err(|err| Error::GeneralError(format!("Error when initialising logger: {}", err)))?;

    info!(
        "Running authd instance from executable at \"{}\"",
        authd_exec.display()
    );

    let pid = process::id();
    info!("authd instance starting (PID: {})...", pid);
    let mut pid_file_path = log_path.clone();
    pid_file_path.push(SAFE_AUTHD_PID_FILE);
    debug!("PID file to be written at: {:?}", &pid_file_path);

    // Open/create PID file
    let pid_file = if pid_file_path.exists() {
        OpenOptions::new()
            .write(true)
            .truncate(false)
            .open(&pid_file_path)?
    } else {
        File::create(&pid_file_path)?
    };

    // Try to lock PID file
    match pid_file.try_lock() {
        Ok(mut pid_file) => {
            // We got the lock on the PID file, therefore write our current PID
            pid_file.set_len(0)?;
            write!(pid_file, "{}", pid).map_err(|err| {
                Error::GeneralError(format!(
                    "Failed to start sn_authd daemon ({}): {}",
                    authd_exec.display(),
                    err.to_string()
                ))
            })?;

            info!("Initialising SAFE Authenticator services...");
            authd_run(listen, None, None).await?;

            // Release PID file lock (this is done automatically anyways if process is killed)
            drop(pid_file);

            Ok(())
        }
        Err(err) => {
            // We cannot start the authd services since PID file lock coudln't be obtained
            let os_error_code = err.raw_os_error().unwrap_or_else(|| 0);
            debug!(
                "Failed to lock PID file with OS error code: {}",
                os_error_code
            );

            // Let's check if the error is due to authd already running
            let is_already_started: bool = if cfg!(target_os = "windows") {
                // On Windows: ERROR_LOCK_VIOLATION == 33
                os_error_code == 33
            } else if cfg!(target_os = "linux") {
                // On Linux: EWOULDBLOCK == EAGAIN == 11
                os_error_code == 11
            } else {
                // On Mac: 35
                os_error_code == 35
            };

            let res_err = if is_already_started {
                // A daemon has been already started keeping the lock on the PID file,
                // although we don't know its status
                Error::AuthdAlreadyStarted(format!(
                    "Failed to start sn_authd daemon ({})",
                    authd_exec.display(),
                ))
            } else {
                Error::GeneralError(format!(
                    "Unknown error when attempting get lock on PID file at {}: {:?}",
                    pid_file_path.display(),
                    err
                ))
            };

            Err(res_err)
        }
    }
}

fn launch_detached_process(listen: &str, log_dir: Option<PathBuf>) -> Result<()> {
    let log_path = get_authd_log_path(log_dir)?;
    let authd_exec = std::env::current_exe()?;

    println!("Starting SAFE Authenticator daemon (sn_authd)...");
    // We execute this same binary but requesting to run in the foreground,
    // and since we spawn it, it will be a detached process running in the background
    let args = [
        "start",
        "--fg",
        "--listen",
        listen,
        "--log-dir",
        &log_path.display().to_string(),
    ];

    debug!(
        "Running '{}' with args {:?} ...",
        authd_exec.display(),
        args
    );

    // Spwan the process
    let mut child = Command::new(&authd_exec)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // Let's give it a sec so it starts/fails
    thread::sleep(Duration::from_millis(1000));

    // If it failed to start already, we can get the error code from it,
    // otherwise we'll assume it started correctly
    if let Ok(Some(status)) = child.try_wait() {
        let exit_code = match status.code() {
            Some(code) => code,
            None => 1,
        };

        let error = Error::from_code(
            exit_code,
            format!(
                "Failed to start sn_authd daemon '{}' (exit code: {})",
                authd_exec.display(),
                exit_code
            ),
        );
        println!("{}", error);
        Err(error)
    } else {
        println!("sn_authd started (PID: {})", child.id());
        Ok(())
    }
}

fn get_authd_log_path(log_dir: Option<PathBuf>) -> Result<PathBuf> {
    match log_dir {
        Some(p) => Ok(p),
        None => {
            let base_dirs = BaseDirs::new().ok_or_else(|| {
                Error::GeneralError("Failed to obtain user's home path".to_string())
            })?;

            let mut path = PathBuf::from(base_dirs.home_dir());
            path.push(".safe");
            path.push("authd");
            path.push("logs");

            if !path.exists() {
                println!("Creating '{}' folder", path.display());
                create_dir_all(path.clone()).map_err(|err| {
                    Error::GeneralError(format!(
                        "Couldn't create target path to store authd log files: {}",
                        err
                    ))
                })?;
            }

            Ok(path)
        }
    }
}
