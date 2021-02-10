// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use crate::operations::config::{
    add_network_to_config, print_networks_settings, read_config_settings,
    read_current_network_conn_info, remove_network_from_config, write_current_network_conn_info,
    NetworkInfo,
};
use anyhow::{bail, Result};
use log::debug;
use std::{collections::HashSet, iter::FromIterator, net::SocketAddr};
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
    /// Add a network to the CLI config using an existing network configuration file
    Add {
        /// Network name. If the network already exists in the config, it will be updated with the new location for the network connection information
        network_name: String,
        /// Location of the network connection information. If this argument is not passed, it takes current network connection information and caches it
        config_location: Option<String>,
    },
    #[structopt(name = "set")]
    /// Set the list of IP addrsses (and port numbers) for a network in the CLI config
    Set {
        /// Network name. If the network doesn't currently exists a new one will be addded to the config, otherwise it's network connection information will be updated
        network_name: String,
        /// List of IP addresses (and port numbers) to set as the contact list for this new network, e.g. 127.0.0.1:12000
        addresses: Vec<SocketAddr>,
    },
    #[structopt(name = "remove")]
    /// Remove a network from the CLI config
    Remove {
        /// Network to remove
        network_name: String,
    },
}

pub fn networks_commander(cmd: Option<NetworksSubCommands>) -> Result<()> {
    match cmd {
        Some(NetworksSubCommands::Switch { network_name }) => {
            let (settings, _) = read_config_settings()?;
            let msg = format!("Switching to '{}' network...", network_name);
            debug!("{}", msg);
            println!("{}", msg);
            let contacts = settings.get_net_info(&network_name)?;

            write_current_network_conn_info(&contacts)?;
            println!(
                "Successfully switched to '{}' network in your system!",
                network_name
            );
            println!("If you need write access to the '{}' network, you'll need to restart authd, unlock a Safe and re-authorise the CLI again", network_name);
        }
        Some(NetworksSubCommands::Check {}) => {
            let (settings, _) = read_config_settings()?;
            println!("Checking current setup network connection information...");
            let (conn_info_file_path, current_conn_info) = read_current_network_conn_info()?;
            let mut matched_network = None;
            for (network_name, network_info) in settings.networks.iter() {
                if network_info.matches(&current_conn_info) {
                    matched_network = Some(network_name);
                    break;
                }
            }

            println!();
            match matched_network {
                Some(name) => {
                    println!("'{}' network matched!", name);
                    println!("Current set network connection information at '{}' matches '{}' network as per current config", conn_info_file_path.display(), name);
                },
                None => println!("Current network setup in your system doesn't match any of your networks in the CLI config. Use 'networks switch' command to switch to any of them")
            }
        }
        Some(NetworksSubCommands::Add {
            network_name,
            config_location,
        }) => {
            let net_info = add_network_to_config(
                &network_name,
                config_location.map(NetworkInfo::ConnInfoUrl),
            )?;
            println!(
                "Network '{}' was added to the list. Connection information is located at '{}'",
                network_name, net_info
            );
        }
        Some(NetworksSubCommands::Set {
            network_name,
            addresses,
        }) => {
            if addresses.is_empty() {
                bail!("Please provide the bootstrapping address/es");
            }
            let addresses = HashSet::from_iter(addresses);
            let net_info =
                add_network_to_config(&network_name, Some(NetworkInfo::Addresses(addresses)))?;
            println!(
                "Network '{}' was added to the list. Contacts: '{}'",
                network_name, net_info
            );
        }
        Some(NetworksSubCommands::Remove { network_name }) => {
            remove_network_from_config(&network_name)?
        }
        None => print_networks_settings()?,
    }

    Ok(())
}
