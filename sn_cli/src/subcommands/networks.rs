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
use comfy_table::{Cell, CellAlignment, Table};
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
    /// Add a network to the CLI config using an existing network contacts
    Add {
        /// Network name. If network_name already exists, then it's updated with the new network contacts
        network_name: String,
        /// Local path or a remote URL to fetch the network contacts from
        contacts_file_location: String,
    },
    #[clap(name = "remove")]
    /// Remove a network from the CLI config
    Remove {
        /// Network to remove
        network_name: String,
    },
    #[clap(name = "sections")]
    /// Display information about the sections of a network
    Sections {
        /// Network to show sections information from, or default network if no name is provided
        network_name: Option<String>,
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
            let (network_contacts, _) = config.read_default_network_contacts().await?;
            let mut matched_network = None;
            for (network_name, network_info) in config.networks_iter() {
                if network_info.matches(network_contacts.genesis_key()) {
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
            contacts_file_location,
        }) => {
            let net_info = if Url::parse(contacts_file_location.as_str()).is_ok() {
                config
                    .add_network(
                        &network_name,
                        NetworkInfo::Remote(contacts_file_location, None),
                    )
                    .await?
            } else {
                let path = PathBuf::from(contacts_file_location);
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
        Some(NetworksSubCommands::Sections { network_name }) => {
            let (network_contacts, location) = if let Some(name) = network_name {
                println!("Network sections information for '{}':", name);
                config.read_network_contacts(&name).await?
            } else {
                println!("Network sections information for default network:");
                config.read_default_network_contacts().await?
            };
            println!("Read from: {}", location);
            println!();

            let genesis_key = network_contacts.genesis_key();
            println!("Genesis Key: {:?}", genesis_key);
            println!();

            println!("Sections:");
            println!();

            let sections_dag = network_contacts.get_sections_dag();
            for sap in &network_contacts.all() {
                let section_key = sap.section_key();
                println!("Prefix '{}'", sap.prefix());
                println!("----------------------------------");
                println!("Section key: {:?}", section_key);
                println!(
                    "Section keys chain: {:?}",
                    sections_dag.get_proof_chain(genesis_key, &section_key)?
                );
                println!();

                println!("Elders:");
                let mut table = Table::new();
                table.load_preset(comfy_table::presets::ASCII_MARKDOWN);
                table.add_row(&vec!["XorName", "Age", "Address"]);

                let mut sorted_elders = sap.elders().collect::<Vec<_>>();
                sorted_elders.sort_by_key(|elder| elder.age());
                for elder in &sorted_elders {
                    table.add_row(vec![
                        elder.name().into(),
                        Cell::new(elder.age().to_string()).set_alignment(CellAlignment::Right),
                        elder.addr().into(),
                    ]);
                }
                println!("{table}");
                println!();
            }
        }
        None => config.print_networks().await,
    }

    Ok(())
}
