// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

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
        #[structopt(short = "e", long = "non_versioned")]
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
