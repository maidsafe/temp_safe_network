// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::authd::{run as authd_run, ErrorExt};
use failure::Error;
use std::ffi::OsString;
use windows_service::service_dispatcher;

define_windows_service!(ffi_authd_service, authd_service);

pub fn start_authd(_listen: &str) -> Result<(), Error> {
    // Register generated `ffi_authd_service` with the system and start the service, blocking
    // this thread until the service is stopped.
    println!("Starting SAFE Authenticator daemon (safe-authd)...");
    service_dispatcher::start("authd", ffi_authd_service)
        .map_err(|err| format_err!("Failed to dispatch service: {}", err))?;

    Ok(())
}

pub fn stop_authd() -> Result<(), Error> {
    println!("Stopping SAFE Authenticator daemon (safe-authd)...");

    // TODO: implementation for stopping authd service

    Ok(())
}

pub fn restart_authd(listen: &str) -> Result<(), Error> {
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

    // FIXME: receive endpoint listen address from arguments
    match authd_run("https://localhost:33000") {
        Ok(()) => {}
        Err(err) => println!("{}", err.pretty()),
    };
}
