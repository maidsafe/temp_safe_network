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
pub enum PnsSubCommands {
    #[structopt(name = "add")]
    /// Add a subname to a public name
    Add {
        /// Set the sub name as default for this public name
        #[structopt(long = "default")]
        default: String,
        /// The safe:// url to add
        #[structopt(long = "link")]
        link: String,
        /// The name to give this key
        #[structopt(long = "name")]
        name: String,
    },
    #[structopt(name = "create")]
    /// Create a new public name
    Create {},
    #[structopt(name = "remove")]
    /// Add a subname to a public name
    Remove {
        /// Set the sub name as default for this public name
        #[structopt(long = "default")]
        default: String,
        /// The name to give this key
        #[structopt(long = "name")]
        name: String,
    },
}
