// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use color_eyre::{eyre::bail, eyre::eyre, eyre::WrapErr, Help, Report, Result};
use comfy_table::Table;
use serde::{Deserialize, Serialize};
use sn_api::keys::deserialize_keypair;
use sn_api::{NodeConfig, PublicKey};
use sn_dbc::Owner;
use std::{
    collections::{BTreeMap, BTreeSet},
    default::Default,
    fmt,
    net::SocketAddr,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};
use structopt::StructOpt;
use tokio::fs;
use tracing::debug;

const CONFIG_NETWORKS_DIRNAME: &str = "networks";

/// Provides an interface for calling a launcher tool for launching and joining the network.
///
/// There will only be 2 implementations of this: one that uses the `sn_launch_tool` and another
/// that uses a fake launch tool for use in unit tests.
///
/// The only reason the trait exists is for enabling unit testing.
pub trait NetworkLauncher {
    fn launch(&mut self, args: Vec<String>, interval: u64) -> Result<(), Report>;
    fn join(&mut self, args: Vec<String>) -> Result<(), Report>;
}

/// A network launcher based on the `sn_launch_tool`, which provides an implementation of a
/// `NetworkLauncher`.
///
/// This is just a thin wrapper around the launch tool.
#[derive(Default)]
pub struct SnLaunchToolNetworkLauncher {}
impl NetworkLauncher for SnLaunchToolNetworkLauncher {
    fn launch(&mut self, args: Vec<String>, interval: u64) -> Result<(), Report> {
        debug!("Running network launch tool with args: {:?}", args);
        println!("Starting a node to join a Safe network...");
        sn_launch_tool::Launch::from_iter_safe(&args)
            .map_err(|e| eyre!(e))
            .and_then(|launch| launch.run())
            .wrap_err("Error launching node")?;

        let interval_duration = Duration::from_secs(interval * 15);
        thread::sleep(interval_duration);

        Ok(())
    }

    fn join(&mut self, args: Vec<String>) -> Result<(), Report> {
        debug!("Running network launch tool with args: {:?}", args);
        println!("Starting a node to join a Safe network...");
        sn_launch_tool::Join::from_iter_safe(&args)
            .map_err(|e| eyre!(e))
            .and_then(|launch| launch.run())
            .wrap_err("Error launching node")?;
        Ok(())
    }
}

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
    pub cli_config_path: PathBuf,
    pub node_config_path: PathBuf,
    pub dbc_owner: Option<Owner>,
}

impl Config {
    pub async fn new(cli_config_path: PathBuf, node_config_path: PathBuf) -> Result<Config> {
        let mut pb = cli_config_path.clone();
        pb.pop();
        fs::create_dir_all(pb.as_path()).await?;

        let settings = if cli_config_path.exists() {
            let content = fs::read(&cli_config_path).await.wrap_err_with(|| {
                format!(
                    "Error reading config file from '{}'",
                    cli_config_path.display(),
                )
            })?;
            if content.is_empty() {
                // During the CLI test run, when running with more than one thread, i.e., running
                // multiple instances of safe at the same time, it seems to be possible for there
                // to be an empty config file, even though I can't determine how the scenario
                // occurs.
                //
                // Checking if the content is empty prevents an error from trying to deserialize an
                // empty byte array. We can just return the default empty settings.
                //
                // This shouldn't have any adverse effects on users, since concurrently running
                // multiple instances of safe is unlikely.
                Settings::default()
            } else {
                let settings = serde_json::from_slice(&content).wrap_err_with(|| {
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
                settings
            }
        } else {
            debug!(
                "Empty config file created at '{}'",
                cli_config_path.display()
            );
            Settings::default()
        };

        let mut dbc_owner_sk_path = pb.clone();
        dbc_owner_sk_path.push("credentials");
        let dbc_owner = Config::get_dbc_owner(&dbc_owner_sk_path).await?;
        let config = Config {
            settings,
            cli_config_path: cli_config_path.clone(),
            node_config_path,
            dbc_owner,
        };
        config.write_settings_to_file().await.wrap_err_with(|| {
            format!("Unable to create config at '{}'", cli_config_path.display())
        })?;
        Ok(config)
    }

    pub async fn read_current_node_config(&self) -> Result<(PathBuf, NodeConfig)> {
        let current_conn_info = fs::read(&self.node_config_path).await.wrap_err_with(|| {
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

    pub async fn add_network(
        &mut self,
        name: &str,
        net_info: Option<NetworkInfo>,
    ) -> Result<NetworkInfo> {
        let net_info = if let Some(info) = net_info {
            info
        } else {
            // Cache current network connection info
            let (_, node_config) = self.read_current_node_config().await?;
            let cache_path = self.cache_node_config(name, &node_config).await?;
            println!(
                "Caching current network connection information into '{}'",
                cache_path.display()
            );
            NetworkInfo::ConnInfoLocation(cache_path.display().to_string())
        };

        match &net_info {
            NetworkInfo::NodeConfig(_) => {}
            NetworkInfo::ConnInfoLocation(location) => {
                let is_invalid_location = match url::Url::parse(location) {
                    Err(_) => true,
                    Ok(location) => !location.has_host(),
                };

                if is_invalid_location {
                    // The location is not a valid URL, so try and parse it as a file.
                    let pb = PathBuf::from(Path::new(&location));
                    if !pb.is_file() {
                        return Err(eyre!("The config location must use an existing file path.")
                            .suggestion(
                                "Please choose an existing file with a network configuration.",
                            ));
                    }
                    deserialise_node_config(&fs::read(pb.as_path()).await?).wrap_err_with(
                        || {
                            eyre!("The file must contain a valid network configuration.")
                                .suggestion(
                                "Please choose another file with a valid network configuration.",
                            )
                        },
                    )?;
                }
            }
        }
        self.settings
            .networks
            .insert(name.to_string(), net_info.clone());

        self.write_settings_to_file().await?;

        debug!("Network '{}' added to settings: {}", name, net_info);
        Ok(net_info)
    }

    pub async fn remove_network(&mut self, name: &str) -> Result<()> {
        match self.settings.networks.remove(name) {
            Some(NetworkInfo::ConnInfoLocation(location)) => {
                self.write_settings_to_file().await?;
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

                    if let Err(err) = fs::remove_file(&location).await {
                        println!(
                            "Failed to remove cached network connection information from '{}': {}",
                            location, err
                        );
                    }
                }
            }
            Some(NetworkInfo::NodeConfig(_)) => {
                self.write_settings_to_file().await?;
                debug!("Network '{}' removed from config", name);
                println!("Network '{}' was removed from the config", name);
            }
            None => println!("No network with name '{}' was found in config", name),
        }

        Ok(())
    }

    pub async fn clear(&mut self) -> Result<()> {
        self.settings = Settings::default();
        self.write_settings_to_file().await
    }

    pub async fn switch_to_network(&self, name: &str) -> Result<()> {
        let mut base_path = self.node_config_path.clone();
        base_path.pop();

        if !base_path.exists() {
            println!(
                "Creating '{}' folder for network connection info",
                base_path.display()
            );
            fs::create_dir_all(&base_path)
                .await
                .wrap_err("Couldn't create folder for network connection info")?;
        }

        let contacts = self.get_network_info(name).await?;
        let conn_info = serialise_node_config(&contacts)?;
        fs::write(&self.node_config_path, conn_info)
            .await
            .wrap_err_with(|| {
                format!(
                    "Unable to write network connection info in '{}'",
                    base_path.display(),
                )
            })
    }

    pub async fn print_networks(&self) {
        let mut table = Table::new();
        table.add_row(&vec!["Networks"]);
        table.add_row(&vec!["Current", "Network name", "Connection info"]);
        let current_node_config = match self.read_current_node_config().await {
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
            table.add_row(&vec![current, network_name, &format!("{:?}", net_info)]);
        }

        println!("{table}");
    }

    ///
    /// Private helpers
    ///

    async fn get_dbc_owner(dbc_sk_path: &Path) -> Result<Option<Owner>> {
        if dbc_sk_path.exists() {
            let keypair = deserialize_keypair(dbc_sk_path)?;
            let sk = keypair.secret_key()?.bls().ok_or_else(|| {
                eyre!("The CLI keypair must be a BLS keypair.")
                    .suggestion("Use the keys create command to generate a BLS keypair.")
            })?;
            return Ok(Some(Owner::from(sk)));
        }
        Ok(None)
    }

    async fn cache_node_config(
        &self,
        network_name: &str,
        node_config: &NodeConfig,
    ) -> Result<PathBuf> {
        let mut pb = self.cli_config_path.clone();
        pb.pop();
        pb.push(CONFIG_NETWORKS_DIRNAME);
        if !pb.exists() {
            println!(
                "Creating '{}' folder for networks connection info cache",
                pb.display()
            );
            fs::create_dir_all(&pb)
                .await
                .wrap_err("Couldn't create folder for networks information cache")?;
        }

        pb.push(format!("{}_node_connection_info.config", network_name));
        let conn_info = serialise_node_config(node_config)?;
        fs::write(&pb, conn_info).await?;
        Ok(pb)
    }

    async fn write_settings_to_file(&self) -> Result<()> {
        let serialised_settings = serde_json::to_string(&self.settings)
            .wrap_err("Failed to serialise config settings")?;
        fs::write(&self.cli_config_path, serialised_settings.as_bytes())
            .await
            .wrap_err_with(|| {
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
        let resp = reqwest::get(location).await.wrap_err_with(|| {
            format!("Failed to fetch connection information from '{}'", location)
        })?;

        let conn_info = resp.text().await.wrap_err_with(|| {
            format!("Failed to fetch connection information from '{}'", location)
        })?;

        conn_info.as_bytes().to_vec()
    } else {
        // Fetch it from a local file then
        fs::read(location).await.wrap_err_with(|| {
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
    use sn_api::{Keypair, Safe};
    use std::net::SocketAddr;
    use std::path::PathBuf;

    #[tokio::test]
    async fn fields_should_be_set_to_correct_values() -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let cli_config_dir = tmp_dir.child(".safe/cli");
        cli_config_dir.create_dir_all()?;

        let cli_config_file = cli_config_dir.child("config.json");
        let node_config_file = tmp_dir.child(".safe/node/node_connection_info.config");
        let dbc_owner_sk_file = cli_config_dir.child("credentials");
        let keypair = Keypair::new_bls();
        let safe = Safe::dry_runner(None);
        safe.serialize_keypair(&keypair, dbc_owner_sk_file.path())?;

        let config = Config::new(
            PathBuf::from(cli_config_file.path()),
            PathBuf::from(node_config_file.path()),
        )
        .await?;

        assert_eq!(config.cli_config_path, cli_config_file.path());
        assert_eq!(config.node_config_path, node_config_file.path());
        assert_eq!(config.settings.networks.len(), 0);
        assert!(config.dbc_owner.is_some());
        Ok(())
    }

    #[tokio::test]
    async fn cli_config_directory_should_be_created() -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let cli_config_dir = tmp_dir.child(".safe/cli");
        let cli_config_file = cli_config_dir.child("config.json");
        let node_config_file = tmp_dir.child(".safe/node/node_connection_info.config");

        let _ = Config::new(
            PathBuf::from(cli_config_file.path()),
            PathBuf::from(node_config_file.path()),
        )
        .await?;

        cli_config_dir.assert(predicate::path::is_dir());
        Ok(())
    }

    #[tokio::test]
    async fn given_config_file_does_not_exist_then_it_should_be_created() -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let cli_config_file = tmp_dir.child(".safe/cli/config.json");
        let node_config_file = tmp_dir.child(".safe/node/node_connection_info.config");
        let _ = Config::new(
            PathBuf::from(cli_config_file.path()),
            PathBuf::from(node_config_file.path()),
        )
        .await?;

        cli_config_file.assert(predicate::path::exists());

        Ok(())
    }

    #[tokio::test]
    async fn given_config_file_exists_then_the_settings_should_be_read() -> Result<()> {
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
        let tmp_dir = assert_fs::TempDir::new()?;
        let cli_config_file = tmp_dir.child(".safe/cli/config.json");
        cli_config_file.write_str(serialized_config)?;
        let node_config_file = tmp_dir.child(".safe/node/node_connection_info.config");
        let config = Config::new(
            PathBuf::from(cli_config_file.path()),
            PathBuf::from(node_config_file.path()),
        )
        .await?;

        let (network_name, network_info) = config
            .networks_iter()
            .next()
            .ok_or_else(|| eyre!("failed to obtain item from networks list"))?;
        assert_eq!(config.networks_iter().count(), 1);
        assert_eq!(network_name, "existing_network");
        match network_info {
            NetworkInfo::NodeConfig((_, contacts)) => {
                assert_eq!(contacts.len(), 2);

                let node: SocketAddr = "127.0.0.1:12000".parse()?;
                assert_eq!(contacts.get(&node), Some(&node));
                let node: SocketAddr = "127.0.0.2:12000".parse()?;
                assert_eq!(contacts.get(&node), Some(&node));
            }
            NetworkInfo::ConnInfoLocation(_) => {
                return Err(eyre!("connection info doesn't apply to this test"));
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn given_an_empty_config_file_empty_settings_should_be_returned() -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let cli_config_file = tmp_dir.child(".safe/cli/config.json");
        cli_config_file.touch()?;
        let node_config_file = tmp_dir.child(".safe/node/node_connection_info.config");
        let config = Config::new(
            PathBuf::from(cli_config_file.path()),
            PathBuf::from(node_config_file.path()),
        )
        .await?;

        assert_eq!(0, config.settings.networks.len());
        assert_eq!(cli_config_file.path(), config.cli_config_path.as_path());
        assert_eq!(node_config_file.path(), config.node_config_path.as_path());

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

    #[tokio::test]
    async fn given_existing_node_config_then_it_should_be_read() -> Result<()> {
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
        )
        .await?;

        let (node_config_path, node_config) = config.read_current_node_config().await?;

        let genesis_key = node_config.0;
        let retrieved_genesis_key_hex = hex::encode(genesis_key.to_bytes());
        let nodes = node_config.1;
        assert_eq!(genesis_key_hex, retrieved_genesis_key_hex);
        assert_eq!(node_config_file.path(), node_config_path);
        assert_eq!(nodes.len(), 11);

        let node: SocketAddr = "127.0.0.1:33314".parse()?;
        assert_eq!(nodes.get(&node), Some(&node));

        Ok(())
    }

    #[tokio::test]
    async fn given_no_existing_node_config_file_the_result_should_be_an_error() -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let cli_config_file = tmp_dir.child(".safe/cli/config.json");
        let node_config_file = tmp_dir.child(".safe/node/node_connection_info.config");
        let config = Config::new(
            PathBuf::from(cli_config_file.path()),
            PathBuf::from(node_config_file.path()),
        )
        .await?;

        let result = config.read_current_node_config().await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "There doesn't seem to be any node configuration setup in your system."
        );

        Ok(())
    }

    #[tokio::test]
    async fn given_node_config_path_points_to_non_node_config_file_the_result_should_be_an_error(
    ) -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let cli_config_file = tmp_dir.child(".safe/cli/config.json");
        let node_config_file = tmp_dir.child(".safe/node/node_connection_info.config");
        node_config_file.write_str("this is not a node config file")?;
        let config = Config::new(
            PathBuf::from(cli_config_file.path()),
            PathBuf::from(node_config_file.path()),
        )
        .await?;

        let result = config.read_current_node_config().await;

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
    use color_eyre::{
        eyre::{bail, eyre},
        Result,
    };
    use predicates::prelude::*;
    use std::collections::BTreeSet;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::path::{Path, PathBuf};

    #[tokio::test]
    async fn given_network_info_not_supplied_then_current_network_config_will_be_cached(
    ) -> Result<()> {
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
        )
        .await?;

        let result = config.add_network("new_network", None).await;

        assert!(result.is_ok());
        new_network_file.assert(predicate::path::is_file());

        let network = config
            .networks_iter()
            .next()
            .ok_or_else(|| eyre!("failed to read network from config"))?;
        let network_name = network.0;
        let network_info = network.1;
        assert_eq!(network_name, "new_network");
        match network_info {
            NetworkInfo::NodeConfig(_) => {
                return Err(eyre!("node config doesn't apply to this test"));
            }
            NetworkInfo::ConnInfoLocation(conn_info_path) => {
                let path = Path::new(conn_info_path);
                assert_eq!(path, new_network_file.path());
            }
        }
        Ok(())
    }

    #[tokio::test]
    async fn given_no_pre_existing_config_and_a_file_path_is_used_then_a_network_should_be_saved(
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
        )
        .await?;

        let result = config
            .add_network(
                "new_network",
                Some(NetworkInfo::ConnInfoLocation(
                    node_config_file.path().display().to_string(),
                )),
            )
            .await;

        assert!(result.is_ok());
        cli_config_file.assert(predicate::path::is_file());

        assert_eq!(config.networks_iter().count(), 1);

        let network = config
            .networks_iter()
            .next()
            .ok_or_else(|| eyre!("failed to read network from config"))?;
        let network_name = network.0;
        let network_info = network.1;
        assert_eq!(network_name, "new_network");
        match network_info {
            NetworkInfo::NodeConfig(_) => {
                bail!("node config doesn't apply to this test");
            }
            NetworkInfo::ConnInfoLocation(path) => {
                assert_eq!(*path, node_config_file.path().display().to_string());
            }
        }
        Ok(())
    }

    #[tokio::test]
    async fn given_no_pre_existing_config_and_a_non_existent_file_path_is_used_then_the_result_should_be_an_error(
    ) -> Result<()> {
        let config_dir = assert_fs::TempDir::new()?.into_persistent();
        let cli_config_file = config_dir.child(".safe/cli/config.json");
        let node_config_file = config_dir.child(".safe/node/node_connection_info.config");
        let mut config = Config::new(
            cli_config_file.path().to_path_buf(),
            node_config_file.path().to_path_buf(),
        )
        .await?;

        let result = config
            .add_network(
                "new_network",
                Some(NetworkInfo::ConnInfoLocation(
                    node_config_file.path().display().to_string(),
                )),
            )
            .await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "The config location must use an existing file path."
        );

        Ok(())
    }

    #[tokio::test]
    async fn given_no_pre_existing_config_and_a_file_that_is_not_a_network_config_is_used_then_the_result_should_be_an_error(
    ) -> Result<()> {
        let config_dir = assert_fs::TempDir::new()?;
        let cli_config_file = config_dir.child(".safe/cli/config.json");
        let node_config_file = config_dir.child(
            Path::new(".safe")
                .join("node")
                .join("node_connection_info.config"),
        );
        node_config_file.write_str("file that is not a network config")?;
        let mut config = Config::new(
            cli_config_file.path().to_path_buf(),
            node_config_file.path().to_path_buf(),
        )
        .await?;

        let result = config
            .add_network(
                "new_network",
                Some(NetworkInfo::ConnInfoLocation(
                    node_config_file.path().display().to_string(),
                )),
            )
            .await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "The file must contain a valid network configuration."
        );
        Ok(())
    }

    #[tokio::test]
    async fn given_no_pre_existing_config_and_a_url_is_used_then_a_network_should_be_saved(
    ) -> Result<()> {
        let config_dir = assert_fs::TempDir::new()?;
        let cli_config_file = config_dir.child(".safe/cli/config.json");
        let node_config_file = config_dir.child(".safe/node/node_connection_info.config");
        let mut config = Config::new(
            cli_config_file.path().to_path_buf(),
            node_config_file.path().to_path_buf(),
        )
        .await?;
        let url = "https://sn-node.s3.eu-west-2.amazonaws.com/config/node_connection_info.config";

        let result = config
            .add_network(
                "new_network",
                Some(NetworkInfo::ConnInfoLocation(String::from(url))),
            )
            .await;

        assert!(result.is_ok());
        cli_config_file.assert(predicate::path::is_file());

        assert_eq!(config.networks_iter().count(), 1);

        let network = config
            .networks_iter()
            .next()
            .ok_or_else(|| eyre!("failed to read network from config"))?;
        let network_name = network.0;
        let network_info = network.1;
        assert_eq!(network_name, "new_network");
        match network_info {
            NetworkInfo::NodeConfig(_) => {
                bail!("node config doesn't apply to this test");
            }
            NetworkInfo::ConnInfoLocation(url) => {
                assert_eq!(*url, String::from(url));
            }
        }
        Ok(())
    }

    #[tokio::test]
    async fn given_a_pre_existing_config_and_a_network_with_the_same_name_exists_then_the_existing_network_should_be_overwritten(
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
        )
        .await?;

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
        let result = config
            .add_network(
                "existing_network",
                Some(NetworkInfo::NodeConfig((secret_key.public_key(), nodes))),
            )
            .await;

        // Assert
        // We still only have 1 network, but the node config was overwritten.
        assert!(result.is_ok());
        assert_eq!(config.networks_iter().count(), 1);

        let network = config
            .networks_iter()
            .next()
            .ok_or_else(|| eyre!("failed to read network from config"))?;
        let network_name = network.0;
        let network_info = network.1;
        assert_eq!(network_name, "existing_network");
        match network_info {
            NetworkInfo::NodeConfig(node_config) => {
                assert_eq!(node_config.1.len(), 2);

                let node: SocketAddr = "10.0.0.1:12000".parse()?;
                assert_eq!(node_config.1.get(&node), Some(&node));
                let node: SocketAddr = "10.0.0.2:12000".parse()?;
                assert_eq!(node_config.1.get(&node), Some(&node));

                let public_key = node_config.0;
                assert_eq!(hex::encode(public_key.to_bytes()), genesis_key);
            }
            NetworkInfo::ConnInfoLocation(_) => {
                bail!("connection info doesn't apply to this test");
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn given_a_pre_existing_config_and_a_new_node_config_is_specified_then_a_new_network_should_be_added(
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
        )
        .await?;

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
        let result = config
            .add_network(
                "new_network",
                Some(NetworkInfo::NodeConfig((secret_key.public_key(), nodes))),
            )
            .await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(config.networks_iter().count(), 2);

        let network = config
            .networks_iter()
            .nth(1)
            .ok_or_else(|| eyre!("failed to read network from config"))?;
        let network_name = network.0;
        let network_info = network.1;
        assert_eq!(network_name, "new_network");
        match network_info {
            NetworkInfo::NodeConfig(node_config) => {
                assert_eq!(node_config.1.len(), 2);

                let node: SocketAddr = "10.0.0.1:12000".parse()?;
                assert_eq!(node_config.1.get(&node), Some(&node));
                let node: SocketAddr = "10.0.0.2:12000".parse()?;
                assert_eq!(node_config.1.get(&node), Some(&node));

                let public_key = node_config.0;
                assert_eq!(hex::encode(public_key.to_bytes()), genesis_key);
            }
            NetworkInfo::ConnInfoLocation(_) => {
                bail!("connection info doesn't apply to this test");
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn given_a_pre_existing_config_and_a_conn_info_location_is_specified_then_a_new_network_should_be_added(
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
        )
        .await?;

        // Act
        let result = config
            .add_network(
                "new_network",
                Some(NetworkInfo::ConnInfoLocation(
                    existing_node_config_file.path().display().to_string(),
                )),
            )
            .await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(config.networks_iter().count(), 2);

        let network = config
            .networks_iter()
            .nth(1)
            .ok_or_else(|| eyre!("failed to read network from config"))?;
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
                    existing_node_config_file.path().display().to_string()
                );
            }
        }

        Ok(())
    }
}
