// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

pub mod auth;
pub mod cat;
pub mod config;
pub mod container;
pub mod dog;
pub mod files;
mod helpers;
pub mod keys;
pub mod networks;
pub mod nrs;
pub mod safe_id;
pub mod setup;
pub mod update;
pub mod wallet;
pub mod xorurl;

use structopt::StructOpt;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum OutputFmt {
    Pretty,
    Json,
    JsonCompact,
    Yaml,
}

impl std::str::FromStr for OutputFmt {
    type Err = String;
    fn from_str(str: &str) -> Result<Self, String> {
        match str {
            "json" => Ok(Self::Json),
            "jsoncompact" => Ok(Self::JsonCompact),
            "yaml" => Ok(Self::Yaml),
            other => {
                Err(format!(
                    "Output serialisation format '{}' not supported. Supported values are json, jsoncompact, and yaml",
                    other
                ))
            }
        }
    }
}

#[derive(StructOpt, Debug)]
pub enum SubCommands {
    #[structopt(name = "config")]
    /// CLI config settings
    Config {
        /// subcommands
        #[structopt(subcommand)]
        cmd: Option<config::ConfigSubCommands>,
    },
    #[structopt(name = "networks")]
    /// Switch between SAFE networks
    Networks {
        /// subcommands
        #[structopt(subcommand)]
        cmd: Option<networks::NetworksSubCommands>,
    },
    #[structopt(name = "auth")]
    /// Authorise the SAFE CLI and interact with a remote Authenticator daemon
    Auth {
        /// subcommands
        #[structopt(subcommand)]
        cmd: Option<auth::AuthSubCommands>,
    },
    // [structopt(name = "container")]
    // /// Create a new SAFE Network account with the credentials provided
    // Container(container::ContainerSubCommands),
    #[structopt(name = "cat")]
    /// Read data on the SAFE Network
    Cat(cat::CatCommands),
    #[structopt(name = "dog")]
    /// Inspect data on the SAFE Network providing only metadata information about the content
    Dog(dog::DogCommands),
    #[structopt(name = "files")]
    /// Manage files on the SAFE Network
    Files(files::FilesSubCommands),
    #[structopt(name = "setup")]
    /// Perform setup tasks
    Setup(setup::SetupSubCommands),
    #[structopt(name = "keypair")]
    /// Generate a key pair without creating and/or storing a SafeKey on the network
    Keypair {},
    #[structopt(name = "nrs")]
    /// Manage public names on the SAFE Network
    Nrs(nrs::NrsSubCommands),
    #[structopt(name = "keys")]
    /// Manage keys on the SAFE Network
    Keys(keys::KeysSubCommands),
    #[structopt(name = "wallet")]
    /// Manage wallets on the SAFE Network
    Wallet(wallet::WalletSubCommands),
    #[structopt(name = "xorurl")]
    /// Obtain the XOR-URL of data without uploading it to the network
    Xorurl {
        /// subcommands
        #[structopt(subcommand)]
        cmd: Option<xorurl::XorurlSubCommands>,
        /// The source file/folder local path
        location: Option<String>,
        /// Recursively crawl folders and files found in the location
        #[structopt(short = "r", long = "recursive")]
        recursive: bool,
    },
    // #[structopt(name = "safe-id")]
    // /// Manage identities on the SAFE Network
    // SafeId(safe_id::SafeIdSubCommands),
    #[structopt(name = "update")]
    /// Update the application to the latest available version
    Update {},
}
