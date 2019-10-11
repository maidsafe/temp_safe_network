// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::helpers::{get_from_arg_or_stdin, get_secret_key};
use super::OutputFmt;
use crate::operations::auth::safe_connect;
use log::{debug, warn};
use safe_api::{BlsKeyPair, Safe};
use structopt::StructOpt;

const PRELOAD_TESTCOINS_DEFAULT_AMOUNT: &str = "1000.111";

#[derive(StructOpt, Debug)]
pub enum KeysSubCommands {
    #[structopt(name = "create")]
    /// Create a new SafeKey
    Create {
        /// The secret key of a SafeKey for paying the operation costs. If not provided, the default wallet from the account will be used, unless '--test-coins' was set
        #[structopt(short = "w", long = "pay-with")]
        pay_with: Option<String>,
        /// Create a SafeKey and allocate test-coins onto it
        #[structopt(long = "test-coins")]
        test_coins: bool,
        /// Preload the SafeKey with a balance
        #[structopt(long = "preload")]
        preload: Option<String>,
        /// Don't generate a key pair and just use the provided public key
        #[structopt(long = "pk")]
        pk: Option<String>,
    },
    #[structopt(name = "balance")]
    /// Query a SafeKey's current balance
    Balance {
        /// The target SafeKey's safe://xor-url to verify it matches/corresponds to the secret key provided. The corresponding secret key will be prompted if not provided with '--sk'
        #[structopt(long = "keyurl")]
        keyurl: Option<String>,
        /// The secret key which corresponds to the target SafeKey. It will be prompted if not provided
        #[structopt(long = "sk")]
        secret: Option<String>,
    },
    #[structopt(name = "transfer")]
    /// Transfer safecoins from one SafeKey to another, or to a Wallet
    Transfer {
        /// Number of safecoins to transfer
        amount: String,
        /// Source SafeKey's secret key, or funds from Account's default SafeKey will be used
        #[structopt(long = "from")]
        from: Option<String>,
        /// The receiving Wallet/SafeKey URL, or pulled from stdin if not provided
        #[structopt(long = "to")]
        to: Option<String>,
        /// The transaction ID, a random one will be generated if not provided. A valid TX Id is a number between 0 and 2^64
        #[structopt(long = "tx-id")]
        tx_id: Option<u64>,
    },
}

pub fn key_commander(
    cmd: Option<KeysSubCommands>,
    output_fmt: OutputFmt,
    safe: &mut Safe,
) -> Result<(), String> {
    match cmd {
        Some(KeysSubCommands::Create {
            preload,
            pk,
            pay_with,
            test_coins,
            ..
        }) => {
            if test_coins && (pk.is_some() | pay_with.is_some()) {
                // We don't support these args with --test-coins
                return Err("When passing '--test-coins' argument only the '--preload' argument can be also provided".to_string());
            } else if !test_coins {
                // We need to connect with an authorised app since we are not creating a SafeKey with test-coins
                safe_connect(safe)?;
            }

            let (xorurl, key_pair, amount) =
                create_new_key(safe, test_coins, pay_with, preload, pk)?;
            print_new_key_output(output_fmt, xorurl, key_pair, amount);
            Ok(())
        }
        Some(KeysSubCommands::Balance { keyurl, secret }) => {
            safe_connect(safe)?;
            let target = keyurl.unwrap_or_else(|| "".to_string());
            let sk = get_secret_key(&target, secret, "the SafeKey to query the balance from")?;
            let current_balance = if target.is_empty() {
                safe.keys_balance_from_sk(&sk)?
            } else {
                safe.keys_balance_from_url(&target, &sk)?
            };

            if OutputFmt::Pretty == output_fmt {
                println!("SafeKey's current balance: {}", current_balance);
            } else {
                println!("{}", current_balance);
            }
            Ok(())
        }
        Some(KeysSubCommands::Transfer {
            amount,
            from,
            to,
            tx_id,
        }) => {
            // TODO: don't connect if --from sk was passed
            safe_connect(safe)?;

            //TODO: if to starts without safe://, i.e. if it's a PK hex string.
            let destination = get_from_arg_or_stdin(
                to,
                Some("...awaiting destination Wallet/SafeKey URL from STDIN stream..."),
            )?;

            let tx_id = safe.keys_transfer(
                &amount,
                from.as_ref().map(String::as_str),
                &destination,
                tx_id,
            )?;

            if OutputFmt::Pretty == output_fmt {
                println!("Success. TX_ID: {}", &tx_id);
            } else {
                println!("{}", &tx_id)
            }

            Ok(())
        }
        None => Err("Missing keys sub-command. Use -h / --help for details.".to_string()),
    }
}

pub fn create_new_key(
    safe: &mut Safe,
    test_coins: bool,
    pay_with: Option<String>,
    preload: Option<String>,
    pk: Option<String>,
) -> Result<(String, Option<BlsKeyPair>, Option<String>), String> {
    let (xorurl, key_pair, amount) = if test_coins {
        warn!("Note that the SafeKey to be created will be preloaded with **test coins** rather than real coins");
        let amount = preload.unwrap_or_else(|| PRELOAD_TESTCOINS_DEFAULT_AMOUNT.to_string());

        if amount == PRELOAD_TESTCOINS_DEFAULT_AMOUNT {
            warn!(
                "You can pass a preload amount with test-coins, 1000.111 will be added by default."
            );
        }

        let (xorurl, key_pair) = safe.keys_create_preload_test_coins(&amount)?;
        (xorurl, key_pair, Some(amount))
    } else {
        // '--pay-with' is either a Wallet XOR-URL, or a secret key
        // TODO: support Wallet XOR-URL, we now support only secret key
        // If the --pay-with is not provided the API will use the account's default wallet/sk
        if pay_with.is_none() {
            debug!("Missing the '--pay-with' argument, using account's default wallet for funds");
        }
        let (xorurl, key_pair) = safe.keys_create(
            pay_with.as_ref().map(String::as_str),
            preload.as_ref().map(String::as_str),
            pk.as_ref().map(String::as_str),
        )?;
        (xorurl, key_pair, preload)
    };

    Ok((xorurl, key_pair, amount))
}

pub fn print_new_key_output(
    output_fmt: OutputFmt,
    xorurl: String,
    key_pair: Option<BlsKeyPair>,
    amount: Option<String>,
) {
    if OutputFmt::Pretty == output_fmt {
        println!("New SafeKey created at: \"{}\"", xorurl);
        if let Some(n) = amount {
            println!("Preloaded with {} coins", n);
        }
        if let Some(pair) = &key_pair {
            println!("Key pair generated:");
            println!("Public Key = {}", pair.pk);
            println!("Secret Key = {}", pair.sk);
        }
    } else if let Some(pair) = &key_pair {
        println!(
            "{}",
            serde_json::to_string(&(&xorurl, pair))
                .unwrap_or_else(|_| "Failed to serialise output to json".to_string())
        );
    } else {
        println!(
            "{}",
            serde_json::to_string(&xorurl)
                .unwrap_or_else(|_| "Failed to serialise output to json".to_string())
        );
    }
}
