// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

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
pub mod seq;
pub mod setup;
pub mod update;
pub mod xorurl;

use structopt::{clap::AppSettings, StructOpt};

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
    #[structopt(
        name = "config",
        no_version,
        global_settings(&[AppSettings::DisableVersion]),
    )]
    /// CLI config settings
    Config {
        /// subcommands
        #[structopt(subcommand)]
        cmd: Option<config::ConfigSubCommands>,
    },
    #[structopt(
        name = "networks",
        no_version,
        global_settings(&[AppSettings::DisableVersion]),
    )]
    /// Switch between SAFE networks
    Networks {
        /// subcommands
        #[structopt(subcommand)]
        cmd: Option<networks::NetworksSubCommands>,
    },
    #[structopt(
        name = "cat",
        no_version,
        global_settings(&[AppSettings::DisableVersion]),
    )]
    /// Read data on the SAFE Network
    Cat(cat::CatCommands),
    #[structopt(
        name = "dog",
        no_version,
        global_settings(&[AppSettings::DisableVersion]),
    )]
    /// Inspect data on the SAFE Network providing only metadata information about the content
    Dog(dog::DogCommands),
    #[structopt(
        name = "files",
        no_version,
        global_settings(&[AppSettings::DisableVersion]),
    )]
    /// Manage files on the SAFE Network
    Files(files::FilesSubCommands),
    #[structopt(
        name = "setup",
        no_version,
        global_settings(&[AppSettings::DisableVersion]),
    )]
    /// Perform setup tasks
    Setup(setup::SetupSubCommands),
    #[structopt(
        name = "nrs",
        no_version,
        global_settings(&[AppSettings::DisableVersion]),
    )]
    /// Manage public names on the SAFE Network
    Nrs(nrs::NrsSubCommands),
    #[structopt(
        name = "keys",
        no_version,
        global_settings(&[AppSettings::DisableVersion]),
    )]
    /// Manage keys on the SAFE Network
    Keys(keys::KeysSubCommands),
    /// Obtain the XOR-URL of data without uploading it to the network, or decode XOR-URLs
    Xorurl {
        /// subcommands
        #[structopt(subcommand)]
        cmd: Option<xorurl::XorurlSubCommands>,
        /// The source file/folder local path
        location: Option<String>,
        /// Recursively crawl folders and files found in the location
        #[structopt(short = "r", long = "recursive")]
        recursive: bool,
        /// Follow symlinks
        #[structopt(short = "l", long = "follow-links")]
        follow_links: bool,
    },
    #[structopt(
        name = "seq",
        no_version,
        global_settings(&[AppSettings::DisableVersion]),
    )]
    /// Manage Sequences on the Safe Network
    Seq(seq::SeqSubCommands),
    // #[structopt(name = "safe-id")]
    // /// Manage identities on the Safe Network
    // SafeId(safe_id::SafeIdSubCommands),
    #[structopt(
        name = "update",
        no_version,
        global_settings(&[AppSettings::DisableVersion]),
    )]
    /// Update the application to the latest available version
    Update {},
    #[structopt(
        name = "node",
        no_version,
        global_settings(&[AppSettings::DisableVersion]),
    )]
    /// Commands to manage Safe Network Nodes
    Node {
        /// subcommands
        #[structopt(subcommand)]
        cmd: Option<node::NodeSubCommands>,
    },
}
