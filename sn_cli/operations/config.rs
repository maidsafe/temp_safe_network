// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use anyhow::{anyhow, Context, Result};
use log::debug;
use prettytable::Table;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs::{self, create_dir_all, remove_file},
    path::PathBuf,
};

const CONFIG_FILENAME: &str = "config.json";
const CONFIG_NETWORKS_DIRNAME: &str = "networks";

#[derive(Deserialize, Debug, Serialize, Default)]
pub struct ConfigSettings {
    pub networks: BTreeMap<String, String>,
    // pub contacts: BTreeMap<String, String>,
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

pub fn add_network_to_config(network_name: &str, config_location: Option<String>) -> Result<()> {
    let location = match config_location {
        Some(location) => location,
        None => {
            // Cache current network connection info
            let (_, conn_info) = read_current_network_conn_info()?;
            let cache_path = cache_conn_info(network_name, &conn_info)?;
            println!(
                "Caching current network connection information into: {}",
                cache_path.display()
            );
            cache_path.display().to_string()
        }
    };

    let (mut settings, file_path) = read_config_settings()?;
    settings
        .networks
        .insert(network_name.to_string(), location.clone());
    write_config_settings(&file_path, settings)?;
    debug!("Network {} - {} added to settings", network_name, location);
    println!(
        "Network '{}' was added to the list. Connection information is located at '{}'",
        network_name, location
    );
    Ok(())
}

pub fn remove_network_from_config(network_name: &str) -> Result<()> {
    let (mut settings, file_path) = read_config_settings()?;
    match settings.networks.remove(network_name) {
        Some(location) => {
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
        None => println!(
            "No network with name '{}' was found in config",
            network_name
        ),
    }

    Ok(())
}

pub fn read_current_network_conn_info() -> Result<(PathBuf, Vec<u8>)> {
    let (_, file_path) = get_current_network_conn_info_path()?;
    let current_conn_info = fs::read(&file_path).with_context(||
        format!(
            "There doesn't seem to be a any network setup in your system. Unable to read current network connection information from '{}'",
            file_path.display()
        )
    )?;
    Ok((file_path, current_conn_info))
}

pub fn write_current_network_conn_info(conn_info: &[u8]) -> Result<()> {
    let (base_path, file_path) = get_current_network_conn_info_path()?;

    if !base_path.exists() {
        println!(
            "Creating '{}' folder for network connection info",
            base_path.display()
        );
        create_dir_all(&base_path).context("Couldn't create folder for network connection info")?;
    }

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

pub fn cache_conn_info(network_name: &str, conn_info: &[u8]) -> Result<PathBuf> {
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
    table.add_row(row![bFg->"Network name", bFg->"Connection info location"]);

    let (settings, _) = read_config_settings()?;
    settings
        .networks
        .iter()
        .for_each(|(network_name, config_location)| {
            table.add_row(row![network_name, config_location,]);
        });
    table.printstd();
    Ok(())
}

pub fn retrieve_conn_info(name: &str, location: &str) -> Result<Vec<u8>> {
    println!(
        "Fetching '{}' network connection information from '{}' ...",
        name, location
    );
    if is_remote_location(location) {
        #[cfg(feature = "self-update")]
        {
            // Fetch info from an HTTP/s location
            let mut resp = reqwest::get(location).with_context(|| {
                format!(
                    "Failed to fetch connection information for network '{}' from '{}'",
                    name, location
                )
            })?;

            let conn_info = resp.text().with_context(|| {
                format!(
                    "Failed to fetch connection information for network '{}' from '{}'",
                    name, location
                )
            })?;
            Ok(conn_info.as_bytes().to_vec())
        }
        #[cfg(not(feature = "self-update"))]
        anyhow!("Self updates are disabled")
    } else {
        // Fetch it from a local file then
        let conn_info = fs::read(location).with_context(|| {
            format!("Unable to read connection information from '{}'", location)
        })?;
        Ok(conn_info)
    }
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
