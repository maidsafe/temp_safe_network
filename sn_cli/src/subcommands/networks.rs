// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::operations::config::{Config, NetworkInfo};
use clap::Subcommand;
use color_eyre::Result;
use std::path::PathBuf;
use tracing::debug;
use url::Url;

#[derive(Subcommand, Debug)]
pub enum NetworksSubCommands {
    #[clap(name = "switch")]
    /// Switch to a different SAFE network
    Switch {
        /// Network to switch to
        network_name: String,
    },
    #[clap(name = "check")]
    /// Check where the default hardlink points and try to match it to the networks in the CLI config
    Check {},
    #[clap(name = "add")]
    /// Add a network to the CLI config using an existing network map
    Add {
        /// Network name. If network_name already exists, then it's updated with the new network map
        network_name: String,
        /// Local path or a remote URL to fetch the network map from
        prefix_location: String,
    },
    #[clap(name = "remove")]
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
            println!("Checking current Network Map...");
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
                        "The default Network Map matches '{}' network as per current config",
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
                "Network '{}' was added to the list. Network Map is located at '{}'",
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
