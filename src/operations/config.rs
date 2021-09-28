// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use color_eyre::{eyre::bail, eyre::eyre, eyre::WrapErr, Result};
use prettytable::Table;
use serde::{Deserialize, Serialize};
use sn_api::{NodeConfig, PublicKey};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    fs::{self, create_dir_all, remove_file},
    net::SocketAddr,
    path::PathBuf,
};
use tracing::debug;

const CONFIG_FILENAME: &str = "config.json";
const CONFIG_NETWORKS_DIRNAME: &str = "networks";

#[derive(Deserialize, Debug, Serialize, Clone)]
pub enum NetworkInfo {
    /// A list of IPv4 addresses wich are the contact peers of the network
    Addresses(NodeConfig),
    /// A URL where the network connection information can be fetched/read from
    ConnInfoUrl(String),
}

impl fmt::Display for NetworkInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Addresses(addresses) => write!(f, "{:?}", addresses),
            Self::ConnInfoUrl(url) => write!(f, "{}", url),
        }
    }
}

impl NetworkInfo {
    pub async fn matches(&self, node_config: &NodeConfig) -> bool {
        match self {
            Self::Addresses(nc) => node_config == nc,
            Self::ConnInfoUrl(config_location) => match retrieve_node_config(config_location).await
            {
                Ok(info) => info == *node_config,
                Err(_) => false,
            },
        }
    }
}

#[derive(Deserialize, Debug, Serialize, Default)]
pub struct Settings {
    networks: BTreeMap<String, NetworkInfo>,
    // contacts: BTreeMap<String, String>,
}

#[derive(Debug)]
pub struct Config {
    settings: Settings,
    file_path: PathBuf,
}

impl Config {
    pub fn read() -> Result<Self> {
        let file_path = config_file_path()?;

        let config = if !file_path.exists() {
            let empty_config = Self {
                settings: Settings::default(),
                file_path: file_path.clone(),
            };
            empty_config.write_settings_to_file().wrap_err_with(|| {
                format!("Unable to create config at '{}'", file_path.display(),)
            })?;
            debug!("Empty config file created at '{}'", file_path.display(),);
            empty_config
        } else {
            let file = fs::File::open(&file_path).wrap_err_with(|| {
                format!("Error opening config file from '{}'", file_path.display(),)
            })?;

            let settings: Settings = serde_json::from_reader(file).wrap_err_with(|| {
                format!(
                    "Format of the config file at '{}' is not valid and couldn't be parsed",
                    file_path.display()
                )
            })?;

            debug!(
                "Config settings retrieved from '{}': {:?}",
                file_path.display(),
                settings
            );

            Self {
                settings,
                file_path,
            }
        };

        Ok(config)
    }

    pub async fn get_network_info(&self, name: &str) -> Result<NodeConfig> {
        match self.settings.networks.get(name) {
            Some(NetworkInfo::ConnInfoUrl(config_location)) => {
                println!(
                    "Fetching '{}' network connection information from '{}' ...",
                    name, config_location
                );

                let node_config = retrieve_node_config(config_location).await?;
                Ok(node_config)
            },
            Some(NetworkInfo::Addresses(addresses)) => Ok(addresses.clone()),
            None => bail!("No network with name '{}' was found in the config. Please use the networks 'add'/'set' subcommand to add it", name)
        }
    }

    pub fn networks_iter(&self) -> impl Iterator<Item = (&String, &NetworkInfo)> {
        self.settings.networks.iter()
    }

    pub fn add_network(
        &mut self,
        name: &str,
        net_info: Option<NetworkInfo>,
    ) -> Result<NetworkInfo> {
        let net_info = if let Some(info) = net_info {
            info
        } else {
            // Cache current network connection info
            let (_, conn_info) = read_current_node_config()?;
            let cache_path = cache_node_config(name, &conn_info)?;
            println!(
                "Caching current network connection information into '{}'",
                cache_path.display()
            );
            NetworkInfo::ConnInfoUrl(cache_path.display().to_string())
        };

        self.settings
            .networks
            .insert(name.to_string(), net_info.clone());

        self.write_settings_to_file()?;

        debug!("Network '{}' added to settings: {}", name, net_info);
        Ok(net_info)
    }

    pub fn remove_network(&mut self, name: &str) -> Result<()> {
        match self.settings.networks.remove(name) {
            Some(NetworkInfo::ConnInfoUrl(location)) => {
                self.write_settings_to_file()?;
                debug!("Network '{}' removed from config", name);
                println!("Network '{}' was removed from the config", name);
                let mut config_local_path = get_cli_config_path()?;
                config_local_path.push(CONFIG_NETWORKS_DIRNAME);
                if PathBuf::from(&location).starts_with(config_local_path) {
                    println!(
                        "Removing cached network connection information from '{}'",
                        location
                    );

                    if let Err(err) = remove_file(&location) {
                        println!(
                            "Failed to remove cached network connection information from '{}': {}",
                            location, err
                        );
                    }
                }
            }
            Some(NetworkInfo::Addresses(_)) => {
                self.write_settings_to_file()?;
                debug!("Network '{}' removed from config", name);
                println!("Network '{}' was removed from the config", name);
            }
            None => println!("No network with name '{}' was found in config", name),
        }

        Ok(())
    }

    pub fn clear(&mut self) -> Result<()> {
        self.settings = Settings::default();
        self.write_settings_to_file()
    }

    pub async fn switch_to_network(&self, name: &str) -> Result<()> {
        let (base_path, file_path) = get_current_network_info_path()?;

        if !base_path.exists() {
            println!(
                "Creating '{}' folder for network connection info",
                base_path.display()
            );
            create_dir_all(&base_path)
                .wrap_err("Couldn't create folder for network connection info")?;
        }

        let contacts = self.get_network_info(name).await?;
        let conn_info = serialise_node_config(&contacts)?;
        fs::write(&file_path, conn_info).wrap_err_with(|| {
            format!(
                "Unable to write network connection info in '{}'",
                base_path.display(),
            )
        })
    }

    pub async fn print_networks(&self) {
        let mut table = Table::new();
        table.add_row(row![bFg->"Networks"]);
        table.add_row(row![bFg->"Current", bFg->"Network name", bFg->"Connection info"]);
        let current_node_config = match read_current_node_config() {
            Ok((_, current_conn_info)) => Some(current_conn_info),
            Err(_) => None, // we simply ignore the error, none of the networks is currently active/set in the system
        };

        for (network_name, net_info) in self.networks_iter() {
            let mut current = "";
            if let Some(node_config) = &current_node_config {
                if net_info.matches(node_config).await {
                    current = "*";
                }
            }
            table.add_row(row![current, network_name, net_info]);
        }

        table.printstd();
    }

    // Private helpers

    fn write_settings_to_file(&self) -> Result<()> {
        let serialised_settings = serde_json::to_string(&self.settings)
            .wrap_err("Failed to serialise config settings")?;

        fs::write(&self.file_path, serialised_settings.as_bytes()).wrap_err_with(|| {
            format!(
                "Unable to write config settings to '{}'",
                self.file_path.display()
            )
        })?;

        debug!(
            "Config settings at '{}' updated with: {:?}",
            self.file_path.display(),
            self.settings
        );

        Ok(())
    }
}

pub fn read_current_node_config() -> Result<(PathBuf, NodeConfig)> {
    let (_, file_path) = get_current_network_info_path()?;
    let current_conn_info = fs::read(&file_path).wrap_err_with(||
        format!(
            "There doesn't seem to be a any network setup in your system. Unable to read current network connection information from '{}'",
            file_path.display()
        )
    )?;

    let node_config = deserialise_node_config(&current_conn_info).wrap_err_with(|| {
        format!(
            "Unable to read current network connection information from '{}'",
            file_path.display()
        )
    })?;

    Ok((file_path, node_config))
}

fn config_file_path() -> Result<PathBuf> {
    let config_local_path = get_cli_config_path()?;
    let file_path = config_local_path.join(CONFIG_FILENAME);
    if !config_local_path.exists() {
        println!(
            "Creating '{}' folder for config file",
            config_local_path.display()
        );
        create_dir_all(config_local_path)
            .wrap_err("Couldn't create project's local config folder")?;
    }

    Ok(file_path)
}

fn cache_node_config(network_name: &str, contacts: &NodeConfig) -> Result<PathBuf> {
    let mut file_path = get_cli_config_path()?;
    file_path.push(CONFIG_NETWORKS_DIRNAME);
    if !file_path.exists() {
        println!(
            "Creating '{}' folder for networks connection info cache",
            file_path.display()
        );
        create_dir_all(&file_path)
            .wrap_err("Couldn't create folder for networks information cache")?;
    }

    file_path.push(format!("{}_node_connection_info.config", network_name));
    let conn_info = serialise_node_config(contacts)?;
    fs::write(&file_path, conn_info).wrap_err_with(|| {
        format!(
            "Unable to cache connection information in '{}'",
            file_path.display(),
        )
    })?;

    Ok(file_path)
}

async fn retrieve_node_config(location: &str) -> Result<NodeConfig> {
    let is_remote_location = location.starts_with("http");
    let contacts_bytes = if is_remote_location {
        #[cfg(feature = "self-update")]
        {
            // Fetch info from an HTTP/s location
            let resp = reqwest::get(location).await.wrap_err_with(|| {
                format!("Failed to fetch connection information from '{}'", location)
            })?;

            let conn_info = resp.text().await.wrap_err_with(|| {
                format!("Failed to fetch connection information from '{}'", location)
            })?;

            conn_info.as_bytes().to_vec()
        }
        #[cfg(not(feature = "self-update"))]
        eyre!("Self updates are disabled")
    } else {
        // Fetch it from a local file then
        fs::read(location).wrap_err_with(|| {
            format!("Unable to read connection information from '{}'", location)
        })?
    };

    deserialise_node_config(&contacts_bytes)
}

fn deserialise_node_config(bytes: &[u8]) -> Result<NodeConfig> {
    let deserialized: (String, BTreeSet<SocketAddr>) = serde_json::from_slice(bytes)
        .wrap_err_with(|| "Format of the contacts addresses is not valid and couldn't be parsed")?;
    let genesis_key = PublicKey::bls_from_hex(&deserialized.0)?
        .bls()
        .ok_or_else(|| eyre!("Unexpectedly failed to obtain (BLS) genesis key."))?;
    Ok((genesis_key, deserialized.1))
}

pub fn serialise_node_config(node_config: &NodeConfig) -> Result<String> {
    let genesis_key_hex = hex::encode(node_config.0.to_bytes());
    serde_json::to_string(&(genesis_key_hex, node_config.1.clone()))
        .wrap_err_with(|| "Failed to serialise network connection info")
}

fn get_current_network_info_path() -> Result<(PathBuf, PathBuf)> {
    let mut node_data_path =
        dirs_next::home_dir().ok_or_else(|| eyre!("Failed to obtain user's home path"))?;

    node_data_path.push(".safe");
    node_data_path.push("node");

    Ok((
        node_data_path.clone(),
        node_data_path.join("node_connection_info.config"),
    ))
}

fn get_cli_config_path() -> Result<PathBuf> {
    let mut project_data_path =
        dirs_next::home_dir().ok_or_else(|| eyre!("Couldn't find user's home directory"))?;
    project_data_path.push(".safe");
    project_data_path.push("cli");

    Ok(project_data_path)
}
