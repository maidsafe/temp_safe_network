// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use crate::operations::vault::*;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum VaultSubCommands {
    #[structopt(name = "install")]
    /// Install latest safe-vault released version in the system.
    Install {
        #[structopt(long = "vault-path")]
        /// Path where to install safe-vault executable (default ~/.safe/vault/)
        vault_path: Option<String>,
    },
    #[structopt(name = "run-baby-fleming")]
    /// Run vaults to form a local single-section SAFE network
    Run {
        #[structopt(long = "vault-path")]
        /// Path where to run safe-vault executable from (default ~/.safe/vault/)
        vault_path: Option<String>,
    },
}

pub fn vault_commander(cmd: Option<VaultSubCommands>) -> Result<(), String> {
    match cmd {
        Some(VaultSubCommands::Install { vault_path }) => vault_install(vault_path),
        Some(VaultSubCommands::Run { vault_path }) => vault_run(vault_path, "baby-fleming-vaults"),
        None => Err("Missing vault subcommand".to_string()),
    }
}
