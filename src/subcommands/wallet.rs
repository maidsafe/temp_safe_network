// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use structopt::StructOpt;

use safe_cli::{BlsKeyPair, Safe};
use unwrap::unwrap;

use crate::subcommands::helpers::{get_target_location, prompt_user};
use crate::subcommands::keys::create_new_key;

#[derive(StructOpt, Debug)]
pub enum WalletSubCommands {
    #[structopt(name = "insert")]
    /// Insert a spendable balance into a Wallet
    Insert {
        /// The source Wallet for funds
        source: String,
        /// The target Wallet to insert the spendable balance
        target: Option<String>,
        /// An existing `Key`'s safe://xor-url. If this is not supplied, a new `Key` will be automatically generated and inserted
        key: Option<String>,
        /// The name to give this spendable balance
        #[structopt(long = "name")]
        name: Option<String>,
        /// Create a Key, allocate test-coins onto it, and add it to the Wallet
        #[structopt(long = "test-coins")]
        test_coins: bool,
        /// Preload the key with a balance
        #[structopt(long = "preload")]
        preload: Option<String>,
        /// Set the sub name as default for this public name
        #[structopt(long = "default")]
        default: bool,
        /// Optionally pass the secret key to make the balance spendable
        #[structopt(short = "sk", long = "secret-key")]
        secret: Option<String>,
    },
    #[structopt(name = "balance")]
    /// Query a Wallet's total balance
    Balance {
        /// The target Wallet to check the total balance
        target: Option<String>,
    },
    #[structopt(name = "check-tx")]
    /// Check the status of a given transaction
    CheckTx {},
    #[structopt(name = "create")]
    /// Create a new Wallet
    Create {},
    #[structopt(name = "transfer")]
    /// Transfer safecoins from one Wallet, Key or pk, to another
    Transfer {
        /// Number of safecoins to transfer
        amount: String,
        /// target Wallet
        to: String,
        /// source Wallet, or pulled from stdin if not present
        from: Option<String>,
    },
    #[structopt(name = "sweep")]
    /// Move all coins within a Wallet to a second given Wallet or Key
    Sweep {
        /// The source Wallet for funds
        #[structopt(long = "from")]
        from: String,
        /// The receiving Wallet/Key
        #[structopt(long = "to")]
        to: String,
    },
}

pub fn wallet_commander(
    cmd: Option<WalletSubCommands>,
    pretty: bool,
    safe: &mut Safe,
) -> Result<(), String> {
    match cmd {
        Some(WalletSubCommands::Create {}) => {
            let xorname = safe.wallet_create()?;

            if pretty {
                println!("Wallet created at: \"{}\"", xorname);
            } else {
                println!("{}", xorname);
            }
            Ok(())
        }
        Some(WalletSubCommands::Balance { target }) => {
            // FIXME: get sk from args or from the account
            let sk =
                String::from("391987fd429b4718a59b165b5799eaae2e56c697eb94670de8886f8fb7387058");
            let target = get_target_location(target)?;
            let balance = safe.wallet_balance(&target, &sk)?;

            if pretty {
                println!(
                    "Wallet at \"{}\" has a total balance of {} safecoins",
                    target, balance
                );
            } else {
                println!("{}", balance);
            }

            Ok(())
        }
        Some(WalletSubCommands::Insert {
            preload,
            source,
            test_coins,
            target,
            key,
            name,
            default,
            secret,
        }) => {
            let target = get_target_location(target)?;

            let (xorname, key_pair) = match key {
                Some(linked_key) => {
                    let mut sk = secret.unwrap_or_else(|| String::from(""));

                    if sk.is_empty() {
                        // Get pk source Key, and prompt user for the corresponding sk
                        sk = prompt_user(
                            &format!(
                                "Enter secret key corresponding to public key at \"{}\": ",
                                linked_key
                            ),
                            "Invalid input",
                        )?;
                    }

                    let pk = safe.fetch_pk_from_xorname(&linked_key)?;

                    (linked_key, Some(BlsKeyPair { pk, sk }))
                }
                None => create_new_key(safe, test_coins, Some(source), preload, None, pretty)?,
            };

            let the_name = match name {
                Some(name_str) => name_str,
                None => xorname.clone(),
            };

            safe.wallet_insert(&target, &the_name, default, &unwrap!(key_pair), &xorname)?;
            if pretty {
                println!(
                    "Spendable balance inserted with name '{}' in Wallet located at \"{}\"",
                    the_name, target
                );
            } else {
                println!("{}", target);
            }
            Ok(())
        }
        Some(WalletSubCommands::Transfer { amount, from, to }) => {
            //TODO: if from/to start without safe://, i.e. if they are PK hex strings.
            let tx_id = safe.wallet_transfer(&amount, from, &to)?;

            if pretty {
                println!("Success. TX_ID: {:?}", &tx_id);
            } else {
                println!("{}", &tx_id)
            }

            Ok(())
        }
        _ => Err("Sub-command not supported yet".to_string()),
    }
}
