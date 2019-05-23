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
pub enum FilesSubCommands {
    #[structopt(name = "add")]
    /// Add a file to a network document / container
    Add {
        /// The soure file location
        #[structopt(short = "s", long = "source")]
        source: String,
        /// desired file name
        #[structopt(long = "name")]
        name: String,
        /// desired file name
        #[structopt(short = "l", long = "link")]
        link: String,
    },
    #[structopt(name = "put")]
    /// Put a file onto the network
    Put {
        /// The soure file location
        #[structopt(short = "s", long = "source")]
        source: String,
        /// The recursively upload folders?
        #[structopt(short = "r", long = "recursive")]
        recursive: bool,
    },
    #[structopt(name = "sync")]
    /// Sync files to the network
    Sync {
        /// The soure file location
        #[structopt(short = "s", long = "source")]
        source: String,
        /// The recursively upload folders?
        #[structopt(short = "r", long = "recursive")]
        recursive: bool,
    },
}
