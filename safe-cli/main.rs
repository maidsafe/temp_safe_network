// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

mod cli;
mod operations;
mod shell;
mod subcommands;

use cli::run;
use log::{debug, error};
use std::process;

#[macro_use]
extern crate prettytable;

#[macro_use]
extern crate human_panic;

#[macro_use]
extern crate self_update;

const APP_ID: &str = "net.maidsafe.cli";
#[allow(dead_code)]
const APP_NAME: &str = "SAFE CLI";
#[allow(dead_code)]
const APP_VENDOR: &str = "MaidSafe.net Ltd";
const PROJECT_DATA_DIR_QUALIFIER: &str = "net";
const PROJECT_DATA_DIR_ORGANISATION: &str = "MaidSafe";
const PROJECT_DATA_DIR_APPLICATION: &str = "safe-cli";

#[tokio::main]
async fn main() {
    setup_panic!();
    env_logger::init();
    debug!("Starting SAFE CLI...");

    if let Err(e) = run().await {
        error!("safe-cli error: {}", e);
        process::exit(1);
    }
}
