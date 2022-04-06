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
use color_eyre::{eyre::eyre, Result};
use sn_api::Safe;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum WalletSubCommands {
    #[structopt(name = "create")]
    /// Create a new Wallet
    Create {
        /// Preload with a DBC
        #[structopt(long = "preload")]
        preload: Option<String>,
    },
    #[structopt(name = "balance")]
    /// Query a Wallet's total balance
    Balance {
        /// The target Wallet to check the total balance
        target: Option<String>,
    },
    #[structopt(name = "deposit")]
    /// Deposit a spendable DBC into a Wallet
    Deposit {
        /// The target Wallet to deposit the spendable DBC on
        target: String,
        /// The name to give this spendable DBC
        #[structopt(long = "name")]
        name: Option<String>,
        /// The DBC to desposit (hex encoded)
        dbc: String,
    },
    #[structopt(name = "reissue")]
    /// Reissue a DBC from a Wallet to a SafeKey
    Reissue {
        /// Number of safecoins to reissue
        amount: String,
        /// Source Wallet URL
        #[structopt(long = "from")]
        from: String,
        /// The receiving SafeKey URL or public key, otherwise pulled from stdin if not provided
        #[structopt(long = "to")]
        to: Option<String>,
    },
}

pub async fn wallet_commander(
    cmd: WalletSubCommands,
    output_fmt: OutputFmt,
    safe: &Safe,
) -> Result<()> {
    match cmd {
        WalletSubCommands::Create { preload: _ } => {
            // Create wallet
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
                Some("...awaiting Wallet address/location from STDIN stream..."),
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
        WalletSubCommands::Deposit { target, name, dbc } => {
            let mut dbc_str =
                hex::decode(dbc).map_err(|err| eyre!("Couldn't hex-decode DBC: {:?}", err))?;

            dbc_str.reverse();
            let dbc = bincode::deserialize(&dbc_str)
                .map_err(|err| eyre!("Couldn't deserialise DBC: {:?}", err))?;

            let the_name = safe.wallet_deposit(&target, name.as_deref(), &dbc).await?;

            if OutputFmt::Pretty == output_fmt {
                println!(
                    "Spendable DBC deposited with name '{}' in Wallet located at \"{}\"",
                    the_name, target
                );
            } else {
                println!("{}", target);
            }

            Ok(())
        }
        WalletSubCommands::Reissue { amount, from, to } => {
            let destination = get_from_arg_or_stdin(
                to,
                Some("...awaiting destination Wallet/SafeKey URL, or public key, from STDIN stream..."),
            )?;

            let dbc = safe.wallet_reissue(&from, &amount, &destination).await?;

            if OutputFmt::Pretty == output_fmt {
                println!("Success. Reissued DBC: {:?}", dbc);
            } else {
                println!("{:?}", dbc)
            }

            Ok(())
        }
    }
}
