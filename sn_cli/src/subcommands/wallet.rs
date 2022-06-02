// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{
    helpers::{dbc_from_hex, dbc_to_hex, get_from_arg_or_stdin, serialise_output},
    OutputFmt,
};
use color_eyre::Result;
use sn_api::Safe;
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
        /// A hex encoded DBC to deposit
        #[structopt(long = "dbc")]
        dbc: Option<String>,
    },
    #[structopt(name = "reissue")]
    /// Reissue a DBC from a wallet to a SafeKey
    Reissue {
        /// The amount to reissue
        amount: String,
        /// The URL of wallet to reissue from
        #[structopt(long = "from")]
        from: String,
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
                dbc_from_hex(&dbc)?
            } else {
                let dbc_hex = get_from_arg_or_stdin(dbc, None)?;
                println!("{}", dbc_hex);
                dbc_from_hex(dbc_hex.trim())?
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
        WalletSubCommands::Reissue { amount, from } => {
            let dbc = safe.wallet_reissue(&from, &amount, None).await?;
            let dbc_hex = dbc_to_hex(&dbc)?;

            if OutputFmt::Pretty == output_fmt {
                println!("Success. Reissued DBC with {} safecoins:", amount);
                println!("-------- DBC DATA --------");
                println!("{}", dbc_hex);
                println!("--------------------------");
            } else {
                println!("{}", dbc_hex);
            }

            Ok(())
        }
    }
}
