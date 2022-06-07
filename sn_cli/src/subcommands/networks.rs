// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::operations::config::{Config, NetworkInfo};
use color_eyre::Result;
use std::path::PathBuf;
use structopt::StructOpt;
use tracing::debug;
use url::Url;

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
        /// Local or Remote location to fetch the NetworkPrefixMap
        prefix_location: String,
    },
    #[structopt(name = "remove")]
    /// Remove a network from the CLI config
    Remove {
        /// Network to remove
        network_name: String,
    },
}

pub async fn networks_commander(
    cmd: Option<NetworksSubCommands>,
    config: &mut Config,
) -> Result<()> {
    match cmd {
        Some(NetworksSubCommands::Switch { network_name }) => {
            let msg = format!("Switching to '{}' network...", network_name);
            debug!("{}", msg);
            println!("{}", msg);
            config.switch_to_network(&network_name).await?;
            println!(
                "Successfully switched to '{}' network in your system!",
                network_name
            );
        }
        Some(NetworksSubCommands::Check {}) => {
            println!("Checking current setup network connection information...");
            let prefix_map = config.read_default_prefix_map().await?;
            let mut matched_network = None;
            for (network_name, network_info) in config.networks_iter() {
                if network_info.matches(&prefix_map.genesis_key()).await {
                    matched_network = Some(network_name);
                    break;
                }
            }

            println!();
            match matched_network {
                Some(name) => {
                    println!("'{}' network matched!", name);
                    println!(
                        "The default NetworkPrefixMap matches '{}' network as per current config",
                        name
                    );
                }
                None => {
                    // should not be possible due to sync?
                    println!("Current network setup in your system doesn't match any of your networks in the CLI config. Use 'networks switch' command to switch to any of them")
                }
            }
        }
        Some(NetworksSubCommands::Add {
            network_name,
            prefix_location,
        }) => {
            let net_info = if Url::parse(prefix_location.as_str()).is_ok() {
                config
                    .add_network(&network_name, NetworkInfo::Remote(prefix_location, None))
                    .await?
            } else {
                let path = PathBuf::from(prefix_location);
                config
                    .add_network(&network_name, NetworkInfo::Local(path, None))
                    .await?
            };
            println!(
                "Network '{}' was added to the list. Connection information is located at '{}'",
                network_name, net_info
            );
        }
        Some(NetworksSubCommands::Remove { network_name }) => {
            config.remove_network(&network_name).await?
        }
        None => config.print_networks().await,
    }

    Ok(())
}
