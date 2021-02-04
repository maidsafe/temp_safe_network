// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{
    helpers::{get_from_arg_or_stdin, get_secret_key, serialise_output},
    OutputFmt,
};
use crate::operations::safe_net::connect;
use log::{debug, warn};
use sn_api::{ed_sk_from_hex, sk_to_hex, Keypair, PublicKey, Safe, SecretKey};
use structopt::StructOpt;

const PRELOAD_DEFAULT_AMOUNT: &str = "0.000000001";
const PRELOAD_TESTCOINS_DEFAULT_AMOUNT: &str = "1000.111";

#[derive(StructOpt, Debug)]
pub enum KeysSubCommands {
    #[structopt(name = "create")]
    /// Create a new SafeKey
    Create {
        /// The secret key of a SafeKey for paying the operation costs. If not provided, the application's default wallet will be used, unless '--test-coins' was set
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
        /// The secret key which corresponds to the target SafeKey. CLI application's default SafeKey will be used by default, otherwise if the CLI has not been given a keypair (authorised), it will be prompted
        #[structopt(long = "sk")]
        secret: Option<String>,
    },
    #[structopt(name = "transfer")]
    /// Transfer safecoins from one SafeKey to another, or to a Wallet
    Transfer {
        /// Number of safecoins to transfer
        amount: String,
        /// Source SafeKey's secret key, or funds from the application's default SafeKey will be used
        #[structopt(long = "from")]
        from: Option<String>,
        /// The receiving Wallet/SafeKey URL or public key, otherwise pulled from stdin if not provided
        #[structopt(long = "to")]
        to: Option<String>,
    },
}

pub async fn key_commander(
    cmd: KeysSubCommands,
    output_fmt: OutputFmt,
    safe: &mut Safe,
) -> Result<(), String> {
    match cmd {
        KeysSubCommands::Create {
            preload,
            pk,
            pay_with,
            test_coins,
            ..
        } => {
            // TODO: support pk argument
            if test_coins && (pk.is_some() | pay_with.is_some()) {
                // We don't support these args with --test-coins
                return Err("When passing '--test-coins' argument only the '--preload' argument can be also provided".to_string());
            } else if !test_coins {
                // We need to connect with an authorised app since we are not creating a SafeKey with test-coins
                connect(safe).await?;
            }

            let (xorurl, key_pair, amount) =
                create_new_key(safe, test_coins, pay_with, preload, pk).await?;
            print_new_key_output(output_fmt, xorurl, key_pair, amount, test_coins);

            Ok(())
        }
        KeysSubCommands::Balance { keyurl, secret } => {
            let target = keyurl.unwrap_or_else(|| "".to_string());
            let sk = match connect(safe).await? {
                Some(keypair) if secret.is_none() => {
                    // we then use the secret from CLI's given credentials
                    println!("Checking balance of CLI's assigned keypair...");
                    keypair.secret_key().map_err(|err| {
                        format!(
                            "Failed to obtain the secret key from app's assigned keypair: {}",
                            err
                        )
                    })?
                }
                Some(_) | None => {
                    // prompt the user for a SK
                    let secret_key =
                        get_secret_key(&target, secret, "the SafeKey to query the balance from")?;

                    SecretKey::Ed25519(ed_sk_from_hex(&secret_key)?)
                }
            };

            let current_balance = if target.is_empty() {
                safe.keys_balance_from_sk(sk).await
            } else {
                safe.keys_balance_from_url(&target, sk).await
            }?;

            if OutputFmt::Pretty == output_fmt {
                println!("SafeKey's current balance: {}", current_balance);
            } else {
                println!("{}", current_balance);
            }
            Ok(())
        }
        KeysSubCommands::Transfer { amount, from, to } => {
            // TODO: don't connect if --from sk was passed
            connect(safe).await?;

            let destination = get_from_arg_or_stdin(
                to,
                Some("...awaiting destination Wallet/SafeKey URL, or public key, from STDIN stream..."),
            )?;

            let tx_id = safe
                .keys_transfer(&amount, from.as_deref(), &destination)
                .await?;

            if OutputFmt::Pretty == output_fmt {
                println!("Success. TX_ID: {}", tx_id);
            } else {
                println!("{}", tx_id)
            }

            Ok(())
        }
    }
}

#[cfg(feature = "simulated-payouts")]
pub async fn create_new_key(
    safe: &mut Safe,
    test_coins: bool,
    pay_with: Option<String>,
    preload: Option<String>,
    _pk: Option<String>,
) -> Result<(String, Option<Keypair>, String), String> {
    if test_coins {
        warn!("Note that the SafeKey to be created will be preloaded with **test coins** rather than real coins");
        let amount = match preload {
            None => {
                warn!(
                    "You can specify a preload amount with --preload argument, 1000.111 will be used by default."
                );
                PRELOAD_TESTCOINS_DEFAULT_AMOUNT.to_string()
            }
            Some(n) => n,
        };

        let (xorurl, key_pair) = safe.keys_create_preload_test_coins(&amount).await?;

        Ok((xorurl, Some(key_pair), amount))
    } else {
        let amount = match preload {
            None => {
                warn!(
                    "You can specify a preload amount with --preload argument, 1 nano will be used by default."
                );
                PRELOAD_DEFAULT_AMOUNT.to_string()
            }
            Some(n) => n,
        };

        // '--pay-with' is either a Wallet XOR-URL, or a secret key
        // TODO: support Wallet XOR-URL, we now support only secret key
        // If the --pay-with is not provided the API will use the application's default wallet/sk
        let (xorurl, key_pair) = match pay_with {
            Some(payee) => {
                safe.keys_create_and_preload_from_sk_string(&payee, &amount)
                    .await?
            }
            None => {
                debug!("Missing the '--pay-with' argument, using app's wallet for funds");
                safe.keys_create_and_preload(&amount).await?
            }
        };

        Ok((xorurl, Some(key_pair), amount))
    }
}

pub fn print_new_key_output(
    output_fmt: OutputFmt,
    xorurl: String,
    key_pair: Option<Keypair>,
    amount: String,
    test_coins: bool,
) {
    if OutputFmt::Pretty == output_fmt {
        println!("New SafeKey created at: \"{}\"", xorurl);
        println!(
            "Preloaded with {} {}",
            amount,
            if test_coins { "testcoins" } else { "coins" }
        );

        if let Some(pair) = &key_pair {
            println!("Key pair generated:");
            match keypair_to_hex_strings(&pair) {
                Ok((pk_hex, sk_hex)) => {
                    println!("Public Key = {}", pk_hex);
                    println!("Secret Key = {}", sk_hex);
                }
                Err(err) => println!("{}", err),
            }
        }
    } else if let Some(pair) = &key_pair {
        match keypair_to_hex_strings(&pair) {
            Ok((pk_hex, sk_hex)) => println!(
                "{}",
                serialise_output(&(xorurl, (pk_hex, sk_hex)), output_fmt)
            ),
            Err(err) => println!("{}", err),
        }
    }
}

pub fn keypair_to_hex_strings(keypair: &Keypair) -> Result<(String, String), String> {
    let pk_hex = match keypair.public_key() {
        PublicKey::Ed25519(pk) => pk.to_bytes().iter().map(|b| format!("{:02x}", b)).collect(),
        PublicKey::Bls(pk) => pk.to_bytes().iter().map(|b| format!("{:02x}", b)).collect(),
        PublicKey::BlsShare(pk) => pk.to_bytes().iter().map(|b| format!("{:02x}", b)).collect(),
    };

    let sk_hex = sk_to_hex(
        keypair
            .secret_key()
            .map_err(|err| format!("Failed to obtain secret key: {}", err))?,
    );

    Ok((pk_hex, sk_hex))
}
