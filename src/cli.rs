// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

// use crate::cli_helpers::*;

// use log::{debug, warn};
// use std::env;
use structopt::StructOpt;

use crate::commands::subcommands::SubCommands;

#[derive(StructOpt, Debug)]
/// Interact with the SAFE Network
pub struct CmdArgs {
    /// The safe:// address of target data
    #[structopt(short = "t", long = "target")]
    target: String,
    /// The account's Root Container address
    #[structopt(short = "r", long = "root")]
    root: bool,
    /// subcommands
    #[structopt(subcommand)]
    cmd: Option<SubCommands>,
    /// Output data serlialisation
    #[structopt(short = "o", long = "output")]
    output: String,
    /// Print human readable responses. (Alias to --output human-readable.)
    #[structopt(short = "hr", long = "human-readable")]
    human: bool,
    /// Increase output verbosity. (More logs!)
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,
    /// Enable to query the output via SPARQL eg.
    #[structopt(short = "q", long = "query")]
    query: String,
    /// Dry run of command. No data will be written. No coins spent.
    #[structopt(long = "dry-run")]
    dry: bool,
}

pub fn run() -> Result<(), String> {
    // Let's first get all the arguments passed in
    let _args = CmdArgs::from_args();

    Ok(())
}
