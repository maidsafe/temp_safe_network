// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

mod authd;
mod errors;
mod notifs;
mod requests;
mod shared;
mod update;

#[cfg(not(target_os = "windows"))]
mod operations;
#[cfg(target_os = "windows")]
mod operations_win;

use env_logger;
use errors::{Error, Result};
use log::debug;
use log::error;
use std::path::PathBuf;
use std::process;
use structopt::{self, StructOpt};
use update::update_commander;

#[macro_use]
extern crate human_panic;

#[macro_use]
extern crate self_update;

#[cfg(not(target_os = "windows"))]
use operations::{
    install_authd, restart_authd, start_authd, start_authd_from_sc, stop_authd, uninstall_authd,
};
#[cfg(target_os = "windows")]
use operations_win::{
    install_authd, restart_authd, start_authd, start_authd_from_sc, stop_authd, uninstall_authd,
};

#[derive(StructOpt, Debug)]
/// SAFE Authenticator daemon subcommands
#[structopt(raw(global_settings = "&[structopt::clap::AppSettings::ColoredHelp]"))]
enum CmdArgs {
    /// Install safe-authd as a service. Only for Windows platforms
    #[structopt(name = "install")]
    Install {},
    /// Uninstall safe-authd service. Only for Windows platforms
    #[structopt(name = "uninstall")]
    Uninstall {},
    /// Start the safe-authd daemon
    #[structopt(name = "start")]
    Start {
        /// File to log TLS keys to for debugging
        #[structopt(long = "keylog")]
        keylog: bool,
        /// TLS private key in PEM format
        #[structopt(parse(from_os_str), short = "k", long = "key", requires = "cert")]
        key: Option<PathBuf>,
        /// TLS certificate in PEM format
        #[structopt(parse(from_os_str), short = "c", long = "cert", requires = "key")]
        cert: Option<PathBuf>,
        /// Enable stateless retries
        #[structopt(long = "stateless-retry")]
        stateless_retry: bool,
        /// Address to listen on
        #[structopt(long = "listen", default_value = "https://localhost:33000")]
        listen: String,
    },
    /// To be invoked by Windows Service Control Manager to start the authd service. Only for Windows platforms
    #[structopt(name = "sc-start")]
    ScStart {},
    /// Stop a running safe-authd
    #[structopt(name = "stop")]
    Stop {},
    /// Restart a running safe-authd
    #[structopt(name = "restart")]
    Restart {
        /// Address to listen on
        #[structopt(long = "listen", default_value = "https://localhost:33000")]
        listen: String,
    },
    /// Update the application to the latest available version
    #[structopt(name = "update")]
    Update {},
}

fn main() {
    setup_panic!();
    env_logger::init();

    // Let's first get all the arguments passed in
    let opt = CmdArgs::from_args();
    debug!("Running authd with options: {:?}", opt);

    if let Err(err) = process_command(opt) {
        error!("safe-authd error: {}", err);
        process::exit(err.error_code());
    }
}

fn process_command(opt: CmdArgs) -> Result<()> {
    match opt {
        CmdArgs::Update {} => update_commander()
            .map_err(|err| Error::GeneralError(format!("Error performing update: {}", err))),
        CmdArgs::Install {} => install_authd(),
        CmdArgs::Uninstall {} => uninstall_authd(),
        CmdArgs::Start { listen, .. } => start_authd(&listen),
        CmdArgs::ScStart {} => start_authd_from_sc(),
        CmdArgs::Stop {} => stop_authd(),
        CmdArgs::Restart { listen } => restart_authd(&listen),
    }
}
