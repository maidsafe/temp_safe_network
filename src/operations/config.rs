// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use color_eyre::{eyre::bail, eyre::eyre, eyre::WrapErr, Help, Result};
use prettytable::Table;
use serde::{Deserialize, Serialize};
use sn_api::{NodeConfig, PublicKey};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    fs::{self, remove_file},
    net::SocketAddr,
    path::{Path, PathBuf},
};
use tracing::debug;

const CONFIG_NETWORKS_DIRNAME: &str = "networks";

#[derive(Deserialize, Debug, Serialize, Clone)]
pub enum NetworkInfo {
    /// The node configuration is a genesis key, which is a BLS public key, and a set of nodes
    /// participating in the network, which are an IPv4 and port address pair.
    NodeConfig(NodeConfig),
    /// A URL or file path where the network connection information can be fetched/read from.
    ConnInfoLocation(String),
}

impl fmt::Display for NetworkInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NodeConfig(addresses) => write!(f, "{:?}", addresses),
            Self::ConnInfoLocation(url) => write!(f, "{}", url),
        }
    }
}

impl NetworkInfo {
    pub async fn matches(&self, node_config: &NodeConfig) -> bool {
        match self {
            Self::NodeConfig(nc) => nc == node_config,
            Self::ConnInfoLocation(config_location) => {
                match retrieve_node_config(config_location).await {
                    Ok(info) => info == *node_config,
                    Err(_) => false,
                }
            }
        }
    }
}

#[derive(Clone, Deserialize, Debug, Serialize, Default)]
pub struct Settings {
    networks: BTreeMap<String, NetworkInfo>,
}

#[derive(Clone, Debug)]
pub struct Config {
    settings: Settings,
    cli_config_path: PathBuf,
    node_config_path: PathBuf,
}

impl Config {
    pub fn new(cli_config_path: PathBuf, node_config_path: PathBuf) -> Result<Config> {
        let mut pb = cli_config_path.clone();
        pb.pop();
        std::fs::create_dir_all(pb.as_path())?;

        let settings: Settings;
        if cli_config_path.exists() {
            let file = fs::File::open(&cli_config_path).wrap_err_with(|| {
                format!(
                    "Error opening config file from '{}'",
                    cli_config_path.display(),
                )
            })?;
            settings = serde_json::from_reader(file).wrap_err_with(|| {
                format!(
                    "Format of the config file at '{}' is not valid and couldn't be parsed",
                    cli_config_path.display()
                )
            })?;
            debug!(
                "Config settings retrieved from '{}': {:?}",
                cli_config_path.display(),
                settings
            );
        } else {
            settings = Settings::default();
            debug!(
                "Empty config file created at '{}'",
                cli_config_path.display()
            );
        }

        let config = Config {
            settings,
            cli_config_path: cli_config_path.clone(),
            node_config_path,
        };
        config.write_settings_to_file().wrap_err_with(|| {
            format!("Unable to create config at '{}'", cli_config_path.display())
        })?;
        Ok(config)
    }

    pub fn read_current_node_config(&self) -> Result<(PathBuf, NodeConfig)> {
        let current_conn_info = fs::read(&self.node_config_path).wrap_err_with(|| {
            eyre!("There doesn't seem to be any node configuration setup in your system.")
                .suggestion(
                    "A node config will be created if you join a network or launch your own.",
                )
        })?;
        let node_config = deserialise_node_config(&current_conn_info).wrap_err_with(|| {
            eyre!(format!(
                "Unable to read current network connection information from '{}'.",
                self.node_config_path.display()
            ))
            .suggestion(
                "This file is likely not a node configuration file.\
                Please point towards another file with a valid node configuration.",
            )
        })?;
        Ok((self.node_config_path.clone(), node_config))
    }

    pub async fn get_network_info(&self, name: &str) -> Result<NodeConfig> {
        match self.settings.networks.get(name) {
            Some(NetworkInfo::ConnInfoLocation(config_location)) => {
                println!(
                    "Fetching '{}' network connection information from '{}' ...",
                    name, config_location
                );

                let node_config = retrieve_node_config(config_location).await?;
                Ok(node_config)
            },
            Some(NetworkInfo::NodeConfig(addresses)) => Ok(addresses.clone()),
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
            let (_, node_config) = self.read_current_node_config()?;
            let cache_path = self.cache_node_config(name, &node_config)?;
            println!(
                "Caching current network connection information into '{}'",
                cache_path.display()
            );
            NetworkInfo::ConnInfoLocation(cache_path.display().to_string())
        };

        match &net_info {
            NetworkInfo::NodeConfig(_) => {}
            NetworkInfo::ConnInfoLocation(location) => {
                let result = url::Url::parse(location);
                if result.is_err() {
                    // The location is not a valid URL, so try and parse it as a file.
                    let pb = PathBuf::from(Path::new(&location));
                    if !pb.is_file() {
                        return Err(eyre!("The config location must use an existing file path.")
                            .suggestion(
                                "Please choose an existing file with a network configuration.",
                            ));
                    }
                    deserialise_node_config(&fs::read(pb.as_path())?).wrap_err_with(|| {
                        eyre!("The file must contain a valid network configuration.").suggestion(
                            "Please choose another file with a valid network configuration.",
                        )
                    })?;
                }
            }
        }
        self.settings
            .networks
            .insert(name.to_string(), net_info.clone());

        self.write_settings_to_file()?;

        debug!("Network '{}' added to settings: {}", name, net_info);
        Ok(net_info)
    }

    pub fn remove_network(&mut self, name: &str) -> Result<()> {
        match self.settings.networks.remove(name) {
            Some(NetworkInfo::ConnInfoLocation(location)) => {
                self.write_settings_to_file()?;
                debug!("Network '{}' removed from config", name);
                println!("Network '{}' was removed from the config", name);
                let mut config_local_path = self.cli_config_path.clone();
                config_local_path.pop();
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
            Some(NetworkInfo::NodeConfig(_)) => {
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
        let mut base_path = self.node_config_path.clone();
        base_path.pop();

        if !base_path.exists() {
            println!(
                "Creating '{}' folder for network connection info",
                base_path.display()
            );
            std::fs::create_dir_all(&base_path)
                .wrap_err("Couldn't create folder for network connection info")?;
        }

        let contacts = self.get_network_info(name).await?;
        let conn_info = serialise_node_config(&contacts)?;
        fs::write(&self.node_config_path, conn_info).wrap_err_with(|| {
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
        let current_node_config = match self.read_current_node_config() {
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

    //
    // Private helpers
    //
    fn cache_node_config(&self, network_name: &str, node_config: &NodeConfig) -> Result<PathBuf> {
        let mut pb = self.cli_config_path.clone();
        pb.pop();
        pb.push(CONFIG_NETWORKS_DIRNAME);
        if !pb.exists() {
            println!(
                "Creating '{}' folder for networks connection info cache",
                pb.display()
            );
            std::fs::create_dir_all(&pb)
                .wrap_err("Couldn't create folder for networks information cache")?;
        }

        pb.push(format!("{}_node_connection_info.config", network_name));
        let conn_info = serialise_node_config(node_config)?;
        fs::write(&pb, conn_info)?;
        Ok(pb)
    }

    fn write_settings_to_file(&self) -> Result<()> {
        let serialised_settings = serde_json::to_string(&self.settings)
            .wrap_err("Failed to serialise config settings")?;
        fs::write(&self.cli_config_path, serialised_settings.as_bytes()).wrap_err_with(|| {
            format!(
                "Unable to write config settings to '{}'",
                self.cli_config_path.display()
            )
        })?;
        debug!(
            "Config settings at '{}' updated with: {:?}",
            self.cli_config_path.display(),
            self.settings
        );
        Ok(())
    }
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
    let deserialized: (String, BTreeSet<SocketAddr>) = serde_json::from_slice(bytes)?;
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

#[cfg(test)]
mod constructor {
    use super::{Config, NetworkInfo};
    use assert_fs::prelude::*;
    use color_eyre::{eyre::eyre, Result};
    use predicates::prelude::*;
    use std::net::SocketAddr;
    use std::path::PathBuf;

    #[test]
    fn fields_should_be_set_to_correct_values() -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let cli_config_file = tmp_dir.child(".safe/cli/config.json");
        let node_config_file = tmp_dir.child(".safe/node/node_connection_info.config");

        let config = Config::new(
            PathBuf::from(cli_config_file.path()),
            PathBuf::from(node_config_file.path()),
        )?;

        assert_eq!(config.cli_config_path, cli_config_file.path());
        assert_eq!(config.node_config_path, node_config_file.path());
        assert_eq!(config.settings.networks.len(), 0);
        Ok(())
    }

    #[test]
    fn cli_config_directory_should_be_created() -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let cli_config_dir = tmp_dir.child(".safe/cli");
        let cli_config_file = cli_config_dir.child("config.json");
        let node_config_file = tmp_dir.child(".safe/node/node_connection_info.config");

        let _ = Config::new(
            PathBuf::from(cli_config_file.path()),
            PathBuf::from(node_config_file.path()),
        )?;

        cli_config_dir.assert(predicate::path::is_dir());
        Ok(())
    }

    #[test]
    fn given_config_file_does_not_exist_then_it_should_be_created() -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let cli_config_file = tmp_dir.child(".safe/cli/config.json");
        let node_config_file = tmp_dir.child(".safe/node/node_connection_info.config");
        let _ = Config::new(
            PathBuf::from(cli_config_file.path()),
            PathBuf::from(node_config_file.path()),
        )?;

        cli_config_file.assert(predicate::path::exists());

        Ok(())
    }

    #[test]
    fn given_config_file_exists_then_the_settings_should_be_read() -> Result<()> {
        let serialized_config = r#"
        {
            "networks": {
                "existing_network": {
                    "NodeConfig":[
                        [140,44,196,143,12,92,218,53,190,33,205,167,109,183,94,205,16,140,197,200,96,112,136,218,221,16,57,54,204,60,58,93,199,119,26,17,105,232,33,188,163,194,145,223,194,95,92,54],
                        ["127.0.0.1:12000","127.0.0.2:12000"]
                    ]
                }
            }
        }"#;
        let tmp_dir = assert_fs::TempDir::new()?.into_persistent();
        let cli_config_file = tmp_dir.child(".safe/cli/config.json");
        cli_config_file.write_str(serialized_config)?;
        let node_config_file = tmp_dir.child(".safe/node/node_connection_info.config");
        let config = Config::new(
            PathBuf::from(cli_config_file.path()),
            PathBuf::from(node_config_file.path()),
        )?;

        let (network_name, network_info) = config
            .networks_iter()
            .next()
            .ok_or_else(|| eyre!("failed to obtain item from networks list"))?;
        assert_eq!(config.networks_iter().count(), 1);
        assert_eq!(network_name, "existing_network");
        match network_info {
            NetworkInfo::NodeConfig((_, contacts)) => {
                assert_eq!(contacts.len(), 2);

                let node: SocketAddr = "127.0.0.1:12000".parse().unwrap();
                assert_eq!(contacts.get(&node), Some(&node));
                let node: SocketAddr = "127.0.0.2:12000".parse().unwrap();
                assert_eq!(contacts.get(&node), Some(&node));
            }
            NetworkInfo::ConnInfoLocation(_) => {
                return Err(eyre!("connection info doesn't apply to this test"));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod read_current_node_config {
    use super::Config;
    use assert_fs::prelude::*;
    use color_eyre::Result;
    use std::net::SocketAddr;
    use std::path::PathBuf;

    #[test]
    fn given_existing_node_config_then_it_should_be_read() -> Result<()> {
        let genesis_key_hex = "89505bbfcac9335a7639a1dca9ed027b98be46b03953e946e53695f678c827f18f6fc22dc888de2bce9078f3fce55095";
        let serialized_node_config = r#"
        [
            "89505bbfcac9335a7639a1dca9ed027b98be46b03953e946e53695f678c827f18f6fc22dc888de2bce9078f3fce55095",
            [
                "127.0.0.1:33314",
                "127.0.0.1:38932",
                "127.0.0.1:39132",
                "127.0.0.1:47795",
                "127.0.0.1:49976",
                "127.0.0.1:53018",
                "127.0.0.1:53421",
                "127.0.0.1:54002",
                "127.0.0.1:54386",
                "127.0.0.1:55890",
                "127.0.0.1:57956"
            ]
        ]"#;

        let tmp_dir = assert_fs::TempDir::new()?;
        let cli_config_file = tmp_dir.child(".safe/cli/config.json");
        let node_config_file = tmp_dir.child(".safe/node/node_connection_info.config");
        node_config_file.write_str(serialized_node_config)?;
        let config = Config::new(
            PathBuf::from(cli_config_file.path()),
            PathBuf::from(node_config_file.path()),
        )?;

        let (node_config_path, node_config) = config.read_current_node_config()?;

        let genesis_key = node_config.0;
        let retrieved_genesis_key_hex = hex::encode(genesis_key.to_bytes());
        let nodes = node_config.1;
        assert_eq!(genesis_key_hex, retrieved_genesis_key_hex);
        assert_eq!(node_config_file.path(), node_config_path);
        assert_eq!(nodes.len(), 11);

        let node: SocketAddr = "127.0.0.1:33314".parse().unwrap();
        assert_eq!(nodes.get(&node), Some(&node));

        Ok(())
    }

    #[test]
    fn given_no_existing_node_config_file_the_result_should_be_an_error() -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let cli_config_file = tmp_dir.child(".safe/cli/config.json");
        let node_config_file = tmp_dir.child(".safe/node/node_connection_info.config");
        let config = Config::new(
            PathBuf::from(cli_config_file.path()),
            PathBuf::from(node_config_file.path()),
        )?;

        let result = config.read_current_node_config();

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "There doesn't seem to be any node configuration setup in your system."
        );

        Ok(())
    }

    #[test]
    fn given_node_config_path_points_to_non_node_config_file_the_result_should_be_an_error(
    ) -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let cli_config_file = tmp_dir.child(".safe/cli/config.json");
        let node_config_file = tmp_dir.child(".safe/node/node_connection_info.config");
        node_config_file.write_str("this is not a node config file")?;
        let config = Config::new(
            PathBuf::from(cli_config_file.path()),
            PathBuf::from(node_config_file.path()),
        )?;

        let result = config.read_current_node_config();

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            format!(
                "Unable to read current network connection information from '{}'.",
                node_config_file.path().display()
            )
        );
        Ok(())
    }
}

#[cfg(test)]
mod add_network {
    use super::{Config, NetworkInfo};
    use assert_fs::prelude::*;
    use color_eyre::{eyre::eyre, Result};
    use predicates::prelude::*;
    use std::collections::BTreeSet;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::path::PathBuf;

    #[test]
    fn given_network_info_not_supplied_then_current_network_config_will_be_cached() -> Result<()> {
        let serialized_node_config = r#"
        [
            "89505bbfcac9335a7639a1dca9ed027b98be46b03953e946e53695f678c827f18f6fc22dc888de2bce9078f3fce55095",
            [
                "127.0.0.1:33314",
                "127.0.0.1:38932",
                "127.0.0.1:39132",
                "127.0.0.1:47795",
                "127.0.0.1:49976",
                "127.0.0.1:53018",
                "127.0.0.1:53421",
                "127.0.0.1:54002",
                "127.0.0.1:54386",
                "127.0.0.1:55890",
                "127.0.0.1:57956"
            ]
        ]"#;

        let tmp_dir = assert_fs::TempDir::new()?;
        let cli_config_file = tmp_dir.child(".safe/cli/config.json");
        let node_config_file = tmp_dir.child(".safe/node/node_connection_info.config");
        let new_network_file =
            tmp_dir.child(".safe/cli/networks/new_network_node_connection_info.config");
        node_config_file.write_str(serialized_node_config)?;
        let mut config = Config::new(
            PathBuf::from(cli_config_file.path()),
            PathBuf::from(node_config_file.path()),
        )?;

        let result = config.add_network("new_network", None);

        assert!(result.is_ok());
        new_network_file.assert(predicate::path::is_file());

        let network = config.networks_iter().next().unwrap();
        let network_name = network.0;
        let network_info = network.1;
        assert_eq!(network_name, "new_network");
        match network_info {
            NetworkInfo::NodeConfig(_) => {
                return Err(eyre!("node config doesn't apply to this test"));
            }
            NetworkInfo::ConnInfoLocation(path) => {
                assert_eq!(
                    *path,
                    String::from(new_network_file.path().to_str().unwrap())
                );
            }
        }
        Ok(())
    }

    #[test]
    fn given_no_pre_existing_config_and_a_file_path_is_used_then_a_network_should_be_saved(
    ) -> Result<()> {
        let existing_node_config = r#"
        [
          "89505bbfcac9335a7639a1dca9ed027b98be46b03953e946e53695f678c827f18f6fc22dc888de2bce9078f3fce55095",
          [
            "127.0.0.1:33314",
            "127.0.0.1:38932",
            "127.0.0.1:39132",
            "127.0.0.1:47795",
            "127.0.0.1:49976",
            "127.0.0.1:53018",
            "127.0.0.1:53421",
            "127.0.0.1:54002",
            "127.0.0.1:54386",
            "127.0.0.1:55890",
            "127.0.0.1:57956"
          ]
        ]"#;

        let config_dir = assert_fs::TempDir::new()?;
        let cli_config_file = config_dir.child(".safe/cli/config.json");
        let node_config_file = config_dir.child("saved_connection_info.config");
        node_config_file.write_str(existing_node_config)?;
        let mut config = Config::new(
            cli_config_file.path().to_path_buf(),
            node_config_file.path().to_path_buf(),
        )?;

        let result = config.add_network(
            "new_network",
            Some(NetworkInfo::ConnInfoLocation(String::from(
                node_config_file.path().to_str().unwrap(),
            ))),
        );

        assert!(result.is_ok());
        cli_config_file.assert(predicate::path::is_file());

        assert_eq!(config.networks_iter().count(), 1);

        let network = config.networks_iter().next().unwrap();
        let network_name = network.0;
        let network_info = network.1;
        assert_eq!(network_name, "new_network");
        match network_info {
            NetworkInfo::NodeConfig(_) => {
                eyre!("node config doesn't apply to this test");
            }
            NetworkInfo::ConnInfoLocation(path) => {
                assert_eq!(
                    *path,
                    String::from(node_config_file.path().to_str().unwrap())
                );
            }
        }
        Ok(())
    }

    #[test]
    fn given_no_pre_existing_config_and_a_non_existent_file_path_is_used_then_the_result_should_be_an_error(
    ) -> Result<()> {
        let config_dir = assert_fs::TempDir::new()?;
        let cli_config_file = config_dir.child(".safe/cli/config.json");
        let node_config_file = config_dir.child(".safe/node/node_connection_info.config");
        let mut config = Config::new(
            cli_config_file.path().to_path_buf(),
            node_config_file.path().to_path_buf(),
        )?;

        let result = config.add_network(
            "new_network",
            Some(NetworkInfo::ConnInfoLocation(String::from(
                node_config_file.path().to_str().unwrap(),
            ))),
        );

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "The config location must use an existing file path."
        );

        Ok(())
    }

    #[test]
    fn given_no_pre_existing_config_and_a_file_that_is_not_a_network_config_is_used_then_the_result_should_be_an_error(
    ) -> Result<()> {
        let config_dir = assert_fs::TempDir::new()?;
        let cli_config_file = config_dir.child(".safe/cli/config.json");
        let node_config_file = config_dir.child(".safe/node/node_connection_info.config");
        node_config_file.write_str("file that is not a network config")?;
        let mut config = Config::new(
            cli_config_file.path().to_path_buf(),
            node_config_file.path().to_path_buf(),
        )?;

        let result = config.add_network(
            "new_network",
            Some(NetworkInfo::ConnInfoLocation(String::from(
                node_config_file.path().to_str().unwrap(),
            ))),
        );

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "The file must contain a valid network configuration."
        );
        Ok(())
    }

    #[test]
    fn given_no_pre_existing_config_and_a_url_is_used_then_a_network_should_be_saved() -> Result<()>
    {
        let config_dir = assert_fs::TempDir::new()?;
        let cli_config_file = config_dir.child(".safe/cli/config.json");
        let node_config_file = config_dir.child(".safe/node/node_connection_info.config");
        let mut config = Config::new(
            cli_config_file.path().to_path_buf(),
            node_config_file.path().to_path_buf(),
        )?;
        let url = "https://sn-node.s3.eu-west-2.amazonaws.com/config/node_connection_info.config";

        let result = config.add_network(
            "new_network",
            Some(NetworkInfo::ConnInfoLocation(String::from(url))),
        );

        assert!(result.is_ok());
        cli_config_file.assert(predicate::path::is_file());

        assert_eq!(config.networks_iter().count(), 1);

        let network = config.networks_iter().next().unwrap();
        let network_name = network.0;
        let network_info = network.1;
        assert_eq!(network_name, "new_network");
        match network_info {
            NetworkInfo::NodeConfig(_) => {
                eyre!("node config doesn't apply to this test");
            }
            NetworkInfo::ConnInfoLocation(url) => {
                assert_eq!(*url, String::from(url));
            }
        }
        Ok(())
    }

    #[test]
    fn given_a_pre_existing_config_and_a_network_with_the_same_name_exists_then_the_existing_network_should_be_overwritten(
    ) -> Result<()> {
        // Arrange
        // Setup existing config.
        let serialized_config = r#"
        {
            "networks": {
                "existing_network": {
                    "NodeConfig":[
                        [140,44,196,143,12,92,218,53,190,33,205,167,109,183,94,205,16,140,197,200,96,112,136,218,221,16,57,54,204,60,58,93,199,119,26,17,105,232,33,188,163,194,145,223,194,95,92,54],
                        ["127.0.0.1:12000","127.0.0.2:12000"]
                    ]
                }
            }
        }"#;
        let config_dir = assert_fs::TempDir::new()?;
        let cli_config_file = config_dir.child(".safe/cli/config.json");
        let node_config_file = config_dir.child(".safe/node/node_connection_info.config");
        cli_config_file.write_str(serialized_config)?;
        let mut config = Config::new(
            cli_config_file.path().to_path_buf(),
            node_config_file.path().to_path_buf(),
        )?;

        // Setup new network info.
        let secret_key = bls::SecretKey::random();
        let genesis_key = hex::encode(secret_key.public_key().to_bytes());
        let mut nodes: BTreeSet<SocketAddr> = BTreeSet::new();
        nodes.insert(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            12000,
        ));
        nodes.insert(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
            12000,
        ));

        // Act
        let result = config.add_network(
            "existing_network",
            Some(NetworkInfo::NodeConfig((secret_key.public_key(), nodes))),
        );

        // Assert
        // We still only have 1 network, but the node config was overwritten.
        assert!(result.is_ok());
        assert_eq!(config.networks_iter().count(), 1);

        let network = config.networks_iter().next().unwrap();
        let network_name = network.0;
        let network_info = network.1;
        assert_eq!(network_name, "existing_network");
        match network_info {
            NetworkInfo::NodeConfig(node_config) => {
                assert_eq!(node_config.1.len(), 2);

                let node: SocketAddr = "10.0.0.1:12000".parse().unwrap();
                assert_eq!(node_config.1.get(&node), Some(&node));
                let node: SocketAddr = "10.0.0.2:12000".parse().unwrap();
                assert_eq!(node_config.1.get(&node), Some(&node));

                let public_key = node_config.0;
                assert_eq!(hex::encode(public_key.to_bytes()), genesis_key);
            }
            NetworkInfo::ConnInfoLocation(_) => {
                eyre!("connection info doesn't apply to this test");
            }
        }

        Ok(())
    }

    #[test]
    fn given_a_pre_existing_config_and_a_new_node_config_is_specified_then_a_new_network_should_be_added(
    ) -> Result<()> {
        let serialized_config = r#"
        {
            "networks": {
                "existing_network": {
                    "NodeConfig":[
                        [140,44,196,143,12,92,218,53,190,33,205,167,109,183,94,205,16,140,197,200,96,112,136,218,221,16,57,54,204,60,58,93,199,119,26,17,105,232,33,188,163,194,145,223,194,95,92,54],
                        ["127.0.0.1:12000","127.0.0.2:12000"]
                    ]
                }
            }
        }"#;
        let config_dir = assert_fs::TempDir::new()?;
        let cli_config_file = config_dir.child(".safe/cli/config.json");
        let node_config_file = config_dir.child(".safe/node/node_connection_info.config");
        cli_config_file.write_str(serialized_config)?;
        let mut config = Config::new(
            cli_config_file.path().to_path_buf(),
            node_config_file.path().to_path_buf(),
        )?;

        // Setup new network info.
        let secret_key = bls::SecretKey::random();
        let genesis_key = hex::encode(secret_key.public_key().to_bytes());
        let mut nodes: BTreeSet<SocketAddr> = BTreeSet::new();
        nodes.insert(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            12000,
        ));
        nodes.insert(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
            12000,
        ));

        // Act
        let result = config.add_network(
            "new_network",
            Some(NetworkInfo::NodeConfig((secret_key.public_key(), nodes))),
        );

        // Assert
        assert!(result.is_ok());
        assert_eq!(config.networks_iter().count(), 2);

        let network = config.networks_iter().nth(1).unwrap();
        let network_name = network.0;
        let network_info = network.1;
        assert_eq!(network_name, "new_network");
        match network_info {
            NetworkInfo::NodeConfig(node_config) => {
                assert_eq!(node_config.1.len(), 2);

                let node: SocketAddr = "10.0.0.1:12000".parse().unwrap();
                assert_eq!(node_config.1.get(&node), Some(&node));
                let node: SocketAddr = "10.0.0.2:12000".parse().unwrap();
                assert_eq!(node_config.1.get(&node), Some(&node));

                let public_key = node_config.0;
                assert_eq!(hex::encode(public_key.to_bytes()), genesis_key);
            }
            NetworkInfo::ConnInfoLocation(_) => {
                eyre!("connection info doesn't apply to this test");
            }
        }

        Ok(())
    }

    #[test]
    fn given_a_pre_existing_config_and_a_conn_info_location_is_specified_then_a_new_network_should_be_added(
    ) -> Result<()> {
        // Arrange
        let serialized_config = r#"
        {
            "networks": {
                "existing_network": {
                    "NodeConfig":[
                        [140,44,196,143,12,92,218,53,190,33,205,167,109,183,94,205,16,140,197,200,96,112,136,218,221,16,57,54,204,60,58,93,199,119,26,17,105,232,33,188,163,194,145,223,194,95,92,54],
                        ["127.0.0.1:12000","127.0.0.2:12000"]
                    ]
                }
            }
        }"#;
        let serialized_node_config = r#"
        [
            "89505bbfcac9335a7639a1dca9ed027b98be46b03953e946e53695f678c827f18f6fc22dc888de2bce9078f3fce55095",
            [
                "127.0.0.1:33314",
                "127.0.0.1:38932",
                "127.0.0.1:39132",
                "127.0.0.1:47795",
                "127.0.0.1:49976",
                "127.0.0.1:53018",
                "127.0.0.1:53421",
                "127.0.0.1:54002",
                "127.0.0.1:54386",
                "127.0.0.1:55890",
                "127.0.0.1:57956"
            ]
        ]"#;
        let config_dir = assert_fs::TempDir::new()?;
        let cli_config_file = config_dir.child(".safe/cli/config.json");
        let node_config_file = config_dir.child(".safe/node/node_connection_info.config");
        let existing_node_config_file = config_dir.child("node_connection_info.config");
        existing_node_config_file.write_str(serialized_node_config)?;
        cli_config_file.write_str(serialized_config)?;
        let mut config = Config::new(
            cli_config_file.path().to_path_buf(),
            node_config_file.path().to_path_buf(),
        )?;

        // Act
        let result = config.add_network(
            "new_network",
            Some(NetworkInfo::ConnInfoLocation(String::from(
                existing_node_config_file.path().to_str().unwrap(),
            ))),
        );

        // Assert
        assert!(result.is_ok());
        assert_eq!(config.networks_iter().count(), 2);

        let network = config.networks_iter().nth(1).unwrap();
        let network_name = network.0;
        let network_info = network.1;
        assert_eq!(network_name, "new_network");
        match network_info {
            NetworkInfo::NodeConfig(_) => {
                return Err(eyre!("node config doesn't apply to this test"));
            }
            NetworkInfo::ConnInfoLocation(path) => {
                assert_eq!(
                    *path,
                    String::from(existing_node_config_file.path().to_str().unwrap())
                );
            }
        }

        Ok(())
    }
}
