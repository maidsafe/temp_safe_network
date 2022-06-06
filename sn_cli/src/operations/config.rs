// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use bls::PublicKey as BlsPublicKey;
use bytes::Bytes;
use color_eyre::{eyre::bail, eyre::eyre, eyre::WrapErr, Help, Report, Result};
use comfy_table::Table;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use sn_api::{NetworkPrefixMap, Safe};
use sn_dbc::Owner;
#[cfg(unix)]
use std::os::unix::fs::symlink;
#[cfg(windows)]
use std::os::windows::fs::symlink_file;
use std::{
    collections::BTreeMap,
    default::Default,
    fmt,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};
use structopt::StructOpt;
use tokio::fs;
use tracing::debug;
use url::Url;

const REMOTE_RETRY_COUNT: usize = 3;

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

// ok so, fetch everythign and store it into prefix_maps_path. have an Some(BlsPublicKey) along with Local and Remote,
// which if present means that prefix map has been downloaded and available in that folder.
#[derive(Deserialize, Debug, Serialize, Clone)]
pub enum NetworkInfo {
    /// Genesis key of a PrefixMap at prefix_maps_path
    GenesisKey(BlsPublicKey),
    /// The Optional genesis key denotes that the PrefixMap has been copied to prefix_maps_path
    Local(PathBuf, Option<BlsPublicKey>),
    Remote(String, Option<BlsPublicKey>),
}

impl fmt::Display for NetworkInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GenesisKey(genesis_key) => write!(f, "{:?}", genesis_key),
            Self::Local(path, genesis_key) => {
                if let Some(gk) = genesis_key {
                    write!(f, "{:?} from {:?}", gk, path)
                } else {
                    write!(f, "{:?}", path)
                }
            }
            Self::Remote(url, genesis_key) => {
                if let Some(gk) = genesis_key {
                    write!(f, "{:?} from {:?}", gk, url)
                } else {
                    write!(f, "{}", url)
                }
            }
        }
    }
}

impl NetworkInfo {
    pub async fn matches(&self, genesis_key: &BlsPublicKey) -> bool {
        match self {
            Self::GenesisKey(gk) => gk == genesis_key,
            Self::Local(_, genesis_key_opt) => match genesis_key_opt {
                Some(gk) => gk == genesis_key,
                None => false,
            },
            Self::Remote(_, genesis_key_opt) => match genesis_key_opt {
                Some(gk) => gk == genesis_key,
                None => false,
            },
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
    pub prefix_maps_dir: PathBuf,
    pub prefix_map_symlink_name: String,
    pub dbc_owner: Option<Owner>,
}

impl Config {
    pub async fn new(
        cli_config_path: PathBuf,
        prefix_maps_dir: PathBuf,
        prefix_map_symlink_name: String,
    ) -> Result<Config> {
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

        fs::create_dir_all(prefix_maps_dir.as_path()).await?;
        let mut dbc_owner_sk_path = pb.clone();
        dbc_owner_sk_path.push("credentials");
        let dbc_owner = Config::get_dbc_owner(&dbc_owner_sk_path).await?;

        let config = Config {
            settings,
            cli_config_path: cli_config_path.clone(),
            prefix_maps_dir,
            prefix_map_symlink_name,
            dbc_owner,
        };
        config.write_settings_to_file().await.wrap_err_with(|| {
            format!("Unable to create config at '{}'", cli_config_path.display())
        })?;
        Ok(config)
    }

    /// Sync the hashmap and the prefix_map_dir
    pub async fn sync(&mut self) -> Result<()> {
        let mut dir_files: BTreeMap<String, bool> = BTreeMap::new();
        let mut prefix_maps_dir = fs::read_dir(&self.prefix_maps_dir).await?;
        while let Some(entry) = prefix_maps_dir.next_entry().await? {
            // check excludes symlink
            if entry.metadata().await?.is_file() {
                let filename = entry
                    .file_name()
                    .into_string()
                    .map_err(|_| eyre!("Error converting OsString to String"))?;
                dir_files.insert(filename, false);
            }
        }

        // get NetworkPrefixMap from cli_config if they are not in prefix_maps_dir
        let mut remove_list: Vec<String> = Vec::new();
        for (network_name, net_info) in self.settings.networks.iter_mut() {
            match net_info {
                NetworkInfo::GenesisKey(genesis_key) => {
                    match dir_files.get_mut(format!("{:?}", genesis_key).as_str()) {
                        Some(present) => {
                            *present = true;
                        }
                        // remove entry from cli_config as there's no way to fetch the PrefixMap
                        None => {
                            remove_list.push(network_name.clone());
                        }
                    }
                }
                NetworkInfo::Local(path, ref mut genesis_key) => {
                    match genesis_key {
                        Some(gk) => match dir_files.get_mut(format!("{:?}", gk).as_str()) {
                            Some(present) => *present = true,
                            None => {
                                if let Ok(prefix_map) = retrieve_local_prefix_map(path).await {
                                    let path = self
                                        .prefix_maps_dir
                                        .join(format!("{:?}", prefix_map.genesis_key()));
                                    write_prefix_map(&path, &prefix_map).await?;
                                    *genesis_key = Some(prefix_map.genesis_key());
                                } else {
                                    remove_list.push(network_name.clone());
                                }
                            }
                        },
                        // PrefixMap has not been fetched, fetch it
                        None => {
                            if let Ok(prefix_map) = retrieve_local_prefix_map(path).await {
                                let path = self
                                    .prefix_maps_dir
                                    .join(format!("{:?}", prefix_map.genesis_key()));
                                write_prefix_map(&path, &prefix_map).await?;
                                *genesis_key = Some(prefix_map.genesis_key());
                            } else {
                                remove_list.push(network_name.clone());
                            }
                        }
                    }
                }
                NetworkInfo::Remote(url, ref mut genesis_key) => match genesis_key {
                    Some(gk) => match dir_files.get_mut(format!("{:?}", gk).as_str()) {
                        Some(present) => *present = true,
                        None => {
                            let url = Url::parse(url)?;
                            if let Ok(prefix_map) = retrieve_remote_prefix_map(&url).await {
                                let path = self
                                    .prefix_maps_dir
                                    .join(format!("{:?}", prefix_map.genesis_key()));
                                write_prefix_map(&path, &prefix_map).await?;
                                *genesis_key = Some(prefix_map.genesis_key());
                            } else {
                                remove_list.push(network_name.clone());
                            }
                        }
                    },
                    None => {
                        let url = Url::parse(url)?;
                        if let Ok(prefix_map) = retrieve_remote_prefix_map(&url).await {
                            let path = self
                                .prefix_maps_dir
                                .join(format!("{:?}", prefix_map.genesis_key()));
                            write_prefix_map(&path, &prefix_map).await?;
                            *genesis_key = Some(prefix_map.genesis_key());
                        } else {
                            remove_list.push(network_name.clone());
                        }
                    }
                },
            }
        }
        for network in remove_list {
            self.settings.networks.remove(network.as_str());
        }

        // add unaccounted NetworkPrefixMap from prefix_maps_dir to cli_config
        for (filename, present) in dir_files.iter() {
            if !present {
                let path = self.prefix_maps_dir.join(filename);
                if let Ok(prefix_map) = retrieve_local_prefix_map(&path).await {
                    let genesis_key = prefix_map.genesis_key();
                    self.settings.networks.insert(
                        format!("{:?}", genesis_key),
                        NetworkInfo::GenesisKey(genesis_key),
                    );
                }
                // else remove the prefix_map if not NetworkPrefixMap type?
            }
        }
        self.write_settings_to_file().await?;
        Ok(())
    }

    pub async fn read_default_prefix_map(&self) -> Result<NetworkPrefixMap> {
        let default_path = self.prefix_maps_dir.join(&self.prefix_map_symlink_name);
        if !default_path.is_symlink() {
            return Err(eyre!("The file {:?} should be a symlink", &default_path));
        }
        let prefix_map = retrieve_local_prefix_map(&default_path)
            .await
            .wrap_err_with(|| {
                eyre!("There doesn't seem to be any default NetworkPrefixMap symlink").suggestion(
                    "A NetworkPrefixMap will be created if you join a network or launch your own.",
                )
            })?;
        Ok(prefix_map)
    }

    // pub async fn get_prefix_map(&mut self, name: &str) -> Result<NetworkPrefixMap> {
    //     match self.settings.networks.get(name) {
    //         Some(NetworkInfo::GenesisKey(genesis_key)) => {
    //             let path = self.prefix_maps_dir.join(format!("{:?}", genesis_key));
    //             Ok(retrieve_local_prefix_map(&path).await?)
    //         }
    //         Some(NetworkInfo::Local(stored_path, ref mut genesis_key)) => {
    //             let prefix_map = if let Some(gk) = genesis_key {
    //                 let path = self.prefix_maps_dir.join(format!("{:?}", gk));
    //                 retrieve_local_prefix_map(&path).await?
    //             } else {
    //                 // LATER: not remove? because sync just tries once. (remote was unavailable?)
    //                 // because genesis_key should be present at this point, but try to fetch it & write
    //                 let prefix_map = retrieve_local_prefix_map(&stored_path).await?;
    //                 let path = self.prefix_maps_dir.join(format!("{:?}", prefix_map.genesis_key()));
    //                 write_prefix_map(&path, &prefix_map).await?;
    //                 *genesis_key = Some(prefix_map.genesis_key());
    //                 prefix_map
    //             };
    //             Ok(prefix_map)
    //         }
    //         Some(NetworkInfo::Remote(stored_url, ref mut genesis_key)) => {
    //             let prefix_map = if let Some(gk) = genesis_key {
    //                 let path = self.prefix_maps_dir.join(format!("{:?}", gk));
    //                 retrieve_local_prefix_map(&path).await?
    //             } else {
    //                 let url = Url::parse(&stored_url)?;
    //                 let prefix_map = retrieve_remote_prefix_map(&url).await?;
    //                 let path = self.prefix_maps_dir.join(format!("{:?}", prefix_map.genesis_key()));
    //                 write_prefix_map(&path, &prefix_map).await?;
    //                 *genesis_key = Some(prefix_map.genesis_key());
    //                 prefix_map
    //             };
    //             Ok(prefix_map)
    //         }
    //         None => bail!("No network with name '{}' was found in the config. Please use the networks 'add'/'set' subcommand to add it", name)
    //     }
    // }

    pub fn networks_iter(&self) -> impl Iterator<Item = (&String, &NetworkInfo)> {
        self.settings.networks.iter()
    }
    pub async fn add_network(
        &mut self,
        name: &str,
        mut net_info: NetworkInfo,
    ) -> Result<NetworkInfo> {
        match net_info {
            NetworkInfo::GenesisKey(_) => {}
            NetworkInfo::Local(ref path, ref mut genesis_key) => {
                let prefix_map = retrieve_local_prefix_map(path).await?;
                let path = self
                    .prefix_maps_dir
                    .join(format!("{:?}", prefix_map.genesis_key()));
                write_prefix_map(&path, &prefix_map).await?;
                *genesis_key = Some(prefix_map.genesis_key());
            }
            NetworkInfo::Remote(ref url, ref mut genesis_key) => {
                let url = Url::parse(url)?;
                let prefix_map = retrieve_remote_prefix_map(&url).await?;
                let path = self
                    .prefix_maps_dir
                    .join(format!("{:?}", prefix_map.genesis_key()));
                write_prefix_map(&path, &prefix_map).await?;
                *genesis_key = Some(prefix_map.genesis_key());
            }
        };
        self.settings
            .networks
            .insert(name.to_string(), net_info.clone());

        self.write_settings_to_file().await?;

        debug!("Network '{}' added to settings: {}", name, net_info);
        Ok(net_info)
    }

    pub async fn remove_network(&mut self, name: &str) -> Result<()> {
        match self.settings.networks.remove(name) {
            Some(NetworkInfo::GenesisKey(genesis_key)) => {
                self.write_settings_to_file().await?;
                let prefix_map_path = self.prefix_maps_dir.join(format!("{:?}", genesis_key));
                if fs::remove_file(&prefix_map_path).await.is_err() {
                    println!(
                        "Failed to remove NetworkPrefixMap from {}",
                        prefix_map_path.display()
                    )
                }
            }
            Some(NetworkInfo::Local(_, genesis_key)) => {
                self.write_settings_to_file().await?;
                if let Some(gk) = genesis_key {
                    let prefix_map_path = self.prefix_maps_dir.join(format!("{:?}", gk));
                    if fs::remove_file(&prefix_map_path).await.is_err() {
                        println!(
                            "Failed to remove NetworkPrefixMap from {}",
                            prefix_map_path.display()
                        )
                    }
                }
                // if None, then the file is not present, since we sync during config init
            }
            Some(NetworkInfo::Remote(_, genesis_key)) => {
                self.write_settings_to_file().await?;
                if let Some(gk) = genesis_key {
                    let prefix_map_path = self.prefix_maps_dir.join(format!("{:?}", gk));
                    if fs::remove_file(&prefix_map_path).await.is_err() {
                        println!(
                            "Failed to remove NetworkPrefixMap from {}",
                            prefix_map_path.display()
                        )
                    }
                }
            }
            None => println!("No network with name '{}' was found in config", name),
        }
        debug!("Network '{}' removed from config", name);
        println!("Network '{}' was removed from the config", name);

        Ok(())
    }

    pub async fn clear(&mut self) -> Result<()> {
        self.settings = Settings::default();
        self.write_settings_to_file().await?;
        // delete all prefix_maps
        let mut prefix_maps_dir = fs::read_dir(&self.prefix_maps_dir).await?;
        while let Some(entry) = prefix_maps_dir.next_entry().await? {
            fs::remove_file(entry.path()).await?;
        }
        Ok(())
    }

    pub async fn switch_to_network(&self, name: &str) -> Result<()> {
        match self.settings.networks.get(name) {
            Some(NetworkInfo::GenesisKey(genesis_key)) => self.update_prefix_map_symlink(genesis_key).await?,
            Some(NetworkInfo::Local(_, genesis_key)) => {
                if let Some(gk) = genesis_key {
                    self.update_prefix_map_symlink(gk).await?;
                }
                // if None, then the file is not present, since we sync during config init
            }
            Some(NetworkInfo::Remote(_, genesis_key)) => {
                if let Some(gk) = genesis_key {
                    self.update_prefix_map_symlink(gk).await?;
                }
            }
            None => bail!("No network with name '{}' was found in the config. Please use the networks 'add'/'set' subcommand to add it", name)
        };
        Ok(())
    }

    pub async fn print_networks(&self) {
        let mut table = Table::new();
        table.add_row(&vec!["Networks"]);
        table.add_row(&vec!["Current", "Network name", "Connection info"]);
        let current_prefix_map = self.read_default_prefix_map().await;

        for (network_name, net_info) in self.networks_iter() {
            let mut current = "";
            if let Ok(prefix_map) = &current_prefix_map {
                if net_info.matches(&prefix_map.genesis_key()).await {
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
            let sk = Safe::deserialize_bls_key(dbc_sk_path)?;
            return Ok(Some(Owner::from(sk)));
        }
        Ok(None)
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

    async fn update_prefix_map_symlink(&self, genesis_key: &BlsPublicKey) -> Result<()> {
        let prefix_map_file = self.prefix_maps_dir.join(format!("{:?}", genesis_key));
        let default_prefix = self.prefix_maps_dir.join(&self.prefix_map_symlink_name);

        if fs::read_link(&default_prefix).await.is_ok() {
            fs::remove_file(&default_prefix).await.wrap_err_with(|| {
                format!(
                    "Error removing previous PrefixMap symlink: {:?}",
                    default_prefix.display()
                )
            })?;
        }
        debug!(
            "Creating symlink for PrefixMap from {:?} to {:?}",
            prefix_map_file.display(),
            default_prefix.display()
        );
        #[cfg(unix)]
        symlink(&prefix_map_file, &default_prefix).wrap_err_with(|| {
            format!(
                "Error creating PrefixMap symlink from {:?} to {:?}",
                prefix_map_file.display(),
                default_prefix.display()
            )
        })?;
        #[cfg(windows)]
        symlink_file(prefix_map_file, default_prefix).wrap_err_with(|| {
            format!(
                "Error creating PrefixMap symlink from {:?} to {:?}",
                prefix_map_file.display(),
                default_prefix.display()
            )
        })?;
        Ok(())
    }
}

async fn write_prefix_map(path: &PathBuf, prefix_map: &NetworkPrefixMap) -> Result<()> {
    let serialized = serialise_prefix_map(prefix_map)?;
    fs::write(path, serialized)
        .await
        .wrap_err_with(|| format!("Unable to write NetworkPrefixMap to '{}'", path.display()))?;
    debug!("NetworkPrefixMap written at {:?}", path.display(),);
    Ok(())
}

async fn retrieve_local_prefix_map(location: &PathBuf) -> Result<NetworkPrefixMap> {
    let bytes = fs::read(location).await.wrap_err_with(|| {
        format!(
            "Unable to read connection information from '{:?}'",
            location
        )
    })?;
    deserialise_prefix_map(&bytes)
}

async fn retrieve_remote_prefix_map(url: &Url) -> Result<NetworkPrefixMap> {
    let mut retry = REMOTE_RETRY_COUNT;
    let mut bytes: Option<Bytes> = None;
    let mut status: StatusCode;
    loop {
        let resp = reqwest::get(url.to_string()).await?;
        status = resp.status();
        if status.is_client_error() || status.is_server_error() {
            if retry <= 1 {
                break;
            } else {
                retry -= 1;
                continue;
            }
        }
        bytes = Some(resp.bytes().await?);
        break;
    }
    match bytes {
        Some(b) => deserialise_prefix_map(&b[..]),
        None => Err(eyre!(
            "{:?}: Failed to fetch connection information (after {} retries) from '{}'",
            status,
            REMOTE_RETRY_COUNT,
            url
        )),
    }
}

fn deserialise_prefix_map(bytes: &[u8]) -> Result<NetworkPrefixMap> {
    let prefix_map: NetworkPrefixMap =
        rmp_serde::from_slice(bytes).wrap_err_with(|| "Failed to deserialize NetworkPrefixMap")?;
    Ok(prefix_map)
}

fn serialise_prefix_map(prefix_map: &NetworkPrefixMap) -> Result<Vec<u8>> {
    rmp_serde::to_vec(prefix_map).wrap_err_with(|| "Failed to serialise NetworkPrefixMap")
}

#[cfg(test)]
mod constructor {
    use super::{Config, NetworkInfo};
    use assert_fs::prelude::*;
    use bls::SecretKey;
    use color_eyre::{eyre::eyre, Result};
    use predicates::prelude::*;
    use sn_api::Safe;
    use std::path::PathBuf;

    #[tokio::test]
    async fn fields_should_be_set_to_correct_values() -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let cli_config_dir = tmp_dir.child(".safe/cli");
        cli_config_dir.create_dir_all()?;
        let prefix_maps_dir = tmp_dir.child(".safe/prefix_maps");
        prefix_maps_dir.create_dir_all()?;
        let prefix_map_symlink_name = String::from("default");

        let cli_config_file = cli_config_dir.child("config.json");
        let dbc_owner_sk_file = cli_config_dir.child("credentials");
        let sk = SecretKey::random();
        Safe::serialize_bls_key(&sk, dbc_owner_sk_file.path())?;

        let config = Config::new(
            PathBuf::from(cli_config_file.path()),
            PathBuf::from(prefix_maps_dir.path()),
            prefix_map_symlink_name.clone(),
        )
        .await?;

        assert_eq!(config.cli_config_path, cli_config_file.path());
        assert_eq!(config.prefix_maps_dir, prefix_maps_dir.path());
        assert_eq!(config.prefix_map_symlink_name, prefix_map_symlink_name);
        assert_eq!(config.settings.networks.len(), 0);
        assert!(config.dbc_owner.is_some());
        Ok(())
    }

    #[tokio::test]
    async fn cli_config_directory_should_be_created() -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let cli_config_dir = tmp_dir.child(".safe/cli");
        let cli_config_file = cli_config_dir.child("config.json");
        let prefix_maps_dir = tmp_dir.child(".safe/prefix_maps");
        let prefix_map_symlink_name = String::from("default");

        let _ = Config::new(
            PathBuf::from(cli_config_file.path()),
            PathBuf::from(prefix_maps_dir.path()),
            prefix_map_symlink_name,
        )
        .await?;

        cli_config_dir.assert(predicate::path::is_dir());
        prefix_maps_dir.assert(predicate::path::is_dir());
        Ok(())
    }

    #[tokio::test]
    async fn given_config_file_does_not_exist_then_it_should_be_created() -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let cli_config_file = tmp_dir.child(".safe/cli/config.json");
        let prefix_maps_dir = tmp_dir.child(".safe/prefix_maps");
        let prefix_map_symlink_name = String::from("default");

        let _ = Config::new(
            PathBuf::from(cli_config_file.path()),
            PathBuf::from(prefix_maps_dir.path()),
            prefix_map_symlink_name,
        )
        .await?;

        cli_config_file.assert(predicate::path::exists());

        Ok(())
    }

    #[tokio::test]
    async fn given_config_file_exists_then_the_settings_should_be_read() -> Result<()> {
        // add all 3 NetworkInfo variants
        let serialized_config = r#"
        {"networks":{
          "PublicKey(030f..2825)":{
             "GenesisKey":[163,15,109,28,26,203,211,208,156,251,90,71,98,171,89,225,173,18,189,187,66,56,137,52,206,69,88,213,185,223,247,133,212,173,29,138,164,236,216,174,167,242,223,192,203,23,81,32]
          },
          "testnet":{
            "Remote":["https://roland-misc.s3.us-west-002.backblazeb2.com/PublicKey(180e..9f94)", [184,14,65,176,22,141,197,197,59,59,175,225,114,243,33,235,247,134,37,206,29,62,209,84,186,163,231,227,3,125,85,157,243,242,100,25,186,58,234,66,176,213,74,222,214,161,152,6]]
          }
        }}"#;
        let tmp_dir = assert_fs::TempDir::new()?;
        let cli_config_file = tmp_dir.child(".safe/cli/config.json");
        cli_config_file.write_str(serialized_config)?;
        let prefix_maps_dir = tmp_dir.child(".safe/prefix_maps");
        let prefix_map_symlink_name = String::from("default");
        let config = Config::new(
            PathBuf::from(cli_config_file.path()),
            PathBuf::from(prefix_maps_dir.path()),
            prefix_map_symlink_name,
        )
        .await?;

        assert_eq!(config.networks_iter().count(), 2);

        let mut iter = config.networks_iter();

        // first network
        let (network_name, network_info) = iter
            .next()
            .ok_or_else(|| eyre!("failed to obtain item from networks list"))?;
        assert_eq!(network_name, "PublicKey(030f..2825)");
        match network_info {
            NetworkInfo::GenesisKey(genesis_key) => {
                let genesis_key_str = format!("{:?}", genesis_key);
                assert_eq!(genesis_key_str, *network_name);
            }
            _ => {
                return Err(eyre!(
                    "The network information should be of type NetworkInfo::GenesisKey"
                ));
            }
        }

        // second network
        let (network_name, network_info) = iter
            .next()
            .ok_or_else(|| eyre!("failed to obtain item from networks list"))?;
        assert_eq!(network_name, "testnet");
        match network_info {
            NetworkInfo::Remote(_, genesis_key) => {
                let genesis_key_str = format!("{:?}", genesis_key);
                assert_eq!(genesis_key_str, String::from("Some(PublicKey(180e..9f94))"));
            }
            _ => {
                return Err(eyre!(
                    "The network information should be of type NetworkInfo::Remote"
                ));
            }
        }
        Ok(())
    }

    #[tokio::test]
    async fn given_an_empty_config_file_empty_settings_should_be_returned() -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let cli_config_file = tmp_dir.child(".safe/cli/config.json");
        cli_config_file.touch()?;
        let prefix_maps_dir = tmp_dir.child(".safe/prefix_maps");
        let prefix_map_symlink_name = String::from("default");
        let config = Config::new(
            PathBuf::from(cli_config_file.path()),
            PathBuf::from(prefix_maps_dir.path()),
            prefix_map_symlink_name.clone(),
        )
        .await?;

        assert_eq!(0, config.settings.networks.len());
        assert_eq!(cli_config_file.path(), config.cli_config_path.as_path());
        assert_eq!(prefix_maps_dir.path(), config.prefix_maps_dir.as_path());

        Ok(())
    }
}

#[cfg(test)]
mod read_prefix_map {
    use super::{retrieve_remote_prefix_map, write_prefix_map, Config};
    use assert_fs::prelude::*;
    use color_eyre::Result;
    use std::path::PathBuf;
    use tokio::fs;
    use url::Url;

    #[tokio::test]
    async fn given_prefix_map_symlink_it_should_be_read() -> Result<()> {
        let remote =
            Url::parse("https://roland-misc.s3.us-west-002.backblazeb2.com/PublicKey(180e..9f94)")?;
        let tmp_dir = assert_fs::TempDir::new()?;
        let cli_config_file = tmp_dir.child(".safe/cli/config.json");
        let prefix_maps_dir = tmp_dir.child(".safe/prefix_maps");
        let prefix_map_symlink_name = String::from("default");
        let config = Config::new(
            PathBuf::from(cli_config_file.path()),
            PathBuf::from(prefix_maps_dir.path()),
            prefix_map_symlink_name,
        )
        .await?;

        let prefix_map = retrieve_remote_prefix_map(&remote).await?;
        let prefix_map_path = config
            .prefix_maps_dir
            .join(format!("{:?}", prefix_map.genesis_key()));
        write_prefix_map(&prefix_map_path, &prefix_map).await?;
        config
            .update_prefix_map_symlink(&prefix_map.genesis_key())
            .await?;

        let retrieved_prefix_map = config.read_default_prefix_map().await?;
        assert_eq!(retrieved_prefix_map, prefix_map);
        assert_eq!(retrieved_prefix_map.genesis_key(), prefix_map.genesis_key());

        Ok(())
    }

    #[tokio::test]
    async fn given_no_prefix_map_symlink_it_should_be_an_error() -> Result<()> {
        let remote =
            Url::parse("https://roland-misc.s3.us-west-002.backblazeb2.com/PublicKey(180e..9f94)")?;
        let tmp_dir = assert_fs::TempDir::new()?;
        let cli_config_file = tmp_dir.child(".safe/cli/config.json");
        let prefix_maps_dir = tmp_dir.child(".safe/prefix_maps");
        let prefix_map_symlink_name = String::from("default");
        let config = Config::new(
            PathBuf::from(cli_config_file.path()),
            PathBuf::from(prefix_maps_dir.path()),
            prefix_map_symlink_name,
        )
        .await?;

        let prefix_map = retrieve_remote_prefix_map(&remote).await?;
        let prefix_map_path = config
            .prefix_maps_dir
            .join(format!("{:?}", prefix_map.genesis_key()));
        write_prefix_map(&prefix_map_path, &prefix_map).await?;
        let retrieved_prefix_map = config.read_default_prefix_map().await;
        assert!(retrieved_prefix_map.is_err(), "Symlink should not exist");

        Ok(())
    }

    #[tokio::test]
    async fn given_no_prefix_map_file_it_should_be_an_error() -> Result<()> {
        let remote =
            Url::parse("https://roland-misc.s3.us-west-002.backblazeb2.com/PublicKey(180e..9f94)")?;
        let tmp_dir = assert_fs::TempDir::new()?;
        let cli_config_file = tmp_dir.child(".safe/cli/config.json");
        let prefix_maps_dir = tmp_dir.child(".safe/prefix_maps");
        let prefix_map_symlink_name = String::from("default");
        let config = Config::new(
            PathBuf::from(cli_config_file.path()),
            PathBuf::from(prefix_maps_dir.path()),
            prefix_map_symlink_name,
        )
        .await?;

        let prefix_map = retrieve_remote_prefix_map(&remote).await?;
        let prefix_map_path = config
            .prefix_maps_dir
            .join(format!("{:?}", prefix_map.genesis_key()));
        write_prefix_map(&prefix_map_path, &prefix_map).await?;
        config
            .update_prefix_map_symlink(&prefix_map.genesis_key())
            .await?;
        fs::remove_file(&prefix_map_path).await?;
        let retrieved_prefix_map = config.read_default_prefix_map().await;
        assert!(
            retrieved_prefix_map.is_err(),
            "PrefixMap pointed by the symlink should not be exist"
        );

        Ok(())
    }
}

#[cfg(test)]
mod sync_prefix_maps_and_settings {
    use super::{retrieve_remote_prefix_map, write_prefix_map, Config};
    use assert_fs::prelude::*;
    use color_eyre::Result;
    use path::PathBuf;
    use std::path;
    use url::Url;

    #[tokio::test]
    async fn empty_cli_config_file_should_be_populated_by_existing_prefix_maps() -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let cli_config_file = tmp_dir.child(".safe/cli/config.json");
        let prefix_maps_dir = tmp_dir.child(".safe/prefix_maps");
        let prefix_map_symlink_name = String::from("default");
        let mut config = Config::new(
            PathBuf::from(cli_config_file.path()),
            PathBuf::from(prefix_maps_dir.path()),
            prefix_map_symlink_name,
        )
        .await?;

        let remotes: Vec<&str> = vec![
            "https://roland-misc.s3.us-west-002.backblazeb2.com/PublicKey(180e..9f94)",
            "https://roland-misc.s3.us-west-002.backblazeb2.com/PublicKey(030f..2825)",
            "https://roland-misc.s3.us-west-002.backblazeb2.com/PublicKey(11d7..ffcc)",
            "https://roland-misc.s3.us-west-002.backblazeb2.com/PublicKey(17bd..a48d)",
        ];
        for remote in &remotes {
            let prefix_map = retrieve_remote_prefix_map(&Url::parse(remote)?).await?;
            let prefix_map_path = config
                .prefix_maps_dir
                .join(format!("{:?}", prefix_map.genesis_key()));
            write_prefix_map(&prefix_map_path, &prefix_map).await?;
            config
                .update_prefix_map_symlink(&prefix_map.genesis_key())
                .await?;
        }
        config.sync().await?;
        assert_eq!(config.settings.networks.len(), remotes.len());
        Ok(())
    }

    #[tokio::test]
    async fn prefix_maps_should_be_fetched_from_cli_config_file() -> Result<()> {
        let serialized_config = r#"
        {"networks":{
          "network_1":{
            "Remote":["https://roland-misc.s3.us-west-002.backblazeb2.com/PublicKey(180e..9f94)", null]
          },
          "network_2":{
            "Remote":["https://roland-misc.s3.us-west-002.backblazeb2.com/PublicKey(030f..2825)", null]
          },
          "network_3":{
            "Remote":["https://roland-misc.s3.us-west-002.backblazeb2.com/PublicKey(11d7..ffcc)", null]
          },
          "network_4":{
            "Remote":["https://roland-misc.s3.us-west-002.backblazeb2.com/PublicKey(17bd..a48d)", null]
          }
        }}"#;
        let tmp_dir = assert_fs::TempDir::new()?;
        let cli_config_file = tmp_dir.child(".safe/cli/config.json");
        cli_config_file.write_str(serialized_config)?;
        let prefix_maps_dir = tmp_dir.child(".safe/prefix_maps");
        let prefix_map_symlink_name = String::from("default");
        let mut config = Config::new(
            PathBuf::from(cli_config_file.path()),
            PathBuf::from(prefix_maps_dir.path()),
            prefix_map_symlink_name,
        )
        .await?;
        let paths = dbg!(std::fs::read_dir(&prefix_maps_dir)?);
        for path in paths {
            println!("Name: {}", path.unwrap().path().display())
        }
        config.sync().await?;
        let paths = dbg!(std::fs::read_dir(prefix_maps_dir)?);
        for path in paths {
            println!("Name: {}", path.unwrap().path().display())
        }

        Ok(())
    }
}
