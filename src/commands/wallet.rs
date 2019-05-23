// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

// use crate::cli_helpers::*;

// use log::{debug, warn};
// use std::env;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
pub enum WalletSubCommands {
    #[structopt(name = "add")]
    /// Add a wallet to another document
    Add {
        /// The safe:// url to add
        #[structopt(long = "link")]
        link: String,
        /// The name to give this wallet
        #[structopt(long = "name")]
        name: String,
    },
    #[structopt(name = "balance")]
    /// Query a new Wallet or PublicKeys CoinBalance
    Balance {},
    #[structopt(name = "check-tx")]
    /// Check the status of a given transaction
    CheckTx {},
    #[structopt(name = "create")]
    /// Create a new Wallet/CoinBalance
    Create {},
    #[structopt(name = "sweep")]
    /// Move all coins within a wallet to a given balance
    Sweep {
        /// The source wallet for funds
        #[structopt(long = "from")]
        from: String,
        /// The receiving wallet/ballance
        #[structopt(long = "to")]
        to: String,
    },
    #[structopt(name = "transfer")]
    /// Manage files on the network
    Transfer {
        /// The safe:// url to add
        #[structopt(long = "amount")]
        amount: String,
        /// The source wallet / balance for funds
        #[structopt(long = "from")]
        from: String,
        /// The receiving wallet/ballance
        #[structopt(long = "to")]
        to: String,
    },
}
