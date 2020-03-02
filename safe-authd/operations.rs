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
use daemonize::{Daemonize, DaemonizeError};
use directories::BaseDirs;
use log::debug;
use std::{
    fs::{create_dir_all, File},
    io::prelude::*,
    path::PathBuf,
    process::Command,
    str, thread,
    time::Duration,
};

const SAFE_AUTHD_PID_FILE: &str = "safe-authd.pid";
const SAFE_AUTHD_STDOUT_FILE: &str = "safe-authd.out";
const SAFE_AUTHD_STDERR_FILE: &str = "safe-authd.err";

pub fn install_authd() -> Result<()> {
    Err(Error::GeneralError("This command is only supported on Windows. You don't need to run this command in other platforms before starting safe-authd".to_string()))
}

pub fn uninstall_authd() -> Result<()> {
    Err(Error::GeneralError(
        "This command is only supported on Windows".to_string(),
    ))
}

pub fn start_authd_from_sc() -> Result<()> {
    Err(Error::GeneralError(
        "This command is only supported on Windows".to_string(),
    ))
}

pub fn start_authd(listen: &str, log_dir: Option<PathBuf>, foreground: bool) -> Result<()> {
    let log_path = get_authd_log_path(log_dir)?;

    let mut stout_file_path = log_path.clone();
    stout_file_path.push(SAFE_AUTHD_STDOUT_FILE);
    let stdout = File::create(stout_file_path).map_err(|err| {
        Error::GeneralError(format!("Failed to open/create file for stdout: {}", err))
    })?;

    let mut stderr_file_path = log_path.clone();
    stderr_file_path.push(SAFE_AUTHD_STDERR_FILE);
    let stderr = File::create(stderr_file_path).map_err(|err| {
        Error::GeneralError(format!("Failed to open/create file for stderr: {}", err))
    })?;

    let mut pid_file_path = log_path.clone();
    pid_file_path.push(SAFE_AUTHD_PID_FILE);
    debug!("PID file to be created at: {:?}", &pid_file_path);

    if foreground {
        println!("Initialising SAFE Authenticator services...");
        authd_run(listen, None, None)
    } else {
        let daemonize = Daemonize::new()
            .pid_file(pid_file_path) // Every method except `new` and `start`
            //.chown_pid_file(true)      // is optional, see `Daemonize` documentation
            .working_directory(log_path) // for default behaviour.
            //.user("nobody")
            //.group("daemon") // Group name
            //.group(2)        // or group id.
            // .umask(0o777) // Set umask, `0o027` by default.
            .stdout(stdout) // Redirect stdout to `safe-authd.out` file.
            .stderr(stderr) // Redirect stderr to `safe-authd.err` file.
            .privileged_action(|| "Executed before drop privileges");

        println!("Starting SAFE Authenticator daemon (safe-authd)...");
        match daemonize.start() {
            Ok(_) => {
                println!("Initialising SAFE Authenticator services...");
                authd_run(listen, None, None)?;
                Ok(())
            }
            Err(err) => {
                let msg = format!("Failed to start safe-authd daemon: {:?}", err);
                if let DaemonizeError::LockPidfile(_pid) = err {
                    // A daemon has been already started keeping the lock on the PID file,
                    // although we don't know its status
                    Err(Error::AuthdAlreadyStarted(msg))
                } else {
                    Err(Error::GeneralError(msg))
                }
            }
        }
    }
}

pub fn stop_authd(log_dir: Option<PathBuf>) -> Result<()> {
    let mut pid_file_path: PathBuf = get_authd_log_path(log_dir)?;
    pid_file_path.push(SAFE_AUTHD_PID_FILE);

    debug!("PID should be: {:?}", &pid_file_path);
    println!("Stopping SAFE Authenticator daemon (safe-authd)...");
    let mut file = File::open(&pid_file_path).map_err(|err| {
        Error::GeneralError(format!(
            "Failed to open safe-authd daemon PID file ('{}') to stop daemon: {}",
            pid_file_path.display(),
            err
        ))
    })?;
    let mut pid = String::new();
    file.read_to_string(&mut pid)?;
    let output = Command::new("kill").arg("-9").arg(&pid).output()?;

    if output.status.success() {
        println!("Success, safe-authd (PID: {}) stopped!", pid);
        Ok(())
    } else {
        Err(Error::GeneralError(format!(
            "Failed to stop safe-authd daemon (PID: {}): {}",
            pid,
            String::from_utf8_lossy(&output.stderr)
        )))
    }
}

pub fn restart_authd(listen: &str, log_dir: Option<PathBuf>, foreground: bool) -> Result<()> {
    match stop_authd(log_dir.clone()) {
        Ok(()) => {
            // Let's give it a sec so it's properlly stopped
            thread::sleep(Duration::from_millis(1000));
        }
        Err(err) => println!("{}", err),
    }
    start_authd(listen, log_dir, foreground)?;
    println!("Success, safe-authd restarted!");
    Ok(())
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
