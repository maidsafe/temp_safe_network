// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

pub mod container;
pub mod files;
mod helpers;
pub mod keys;
pub mod pns;
pub mod safe_id;
pub mod wallet;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum SubCommands {
    #[structopt(name = "container")]
    /// Create a new SAFE Network account with the credentials provided
    Container {
        /// subcommands
        #[structopt(subcommand)]
        cmd: Option<container::ContainerSubCommands>,
    },
    #[structopt(name = "auth")]
    /// Authorise the CLI
    Auth {},
    #[structopt(name = "cat")]
    /// Read data on the network.
    Cat {
        /// The key to cat
        #[structopt(short = "k", long = "key")]
        key: String,
        /// Version of the resource to cat
        #[structopt(long = "version")]
        version: String,
    },
    #[structopt(name = "files")]
    /// Manage files on the network
    Files {
        /// subcommands
        #[structopt(subcommand)]
        cmd: Option<files::FilesSubCommands>,
    },
    #[structopt(name = "pns")]
    /// Manage public names on the network
    Pns {
        /// subcommands
        #[structopt(subcommand)]
        cmd: Option<pns::PnsSubCommands>,
    },
    #[structopt(name = "keys")]
    /// Manage keys on the network
    Keys {
        /// subcommands
        #[structopt(subcommand)]
        cmd: Option<keys::KeysSubCommands>,
    },
    #[structopt(name = "wallet")]
    /// Manage wallets on the network
    Wallet {
        /// subcommands
        #[structopt(subcommand)]
        cmd: Option<wallet::WalletSubCommands>,
    },
    #[structopt(name = "safe-id")]
    /// Manage identities on the network
    SafeId {
        /// subcommands
        #[structopt(subcommand)]
        cmd: Option<safe_id::SafeIdSubCommands>,
    },
}
