// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use anyhow::{anyhow, bail, Context, Result};
use log::debug;
use prettytable::Table;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashSet},
    fmt,
    fs::{self, create_dir_all, remove_file},
    net::SocketAddr,
    path::PathBuf,
};

const CONFIG_FILENAME: &str = "config.json";
const CONFIG_NETWORKS_DIRNAME: &str = "networks";

#[derive(Deserialize, Debug, Serialize, Clone)]
pub enum NetworkInfo {
    /// A list of IPv4 addresses wich are the contact peers of the network
    Addresses(HashSet<SocketAddr>),
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
    pub fn matches(&self, conn_info: &HashSet<SocketAddr>) -> bool {
        match self {
            Self::Addresses(addresses) => addresses == conn_info,
            Self::ConnInfoUrl(config_location) => match retrieve_conn_info(config_location) {
                Ok(info) => info == *conn_info,
                Err(_) => false,
            },
        }
    }
}

#[derive(Deserialize, Debug, Serialize, Default)]
pub struct ConfigSettings {
    pub networks: BTreeMap<String, NetworkInfo>,
    // pub contacts: BTreeMap<String, String>,
}

impl ConfigSettings {
    pub fn get_net_info(&self, name: &str) -> Result<HashSet<SocketAddr>> {
        match self.networks.get(name) {
            Some(NetworkInfo::ConnInfoUrl(config_location)) => {
                println!(
                    "Fetching '{}' network connection information from '{}' ...",
                    name, config_location
                );

                retrieve_conn_info(config_location)
            },
            Some(NetworkInfo::Addresses(addresses)) => Ok(addresses.clone()),
            None => bail!("No network with name '{}' was found in the config. Please use the 'networks add' command to add it", name)
        }
    }
}

pub fn read_config_settings() -> Result<(ConfigSettings, PathBuf)> {
    let file_path = config_file_path()?;
    let file = fs::File::open(&file_path)
        .with_context(|| format!("Error reading config file from '{}'", file_path.display(),))?;

    let settings: ConfigSettings = serde_json::from_reader(file)
        .context("Format of the config file is not valid and couldn't be parsed")?;

    debug!(
        "Config settings retrieved from {}: {:?}",
        file_path.display(),
        settings
    );
    Ok((settings, file_path))
}

pub fn write_config_settings(file_path: &PathBuf, settings: ConfigSettings) -> Result<()> {
    let serialised_settings =
        serde_json::to_string(&settings).context("Failed to add config to file")?;

    fs::write(&file_path, serialised_settings.as_bytes())
        .with_context(|| format!("Unable to write config in {}", file_path.display()))?;

    debug!(
        "Config settings at {} updated with: {:?}",
        file_path.display(),
        settings
    );

    Ok(())
}

pub fn add_network_to_config(
    network_name: &str,
    net_info: Option<NetworkInfo>,
) -> Result<NetworkInfo> {
    let net_info = if let Some(info) = net_info {
        info
    } else {
        // Cache current network connection info
        let (_, conn_info) = read_current_network_conn_info()?;
        let cache_path = cache_conn_info(network_name, &conn_info)?;
        println!(
            "Caching current network connection information into: {}",
            cache_path.display()
        );
        NetworkInfo::ConnInfoUrl(cache_path.display().to_string())
    };

    let (mut settings, file_path) = read_config_settings()?;
    settings
        .networks
        .insert(network_name.to_string(), net_info.clone());

    write_config_settings(&file_path, settings)?;

    debug!("Network {} - {} added to settings", network_name, net_info);
    Ok(net_info)
}

pub fn remove_network_from_config(network_name: &str) -> Result<()> {
    let (mut settings, file_path) = read_config_settings()?;
    match settings.networks.remove(network_name) {
        Some(NetworkInfo::ConnInfoUrl(location)) => {
            write_config_settings(&file_path, settings)?;
            debug!("Network {} removed from settings", network_name);
            println!("Network '{}' was removed from the list", network_name);
            let mut config_local_path = get_cli_config_path()?;
            config_local_path.push(CONFIG_NETWORKS_DIRNAME);
            if PathBuf::from(&location).starts_with(config_local_path) {
                println!(
                    "Removing cached network connection information from {}",
                    location
                );
                remove_file(&location).with_context(|| {
                    format!(
                        "Failed to remove cached network connection information from {}",
                        location
                    )
                })?;
            }
        }
        Some(NetworkInfo::Addresses(_)) => {
            debug!("Network {} removed from settings", network_name);
            println!("Network '{}' was removed from the list", network_name);
        }
        None => println!(
            "No network with name '{}' was found in config",
            network_name
        ),
    }

    Ok(())
}

pub fn read_current_network_conn_info() -> Result<(PathBuf, HashSet<SocketAddr>)> {
    let (_, file_path) = get_current_network_conn_info_path()?;
    let current_conn_info = fs::read(&file_path).with_context(||
        format!(
            "There doesn't seem to be a any network setup in your system. Unable to read current network connection information from '{}'",
            file_path.display()
        )
    )?;

    let contacts = deserialise_contacts(&current_conn_info).with_context(|| {
        format!(
            "Unable to read current network connection information from '{}'",
            file_path.display()
        )
    })?;

    Ok((file_path, contacts))
}

pub fn write_current_network_conn_info(contacts: &HashSet<SocketAddr>) -> Result<()> {
    let (base_path, file_path) = get_current_network_conn_info_path()?;

    if !base_path.exists() {
        println!(
            "Creating '{}' folder for network connection info",
            base_path.display()
        );
        create_dir_all(&base_path).context("Couldn't create folder for network connection info")?;
    }

    let conn_info = serialise_contacts(contacts)?;
    fs::write(&file_path, conn_info).with_context(|| {
        format!(
            "Unable to write network connection info in {}",
            base_path.display(),
        )
    })
}

pub fn config_file_path() -> Result<PathBuf> {
    let config_local_path = get_cli_config_path()?;
    let file_path = config_local_path.join(CONFIG_FILENAME);
    if !config_local_path.exists() {
        println!(
            "Creating '{}' folder for config file",
            config_local_path.display()
        );
        create_dir_all(config_local_path)
            .context("Couldn't create project's local config folder")?;
    }

    if !file_path.exists() {
        let empty_settings = ConfigSettings::default();
        write_config_settings(&file_path, empty_settings)
            .with_context(|| format!("Unable to create config in {}", file_path.display(),))?;
    }

    Ok(file_path)
}

pub fn cache_conn_info(network_name: &str, contacts: &HashSet<SocketAddr>) -> Result<PathBuf> {
    let mut file_path = get_cli_config_path()?;
    file_path.push(CONFIG_NETWORKS_DIRNAME);
    if !file_path.exists() {
        println!(
            "Creating '{}' folder for networks connection info cache",
            file_path.display()
        );
        create_dir_all(&file_path)
            .context("Couldn't create folder for networks information cache")?;
    }

    file_path.push(format!("{}_node_connection_info.config", network_name));
    let conn_info = serialise_contacts(contacts)?;
    fs::write(&file_path, conn_info).with_context(|| {
        format!(
            "Unable to cache connection information in {}",
            file_path.display(),
        )
    })?;

    Ok(file_path)
}

pub fn print_networks_settings() -> Result<()> {
    let mut table = Table::new();
    table.add_row(row![bFg->"Networks"]);
    table.add_row(row![bFg->"Current", bFg->"Network name", bFg->"Connection info"]);
    let current_conn_info = match read_current_network_conn_info() {
        Ok((_, current_conn_info)) => Some(current_conn_info),
        Err(_) => None, // we simply ignore the error, none of the networks is currently active/set in the system
    };

    let (settings, _) = read_config_settings()?;
    settings
        .networks
        .iter()
        .for_each(|(network_name, net_info)| {
            let mut current = "";
            if let Some(conn_info) = &current_conn_info {
                if net_info.matches(conn_info) {
                    current = "*";
                }
            }
            table.add_row(row![current, network_name, net_info]);
        });
    table.printstd();
    Ok(())
}

pub fn retrieve_conn_info(location: &str) -> Result<HashSet<SocketAddr>> {
    let contacts_bytes = if is_remote_location(location) {
        #[cfg(feature = "self-update")]
        {
            // Fetch info from an HTTP/s location
            let mut resp = reqwest::get(location).with_context(|| {
                format!("Failed to fetch connection information from '{}'", location)
            })?;

            let conn_info = resp.text().with_context(|| {
                format!("Failed to fetch connection information from '{}'", location)
            })?;

            conn_info.as_bytes().to_vec()
        }
        #[cfg(not(feature = "self-update"))]
        anyhow!("Self updates are disabled")
    } else {
        // Fetch it from a local file then
        fs::read(location)
            .with_context(|| format!("Unable to read connection information from '{}'", location))?
    };

    deserialise_contacts(&contacts_bytes)
}

fn deserialise_contacts(bytes: &[u8]) -> Result<HashSet<SocketAddr>> {
    serde_json::from_slice(bytes)
        .with_context(|| "Format of the contacts addresses is not valid and couldn't be parsed")
}

pub fn serialise_contacts(contacts: &HashSet<SocketAddr>) -> Result<String> {
    serde_json::to_string(contacts).with_context(|| "Failed to serialise network connection info")
}

#[inline]
fn is_remote_location(location: &str) -> bool {
    location.starts_with("http")
}

fn get_current_network_conn_info_path() -> Result<(PathBuf, PathBuf)> {
    let mut node_data_path =
        dirs_next::home_dir().ok_or_else(|| anyhow!("Failed to obtain user's home path"))?;

    node_data_path.push(".safe");
    node_data_path.push("node");

    Ok((
        node_data_path.clone(),
        node_data_path.join("node_connection_info.config"),
    ))
}

fn get_cli_config_path() -> Result<PathBuf> {
    let mut project_data_path =
        dirs_next::home_dir().ok_or_else(|| anyhow!("Couldn't find user's home directory"))?;
    project_data_path.push(".safe");
    project_data_path.push("cli");

    Ok(project_data_path)
}
