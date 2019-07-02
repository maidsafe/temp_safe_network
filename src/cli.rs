// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use log::debug;
use structopt::StructOpt;

use crate::subcommands::auth::{auth_commander, auth_connect};
use crate::subcommands::cat::cat_command;
use crate::subcommands::files::files_commander;
use crate::subcommands::keys::key_commander;
use crate::subcommands::wallet::wallet_commander;
use crate::subcommands::{OutputFmt, SubCommands};
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
    /// Output data serlialisation. Currently only supported 'json'
    #[structopt(short = "o", long = "output", raw(global = "true"))]
    output_fmt: Option<String>,
    /// Sets JSON as output serialisation format (alias of '--output json')
    #[structopt(long = "json", raw(global = "true"))]
    output_json: bool,
    /// Increase output verbosity. (More logs!)
    #[structopt(short = "v", long = "verbose", raw(global = "true"))]
    verbose: bool,
    /// Enable to query the output via SPARQL eg.
    #[structopt(short = "q", long = "query", raw(global = "true"))]
    query: Option<String>,
    /// Dry run of command. No data will be written. No coins spent.
    #[structopt(long = "dry-run", raw(global = "true"))]
    dry: bool,
    /// Base encoding to be used for XOR-URLs generated. Currently supported: base32z (default) and base32
    #[structopt(long = "xorurl", raw(global = "true"))]
    xorurl_base: Option<String>,
}

pub fn run() -> Result<(), String> {
    // Let's first get all the arguments passed in
    let args = CmdArgs::from_args();

    let mut safe = Safe::new(args.xorurl_base.clone().unwrap_or_else(|| "".to_string()));

    let output_fmt = if args.output_json {
        OutputFmt::Json
    } else {
        let fmt = args.output_fmt.clone().unwrap_or_else(|| "".to_string());
        match fmt.as_ref() {
            "json" => OutputFmt::Json,
            "" => OutputFmt::Pretty,
            other => {
                return Err(format!(
                    "Output serialisation format '{}' not supported",
                    other
                ))
            }
        }
    };

    debug!("Processing command: {:?}", args);

    match args.cmd {
        SubCommands::Auth { cmd } => auth_commander(cmd, &mut safe),
        SubCommands::Cat { location, version } => {
            cat_command(location, version, output_fmt, &mut safe)
        }
        SubCommands::Keypair {} => {
            let key_pair = safe.keypair()?;
            if OutputFmt::Pretty == output_fmt {
                println!("Key pair generated:");
            }
            println!("pk={}", key_pair.pk);
            println!("sk={}", key_pair.sk);
            Ok(())
        }
        _ => {
            // We treat SubCommands::Auth separatelly since we need to connect before
            // handling any command but auth
            auth_connect(&mut safe)?;
            match args.cmd {
                SubCommands::Keys { cmd } => key_commander(cmd, output_fmt, &mut safe),
                SubCommands::Wallet { cmd } => wallet_commander(cmd, output_fmt, &mut safe),
                SubCommands::Files { cmd } => files_commander(cmd, output_fmt, &mut safe),
                _ => Err("Command not supported yet".to_string()),
            }
        }
    }
}
