// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::authd::run as authd_run;
use daemonize::Daemonize;
use failure::{Error, Fail};
use log::debug;
use std::env::temp_dir;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use std::{fmt, str};

const SAFE_AUTHD_PID_FILE: &str = "safe-authd.pid";
const SAFE_AUTHD_STDOUT_FILE: &str = "safe-authd.out";
const SAFE_AUTHD_STDERR_FILE: &str = "safe-authd.err";

pub struct PrettyErr<'a>(&'a dyn Fail);
impl<'a> fmt::Display for PrettyErr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)?;
        let mut x: &dyn Fail = self.0;
        while let Some(cause) = x.cause() {
            f.write_str(": ")?;
            fmt::Display::fmt(&cause, f)?;
            x = cause;
        }
        Ok(())
    }
}

pub trait ErrorExt {
    fn pretty(&self) -> PrettyErr<'_>;
}

impl ErrorExt for Error {
    fn pretty(&self) -> PrettyErr<'_> {
        PrettyErr(self.as_fail())
    }
}

pub fn get_certificate_base_path() -> Result<String, Error> {
    match directories::ProjectDirs::from("net", "maidsafe", "authd") {
        Some(dirs) => Ok(dirs.data_local_dir().display().to_string()),
        None => Err(format_err!(
            "Failed to obtain local project directory where to write certificate from"
        )),
    }
}

pub fn install_authd() -> Result<(), Error> {
    Err(format_err!("This command is only supported on Windows. You don't need to run this command in other platforms before starting safe-authd"))
}

pub fn uninstall_authd() -> Result<(), Error> {
    Err(format_err!("This command is only supported on Windows"))
}

pub fn start_authd(listen: &str) -> Result<(), Error> {
    let mut stout_file: PathBuf = temp_dir();
    stout_file.push(SAFE_AUTHD_STDOUT_FILE);
    let mut stderr_file: PathBuf = temp_dir();
    stderr_file.push(SAFE_AUTHD_STDERR_FILE);
    let mut pid_file: PathBuf = temp_dir();
    pid_file.push(SAFE_AUTHD_PID_FILE);
    let stdout = File::create(stout_file)
        .map_err(|err| format_err!("Failed to open/create file for stdout: {}", err))?;
    let stderr = File::create(stderr_file)
        .map_err(|err| format_err!("Failed to open/create file for stderr: {}", err))?;

    debug!("PID file to be created at: {:?}", &pid_file);

    let daemonize = Daemonize::new()
        .pid_file(pid_file) // Every method except `new` and `start`
        //.chown_pid_file(true)      // is optional, see `Daemonize` documentation
        .working_directory(temp_dir()) // for default behaviour.
        //.user("nobody")
        //.group("daemon") // Group name
        //.group(2)        // or group id.
        // .umask(0o777) // Set umask, `0o027` by default.
        .stdout(stdout) // Redirect stdout to `/tmp/safe-authd.out`.
        .stderr(stderr) // Redirect stderr to `/tmp/safe-authd.err`.
        .privileged_action(|| "Executed before drop privileges");

    println!("Starting SAFE Authenticator daemon (safe-authd)...");
    match daemonize.start() {
        Ok(_) => {
            println!("Initialising SAFE Authenticator services...");
            authd_run(listen)?;
        }
        Err(err) => eprintln!("Failed to start safe-authd daemon: {}", err),
    }

    Ok(())
}

pub fn stop_authd() -> Result<(), Error> {
    let mut pid_file: PathBuf = temp_dir();
    pid_file.push(SAFE_AUTHD_PID_FILE);

    debug!("PID should be: {:?}", &pid_file);
    println!("Stopping SAFE Authenticator daemon (safe-authd)...");
    let mut file = File::open(&pid_file)?;
    let mut pid = String::new();
    file.read_to_string(&mut pid)?;
    let output = Command::new("kill").arg("-9").arg(&pid).output()?;

    if output.status.success() {
        io::stdout().write_all(&output.stdout)?;
        println!("Success, safe-authd stopped!");
        Ok(())
    } else {
        io::stdout().write_all(&output.stderr)?;
        bail!("Failed to stop safe-authd daemon");
    }
}

pub fn restart_authd(listen: &str) -> Result<(), Error> {
    stop_authd()?;
    start_authd(listen)?;
    println!("Success, safe-authd restarted!");
    Ok(())
}
