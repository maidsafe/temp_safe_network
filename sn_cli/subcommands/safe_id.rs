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
pub enum SafeIdSubCommands {
    #[structopt(name = "create")]
    /// Create a new SafeId
    Create {
        /// The SafeId name
        #[structopt(long = "name")]
        name: String,
        /// The SafeId surname
        #[structopt(long = "surname")]
        surname: String,
        /// The SafeId email
        #[structopt(long = "email")]
        email: String,
        /// The SafeId website
        #[structopt(long = "website")]
        website: String,
        /// The SafeId wallet
        #[structopt(long = "wallet")]
        wallet: String,
    },
    #[structopt(name = "update")]
    /// Manage files on the network
    Update {
        /// The SafeId name
        #[structopt(long = "name")]
        name: String,
        /// The SafeId surname
        #[structopt(long = "surname")]
        surname: String,
        /// The SafeId email
        #[structopt(long = "email")]
        email: String,
        /// The SafeId website
        #[structopt(long = "website")]
        website: String,
        /// The SafeId wallet
        #[structopt(long = "wallet")]
        wallet: String,
    },
}
