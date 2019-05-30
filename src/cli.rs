// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use log::{debug, warn};
use std::io::{self, stdin, stdout, Write};
use structopt::StructOpt;

use crate::subcommands::keys::KeysSubCommands;
use crate::subcommands::mutable_data::MutableDataSubCommands;
use crate::subcommands::SubCommands;
use safe_cli::{BlsKeyPair, Safe};

#[derive(StructOpt, Debug)]
/// Interact with the SAFE Network
#[structopt(raw(global_settings = "&[structopt::clap::AppSettings::ColoredHelp]"))]
struct CmdArgs {
    /// The safe:// address of target data
    #[structopt(short = "t", long = "target", raw(global = "true"))]
    target: Option<String>,
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
    /// subcommands
    #[structopt(subcommand)]
    cmd: SubCommands,
}

pub fn run() -> Result<(), String> {
    // Let's first get all the arguments passed in
    let args = CmdArgs::from_args();

    let mut safe = Safe::new();

    debug!("Processing command: {:?}", args);

    // Is it a keys command?
    if let SubCommands::Keys { cmd } = args.cmd {
        // Is it a create subcommand?
        match cmd {
            Some(KeysSubCommands::Create {
                anon,
                preload,
                pk,
                from,
                test_coins,
                ..
            }) => {
                let create_new_key = |safe: &mut Safe, from, preload: Option<String>, pk| {
                    if test_coins {
                        /*if cfg!(not(feature = "mock-network")) {
                            warn!("Ignoring \"--test-coins\" flag since it's only available for \"mock-network\" feature");
                            println!("Ignoring \"--test-coins\" flag since it's only available for \"mock-network\" feature");
                            safe.keys_create(from, preload, pk)
                        } else {*/
                        warn!("Note that the Key to be created will be preloaded with **test coins** rather than real coins");
                        println!("Note that the Key to be created will be preloaded with **test coins** rather than real coins");
                        let amount = preload.unwrap_or("0".to_string());
                        safe.keys_create_test_coins(amount, pk)
                    // }
                    } else {
                        safe.keys_create(from, preload, pk)
                    }
                };

                // '--from' is either a Wallet XOR-URL, a Key XOR-URL, or a pk
                let from_key_pair = match from {
                    Some(from_xorname) => {
                        // TODO: support Key XOR-URL and pk, we now support only Key XOR name
                        // Prompt the user for the secret key since 'from' is a Key and not a Wallet
                        let sk = prompt_user(
                            &format!(
                                "Enter secret key corresponding to public key at XOR name [{}]: ",
                                from_xorname
                            ),
                            "Invalid input",
                        );

                        let pk = safe.fetch_key_pk(&from_xorname, &sk);
                        Some(BlsKeyPair { pk, sk })
                    }
                    None => None,
                };

                // Want an anonymous Key?
                if anon {
                    let (xorname, key_pair) = create_new_key(&mut safe, from_key_pair, preload, pk);
                    println!("New Key created at XOR name: \"{}\"", xorname);
                    println!("This was not linked from any container.");
                    if let Some(pair) = key_pair {
                        println!("Key pair generated: pk=\"{}\", sk=\"{}\"", pair.pk, pair.sk);
                    }
                } else {
                    // TODO: create Key and add it to the provided --target Wallet
                }
            }
            Some(KeysSubCommands::Balance {}) => {
                let sk = String::from(
                    "391987fd429b4718a59b165b5799eaae2e56c697eb94670de8886f8fb7387058",
                ); // FIXME: get sk from args or account
                let target = get_target_location(args.target)?;
                let current_balance = safe.keys_balance_from_xorname(&target, &sk);
                println!("Key's current balance: {}", current_balance);
            }
            Some(KeysSubCommands::Add { .. }) => println!("keys add ...coming soon!"),
            None => return Err("Missing keys sub-command. Use --help for details.".to_string()),
        };
    } else if let SubCommands::MutableData { cmd } = args.cmd {
        match cmd {
            Some(MutableDataSubCommands::Create {
                name,
                permissions,
                tag,
                sequenced,
            }) => {
                let xor = safe.md_create(name, tag, permissions, sequenced);
                println!("{:?}", xor);
            }
            None => return Err("Missing mutable-data subcommand".to_string()),
        }
    }

    Ok(())
}

fn get_target_location(target_arg: Option<String>) -> Result<String, String> {
    match target_arg {
        Some(t) => Ok(t),
        None => {
            // try reading target from stdin then
            println!("Reading target from STDIN...");
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(n) => {
                    debug!(
                        "Read ({} bytes) from STDIN for target location: {}",
                        n, input
                    );
                    input.truncate(input.len() - 1);
                    Ok(input)
                }
                Err(_) => Err("There is no `--target` specified and no STDIN stream".to_string()),
            }
        }
    }
}

fn prompt_user(prompt_msg: &str, error_msg: &str) -> String {
    let mut user_input = String::new();
    print!("{}", prompt_msg);
    let _ = stdout().flush();
    stdin().read_line(&mut user_input).expect(error_msg);
    if let Some('\n') = user_input.chars().next_back() {
        user_input.pop();
    }
    if let Some('\r') = user_input.chars().next_back() {
        user_input.pop();
    }

    user_input
}
