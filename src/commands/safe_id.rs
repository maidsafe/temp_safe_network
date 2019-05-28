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
pub enum SafeIdSubCommands {
    #[structopt(name = "create")]
    /// Create a new Wallet/CoinBalance
    Create {
        /// The SafeId name
        #[structopt(long = "name")]
        name: String,
        /// The SafeId surname
        #[structopt(long = "surname")]
        surname: String,
        /// The SafeId email
        #[structopt(long = "email")]
        email: String,
        /// The SafeId website
        #[structopt(long = "website")]
        website: String,
        /// The SafeId wallet
        #[structopt(long = "wallet")]
        wallet: String,
    },
    #[structopt(name = "update")]
    /// Manage files on the network
    Update {
        /// The SafeId name
        #[structopt(long = "name")]
        name: String,
        /// The SafeId surname
        #[structopt(long = "surname")]
        surname: String,
        /// The SafeId email
        #[structopt(long = "email")]
        email: String,
        /// The SafeId website
        #[structopt(long = "website")]
        website: String,
        /// The SafeId wallet
        #[structopt(long = "wallet")]
        wallet: String,
    },
}
