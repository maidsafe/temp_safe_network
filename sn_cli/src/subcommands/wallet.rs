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
use color_eyre::{eyre::eyre, Help, Result};
use sn_api::Safe;
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
    /// Deposit a spendable DBC in a wallet
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
        /// spend it.
        #[structopt(long = "public-key")]
        public_key_hex: Option<String>,
    },
}

pub async fn wallet_commander(
    cmd: WalletSubCommands,
    output_fmt: OutputFmt,
    safe: &Safe,
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
        } => {
            let dbc = if let Some(dbc) = dbc {
                let path = Path::new(&dbc);
                if path.exists() {
                    if path.is_dir() {
                        return Err(eyre!("The path supplied refers to a directory.")
                            .suggestion("A file path must be specified for the DBC data."));
                    }
                    let dbc_data = std::fs::read_to_string(path)?;
                    sn_dbc::Dbc::from_hex(dbc_data.trim()).map_err(|e| {
                        eyre!(e.to_string()).suggestion(
                            "This file does not appear to have DBC data. \
                            Please select another file with valid hex-encoded DBC data.",
                        )
                    })?
                } else {
                    sn_dbc::Dbc::from_hex(&dbc)?
                }
            } else {
                let dbc_hex = get_from_arg_or_stdin(dbc, None)?;
                println!("{}", dbc_hex);
                sn_dbc::Dbc::from_hex(dbc_hex.trim())?
            };
            let the_name = safe
                .wallet_deposit(&wallet_url, name.as_deref(), &dbc)
                .await?;

            if OutputFmt::Pretty == output_fmt {
                println!(
                    "Spendable DBC deposited with name '{}' in wallet located at \"{}\"",
                    the_name, wallet_url
                );
            } else {
                println!("{}", serialise_output(&(wallet_url, the_name), output_fmt));
            }

            Ok(())
        }
        WalletSubCommands::Reissue {
            amount,
            from,
            public_key_hex,
        } => {
            let pk = if let Some(pk_hex) = public_key_hex.clone() {
                let pk_bytes = hex::decode(pk_hex)?;
                let pk_bytes: [u8; bls::PK_SIZE] = pk_bytes.try_into().map_err(|_| {
                    eyre!("Could not decode supplied public key").suggestion(
                        "Verify that this is a hex encoded BLS key. \
                            You can use the `keys create` command to see the format of the key.",
                    )
                })?;
                Some(bls::PublicKey::from_bytes(pk_bytes)?)
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
                if let Some(pk_hex) = public_key_hex {
                    println!("This DBC is owned by public key {}", pk_hex);
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
