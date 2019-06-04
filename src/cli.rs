// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use log::debug;
use structopt::StructOpt;

use crate::subcommands::keys::key_commander;
// use crate::subcommands::mutable_data::MutableDataSubCommands;
use crate::subcommands::wallet::wallet_commander;
use crate::subcommands::SubCommands;
use safe_cli::Safe;

#[derive(StructOpt, Debug)]
/// Interact with the SAFE Network
#[structopt(raw(global_settings = "&[structopt::clap::AppSettings::ColoredHelp]"))]
struct CmdArgs {
    /// subcommands
    #[structopt(subcommand)]
    cmd: SubCommands,
    /// The account's Root Container address
    #[structopt(long = "root", raw(global = "true"))]
    root: bool,
    /// Output data serlialisation
    #[structopt(short = "o", long = "output", raw(global = "true"))]
    output: Option<String>,
    /// Print human readable responses. (Alias to --output human-readable.)
    #[structopt(short = "hr", long = "human-readable", raw(global = "true"))]
    human: bool,
    /// Increase output verbosity. (More logs!)
    #[structopt(short = "v", long = "verbose", raw(global = "true"))]
    verbose: bool,
    /// Enable to query the output via SPARQL eg.
    #[structopt(short = "q", long = "query", raw(global = "true"))]
    query: Option<String>,
    /// Dry run of command. No data will be written. No coins spent.
    #[structopt(long = "dry-run", raw(global = "true"))]
    dry: bool,
}

pub fn run() -> Result<(), String> {
    // Let's first get all the arguments passed in
    let args = CmdArgs::from_args();

    let mut safe = Safe::new();

    debug!("Processing command: {:?}", args);

    match args.cmd {
        SubCommands::Keys { cmd } => key_commander(cmd, &mut safe),
        SubCommands::Wallet { cmd } => wallet_commander(cmd, &mut safe),
        _ => return Err("Command not supported yet".to_string()),
    }
}
