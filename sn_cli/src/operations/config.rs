// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use bls::PublicKey as BlsPublicKey;
use bytes::Bytes;
use clap::Parser;
use color_eyre::{eyre::bail, eyre::eyre, eyre::WrapErr, Help, Report, Result};
use comfy_table::Table;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use sn_api::{NetworkPrefixMap, Safe, DEFAULT_PREFIX_HARDLINK_NAME};
use sn_dbc::Owner;
use std::{
    collections::BTreeMap,
    default::Default,
    fmt,
    io::Write,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};
use tempfile::NamedTempFile;
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

#[derive(Deserialize, Debug, Serialize, Clone)]
pub enum NetworkInfo {
    /// The Local path pointing to a network map. The optional genesis key denotes that the network map has been copied
    /// to prefix_maps_dir
    Local(PathBuf, Option<BlsPublicKey>),
    /// The remote url pointing to a network map. The optional genesis key denotes that the network map has been copied
    //     /// to prefix_maps_dir
    Remote(String, Option<BlsPublicKey>),
}

impl fmt::Display for NetworkInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Local(path, genesis_key) => {
                if let Some(gk) = genesis_key {
                    write!(f, "{:?}, path: {:?}", gk, path)
                } else {
                    write!(f, "{:?}", path)
                }
            }
            Self::Remote(url, genesis_key) => {
                if let Some(gk) = genesis_key {
                    write!(f, "{:?}, url: {:?}", gk, url)
                } else {
                    write!(f, "{}", url)
                }
            }
        }
    }
}

impl NetworkInfo {
    pub fn matches(&self, genesis_key: &BlsPublicKey) -> bool {
        match self {
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
    pub dbc_owner: Option<Owner>,
}

impl Config {
    pub async fn new(cli_config_path: PathBuf, prefix_maps_dir: PathBuf) -> Result<Config> {
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
        let dbc_owner = Config::get_dbc_owner(&dbc_owner_sk_path)?;

        let config = Config {
            settings,
            cli_config_path: cli_config_path.clone(),
            prefix_maps_dir,
            dbc_owner,
        };
        config.write_settings_to_file().await.wrap_err_with(|| {
            format!("Unable to create config at '{}'", cli_config_path.display())
        })?;
        Ok(config)
    }

    /// Sync settings and the prefix_map_dir
    pub async fn sync(&mut self) -> Result<()> {
        let mut dir_files_checklist: BTreeMap<String, bool> = BTreeMap::new();
        let mut prefix_maps_dir = fs::read_dir(&self.prefix_maps_dir).await?;
        while let Some(entry) = prefix_maps_dir.next_entry().await? {
            if entry.metadata().await?.is_file() {
                let filename = entry
                    .file_name()
                    .into_string()
                    .map_err(|_| eyre!("Error converting OsString to String"))?;
                if filename != *DEFAULT_PREFIX_HARDLINK_NAME {
                    dir_files_checklist.insert(filename, false);
                }
            }
        }

        // use settings from cli_config_file instead of self.settings
        let content = fs::read(&self.cli_config_path).await.wrap_err_with(|| {
            format!(
                "Error reading config file from '{}'",
                &self.cli_config_path.display(),
            )
        })?;
        self.settings = if content.is_empty() {
            Settings::default()
        } else {
            serde_json::from_slice(&content).wrap_err_with(|| {
                format!(
                    "Format of the config file at '{}' is not valid and couldn't be parsed",
                    &self.cli_config_path.display()
                )
            })?
        };

        // get NetworkPrefixMap from cli_config if they are not in prefix_maps_dir
        let mut remove_list: Vec<String> = Vec::new();
        for (network_name, net_info) in self.settings.networks.iter_mut() {
            match net_info {
                NetworkInfo::Local(path, ref mut genesis_key) => {
                    match genesis_key {
                        Some(gk) => match dir_files_checklist.get_mut(format!("{:?}", gk).as_str())
                        {
                            Some(present) => *present = true,
                            None => {
                                if let Ok(prefix_map) = Self::retrieve_local_prefix_map(path).await
                                {
                                    let path = self
                                        .prefix_maps_dir
                                        .join(format!("{:?}", prefix_map.genesis_key()));
                                    Self::write_prefix_map(&path, &prefix_map).await?;
                                    *genesis_key = Some(*prefix_map.genesis_key());
                                } else {
                                    remove_list.push(network_name.clone());
                                }
                            }
                        },
                        // NetworkPrefixMap has not been fetched, fetch it
                        None => {
                            if let Ok(prefix_map) = Self::retrieve_local_prefix_map(path).await {
                                let path = self
                                    .prefix_maps_dir
                                    .join(format!("{:?}", prefix_map.genesis_key()));
                                Self::write_prefix_map(&path, &prefix_map).await?;
                                *genesis_key = Some(*prefix_map.genesis_key());
                            } else {
                                remove_list.push(network_name.clone());
                            }
                        }
                    }
                }
                NetworkInfo::Remote(url, ref mut genesis_key) => match genesis_key {
                    Some(gk) => match dir_files_checklist.get_mut(format!("{:?}", gk).as_str()) {
                        Some(present) => *present = true,
                        None => {
                            let url = Url::parse(url)?;
                            if let Ok(prefix_map) = Self::retrieve_remote_prefix_map(&url).await {
                                let path = self
                                    .prefix_maps_dir
                                    .join(format!("{:?}", prefix_map.genesis_key()));
                                Self::write_prefix_map(&path, &prefix_map).await?;
                                *genesis_key = Some(*prefix_map.genesis_key());
                            } else {
                                remove_list.push(network_name.clone());
                            }
                        }
                    },
                    None => {
                        let url = Url::parse(url)?;
                        if let Ok(prefix_map) = Self::retrieve_remote_prefix_map(&url).await {
                            let path = self
                                .prefix_maps_dir
                                .join(format!("{:?}", prefix_map.genesis_key()));
                            Self::write_prefix_map(&path, &prefix_map).await?;
                            *genesis_key = Some(*prefix_map.genesis_key());
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
        for (filename, present) in dir_files_checklist.iter() {
            if !present {
                let path = self.prefix_maps_dir.join(filename);
                if let Ok(prefix_map) = Self::retrieve_local_prefix_map(&path).await {
                    let genesis_key = *prefix_map.genesis_key();
                    self.settings.networks.insert(
                        format!("{:?}", genesis_key),
                        NetworkInfo::Local(path, Some(genesis_key)),
                    );
                }
                // else remove the prefix_map if not NetworkPrefixMap type?
            }
        }
        self.write_settings_to_file().await?;
        Ok(())
    }

    pub async fn read_default_prefix_map(&self) -> Result<NetworkPrefixMap> {
        let default_path = self.prefix_maps_dir.join(DEFAULT_PREFIX_HARDLINK_NAME);
        let prefix_map = Self::retrieve_local_prefix_map(&default_path)
            .await
            .wrap_err_with(|| {
                eyre!("There doesn't seem to be any default Network Map").suggestion(
                    "A Network Map will be created if you join a network or launch your own.",
                )
            })?;
        Ok(prefix_map)
    }

    pub fn networks_iter(&self) -> impl Iterator<Item = (&String, &NetworkInfo)> {
        self.settings.networks.iter()
    }

    pub async fn add_network(
        &mut self,
        name: &str,
        mut net_info: NetworkInfo,
    ) -> Result<NetworkInfo> {
        match net_info {
            NetworkInfo::Local(ref mut path, ref mut genesis_key) => {
                if !path.is_absolute() {
                    *path = fs::canonicalize(&path).await?
                }
                let prefix_map = Self::retrieve_local_prefix_map(path).await?;
                let path = self
                    .prefix_maps_dir
                    .join(format!("{:?}", prefix_map.genesis_key()));
                Self::write_prefix_map(&path, &prefix_map).await?;
                *genesis_key = Some(*prefix_map.genesis_key());
            }
            NetworkInfo::Remote(ref url, ref mut genesis_key) => {
                let url = Url::parse(url)?;
                let prefix_map = Self::retrieve_remote_prefix_map(&url).await?;
                let path = self
                    .prefix_maps_dir
                    .join(format!("{:?}", prefix_map.genesis_key()));
                Self::write_prefix_map(&path, &prefix_map).await?;
                *genesis_key = Some(*prefix_map.genesis_key());
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
            Some(NetworkInfo::Local(_, genesis_key)) => {
                self.write_settings_to_file().await?;
                if let Some(gk) = genesis_key {
                    let prefix_map_path = self.prefix_maps_dir.join(format!("{:?}", gk));
                    if fs::remove_file(&prefix_map_path).await.is_err() {
                        println!(
                            "Failed to remove network map from {}",
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
                            "Failed to remove network map from {}",
                            prefix_map_path.display()
                        )
                    }
                }
            }
            None => println!("No network with name '{}' was found in config", name),
        }
        if fs::remove_file(&self.prefix_maps_dir.join(DEFAULT_PREFIX_HARDLINK_NAME))
            .await
            .is_err()
        {
            debug!("Cannot remove default NetworkPrefixMap!");
        };
        debug!("Network '{}' removed from config", name);
        println!("Network '{}' was removed from the config", name);

        Ok(())
    }

    pub async fn clear(&mut self) -> Result<()> {
        self.settings = Settings::default();
        self.write_settings_to_file().await?;
        // delete all prefix maps
        let mut prefix_maps_dir = fs::read_dir(&self.prefix_maps_dir).await?;
        while let Some(entry) = prefix_maps_dir.next_entry().await? {
            fs::remove_file(entry.path()).await?;
        }
        Ok(())
    }

    pub async fn switch_to_network(&self, name: &str) -> Result<()> {
        match self.settings.networks.get(name) {
            Some(NetworkInfo::Local(_, genesis_key)) => {
                match genesis_key {
                    Some(gk) => self.set_default_prefix_map(format!("{:?}", gk)).await?,
                    //  can't be none since we sync during get_config
                    None => bail!("Cannot switch to {}, since the network file is not found! Please re-run the same command!", name)
                }
            }
            Some(NetworkInfo::Remote(_, genesis_key)) => {
                match genesis_key {
                    Some(gk) => self.set_default_prefix_map(format!("{:?}", gk)).await?,
                    None => bail!("Cannot switch to {}, since the network file is not found! Please re-run the same command!", name)
                }
            }
            None => bail!("No network with name '{}' was found in the config. Please use the networks 'add'/'set' subcommand to add it", name)
        };
        Ok(())
    }

    pub async fn print_networks(&self) {
        let mut table = Table::new();
        table.add_row(&vec!["Networks"]);
        table.add_row(&vec!["Current", "Network name", "Network map info"]);
        let current_prefix_map = self.read_default_prefix_map().await;

        for (network_name, net_info) in self.networks_iter() {
            let mut current = "";
            if let Ok(prefix_map) = &current_prefix_map {
                if net_info.matches(prefix_map.genesis_key()) {
                    current = "*";
                }
            }
            let simplified_net_info = match net_info {
                NetworkInfo::Local(path, _) => format!("Local: {:?}", path),
                NetworkInfo::Remote(url, _) => format!("Remote: {:?}", url),
            };
            table.add_row(&vec![current, network_name, simplified_net_info.as_str()]);
        }

        println!("{table}");
    }

    pub async fn set_default_prefix_map(&self, genesis_key: String) -> Result<()> {
        let prefix_map_file = self.prefix_maps_dir.join(genesis_key);
        let default_prefix = self.prefix_maps_dir.join(DEFAULT_PREFIX_HARDLINK_NAME);

        if default_prefix.exists() {
            fs::remove_file(&default_prefix).await.wrap_err_with(|| {
                format!(
                    "Error removing default NetworkPrefixMap hardlink: {:?}",
                    default_prefix.display()
                )
            })?;
        }
        debug!(
            "Creating hardlink for NetworkPrefixMap from {:?} to {:?}",
            prefix_map_file.display(),
            default_prefix.display()
        );
        fs::hard_link(&prefix_map_file, &default_prefix)
            .await
            .wrap_err_with(|| {
                format!(
                    "Error creating hardlink from {:?} to {:?}",
                    prefix_map_file.display(),
                    default_prefix.display()
                )
            })?;
        Ok(())
    }

    pub async fn write_prefix_map(path: &PathBuf, prefix_map: &NetworkPrefixMap) -> Result<()> {
        let prefix_map_dir = path
            .parent()
            .ok_or_else(|| eyre!("Path {:?} should be inside a folder", path.display()))?;
        let mut temp_file = NamedTempFile::new_in(prefix_map_dir)
            .wrap_err_with(|| "Error creating temp file".to_string())?;

        let serialized = Self::serialise_prefix_map(prefix_map)
            .wrap_err("Failed to serialise NetworkPrefixMap")?;
        temp_file
            .write_all(serialized.as_slice())
            .wrap_err_with(|| {
                format!(
                    "Unable to write temp NetworkPrefixMap to '{}'",
                    temp_file.path().display()
                )
            })?;
        fs::rename(temp_file.path(), path).await.wrap_err_with(|| {
            format!(
                "Error while renaming NetworkPrefixMap file from {:?} to {:?}",
                temp_file.path().display(),
                path.display()
            )
        })?;
        debug!("NetworkPrefixMap written at {:?}", path.display());

        Ok(())
    }

    pub async fn retrieve_local_prefix_map(location: &Path) -> Result<NetworkPrefixMap> {
        let bytes = fs::read(location)
            .await
            .wrap_err_with(|| format!("Unable to read NetworkPrefixMap from '{:?}'", location))?;
        Self::deserialise_prefix_map(&bytes)
    }

    pub async fn retrieve_remote_prefix_map(url: &Url) -> Result<NetworkPrefixMap> {
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
            Some(b) => Self::deserialise_prefix_map(&b[..]),
            None => Err(eyre!(
                "{:?} Failed to fetch network map ({} retries) from '{}'",
                status,
                REMOTE_RETRY_COUNT,
                url
            )),
        }
    }

    ///
    /// Private helpers
    ///

    fn get_dbc_owner(dbc_sk_path: &Path) -> Result<Option<Owner>> {
        if dbc_sk_path.exists() {
            let sk = Safe::deserialize_bls_key(dbc_sk_path)?;
            return Ok(Some(Owner::from(sk)));
        }
        Ok(None)
    }

    async fn write_settings_to_file(&self) -> Result<()> {
        let cli_dir = self.cli_config_path.parent().ok_or_else(|| {
            eyre!(
                "cli_config_path {} should be inside a folder",
                self.cli_config_path.display()
            )
        })?;
        let mut temp_file = NamedTempFile::new_in(cli_dir)
            .wrap_err_with(|| "Error creating temp file".to_string())?;

        let serialised_settings = serde_json::to_string(&self.settings)
            .wrap_err("Failed to serialise config settings")?;
        temp_file
            .write_all(serialised_settings.as_bytes())
            .wrap_err_with(|| {
                format!(
                    "Unable to write config settings to '{}'",
                    temp_file.path().display()
                )
            })?;
        fs::rename(temp_file.path(), &self.cli_config_path)
            .await
            .wrap_err_with(|| {
                format!(
                    "Error while renaming config.json file from {} to {}",
                    temp_file.path().display(),
                    &self.cli_config_path.display()
                )
            })?;
        debug!(
            "Config settings at '{}' updated with: {:?}",
            self.cli_config_path.display(),
            self.settings
        );

        Ok(())
    }

    fn deserialise_prefix_map(bytes: &[u8]) -> Result<NetworkPrefixMap> {
        let prefix_map = rmp_serde::from_slice(bytes)
            .wrap_err_with(|| "Failed to deserialize NetworkPrefixMap")?;
        Ok(prefix_map)
    }

    fn serialise_prefix_map(prefix_map: &NetworkPrefixMap) -> Result<Vec<u8>> {
        rmp_serde::to_vec(prefix_map).wrap_err_with(|| "Failed to serialise NetworkPrefixMap")
    }
}

#[cfg(test)]
pub mod test_utils {
    use super::{Config, NetworkInfo};
    use crate::operations::config::Settings;
    use assert_fs::{prelude::*, TempDir};
    use color_eyre::{eyre::eyre, Result};
    use sn_api::{NetworkPrefixMap, DEFAULT_PREFIX_HARDLINK_NAME};
    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};
    use tokio::fs;

    pub async fn store_dummy_prefix_maps(
        path: &Path,
        n_prefix_maps: usize,
    ) -> Result<Vec<NetworkPrefixMap>> {
        let mut prefix_maps: Vec<NetworkPrefixMap> = Vec::new();

        for _ in 0..n_prefix_maps {
            let sk = bls::SecretKey::random();
            let prefix_map = NetworkPrefixMap::new(sk.public_key());
            let filename = format!("{:?}", prefix_map.genesis_key());

            Config::write_prefix_map(&path.join(filename), &prefix_map).await?;
            prefix_maps.push(prefix_map);
        }
        Ok(prefix_maps)
    }

    impl Config {
        // Optionally write a cli/config.json file before creating the Config
        pub async fn create_config(tmp_dir: &TempDir, config: Option<Settings>) -> Result<Config> {
            let cli_config_dir = tmp_dir.child(".safe/cli");
            let cli_config_file = cli_config_dir.child("config.json");

            // write settings to config.json which can be read during Config::new
            if let Some(settings) = config {
                cli_config_file.write_str(serde_json::to_string(&settings)?.as_str())?;
            }

            let prefix_maps_dir = tmp_dir.child(".safe/prefix_maps");
            Config::new(
                PathBuf::from(cli_config_file.path()),
                PathBuf::from(prefix_maps_dir.path()),
            )
            .await
        }

        pub async fn store_dummy_prefix_maps_and_set_default(
            &self,
            n_prefix_maps: usize,
        ) -> Result<Vec<NetworkPrefixMap>> {
            let prefix_maps = store_dummy_prefix_maps(&self.prefix_maps_dir, n_prefix_maps).await?;
            // set one as default
            let prefix_map = prefix_maps.clone().pop().unwrap();
            self.set_default_prefix_map(format!("{:?}", prefix_map.genesis_key()))
                .await?;
            Ok(prefix_maps)
        }

        /// Compare the network entries by creating a checklist using the settings and marking
        /// them as 'true' if the network is present in the system.
        pub async fn compare_settings_and_prefix_maps_dir(&self) -> Result<()> {
            let mut prefix_map_checklist: BTreeMap<String, bool> = BTreeMap::new();

            // get list of all NetworkPrefixMaps from settings
            for (_, net_info) in self.networks_iter() {
                let genesis_key =
                    match net_info {
                        NetworkInfo::Local(_, genesis_key) => genesis_key
                            .ok_or_else(|| eyre!("gk should must be present after sync"))?,
                        NetworkInfo::Remote(_, genesis_key) => genesis_key
                            .ok_or_else(|| eyre!("gk should must be present after sync"))?,
                    };

                // same gk can be present if multiple network entries in settings points to the same
                // prefix map
                let _ = prefix_map_checklist.insert(format!("{:?}", genesis_key), false);
            }

            // mark them as true if the same entries are found in the prefix_maps_dir
            let mut prefix_maps_dir = fs::read_dir(&self.prefix_maps_dir).await?;
            while let Some(entry) = prefix_maps_dir.next_entry().await? {
                if entry.metadata().await?.is_file() {
                    let filename = entry
                        .file_name()
                        .into_string()
                        .map_err(|_| eyre!("Error converting OsString to String"))?;
                    if filename != *DEFAULT_PREFIX_HARDLINK_NAME {
                        let already_present = prefix_map_checklist.insert(filename, true);
                        // cannot insert new entries. Denotes that an extra network is found in
                        // prefix_maps_dir
                        if already_present.is_none() {
                            return Err(eyre!("Extra network found in the system!"));
                        }
                    }
                }
            }

            for (_, present) in prefix_map_checklist.iter() {
                if !present {
                    return Err(eyre!("Extra network found in the settings!"));
                }
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod constructor {
    use super::{Config, NetworkInfo};
    use crate::operations::config::{test_utils::store_dummy_prefix_maps, Settings};
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

        let cli_config_file = cli_config_dir.child("config.json");
        let dbc_owner_sk_file = cli_config_dir.child("credentials");
        let sk = SecretKey::random();
        Safe::serialize_bls_key(&sk, dbc_owner_sk_file.path())?;

        let config = Config::new(
            PathBuf::from(cli_config_file.path()),
            PathBuf::from(prefix_maps_dir.path()),
        )
        .await?;

        assert_eq!(config.cli_config_path, cli_config_file.path());
        assert_eq!(config.prefix_maps_dir, prefix_maps_dir.path());
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

        let _ = Config::new(
            PathBuf::from(cli_config_file.path()),
            PathBuf::from(prefix_maps_dir.path()),
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

        let _ = Config::new(
            PathBuf::from(cli_config_file.path()),
            PathBuf::from(prefix_maps_dir.path()),
        )
        .await?;

        cli_config_file.assert(predicate::path::exists());

        Ok(())
    }

    #[tokio::test]
    async fn given_config_file_exists_then_the_settings_should_be_read() -> Result<()> {
        // write cli/config.json file
        let tmp_dir = assert_fs::TempDir::new()?;
        let prefix_map = store_dummy_prefix_maps(&tmp_dir, 1).await?.pop().unwrap();
        let prefix_map_path = tmp_dir
            .path()
            .join(format!("{:?}", prefix_map.genesis_key()));
        let mut settings = Settings::default();
        settings.networks.insert(
            "network_1".to_string(),
            NetworkInfo::Remote("https://safe-testnet-tool.s3.eu-west-2.amazonaws.com/sn_cli_resources/PublicKey(08d5..60a1)".to_string(), None)
        );
        settings.networks.insert(
            "network_2".to_string(),
            NetworkInfo::Local(prefix_map_path, Some(*prefix_map.genesis_key())),
        );
        let config = Config::create_config(&tmp_dir, Some(settings)).await?;

        assert_eq!(config.networks_iter().count(), 2);
        let mut iter = config.networks_iter();
        // first network
        let (network_name, network_info) = iter
            .next()
            .ok_or_else(|| eyre!("failed to obtain item from networks list"))?;
        assert_eq!(network_name, "network_1");
        assert!(matches!(network_info, NetworkInfo::Remote(_, None)));

        // second network
        let (network_name, network_info) = iter
            .next()
            .ok_or_else(|| eyre!("failed to obtain item from networks list"))?;
        assert_eq!(network_name, "network_2");
        assert!(matches!(
                network_info,
                NetworkInfo::Local(_, Some(genesis_key)) if genesis_key == prefix_map.genesis_key()
        ));
        Ok(())
    }

    #[tokio::test]
    async fn given_an_empty_config_file_empty_settings_should_be_returned() -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let cli_config_file = tmp_dir.child(".safe/cli/config.json");
        cli_config_file.touch()?;
        let prefix_maps_dir = tmp_dir.child(".safe/prefix_maps");
        let config = Config::new(
            PathBuf::from(cli_config_file.path()),
            PathBuf::from(prefix_maps_dir.path()),
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
    use super::Config;
    use color_eyre::Result;
    use sn_api::DEFAULT_PREFIX_HARDLINK_NAME;
    use tokio::fs;

    #[tokio::test]
    async fn given_default_prefix_map_it_should_be_read() -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let config = Config::create_config(&tmp_dir, None).await?;
        let prefix_map = config
            .store_dummy_prefix_maps_and_set_default(1)
            .await?
            .pop()
            .unwrap();

        let retrieved_prefix_map = config.read_default_prefix_map().await?;
        assert_eq!(retrieved_prefix_map, prefix_map);

        Ok(())
    }

    #[tokio::test]
    async fn given_no_default_prefix_map_hardlink_it_should_be_an_error() -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let config = Config::create_config(&tmp_dir, None).await?;
        let _ = config.store_dummy_prefix_maps_and_set_default(1).await?;
        fs::remove_file(&config.prefix_maps_dir.join(DEFAULT_PREFIX_HARDLINK_NAME)).await?;

        let retrieved_prefix_map = config.read_default_prefix_map().await;
        assert!(retrieved_prefix_map.is_err(), "Hardlink should not exist");

        Ok(())
    }

    #[tokio::test]
    async fn given_no_prefix_map_file_it_should_be_an_error() -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let config = Config::create_config(&tmp_dir, None).await?;
        let retrieved_prefix_map = config.read_default_prefix_map().await;
        assert!(retrieved_prefix_map.is_err(), "PrefixMap should not exist");

        Ok(())
    }
}

#[cfg(test)]
mod sync_prefix_maps_and_settings {
    use super::Config;
    use crate::operations::config::{test_utils::store_dummy_prefix_maps, NetworkInfo, Settings};
    use color_eyre::eyre::eyre;
    use color_eyre::Result;
    use tokio::fs;

    #[tokio::test]
    async fn empty_cli_config_file_should_be_populated_by_existing_prefix_maps() -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let mut config = Config::create_config(&tmp_dir, None).await?;
        let _ = config.store_dummy_prefix_maps_and_set_default(4).await?;

        config.sync().await?;
        // sync should not read "default" file; hence 4 networks as expected
        assert_eq!(config.settings.networks.len(), 4);
        config.compare_settings_and_prefix_maps_dir().await?;
        Ok(())
    }

    #[tokio::test]
    async fn prefix_maps_should_be_fetched_from_cli_config_file() -> Result<()> {
        // write cli/config.json file
        let tmp_dir = assert_fs::TempDir::new()?;
        let prefix_maps = store_dummy_prefix_maps(&tmp_dir, 2).await?;
        let mut settings = Settings::default();
        settings.networks.insert(
            "network_1".to_string(),
            NetworkInfo::Remote("https://safe-testnet-tool.s3.eu-west-2.amazonaws.com/sn_cli_resources/PublicKey(08d5..60a1)".to_string(), None)
        );
        settings.networks.insert(
            "network_2".to_string(),
            NetworkInfo::Remote("https://safe-testnet-tool.s3.eu-west-2.amazonaws.com/sn_cli_resources/PublicKey(16b0..6ec8)".to_string(), None)
        );
        prefix_maps
            .iter()
            .enumerate()
            .for_each(|(idx, prefix_map)| {
                let prefix_map_path = tmp_dir
                    .path()
                    .join(format!("{:?}", prefix_map.genesis_key()));
                settings.networks.insert(
                    format!("local_network_{}", idx + 1),
                    NetworkInfo::Local(prefix_map_path, Some(*prefix_map.genesis_key())),
                );
            });
        let mut config = Config::create_config(&tmp_dir, Some(settings)).await?;

        config.sync().await?;
        assert_eq!(config.settings.networks.len(), 4);
        config.compare_settings_and_prefix_maps_dir().await?;
        Ok(())
    }

    #[tokio::test]
    async fn unreachable_remote_and_local_variants_should_be_removed_from_cli_config_file(
    ) -> Result<()> {
        // write cli/config.json file
        let tmp_dir = assert_fs::TempDir::new()?;
        let mut settings = Settings::default();
        settings.networks.insert(
            "network_1".to_string(),
            NetworkInfo::Remote("https://safe-testnet-tool.s3.eu-west-2.amazonaws.com/sn_cli_resources/PublicKey(0000..0000)".to_string(), None)
        );
        settings.networks.insert(
            "network_2".to_string(),
            NetworkInfo::Local(tmp_dir.path().join("PublicKey(0000.0000)"), None),
        );
        let mut config = Config::create_config(&tmp_dir, Some(settings)).await?;

        config.sync().await?;
        assert_eq!(config.settings.networks.len(), 0);
        config.compare_settings_and_prefix_maps_dir().await?;
        Ok(())
    }

    #[tokio::test]
    async fn genesis_key_field_should_be_set() -> Result<()> {
        // write cli/config.json file
        let tmp_dir = assert_fs::TempDir::new()?;
        let prefix_map = store_dummy_prefix_maps(&tmp_dir, 1).await?.pop().unwrap();
        let prefix_map_path = tmp_dir
            .path()
            .join(format!("{:?}", prefix_map.genesis_key()));
        let mut settings = Settings::default();
        settings.networks.insert(
            "network_1".to_string(),
            NetworkInfo::Remote("https://safe-testnet-tool.s3.eu-west-2.amazonaws.com/sn_cli_resources/PublicKey(08d5..60a1)".to_string(), None)
        );
        settings.networks.insert(
            "network_2".to_string(),
            NetworkInfo::Local(prefix_map_path, None),
        );
        let mut config = Config::create_config(&tmp_dir, Some(settings)).await?;

        config.sync().await?;
        // network_1
        let mut iter = config.networks_iter();
        let (network_name, network_info) = iter
            .next()
            .ok_or_else(|| eyre!("failed to obtain item from networks list"))?;
        assert_eq!(network_name, "network_1");
        assert!(matches!(network_info, NetworkInfo::Remote(_, Some(_))));
        // network_2
        let (network_name, network_info) = iter
            .next()
            .ok_or_else(|| eyre!("failed to obtain item from networks list"))?;
        assert_eq!(network_name, "network_2");
        assert!(matches!(network_info, NetworkInfo::Local(_, Some(_))));

        config.compare_settings_and_prefix_maps_dir().await?;

        Ok(())
    }

    #[tokio::test]
    async fn network_list_should_be_fetched_from_cli_config_file() -> Result<()> {
        // write cli/config.json file
        let tmp_dir = assert_fs::TempDir::new()?;
        let mut prefix_maps = store_dummy_prefix_maps(&tmp_dir, 2).await?;
        let prefix_map_path = tmp_dir
            .path()
            .join(format!("{:?}", prefix_maps.pop().unwrap().genesis_key()));
        let mut settings = Settings::default();
        settings.networks.insert(
            "network_1".to_string(),
            NetworkInfo::Local(prefix_map_path, None),
        );
        let mut config = Config::create_config(&tmp_dir, Some(settings)).await?;

        let prefix_map_path = tmp_dir
            .path()
            .join(format!("{:?}", prefix_maps.pop().unwrap().genesis_key()));
        // will be ignored during sync since the network list is fetched from the config file
        config.settings.networks.insert(
            "network_2".to_string(),
            NetworkInfo::Local(prefix_map_path.clone(), None),
        );
        config.sync().await?;
        assert_eq!(config.settings.networks.len(), 1);

        // but it will be read if it's present in the cli/config.json file
        config.settings.networks.insert(
            "network_2".to_string(),
            NetworkInfo::Local(prefix_map_path, None),
        );
        config.write_settings_to_file().await?;
        config.sync().await?;
        assert_eq!(config.settings.networks.len(), 2);

        Ok(())
    }

    #[tokio::test]
    async fn multiple_networks_with_the_same_prefix_map() -> Result<()> {
        // write cli/config.json file
        let tmp_dir = assert_fs::TempDir::new()?;
        let prefix_map = store_dummy_prefix_maps(&tmp_dir, 1).await?.pop().unwrap();
        let prefix_map_path = tmp_dir
            .path()
            .join(format!("{:?}", prefix_map.genesis_key()));
        let mut settings = Settings::default();
        settings.networks.insert(
            "network_1".to_string(),
            NetworkInfo::Local(prefix_map_path.clone(), None),
        );
        settings.networks.insert(
            "network_1_copy".to_string(),
            NetworkInfo::Local(prefix_map_path, None),
        );
        let mut config = Config::create_config(&tmp_dir, Some(settings)).await?;

        config.sync().await?;
        assert_eq!(config.settings.networks.len(), 2);
        config.compare_settings_and_prefix_maps_dir().await?;
        Ok(())
    }

    #[tokio::test]
    async fn local_variant_with_path_inside_prefix_map_dir() -> Result<()> {
        // This is the case if prefix map was directly pasted into the dir. It should behave as expected
        let tmp_dir = assert_fs::TempDir::new()?;
        let mut config = Config::create_config(&tmp_dir, None).await?;
        let prefix_map = config
            .store_dummy_prefix_maps_and_set_default(1)
            .await?
            .pop()
            .unwrap();

        config.sync().await?;

        assert_eq!(config.settings.networks.len(), 1);
        let (network_name, network_info) = config
            .networks_iter()
            .next()
            .ok_or_else(|| eyre!("failed to obtain item from networks list"))?;
        assert_eq!(*network_name, format!("{:?}", prefix_map.genesis_key()));
        assert!(matches!(
            network_info,
            NetworkInfo::Local(_, Some(genesis_key)) if genesis_key == prefix_map.genesis_key()
        ));

        fs::remove_file(
            config
                .prefix_maps_dir
                .join(format!("{:?}", prefix_map.genesis_key())),
        )
        .await?;
        config.sync().await?;
        assert_eq!(config.settings.networks.len(), 0);
        config.compare_settings_and_prefix_maps_dir().await?;

        Ok(())
    }
}

#[cfg(test)]
mod networks {
    use super::{test_utils::store_dummy_prefix_maps, Config, NetworkInfo};
    use color_eyre::eyre::eyre;
    use color_eyre::Result;

    #[tokio::test]
    async fn local_and_remote_networks_should_be_added() -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let prefix_map = store_dummy_prefix_maps(&tmp_dir, 1).await?.pop().unwrap();
        let prefix_map_path = tmp_dir
            .path()
            .join(format!("{:?}", prefix_map.genesis_key()));
        let mut config = Config::create_config(&tmp_dir, None).await?;

        let network_1 = NetworkInfo::Remote(
            "https://safe-testnet-tool.s3.eu-west-2.amazonaws.com/sn_cli_resources/PublicKey(08d5..60a1)".to_string(),
            None,
        );
        let network_2 = NetworkInfo::Local(prefix_map_path, None);
        config.add_network("network_1", network_1).await?;
        config.add_network("network_2", network_2).await?;

        assert_eq!(config.settings.networks.len(), 2);
        config.compare_settings_and_prefix_maps_dir().await?;

        Ok(())
    }

    #[tokio::test]
    async fn add_local_network_where_path_lies_inside_prefix_maps_dir() -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let mut config = Config::create_config(&tmp_dir, None).await?;
        let prefix_map = config
            .store_dummy_prefix_maps_and_set_default(1)
            .await?
            .pop()
            .unwrap();
        let path = config
            .prefix_maps_dir
            .join(format!("{:?}", prefix_map.genesis_key()));

        let network_1 = NetworkInfo::Local(path, None);
        config.add_network("network_1", network_1).await?;

        assert_eq!(config.settings.networks.len(), 1);
        config.compare_settings_and_prefix_maps_dir().await?;
        Ok(())
    }

    #[tokio::test]
    async fn removing_network_should_give_the_desirable_output() -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let prefix_map = store_dummy_prefix_maps(&tmp_dir, 1).await?.pop().unwrap();
        let prefix_map_path = tmp_dir
            .path()
            .join(format!("{:?}", prefix_map.genesis_key()));
        let mut config = Config::create_config(&tmp_dir, None).await?;

        let network_1 = NetworkInfo::Local(prefix_map_path, None);
        config.add_network("network_1", network_1).await?;
        assert_eq!(config.settings.networks.len(), 1);

        config.remove_network("a_random_network").await?;
        assert_eq!(config.settings.networks.len(), 1);
        config.compare_settings_and_prefix_maps_dir().await?;

        config.remove_network("network_1").await?;
        assert_eq!(config.settings.networks.len(), 0);
        config.compare_settings_and_prefix_maps_dir().await?;

        Ok(())
    }

    #[tokio::test]
    async fn switching_network_should_change_the_default_prefix_map() -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let mut prefix_maps = store_dummy_prefix_maps(&tmp_dir, 2).await?;
        let mut config = Config::create_config(&tmp_dir, None).await?;

        let prefix_map_1 = prefix_maps.pop().unwrap();
        let network_1 = NetworkInfo::Local(
            tmp_dir
                .path()
                .join(format!("{:?}", prefix_map_1.genesis_key())),
            None,
        );
        let prefix_map_2 = prefix_maps.pop().unwrap();
        let network_2 = NetworkInfo::Local(
            tmp_dir
                .path()
                .join(format!("{:?}", prefix_map_2.genesis_key())),
            None,
        );
        config.add_network("network_1", network_1).await?;
        config.add_network("network_2", network_2).await?;

        config.switch_to_network("network_1").await?;
        let default = config.read_default_prefix_map().await?;
        let net_info = config
            .settings
            .networks
            .get("network_1")
            .ok_or_else(|| eyre!("network_1 should be present"))?;
        assert_eq!(default, prefix_map_1);
        assert!(matches!(
            net_info,
            NetworkInfo::Local(_, Some(genesis_key)) if genesis_key == default.genesis_key()
        ));

        config.switch_to_network("network_2").await?;
        let default = config.read_default_prefix_map().await?;
        let net_info = config
            .settings
            .networks
            .get("network_2")
            .ok_or_else(|| eyre!("network_2 should be present"))?;
        assert_eq!(default, prefix_map_2);
        assert!(matches!(
            net_info,
            NetworkInfo::Local(_, Some(genesis_key)) if genesis_key == default.genesis_key()
        ));

        Ok(())
    }

    #[tokio::test]
    async fn switching_to_a_random_network_should_return_error() -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let config = Config::create_config(&tmp_dir, None).await?;

        let switch_result = config.switch_to_network("network_1").await;
        let default = config.read_default_prefix_map().await;
        assert!(switch_result.is_err());
        assert!(default.is_err());
        Ok(())
    }
}
