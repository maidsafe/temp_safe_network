// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use clap::{AppSettings, Subcommand};

pub mod cat;
pub mod config;
pub mod dog;
pub mod files;
mod files_get;
mod helpers;
pub mod keys;
pub mod networks;
pub mod node;
pub mod nrs;
pub mod safe_id;
pub mod setup;
pub mod update;
pub mod wallet;
pub mod xorurl;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
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

#[derive(Subcommand, Debug)]
pub enum SubCommands {
    #[clap(
        name = "config",
        global_settings(&[AppSettings::DisableVersion])
    )]
    /// CLI config settings
    Config {
        /// subcommands
        #[clap(subcommand)]
        cmd: Option<config::ConfigSubCommands>,
    },
    #[clap(
        name = "networks",
        global_settings(&[AppSettings::DisableVersion]),
    )]
    /// Switch between SAFE networks
    Networks {
        /// subcommands
        #[clap(subcommand)]
        cmd: Option<networks::NetworksSubCommands>,
    },
    #[clap(
        name = "cat",
        global_settings(&[AppSettings::DisableVersion]),
    )]
    /// Read data on the SAFE Network
    Cat(cat::CatCommands),
    #[clap(
        name = "dog",
        global_settings(&[AppSettings::DisableVersion]),
    )]
    /// Inspect data on the SAFE Network providing only metadata information about the content
    Dog(dog::DogCommands),
    #[clap(name = "files", subcommand, global_settings(&[AppSettings::DisableVersion]))]
    /// Manage files on the SAFE Network
    Files(files::FilesSubCommands),
    #[clap(name = "setup", subcommand, global_settings(&[AppSettings::DisableVersion]))]
    /// Perform setup tasks
    Setup(setup::SetupSubCommands),
    #[clap(name = "nrs", subcommand, global_settings(&[AppSettings::DisableVersion]))]
    /// Manage public names on the SAFE Network
    Nrs(nrs::NrsSubCommands),
    #[clap(name = "keys", subcommand, global_settings(&[AppSettings::DisableVersion]))]
    /// Manage keys on the SAFE Network
    Keys(keys::KeysSubCommands),
    #[clap(name = "wallet", subcommand, global_settings(&[AppSettings::DisableVersion]))]
    /// Manage wallets on the SAFE Network
    Wallet(wallet::WalletSubCommands),
    /// Obtain the XOR-URL of data without uploading it to the network, or decode XOR-URLs
    Xorurl {
        /// subcommands
        #[clap(subcommand)]
        cmd: Option<xorurl::XorurlSubCommands>,
        /// The source file/folder local path
        location: Option<String>,
        /// Recursively crawl folders and files found in the location
        #[clap(short = 'r', long = "recursive")]
        recursive: bool,
        /// Follow symlinks
        #[clap(short = 'l', long = "follow-links")]
        follow_links: bool,
    },
    #[clap(name = "update", global_settings(&[AppSettings::DisableVersion]))]
    /// Update the application to the latest available version
    Update {
        /// Remove prompt to confirm the update.
        #[clap(short = 'y', long = "no-confirm")]
        no_confirm: bool,
    },
    #[clap(name = "node", global_settings(&[AppSettings::DisableVersion]))]
    /// Commands to manage Safe Network Nodes
    Node {
        /// subcommands
        #[clap(subcommand)]
        cmd: Option<node::NodeSubCommands>,
    },
}
