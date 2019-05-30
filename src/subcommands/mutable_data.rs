// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum MutableDataSubCommands {
    #[structopt(name = "create")]
    /// Create a new MutableData
    Create {
        /// The name to give this MutableData. Randomly generated, if none provided
        #[structopt(long = "name")]
        name: Option<String>,
        /// The tag number to assign this MutableData. Randomly generated, if none provided.
        #[structopt(long = "tag")]
        tag: Option<u64>,
        /// Comma separated permissions: read, insert, update, delete, permissions. If none, provided all will assigned by default.
        #[structopt(long = "permissions")]
        permissions: Option<String>,
        /// Do you need to track data entry sequences?
        #[structopt(long = "sequenced")]
        sequenced: bool,
        // Data entry
        // #[structopt(long = "data")]
        // data: Option<String>,
    },
}
