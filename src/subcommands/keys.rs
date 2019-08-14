// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::helpers::get_secret_key;
use super::OutputFmt;
use log::{debug, warn};
use safe_cli::{BlsKeyPair, Safe};
use structopt::StructOpt;

const PRELOAD_TESTCOINS_DEFAULT_AMOUNT: &str = "1000.111";

#[derive(StructOpt, Debug)]
pub enum KeysSubCommands {
    #[structopt(name = "create")]
    /// Create a new Key
    Create {
        /// The secret key of a Key for paying the operation costs. If not provided, the default wallet from the account will be used, unless '--test-coins' was set
        #[structopt(short = "w", long = "pay-with")]
        pay_with: Option<String>,
        /// Create a Key and allocate test-coins onto it
        #[structopt(long = "test-coins")]
        preload_test_coins: bool,
        /// Preload the Key with a balance
        #[structopt(long = "preload")]
        preload: Option<String>,
        /// Don't generate a key pair and just use the provided public key
        #[structopt(long = "pk")]
        pk: Option<String>,
    },
    #[structopt(name = "balance")]
    /// Query a Key's current balance
    Balance {
        /// The target Key's safe://xor-url to verify it matches/corresponds to the secret key provided. The corresponding secret key will be prompted if not provided with '--sk'
        #[structopt(long = "keyurl")]
        keyurl: Option<String>,
        /// The secret key which corresponds to the target Key. It will be prompted if not provided
        #[structopt(long = "sk")]
        secret: Option<String>,
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
            preload_test_coins,
            ..
        }) => {
            create_new_key(safe, preload_test_coins, pay_with, preload, pk, output_fmt)?;
            Ok(())
        }
        Some(KeysSubCommands::Balance { keyurl, secret }) => {
            let target = keyurl.unwrap_or_else(|| "".to_string());
            let sk = get_secret_key(&target, secret, "the Key to query the balance from")?;
            let current_balance = if target.is_empty() {
                safe.keys_balance_from_sk(&sk)?
            } else {
                safe.keys_balance_from_url(&target, &sk)?
            };

            if OutputFmt::Pretty == output_fmt {
                println!("Key's current balance: {}", current_balance);
            } else {
                println!("{}", current_balance);
            }
            Ok(())
        }
        None => Err("Missing keys sub-command. Use --help for details.".to_string()),
    }
}

pub fn create_new_key(
    safe: &mut Safe,
    preload_test_coins: bool,
    pay_with: Option<String>,
    preload: Option<String>,
    pk: Option<String>,
    output_fmt: OutputFmt,
) -> Result<(String, Option<BlsKeyPair>), String> {
    let (xorurl, key_pair) = if preload_test_coins {
        if cfg!(not(feature = "mock-network")) && cfg!(not(feature = "scl-mock")) {
            warn!("'--test-coins' flag is not supported since it's only available for \"mock-network\" feature");
            return Err("'--test-coins' flag is not supported since it's only available for \"mock-network\" feature".to_string());
        }

        warn!("Note that the Key to be created will be preloaded with **test coins** rather than real coins");
        let amount = preload.unwrap_or_else(|| PRELOAD_TESTCOINS_DEFAULT_AMOUNT.to_string());

        if amount == PRELOAD_TESTCOINS_DEFAULT_AMOUNT {
            warn!(
                "You can pass a preload amount with test-coins, 1000.111 will be added by default."
            );
        }

        safe.keys_create_preload_test_coins(&amount, pk)?
    } else {
        // '--pay-with' is either a Wallet XOR-URL, or a secret key
        // TODO: support Wallet XOR-URL, we now support only secret key
        // If the --pay-with is not provided the API will use the account's default wallet/sk
        if pay_with.is_none() {
            debug!("Missing the '--pay-with' argument, using account's default wallet for funds");
        }
        safe.keys_create(pay_with, preload, pk)?
    };

    if OutputFmt::Pretty == output_fmt {
        println!("New Key created at: \"{}\"", xorurl);
        if let Some(pair) = &key_pair {
            println!("Key pair generated:");
            println!("pk = {}", pair.pk);
            println!("sk = {}", pair.sk);
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

    Ok((xorurl, key_pair))
}
