// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::operations::config::{Config, NetworkInfo};
use color_eyre::{eyre::bail, eyre::eyre, Result};
use sn_api::PublicKey;
use std::collections::BTreeSet;
use std::net::SocketAddr;
use structopt::StructOpt;
use tracing::debug;

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
        /// The genesis key for the network you want to join. The genesis key is either generated
        /// by the first node of the network, or it's generated before the launch and supplied to
        /// the first node. You should use the hex string representation of the key.
        genesis_key_hex: String,
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
            println!("If you need write access to the '{}' network, you'll need to restart authd (safe auth restart), unlock a Safe and re-authorise the CLI again", network_name);
        }
        Some(NetworksSubCommands::Check {}) => {
            println!("Checking current setup network connection information...");
            let (node_config_path, current_node_config) = config.read_current_node_config().await?;
            let mut matched_network = None;
            for (network_name, network_info) in config.networks_iter() {
                if network_info.matches(&current_node_config).await {
                    matched_network = Some(network_name);
                    break;
                }
            }

            println!();
            match matched_network {
                Some(name) => {
                    println!("'{}' network matched!", name);
                    println!("Current set network connection information at '{}' matches '{}' network as per current config", node_config_path.display(), name);
                },
                None => println!("Current network setup in your system doesn't match any of your networks in the CLI config. Use 'networks switch' command to switch to any of them")
            }
        }
        Some(NetworksSubCommands::Add {
            network_name,
            config_location,
        }) => {
            let net_info = config
                .add_network(
                    &network_name,
                    config_location.map(NetworkInfo::ConnInfoLocation),
                )
                .await?;
            println!(
                "Network '{}' was added to the list. Connection information is located at '{}'",
                network_name, net_info
            );
        }
        Some(NetworksSubCommands::Set {
            network_name,
            genesis_key_hex,
            addresses,
        }) => {
            if addresses.is_empty() {
                bail!("Please provide the bootstrapping address/es");
            }
            let mut set = BTreeSet::new();
            for address in addresses {
                set.insert(address);
            }

            let genesis_key = PublicKey::bls_from_hex(&genesis_key_hex)?
                .bls()
                .ok_or_else(|| eyre!("Unexpectedly failed to obtain (BLS) genesis key."))?;
            let net_info = config
                .add_network(
                    &network_name,
                    Some(NetworkInfo::NodeConfig((genesis_key, set))),
                )
                .await?;
            println!(
                "Network '{}' was added to the list. Contacts: '{}'",
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

#[cfg(test)]
mod networks_set_command {
    use super::networks_commander;
    use crate::operations::config::{Config, NetworkInfo};
    use assert_fs::prelude::*;
    use color_eyre::{eyre::eyre, Result};
    use predicates::prelude::*;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    #[tokio::test]
    async fn given_no_pre_existing_config_and_multiple_nodes_are_specified_then_a_network_should_be_saved(
    ) -> Result<()> {
        // Arrange
        let secret_key = bls::SecretKey::random();
        let genesis_key = hex::encode(secret_key.public_key().to_bytes());

        let cmd = super::NetworksSubCommands::Set {
            network_name: String::from("new_network"),
            genesis_key_hex: genesis_key.clone(),
            addresses: vec![
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 12000),
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2)), 12000),
            ],
        };
        let config_dir = assert_fs::TempDir::new()?;
        let cli_config_file = config_dir.child(".safe/cli/config.json");
        let node_config_file = config_dir.child(".safe/node/node_connection_info.config");
        let mut config = Config::new(
            cli_config_file.path().to_path_buf(),
            node_config_file.path().to_path_buf(),
        )
        .await?;

        // Act
        let result = networks_commander(Some(cmd), &mut config).await;

        // Assert
        assert!(result.is_ok());
        cli_config_file.assert(predicate::path::is_file());

        assert_eq!(config.networks_iter().count(), 1);

        let (network_name, network_info) = config
            .networks_iter()
            .next()
            .ok_or_else(|| eyre!("failed to obtain item from networks list"))?;
        assert_eq!(network_name, "new_network");
        match network_info {
            NetworkInfo::NodeConfig((public_key, contacts)) => {
                assert_eq!(contacts.len(), 2);

                let node: SocketAddr = "127.0.0.1:12000".parse()?;
                assert_eq!(contacts.get(&node), Some(&node));
                let node: SocketAddr = "127.0.0.2:12000".parse()?;
                assert_eq!(contacts.get(&node), Some(&node));

                assert_eq!(hex::encode(public_key.to_bytes()), genesis_key);
            }
            NetworkInfo::ConnInfoLocation(_) => {
                return Err(eyre!("connection info doesn't apply to this test"));
            }
        }

        Ok(())
    }
}
