// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use structopt::StructOpt;

use super::helpers::{get_from_arg_or_stdin, get_secret_key};
use super::keys::create_new_key;
use super::OutputFmt;
use log::debug;
use safe_cli::Safe;

#[derive(StructOpt, Debug)]
pub enum WalletSubCommands {
    #[structopt(name = "insert")]
    /// Insert a spendable balance into a Wallet
    Insert {
        /// The target Wallet to insert the spendable balance
        target: String,
        /// The secret key of a Key for paying the operation costs. If not provided, the default wallet from the account will be used
        #[structopt(short = "w", long = "pay-with")]
        pay_with: Option<String>,
        /// Pass the secret key needed to make the balance spendable, it will be prompted if not provided
        #[structopt(long = "sk")]
        secret: Option<String>,
        /// The name to give this spendable balance
        #[structopt(long = "name")]
        name: Option<String>,
        /// The Key's safe://xor-url to verify it matches/corresponds to the secret key provided. The corresponding secret key will be prompted if not provided with '--sk'
        #[structopt(long = "keyurl")]
        keyurl: Option<String>,
        /// Set the inserted Key as the default one in the target Wallet
        #[structopt(long = "default")]
        default: bool,
    },
    #[structopt(name = "balance")]
    /// Query a Wallet's total balance
    Balance {
        /// The target Wallet to check the total balance
        target: Option<String>,
    },
    /*#[structopt(name = "check-tx")]
    /// Check the status of a given transaction
    CheckTx {},*/
    #[structopt(name = "create")]
    /// Create a new Wallet
    Create {
        /// The secret key of a Key for paying the operation costs
        #[structopt(short = "w", long = "pay-with")]
        pay_with: Option<String>,
        /// If true, do not create a spendable balance
        #[structopt(long = "no-balance")]
        no_balance: bool,
        /// The name to give the spendable balance
        #[structopt(long = "name")]
        name: Option<String>,
        /// An existing Key's safe://xor-url. If this is not supplied, a new Key will be automatically generated and inserted. The corresponding secret key will be prompted if not provided with '--sk'
        #[structopt(long = "keyurl")]
        keyurl: Option<String>,
        /// Pass the secret key needed to make the balance spendable, it will be prompted if not provided
        #[structopt(long = "sk")]
        secret: Option<String>,
        /// Create a Key, allocate test-coins onto it, and add the Key to the Wallet
        #[structopt(long = "test-coins")]
        test_coins: bool,
        /// Preload the key with a balance
        #[structopt(long = "preload")]
        preload: Option<String>,
    },
    #[structopt(name = "transfer")]
    /// Transfer safecoins from one Wallet, Key or pk, to another
    Transfer {
        /// Number of safecoins to transfer
        amount: String,
        /// The receiving Wallet/Key URL
        to: String,
        /// Source Wallet URL, or pulled from stdin if not provided
        from: Option<String>,
    },
    /*#[structopt(name = "sweep")]
    /// Move all coins within a Wallet to a second given Wallet or Key
    Sweep {
        /// The source Wallet for funds
        #[structopt(long = "from")]
        from: String,
        /// The receiving Wallet/Key
        #[structopt(long = "to")]
        to: String,
    },*/
}

pub fn wallet_commander(
    cmd: Option<WalletSubCommands>,
    output_fmt: OutputFmt,
    safe: &mut Safe,
) -> Result<(), String> {
    match cmd {
        Some(WalletSubCommands::Create {
            preload,
            test_coins,
            no_balance,
            keyurl,
            name,
            pay_with,
            secret,
        }) => {
            // create wallet
            let wallet_xorname = safe.wallet_create()?;

            if !no_balance {
                // get or create keypair
                let sk = match keyurl {
                    Some(linked_key) => {
                        let sk = get_secret_key(&linked_key, secret, "the Key to insert")?;
                        let _pk = safe.validate_sk_for_url(&sk, &linked_key)?;
                        sk
                    }
                    None => {
                        let (_xorurl, key_pair) =
                            create_new_key(safe, test_coins, pay_with, preload, None, output_fmt)?;
                        let unwrapped_key_pair =
                            key_pair.ok_or("Failed to read the generated key pair")?;
                        unwrapped_key_pair.sk
                    }
                };

                // insert and set as default
                safe.wallet_insert(&wallet_xorname, name, true, &sk)?;
            }

            if OutputFmt::Pretty == output_fmt {
                println!("Wallet created at: \"{}\"", &wallet_xorname);
            } else {
                println!("{}", &wallet_xorname);
            }
            Ok(())
        }
        Some(WalletSubCommands::Balance { target }) => {
            let target = get_from_arg_or_stdin(
                target,
                Some("...awaiting Wallet address/location from STDIN stream..."),
            )?;

            debug!("Got target location {:?}", target);
            let balance = safe.wallet_balance(&target)?;

            if OutputFmt::Pretty == output_fmt {
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
            target,
            keyurl,
            name,
            default,
            secret,
            pay_with,
        }) => {
            if pay_with.is_some() {
                println!("The '--pay-with' argument is being ignored for now as it's not supported yet for this command.");
            }

            let sk = match keyurl {
                Some(linked_key) => {
                    let sk = get_secret_key(&linked_key, secret, "the Key to insert")?;
                    let _pk = safe.validate_sk_for_url(&sk, &linked_key)?;
                    sk
                }
                None => get_secret_key("", secret, "the Key to insert")?,
            };

            let the_name = safe.wallet_insert(&target, name, default, &sk)?;
            if OutputFmt::Pretty == output_fmt {
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
            let source = get_from_arg_or_stdin(
                from,
                Some("...awaiting source Wallet/Key URL to be used for funds from STDIN stream..."),
            )?;

            let tx_id = safe.wallet_transfer(&amount, Some(&source), &to)?;

            if OutputFmt::Pretty == output_fmt {
                println!("Success. TX_ID: {}", &tx_id);
            } else {
                println!("{}", &tx_id)
            }

            Ok(())
        }
        _ => Err("Sub-command not supported yet".to_string()),
    }
}
