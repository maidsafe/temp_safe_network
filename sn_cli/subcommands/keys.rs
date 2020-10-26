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
use sn_api::{bls_sk_from_hex, ed_sk_from_hex, Keypair, Safe, SecretKey};
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
        /// The secret key is a BLS secret key. (Defaults to an ED25519 Secret Key)
        #[structopt(long = "bls")]
        is_bls: bool,
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
        /// The from secret key is a BLS secret key. (Defaults to an ED25519 Secret Key)
        #[structopt(long = "from-is-bls")]
        from_is_bls: bool,
        /// The target secret key is a BLS secret key. (Defaults to an ED25519 Secret Key)
        #[structopt(long = "to-is-bls")]
        to_is_bls: bool,
        // TODO: BlsShare when we have multisig
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
            if test_coins && (pk.is_some() | pay_with.is_some()) {
                // We don't support these args with --test-coins
                return Err("When passing '--test-coins' argument only the '--preload' argument can be also provided".to_string());
            } else if !test_coins {
                // We need to connect with an authorised app since we are not creating a SafeKey with test-coins
                connect(safe).await?;
            }

            let (xorurl, key_pair, amount) =
                create_new_key(safe, test_coins, pay_with, preload, pk).await?;
            print_new_key_output(output_fmt, xorurl, key_pair, amount);
            Ok(())
        }
        KeysSubCommands::Balance {
            keyurl,
            secret,
            is_bls,
        } => {
            connect(safe).await?;
            let target = keyurl.unwrap_or_else(|| "".to_string());
            let sk = get_secret_key(&target, secret, "the SafeKey to query the balance from")?;
            let sk = if is_bls {
                SecretKey::from(bls_sk_from_hex(&sk)?)
            } else {
                SecretKey::Ed25519(ed_sk_from_hex(&sk)?)
            };

            let current_balance = if target.is_empty() {
                safe.keys_balance_from_sk(sk).await
            } else {
                safe.keys_balance_from_url(&target, sk.clone()).await
            }?;

            if OutputFmt::Pretty == output_fmt {
                println!("SafeKey's current balance: {}", current_balance);
            } else {
                println!("{}", current_balance);
            }
            Ok(())
        }
        KeysSubCommands::Transfer {
            amount,
            from,
            to,
            to_is_bls: _,
            from_is_bls: _,
        } => {
            // TODO: don't connect if --from sk was passed
            connect(safe).await?;

            //TODO: if to starts without safe://, i.e. if it's a PK hex string.
            let destination = get_from_arg_or_stdin(
                to,
                Some("...awaiting destination Wallet/SafeKey URL from STDIN stream..."),
            )?;

            let tx_id = safe
                .keys_transfer(&amount, from.as_deref(), &destination)
                .await?;

            if OutputFmt::Pretty == output_fmt {
                println!("Successful. TX_ID: {:?}", &tx_id);
            } else {
                println!("{:?}", &tx_id)
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
) -> Result<(String, Option<Keypair>, Option<String>), String> {
    let (xorurl, key_pair, amount) = if test_coins {
        warn!("Note that the SafeKey to be created will be preloaded with **test coins** rather than real coins");
        let amount = preload.unwrap_or_else(|| PRELOAD_TESTCOINS_DEFAULT_AMOUNT.to_string());

        if amount == PRELOAD_TESTCOINS_DEFAULT_AMOUNT {
            warn!(
                "You can pass a preload amount with test-coins, 1000.111 will be added by default."
            );
        }

        let (xorurl, key_pair) = safe.keys_create_preload_test_coins(&amount).await?;
        (xorurl, key_pair, Some(amount))
    } else {
        let key_pair;
        let xorurl;

        // '--pay-with' is either a Wallet XOR-URL, or a secret key
        // TODO: support Wallet XOR-URL, we now support only secret key
        // If the --pay-with is not provided the API will use the account's default wallet/sk
        if pay_with.is_none() {
            debug!("Missing the '--pay-with' argument, using account's default wallet for funds");

            let payee = safe
                .keypair()
                .await?
                .secret_key()
                .map_err(|e| format!("Secret key error: {:?}", e))?;
            let keys_info = safe
                .keys_create_and_preload_from_sk_string(&payee.to_string(), preload.as_deref())
                .await?;

            xorurl = keys_info.0;
            key_pair = keys_info.1;
        } else {
            let keys_info = safe
                .keys_create_and_preload_from_sk_string(&pay_with.unwrap(), preload.as_deref())
                .await?;

            xorurl = keys_info.0;
            key_pair = keys_info.1;
        }

        (xorurl, key_pair, preload)
    };

    Ok((xorurl, Some(key_pair), amount))
}

pub fn print_new_key_output(
    output_fmt: OutputFmt,
    xorurl: String,
    key_pair: Option<Keypair>,
    amount: Option<String>,
) {
    if OutputFmt::Pretty == output_fmt {
        println!("New SafeKey created at: \"{}\"", xorurl);
        if let Some(n) = amount {
            println!("Preloaded with {} coins", n);
        }
        if let Some(pair) = &key_pair {
            println!("Key pair generated:");
            println!("Public Key = {}", pair.public_key());
            match pair.secret_key() {
                Ok(sk) => println!("Secret Key = {}", sk),
                Err(error) => println!("{:?}", error),
            }
        }
    } else if let Some(pair) = &key_pair {
        println!("{}", serialise_output(&(pair), output_fmt));
    }
}
