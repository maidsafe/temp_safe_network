// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::operations::config::{
    add_network_to_config, print_networks_settings, read_config_settings,
    read_current_network_conn_info, remove_network_from_config, retrieve_conn_info,
    write_current_network_conn_info,
};
use log::debug;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum NetworksSubCommands {
    #[structopt(name = "switch")]
    /// Switch to a different SAFE network
    Switch {
        /// Network to switch to
        network_name: String,
    },
    #[structopt(name = "check")]
    /// Check current network configuration and try to match it to networks in the CLI config
    Check {},
    #[structopt(name = "add")]
    /// Add a network to the CLI config
    Add {
        /// Network name
        network_name: String,
        /// Location of the network connection information. If this argument is not passed, it takes current network connection information and caches it
        config_location: Option<String>,
    },
    #[structopt(name = "remove")]
    /// Remove a network from the CLI config
    Remove {
        /// Network to remove
        network_name: String,
    },
}

pub fn networks_commander(cmd: Option<NetworksSubCommands>) -> Result<(), String> {
    match cmd {
        Some(NetworksSubCommands::Switch { network_name }) => {
            let (settings, _) = read_config_settings()?;
            let msg = format!("Switching to '{}' network...", network_name);
            debug!("{}", msg);
            println!("{}", msg);
            match settings.networks.get(&network_name) {
                Some(config_location) => {
                    let conn_info = retrieve_conn_info(&network_name, config_location)?;
                    write_current_network_conn_info(&conn_info)?;
                    println!("Successfully switched to '{}' network in your system!", network_name);
                    println!("You'll need to login again, and re-authorise the CLI, if you need write access to the '{}' network", network_name);
                },
                None => return Err(format!("No network with name '{}' was found in the config. Please use the 'config add network' command to add it", network_name))
            }
        }
        Some(NetworksSubCommands::Check {}) => {
            let (settings, _) = read_config_settings()?;
            println!("Checking current setup network connection information...");
            let (conn_info_file_path, current_conn_info) = read_current_network_conn_info()?;
            let mut matched_network = None;
            for (network_name, config_location) in settings.networks.iter() {
                match retrieve_conn_info(&network_name, config_location) {
                    Ok(conn_info) => {
                        if current_conn_info == conn_info {
                            matched_network = Some(network_name);
                            break;
                        }
                    }
                    Err(err) => println!("Ignoring '{}' network: {}", network_name, err),
                }
            }
            println!();
            match matched_network {
                Some(name) => println!("'{}' network matched. Current set network connection information at '{}' matches '{}' network as per current config", name, conn_info_file_path.display(), name),
                None => println!("Current network setup in your system doesn't match any of your networks in the CLI config. Use 'networks switch' command to switch to any of them")
            }
        }
        Some(NetworksSubCommands::Add {
            network_name,
            config_location,
        }) => add_network_to_config(&network_name, config_location)?,
        Some(NetworksSubCommands::Remove { network_name }) => {
            remove_network_from_config(&network_name)?
        }
        None => print_networks_settings()?,
    }

    Ok(())
}
