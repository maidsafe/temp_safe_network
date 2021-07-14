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
use crate::operations::{
    auth_and_connect::{create_credentials_file, read_credentials},
    safe_net::connect,
};
use anyhow::{anyhow, bail, Context, Result};
use hex::encode;
use log::{debug, warn};
use sn_api::{
    fetch::{SafeData, SafeUrl},
    sk_to_hex, Keypair, PublicKey, Safe, SecretKey, XorName,
};
use std::io::Write;
use structopt::StructOpt;

const PRELOAD_DEFAULT_AMOUNT: &str = "0.000000001";
const PRELOAD_TESTCOINS_DEFAULT_AMOUNT: &str = "1000.111";

#[derive(StructOpt, Debug)]
pub enum KeysSubCommands {
    /// Show information about a SafeKey, by default it will show info about the one owned by CLI (if found)
    Show {
        /// Show Secret Key as well
        #[structopt(long = "show-sk")]
        show_sk: bool,
        /// The SafeKey's URL to decode and show its Public Key. If this is not provided, the SafeKey owned by CLI (if found) will be shown
        keyurl: Option<String>,
    },
    #[structopt(name = "create")]
    /// Create a new SafeKey
    Create {
        /// The secret key of a SafeKey for paying the operation costs. If not provided, the application's default wallet will be used, unless '--test-coins' was set
        #[structopt(short = "w", long = "pay-with")]
        pay_with: Option<String>,
        /// Create a SafeKey and allocate test-coins onto it
        #[structopt(long = "test-coins")]
        test_coins: bool,
        /// Set the newly created keys to be used by CLI
        #[structopt(long = "for-cli")]
        for_cli: bool,
    },
}

pub async fn key_commander(
    cmd: KeysSubCommands,
    output_fmt: OutputFmt,
    safe: &mut Safe,
) -> Result<()> {
    match cmd {
        KeysSubCommands::Show { show_sk, keyurl } => {
            if let Some(url) = keyurl {
                if show_sk {
                    bail!("The 'show-sk' flag cannot be set when providing a SafeKey URL");
                }

                match safe.fetch(&url, None).await {
                    Ok(SafeData::SafeKey {
                        xorurl, xorname, ..
                    }) => {
                        // Get pk from xorname. We assume Ed25519 key for now, which is
                        // 32 bytes long, just like a xorname.
                        // TODO: support for BLS keys which are longer.
                        let pk = ed25519_dalek::PublicKey::from_bytes(&xorname).map_err(|err| {
                            anyhow!(
                                "Failed to derive Ed25519 PublicKey from SafeKey at '{}': {:?}",
                                url,
                                err
                            )
                        })?;

                        println!("SafeKey found at {}:", url);
                        println!("XOR-URL: {}", xorurl);
                        println!("Public Key: {}", encode(pk));
                    }
                    Ok(other) => bail!(format!(
                        "The Safe-URL provided is not targetting a SafeKey: {:?}",
                        other
                    )),
                    Err(err) => bail!(err),
                }
            } else {
                match read_credentials()? {
                    (file_path, Some(keypair)) => {
                        let xorname = XorName::from(keypair.public_key());
                        let xorurl = SafeUrl::encode_safekey(xorname, safe.xorurl_base)?;
                        let (pk_hex, sk_hex) = keypair_to_hex_strings(&keypair)?;

                        println!("Current CLI's SafeKey found at {}:", file_path.display());
                        println!("XOR-URL: {}", xorurl);
                        println!("Public Key: {}", pk_hex);
                        if show_sk {
                            println!("Secret Key: {}", sk_hex);
                        }
                    }
                    (file_path, None) => println!("No SafeKey found at {}", file_path.display()),
                }
            }

            Ok(())
        }
        KeysSubCommands::Create {
            for_cli,
            ..
        } => {


            let (xorurl, key_pair) =
                create_new_key(safe).await?;
            print_new_key_output(output_fmt, xorurl, Some(&key_pair), );

            if for_cli {
                println!("Setting new SafeKey to be used by CLI...");
                let (mut file, file_path) = create_credentials_file()?;
                let serialised_keypair = serde_json::to_string(&key_pair)
                    .context("Unable to serialise the credentials created")?;

                file.write_all(serialised_keypair.as_bytes())
                    .with_context(|| {
                        format!("Unable to write credentials in {}", file_path.display(),)
                    })?;

                println!(
                    "New credentials were successfully stored in {}",
                    file_path.display()
                );
                println!("Safe CLI now has write access to the network");
            }

            Ok(())
        }
    }
}



pub fn print_new_key_output(
    output_fmt: OutputFmt,
    xorurl: String,
    key_pair: Option<&Keypair>,
) {
    if OutputFmt::Pretty == output_fmt {
        println!("New SafeKey created: \"{}\"", xorurl);

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

pub fn keypair_to_hex_strings(keypair: &Keypair) -> Result<(String, String)> {
    let pk_hex = match keypair.public_key() {
        PublicKey::Ed25519(pk) => pk.to_bytes().iter().map(|b| format!("{:02x}", b)).collect(),
        PublicKey::Bls(pk) => pk.to_bytes().iter().map(|b| format!("{:02x}", b)).collect(),
        PublicKey::BlsShare(pk) => pk.to_bytes().iter().map(|b| format!("{:02x}", b)).collect(),
    };

    let sk_hex = sk_to_hex(
        keypair
            .secret_key()
            .context("Failed to obtain secret key")?,
    );

    Ok((pk_hex, sk_hex))
}



#[cfg(feature = "testing")]
pub async fn create_new_key(
    safe: &mut Safe,
) -> Result<(String, Keypair)> {

        // '--pay-with' is either a Wallet XOR-URL, or a secret key
        let key_pair =  safe.generate_random_ed_keypair();


        let xorname = XorName::from(key_pair.public_key());
        let xorurl = SafeUrl::encode_safekey(xorname, safe.xorurl_base)?;
        // // TODO: support Wallet XOR-URL, we now support only secret key
        // // If the --pay-with is not provided the API will use the application's default wallet/sk
        // let (xorurl, key_pair) = match pay_with {
        //     Some(payee) => {
        //         safe.keys_create_and_preload_from_sk_string(&payee, &amount)
        //             .await?
        //     }
        //     None => {
        //         debug!("Missing the '--pay-with' argument, using app's wallet for funds");
        //     }
        // };

        Ok((xorurl, key_pair))

}
