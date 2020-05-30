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
mod operations;
mod requests;
mod shared;
mod update;

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

use operations::{restart_authd, start_authd, stop_authd};

#[derive(StructOpt, Debug)]
/// SAFE Authenticator daemon subcommands
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
        /// Path where to store authd log files (default ~/.safe/authd/logs/)
        #[structopt(long = "log-dir")]
        log_dir: Option<PathBuf>,
        /// Run in foreground instead of daemon mode
        #[structopt(long = "fg")]
        fg: bool,
    },
    /// Stop a running safe-authd
    #[structopt(name = "stop")]
    Stop {
        /// Path where to store authd log files (default ~/.safe/authd/logs/)
        #[structopt(long)]
        log_dir: Option<PathBuf>,
    },
    /// Restart a running safe-authd
    #[structopt(name = "restart")]
    Restart {
        /// Address to listen on
        #[structopt(long = "listen", default_value = "https://localhost:33000")]
        listen: String,
        /// Path where to store authd log files (default ~/.safe/authd/logs/)
        #[structopt(long = "log-dir")]
        log_dir: Option<PathBuf>,
        /// Run in foreground instead of daemon mode
        #[structopt(long = "fg")]
        fg: bool,
    },
    /// Update the application to the latest available version
    #[structopt(name = "update")]
    Update {},
}

#[tokio::main]
async fn main() {
    setup_panic!();

    // Let's first get all the arguments passed in
    let opt = CmdArgs::from_args();
    debug!("Running authd with options: {:?}", opt);

    if let Err(err) = process_command(opt).await {
        error!("safe-authd error: {}", err);
        process::exit(err.error_code());
    }
}

async fn process_command(opt: CmdArgs) -> Result<()> {
    match opt {
        CmdArgs::Update {} => {
            // We run this command in a separate thread to overcome a conflict with
            // the self_update crate as it seems to be creating its own runtime.
            let handler = std::thread::spawn(|| {
                update_commander()
                    .map_err(|err| Error::GeneralError(format!("Error performing update: {}", err)))
            });
            handler.join().map_err(|err| {
                Error::GeneralError(format!("Failed to run self update: {:?}", err))
            })?
        }
        CmdArgs::Start {
            listen,
            log_dir,
            fg,
            ..
        } => start_authd(&listen, log_dir, fg).await,
        CmdArgs::Stop { log_dir } => stop_authd(log_dir),
        CmdArgs::Restart {
            listen,
            log_dir,
            fg,
        } => restart_authd(&listen, log_dir, fg).await,
    }
}
