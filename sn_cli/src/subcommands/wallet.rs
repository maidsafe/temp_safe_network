// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{
    helpers::{get_from_arg_or_stdin, serialise_output},
    OutputFmt,
};
use crate::operations::config::Config;
use bls::{PublicKey, SecretKey};
use color_eyre::{eyre::eyre, Help, Result};
use sn_api::{Error, Safe};
use sn_dbc::Dbc;
use std::path::Path;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum WalletSubCommands {
    #[structopt(name = "create")]
    /// Create a new wallet
    Create {},
    #[structopt(name = "balance")]
    /// Query a wallet's balance
    Balance {
        /// The URL of wallet to query
        target: Option<String>,
    },
    #[structopt(name = "deposit")]
    /// Deposit a spendable DBC in a wallet. If the DBC is not bearer, we will try to deposit using
    /// the secret key configured for use with safe. If you wish to use a different key, use the
    /// --secret-key argument.
    Deposit {
        /// The URL of the wallet for the deposit
        wallet_url: String,
        /// The name to give this spendable DBC
        #[structopt(long = "name")]
        name: Option<String>,
        /// A path to a file containing hex encoded DBC data, or you can supply the data directly.
        /// Depending on the shell or OS in use, due to the length of the data string, supplying
        /// directly may not work.
        #[structopt(long = "dbc")]
        dbc: Option<String>,
        #[structopt(long = "secret-key")]
        /// Use this argument to specify a secret key for an owned DBC. It should be a hex-encoded
        /// BLS key.
        secret_key_hex: Option<String>,
    },
    #[structopt(name = "reissue")]
    /// Reissue a DBC from a wallet to a SafeKey.
    Reissue {
        /// The amount to reissue
        amount: String,
        /// The URL of wallet to reissue from
        #[structopt(long = "from")]
        from: String,
        /// To reissue the DBC to a particular owner, provide their public key. This should be a
        /// hex-encoded BLS key. Otherwise the DBC will be reissued as bearer, meaning anyone can
        /// spend it. This argument and the --owned argument are mutually exclusive.
        #[structopt(long = "public-key")]
        public_key_hex: Option<String>,
        /// Set this flag to reissue as an owned DBC, using the public key configured for use with
        /// safe. This argument and the --public-key argument are mutually exclusive.
        #[structopt(long = "owned")]
        owned: bool,
    },
}

pub async fn wallet_commander(
    cmd: WalletSubCommands,
    output_fmt: OutputFmt,
    safe: &Safe,
    config: &Config,
) -> Result<()> {
    match cmd {
        WalletSubCommands::Create {} => {
            let wallet_xorurl = safe.wallet_create().await?;

            if OutputFmt::Pretty == output_fmt {
                println!("Wallet created at: \"{}\"", wallet_xorurl);
            } else {
                println!("{}", serialise_output(&wallet_xorurl, output_fmt));
            }

            Ok(())
        }
        WalletSubCommands::Balance { target } => {
            let target = get_from_arg_or_stdin(
                target,
                Some("...awaiting wallet address/location from STDIN stream..."),
            )?;

            let balance = safe.wallet_balance(&target).await?;

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
        WalletSubCommands::Deposit {
            wallet_url,
            name,
            dbc,
            secret_key_hex,
        } => {
            let dbc = if let Some(dbc) = dbc {
                let path = Path::new(&dbc);
                if path.exists() {
                    if path.is_dir() {
                        return Err(eyre!("The path supplied refers to a directory.")
                            .suggestion("A file path must be specified for the DBC data."));
                    }
                    let dbc_data = std::fs::read_to_string(path)?;
                    Dbc::from_hex(dbc_data.trim()).map_err(|e| {
                        eyre!(e.to_string()).suggestion(
                            "This file does not appear to have DBC data. \
                            Please select another file with valid hex-encoded DBC data.",
                        )
                    })?
                } else {
                    Dbc::from_hex(&dbc)?
                }
            } else {
                let dbc_hex = get_from_arg_or_stdin(dbc, None)?;
                Dbc::from_hex(dbc_hex.trim())?
            };

            let sk = if dbc.is_bearer() {
                None
            } else if let Some(sk_hex) = secret_key_hex {
                // This is an owned DBC and its secret key has been supplied
                Some(SecretKey::from_hex(&sk_hex)?)
            } else {
                // This is an owned DBC but its secret key was not provided,
                // thus attempt to use the key configured for use with the CLI.
                Some(read_key_from_configured_credentials(
                    config,
                    "This is an owned DBC. To deposit, it requires a secret key. \
                         A secret key was not supplied and there were no credentials \
                         configured for use with safe."
                        .to_string(),
                    "Please run the command again using the --secret-key \
                         argument to specify the key."
                        .to_string(),
                )?)
            };

            let name = safe
                .wallet_deposit(&wallet_url, name.as_deref(), &dbc, sk)
                .await
                .map_err(|e| match e {
                    Error::DbcDepositInvalidSecretKey => {
                        eyre!("The supplied secret key did not match the public key for this DBC.")
                            .suggestion(
                                "Please run the command again with the correct key for the \
                                --secret-key argument.",
                            )
                    }
                    _ => e.into(),
                })?;
            if OutputFmt::Pretty == output_fmt {
                println!(
                    "Spendable DBC deposited with name '{}' in wallet located at \"{}\"",
                    name, wallet_url
                );
            } else {
                println!("{}", serialise_output(&(wallet_url, name), output_fmt));
            }

            Ok(())
        }
        WalletSubCommands::Reissue {
            amount,
            from,
            public_key_hex,
            owned,
        } => {
            if owned && public_key_hex.is_some() {
                return Err(eyre!(
                    "The --owned and --public-key arguments are mutually exclusive."
                )
                .suggestion(
                    "Please run the command again and use one or the other, but not both, of these \
                    arguments."));
            }
            let pk = if let Some(pk_hex) = public_key_hex {
                Some(PublicKey::from_hex(&pk_hex)?)
            } else if owned {
                let sk = read_key_from_configured_credentials(
                    config,
                    "The --owned argument requires credentials to be configured for safe."
                        .to_string(),
                    "Run the 'keys create --for-cli' command to generate a credentials then run \
                    this command again."
                        .to_string(),
                )?;
                Some(sk.public_key())
            } else {
                None
            };
            let dbc = safe.wallet_reissue(&from, &amount, pk).await?;
            let dbc_hex = dbc.to_hex()?;

            if OutputFmt::Pretty == output_fmt {
                println!("Reissued DBC with {} safecoins.", amount);
                println!("-------- DBC DATA --------");
                println!("{}", dbc_hex);
                println!("--------------------------");
                if let Some(pk) = pk {
                    println!("This DBC is owned by public key {}", pk.to_hex());
                } else {
                    println!("This is a bearer DBC that can be spent by anyone.");
                }
            } else {
                println!("{}", dbc_hex);
            }

            Ok(())
        }
    }
}

/// Helper to get the secret key from the credentials that are configured for use with safe.
///
/// Different error and suggestion messages need to be provided depending on the context in which
/// it is used.
///
/// Returns an error if the credentials file is missing.
fn read_key_from_configured_credentials(
    config: &Config,
    error: String,
    suggestion: String,
) -> Result<SecretKey> {
    let mut credentials_path = config.cli_config_path.clone();
    credentials_path.pop();
    credentials_path.push("credentials");
    if !credentials_path.exists() {
        return Err(eyre!(error).suggestion(suggestion));
    }
    let sk_hex = std::fs::read_to_string(credentials_path)?;
    Ok(SecretKey::from_hex(&sk_hex)?)
}
