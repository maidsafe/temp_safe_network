// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

// #[cfg(not(any(feature = "fake-auth", feature = "scl-mock")))]
pub mod auth;
pub mod cat;
pub mod container;
pub mod dog;
// #[cfg(any(feature = "fake-auth", feature = "scl-mock"))]
// pub mod fake_auth;
pub mod files;
mod helpers;
pub mod keys;
pub mod nrs;
pub mod safe_id;
pub mod update;
pub mod wallet;

// #[cfg(not(any(feature = "fake-auth", feature = "scl-mock")))]
use auth::AuthSubCommands;
// #[cfg(any(feature = "fake-auth", feature = "scl-mock"))]
// pub use fake_auth::{self as auth, AuthSubCommands};

use structopt::StructOpt;

#[derive(PartialEq, Clone, Copy)]
pub enum OutputFmt {
    Pretty,
    Json,
}

#[derive(StructOpt, Debug)]
pub enum SubCommands {
    #[structopt(name = "auth")]
    /// Authorise the CLI
    Auth {
        /// subcommands
        #[structopt(subcommand)]
        cmd: Option<AuthSubCommands>,
        #[structopt(long = "port", raw(global = "true"))]
        /// Port number of the Authenticator where to send requests to. If not provided, default port 33000 is assumed.
        port: Option<u16>,
    },
    #[structopt(name = "container")]
    /// Create a new SAFE Network account with the credentials provided
    Container {
        /// subcommands
        #[structopt(subcommand)]
        cmd: Option<container::ContainerSubCommands>,
    },
    #[structopt(name = "cat")]
    /// Read data on the SAFE Network.
    Cat(cat::CatCommands),
    #[structopt(name = "dog")]
    /// Inspect data on the SAFE Network providing only metadata information about the content
    Dog(dog::DogCommands),
    #[structopt(name = "files")]
    /// Manage files on the SAFE Network
    Files {
        /// subcommands
        #[structopt(subcommand)]
        cmd: Option<files::FilesSubCommands>,
    },
    #[structopt(name = "keypair")]
    /// Generate a key pair without creating and/or storing a SafeKey on the network
    Keypair {},
    #[structopt(name = "nrs")]
    /// Manage public names on the SAFE Network
    Nrs {
        /// subcommands
        #[structopt(subcommand)]
        cmd: Option<nrs::NrsSubCommands>,
    },
    #[structopt(name = "keys")]
    /// Manage keys on the SAFE Network
    Keys {
        /// subcommands
        #[structopt(subcommand)]
        cmd: Option<keys::KeysSubCommands>,
    },
    #[structopt(name = "wallet")]
    /// Manage wallets on the SAFE Network
    Wallet {
        /// subcommands
        #[structopt(subcommand)]
        cmd: Option<wallet::WalletSubCommands>,
    },
    #[structopt(name = "safe-id")]
    /// Manage identities on the SAFE Network
    SafeId {
        /// subcommands
        #[structopt(subcommand)]
        cmd: Option<safe_id::SafeIdSubCommands>,
    },
    #[structopt(name = "update")]
    /// Update the application to the latest available version
    Update {},
}
