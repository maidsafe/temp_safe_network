// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

/*
use super::notifs::monitor_pending_auth_reqs;
use super::requests::process_request;
use super::shared::*;
use failure::{Error, Fail, ResultExt};
use futures::{Future, Stream};
use safe_api::SafeAuthenticator;
use slog::{Drain, Logger};
use std::collections::BTreeSet;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, Write};
use std::net::SocketAddr;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::{ascii, fmt, fs, str};
use tokio::runtime::current_thread::Runtime;
*/

use super::authd::run as authd_run;
use std::ffi::OsString;
use windows_service::service_dispatcher;

define_windows_service!(ffi_authd_service, authd_service);

//const SAFE_AUTHD_PID_FILE: &str = "/tmp/safe-authd.pid";
//const SAFE_AUTHD_STDOUT_FILE: &str = "/tmp/safe-authd.out";
//const SAFE_AUTHD_STDERR_FILE: &str = "/tmp/safe-authd.err";

pub fn start_authd(_listen: SocketAddr) -> Result<(), Error> {
    /*
        let stdout = File::create(SAFE_AUTHD_STDOUT_FILE)
            .map_err(|err| format_err!("Failed to open/create file for stdout: {}", err))?;
        let stderr = File::create(SAFE_AUTHD_STDERR_FILE)
            .map_err(|err| format_err!("Failed to open/create file for stderr: {}", err))?;
    */
    // Register generated `ffi_authd_service` with the system and start the service, blocking
    // this thread until the service is stopped.
    println!("Starting SAFE Authenticator daemon (safe-authd)...");
    service_dispatcher::start("authd", ffi_authd_service)
        .map_err(|err| format_err!("Failed to dispatch service: {}", err))?;

    Ok(())
}

pub fn stop_authd() -> Result<(), Error> {
    println!("Stopping SAFE Authenticator daemon (safe-authd)...");
    /*
        let mut file = File::open(SAFE_AUTHD_PID_FILE)?;
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
    */
}

pub fn restart_authd(listen: SocketAddr) -> Result<(), Error> {
    stop_authd()?;
    start_authd(listen)?;
    println!("Success, safe-authd restarted!");
    Ok(())
}

fn authd_service(arguments: Vec<OsString>) {
    // The entry point where execution will start on a background thread after a call to
    // `service_dispatcher::start` from `main`.
    println!(
        "Initialising SAFE Authenticator services...: {:?}",
        arguments
    );
    authd_run()
}
