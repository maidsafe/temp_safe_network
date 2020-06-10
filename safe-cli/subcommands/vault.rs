// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use crate::operations::config::{
    read_config_settings, read_current_network_conn_info, retrieve_conn_info,
};
use crate::operations::vault::*;
use log::debug;
use std::path::PathBuf;
use structopt::StructOpt;

const VAULTS_DATA_FOLDER: &str = "baby-fleming-vaults";

const LOCAL_VAULT_DIR: &str = "local-vault";

#[derive(StructOpt, Debug)]
pub enum VaultSubCommands {
    #[structopt(name = "install")]
    /// Install latest safe-vault released version in the system
    Install {
        #[structopt(long = "vault-path")]
        /// Path where to install safe-vault executable (default ~/.safe/vault/). The SAFE_VAULT_PATH env var can also be used to set the path
        #[structopt(long = "vault-path", env = "SAFE_VAULT_PATH")]
        vault_path: Option<PathBuf>,
    },
    #[structopt(name = "join")]
    /// Join an already running network
    Join {
        /// Network to have the vault to join to
        network_name: Option<String>,
        #[structopt(long = "vault-path")]
        /// Path where to run safe-vault executable from (default ~/.safe/vault/). The SAFE_VAULT_PATH env var can also be used to set the path
        #[structopt(long = "vault-path", env = "SAFE_VAULT_PATH")]
        vault_path: Option<PathBuf>,
        /// Vebosity level for vaults logs
        #[structopt(short = "y", parse(from_occurrences))]
        verbosity: u8,
        /// Hardcoded contacts (endpoints) to be used to bootstrap to an already running network (this overrides any value passed as 'network_name').
        #[structopt(short = "h", long = "hcc")]
        hard_coded_contacts: Option<String>,
    },
    #[structopt(name = "run-baby-fleming")]
    /// Run vaults to form a local single-section SAFE network
    Run {
        /// Path where to run safe-vault executable from (default ~/.safe/vault/). The SAFE_VAULT_PATH env var can also be used to set the path
        #[structopt(long = "vault-path", env = "SAFE_VAULT_PATH")]
        vault_path: Option<PathBuf>,
        /// Vebosity level for vaults logs
        #[structopt(short = "y", parse(from_occurrences))]
        verbosity: u8,
        /// Interval in seconds between launching each of the vaults
        #[structopt(short = "i", long, default_value = "1")]
        interval: u64,
        /// IP to be used to launch the local vaults.
        #[structopt(long = "ip")]
        ip: Option<String>,
        /// Start authd and login with
        #[structopt(short = "t", long = "testing")]
        test: bool,
    },
    /// Shutdown all running vaults processes
    #[structopt(name = "killall")]
    Killall {
        /// Path of the safe-vault executable used to launch the processes with (default ~/.safe/vault/safe_vault). The SAFE_VAULT_PATH env var can be also used to set this path
        #[structopt(long = "vault-path", env = "SAFE_VAULT_PATH")]
        vault_path: Option<PathBuf>,
    },
    #[structopt(name = "update")]
    /// Update to latest safe-vault released version
    Update {
        #[structopt(long = "vault-path")]
        /// Path of the safe-vault executable to update (default ~/.safe/vault/). The SAFE_VAULT_PATH env var can be also used to set the path
        #[structopt(long = "vault-path", env = "SAFE_VAULT_PATH")]
        vault_path: Option<PathBuf>,
    },
}

pub fn vault_commander(cmd: Option<VaultSubCommands>) -> Result<(), String> {
    match cmd {
        Some(VaultSubCommands::Install { vault_path }) => vault_install(vault_path),
        Some(VaultSubCommands::Join {
            network_name,
            vault_path,
            verbosity,
            hard_coded_contacts,
        }) => {
            let network_contacts: Result<String, String> = if let Some(contacts) =
                hard_coded_contacts
            {
                let msg = format!("Joining network with contacts {}...", contacts);
                debug!("{}", msg);
                println!("{}", msg);
                Ok(contacts)
            } else {
                let contacts = if let Some(name) = network_name {
                    let (settings, _) = read_config_settings()?;
                    let msg = format!("Joining the '{}' network...", name);
                    debug!("{}", msg);
                    println!("{}", msg);
                    match settings.networks.get(&name) {
                        Some(config_location) => retrieve_conn_info(&name, config_location),
                        None => Err(format!("No network with name '{}' was found in the config. Please use the 'networks add' command to add it", name))
                    }
                } else {
                    let (_, contacts) = read_current_network_conn_info()?;
                    Ok(contacts)
                }?;

                let mut contacts_str = std::str::from_utf8(&contacts)
                    .map_err(|err| {
                        format!(
                            "Failed to parse network contact information from the config: {}",
                            err
                        )
                    })?
                    .to_string();

                contacts_str = contacts_str.replace("\"", "");
                let len_withoutcrlf = contacts_str.trim_end().len();
                contacts_str.truncate(len_withoutcrlf);
                debug!("Joining network with contacts {}...", contacts_str);

                Ok(contacts_str)
            };

            vault_join(vault_path, LOCAL_VAULT_DIR, verbosity, &network_contacts?)
        }
        Some(VaultSubCommands::Run {
            vault_path,
            verbosity,
            interval,
            ip,
            test,
        }) => vault_run(
            vault_path,
            VAULTS_DATA_FOLDER,
            verbosity,
            &interval.to_string(),
            ip,
            test,
        ),
        Some(VaultSubCommands::Killall { vault_path }) => vault_shutdown(vault_path),
        Some(VaultSubCommands::Update { vault_path }) => vault_update(vault_path),
        None => Err("Missing vault subcommand".to_string()),
    }
}
