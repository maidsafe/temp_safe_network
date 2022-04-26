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
    /// Create a new Wallet
    Create {},
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
    },
}

pub async fn wallet_commander(
    cmd: WalletSubCommands,
    output_fmt: OutputFmt,
    safe: &Safe,
) -> Result<()> {
    match cmd {
        WalletSubCommands::Create {} => {
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
            let dbc = dbc_from_hex(&dbc)?;
            let the_name = safe.wallet_deposit(&target, name.as_deref(), &dbc).await?;

            if OutputFmt::Pretty == output_fmt {
                println!(
                    "Spendable DBC deposited with name '{}' in Wallet located at \"{}\"",
                    the_name, target
                );
            } else {
                println!("{}", serialise_output(&(target, the_name), output_fmt));
            }

            Ok(())
        }
        WalletSubCommands::Reissue { amount, from } => {
            let dbc = safe.wallet_reissue(&from, &amount).await?;
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
