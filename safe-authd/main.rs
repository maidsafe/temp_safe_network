// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod authd;
mod notifs;
mod quic_client;
mod requests;
mod shared;
mod update;

use env_logger;
use log::debug;
use log::error;
use std::net::ToSocketAddrs;
use std::path::PathBuf;
use std::process;
use structopt::{self, StructOpt};
use update::update_commander;
use url::Url;

#[macro_use]
extern crate human_panic;

#[macro_use]
extern crate failure;
#[macro_use]
extern crate slog;

#[cfg(not(feature = "mock-network"))]
#[macro_use]
extern crate self_update;

use authd::{restart_authd, start_authd, stop_authd, ErrorExt};

#[derive(StructOpt, Debug)]
/// SAFE Authenticator daemon
#[structopt(raw(global_settings = "&[structopt::clap::AppSettings::ColoredHelp]"))]
enum CmdArgs {
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

    if let Err(e) = process_command(opt) {
        error!("safe-authd error: {}", e);
        process::exit(1);
    }
}

fn process_command(opt: CmdArgs) -> Result<(), String> {
    match opt {
        CmdArgs::Update {} => {
            update_commander().map_err(|err| format!("Error performing update: {}", err))
        }
        CmdArgs::Start { listen, .. } => {
            let url = Url::parse(&listen).map_err(|_| "Invalid end point address".to_string())?;
            let endpoint = url
                .to_socket_addrs()
                .map_err(|_| "Invalid end point address".to_string())?
                .next()
                .ok_or_else(|| "The end point is an invalid address".to_string())?;
            if let Err(e) = start_authd(endpoint) {
                Err(format!("{}", e.pretty()))
            } else {
                Ok(())
            }
        }
        CmdArgs::Stop {} => {
            if let Err(e) = stop_authd() {
                Err(format!("{}", e.pretty()))
            } else {
                Ok(())
            }
        }
        CmdArgs::Restart { listen } => {
            let url = Url::parse(&listen).map_err(|_| "Invalid end point address".to_string())?;
            let endpoint = url
                .to_socket_addrs()
                .map_err(|_| "Invalid end point address".to_string())?
                .next()
                .ok_or_else(|| "The end point is an invalid address".to_string())?;
            if let Err(e) = restart_authd(endpoint) {
                Err(format!("{}", e.pretty()))
            } else {
                Ok(())
            }
        }
    }
}
