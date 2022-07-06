// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum SafeIdSubCommands {
    #[clap(name = "create")]
    /// Create a new SafeId
    Create {
        /// The SafeId name
        #[clap(long = "name")]
        name: String,
        /// The SafeId surname
        #[clap(long = "surname")]
        surname: String,
        /// The SafeId email
        #[clap(long = "email")]
        email: String,
        /// The SafeId website
        #[clap(long = "website")]
        website: String,
        /// The SafeId wallet
        #[clap(long = "wallet")]
        wallet: String,
    },
    #[clap(name = "update")]
    /// Manage files on the network
    Update {
        /// The SafeId name
        #[clap(long = "name")]
        name: String,
        /// The SafeId surname
        #[clap(long = "surname")]
        surname: String,
        /// The SafeId email
        #[clap(long = "email")]
        email: String,
        /// The SafeId website
        #[clap(long = "website")]
        website: String,
        /// The SafeId wallet
        #[clap(long = "wallet")]
        wallet: String,
    },
}
