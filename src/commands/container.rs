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
pub enum ContainerSubCommands {
    #[structopt(name = "create")]
    /// Create a network container
    Create {
        /// Desired container name
        #[structopt(long = "name")]
        name: String,
        /// Location to add into the container
        #[structopt(long = "link")]
        link: String,
        /// Publish the container
        #[structopt(short = "p", long = "publish")]
        publish: bool,
        /// Do not require new versions for container edits
        #[structopt(short = "n", long = "non_versioned")]
        non_versioned: bool,
    },
    #[structopt(name = "add")]
    /// Add a container to another container on the network
    Add {
        /// Desired container name
        #[structopt(long = "name")]
        name: String,
        /// Location to add into the container
        #[structopt(long = "link")]
        link: String,
    },
    #[structopt(name = "edit")]
    /// Edit files to the network
    Edit {
        /// The key to edit
        #[structopt(short = "k", long = "key")]
        key: String,
        /// The value to edit
        #[structopt(short = "d", long = "value")]
        value: bool,
    },
}
