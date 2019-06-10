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
    /// Insert a spendable balance into a wallet
    Insert {
        /// The source wallet for funds
        source: String,
        /// The target wallet to store the spendable balance.
        target: Option<String>,
		/// An existing key safe://xor-url to add to the wallet. If this is not supplied a new key will be generated and inserted.
        key: Option<String>,
        /// The name to give this spendable balance
        #[structopt(long = "name")]
        name: Option<String>,
        /// Create a Key, allocate test-coins onto it, and add it to the wallet
        #[structopt(long = "test-coins")]
        test_coins: bool,
        /// Preload the key with a coinbalance
        #[structopt(long = "preload")]
        preload: Option<String>,
        /// Set the sub name as default for this public name
        #[structopt(long = "default")]
        default: bool,
        /// Optionally pass the secret key for the balance being inserted.
        #[structopt(short = "sk", long = "secret-key")]
        secret: Option<String>,
    },
    #[structopt(name = "balance")]
    /// Query a new Wallet or PublicKeys CoinBalance
    Balance {
        /// The target wallet to check the total balance.
        target: Option<String>,
    },
    #[structopt(name = "check-tx")]
    /// Check the status of a given transaction
    CheckTx {},
    #[structopt(name = "create")]
    /// Create a new Wallet/CoinBalance
    Create {},
    #[structopt(name = "transfer")]
    /// Transfer safecoins from one wallet, spendable balance or pk to another.
    Transfer {
        /// Number of safecoins to transfer
        amount: String,
        /// target wallet
        to: String,
        /// source wallet, or pulled from stdin if not present
        from: Option<String>,
    },
    #[structopt(name = "sweep")]
    /// Move all coins within a wallet to a given balance
    Sweep {
        /// The source wallet for funds
        #[structopt(long = "from")]
        from: String,
        /// The receiving wallet/ballance
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
            let xorname = safe.wallet_create();

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
            let balance = safe.wallet_balance(&target, &sk);

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
                    let mut sk = secret.unwrap_or(String::from(""));

                    if sk.is_empty() {
                        // Get pk source Key, and prompt user for the corresponding sk
                        sk = prompt_user(
                            &format!(
                                "Enter secret key corresponding to public key at \"{}\": ",
                                linked_key
                            ),
                            "Invalid input",
                        );
                    }

                    let pk = safe.keys_fetch_pk(&linked_key);

                    (linked_key, Some(BlsKeyPair { pk, sk }))
                }
                None => create_new_key(safe, test_coins, Some(source), preload, None, pretty),
            };

            let the_name = match name {
                Some(name_str) => name_str,
                None => xorname.clone(),
            };

            safe.wallet_insert(&target, &the_name, default, &unwrap!(key_pair), &xorname);
            if pretty {
                println!(
                    "Spendable balance added with name '{}' in wallet located at XOR-URL \"{}\"",
                    the_name, target
                );
            } else {
                println!("{}", target);
            }
            Ok(())
        }
        Some(WalletSubCommands::Transfer { amount, from, to }) => {
            //TODO: if from/to start withOUT safe:// PKs.
            let tx_id = safe.wallet_transfer(&amount, from, &to).unwrap();

            if pretty {
                println!("Success. TX_ID: {:?}", &tx_id);
            } else {
                println!("{}", &tx_id)
            }

            Ok(())
        }
        _ => return Err("Sub-command not supported yet".to_string()),
    }
}
