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
use sn_api::{Safe, SectionTree, DEFAULT_NETWORK_CONTACTS_FILE_NAME};
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
        println!("Starting nodes to join the Safe network...");
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
    /// to network_contacts_dir
    Local(PathBuf, Option<BlsPublicKey>),
    /// The remote url pointing to a network map. The optional genesis key denotes that the network map has been copied
    /// to network_contacts_dir
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
    pub network_contacts_dir: PathBuf,
    pub dbc_owner: Option<Owner>,
}

impl Config {
    pub async fn new(cli_config_path: PathBuf, network_contacts_dir: PathBuf) -> Result<Config> {
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

        fs::create_dir_all(network_contacts_dir.as_path()).await?;
        let mut dbc_owner_sk_path = pb.clone();
        dbc_owner_sk_path.push("credentials");
        let dbc_owner = Config::get_dbc_owner(&dbc_owner_sk_path)?;

        let config = Config {
            settings,
            cli_config_path: cli_config_path.clone(),
            network_contacts_dir,
            dbc_owner,
        };
        config.write_settings_to_file().await.wrap_err_with(|| {
            format!("Unable to create config at '{}'", cli_config_path.display())
        })?;
        Ok(config)
    }

    /// Sync settings and the network_contacts_dir
    pub async fn sync(&mut self) -> Result<()> {
        // if default hardlink is present, make sure its original file is also present; Else the
        // default hardlink might be overwritten while switching networks
        if let Ok((default_network_contacts, _)) = self.read_default_network_contacts().await {
            self.write_network_contacts(&default_network_contacts)
                .await?;
        };

        let mut dir_files_checklist: BTreeMap<String, bool> = BTreeMap::new();
        let mut network_contacts_dir = fs::read_dir(&self.network_contacts_dir).await?;
        while let Some(entry) = network_contacts_dir.next_entry().await? {
            if entry.metadata().await?.is_file() {
                let filename = entry
                    .file_name()
                    .into_string()
                    .map_err(|_| eyre!("Error converting OsString to String"))?;
                if filename != *DEFAULT_NETWORK_CONTACTS_FILE_NAME {
                    dir_files_checklist.insert(filename, false);
                }
            }
        }

        // get SectionTree from cli_config if they are not in network_contacts_dir
        let mut remove_list: Vec<String> = Vec::new();
        for (network_name, net_info) in self.settings.networks.iter_mut() {
            match net_info {
                NetworkInfo::Local(path, ref mut genesis_key) => {
                    match genesis_key {
                        Some(gk) => match dir_files_checklist.get_mut(format!("{:?}", gk).as_str())
                        {
                            Some(present) => *present = true,
                            None => {
                                if let Ok(network_contacts) =
                                    Self::retrieve_local_network_contacts(path).await
                                {
                                    Self::write_network_contacts_to_dir(
                                        &self.network_contacts_dir,
                                        &network_contacts,
                                    )
                                    .await?;
                                    *genesis_key = Some(*network_contacts.genesis_key());
                                } else {
                                    remove_list.push(network_name.clone());
                                }
                            }
                        },
                        // SectionTree has not been fetched, fetch it
                        None => {
                            if let Ok(network_contacts) =
                                Self::retrieve_local_network_contacts(path).await
                            {
                                Self::write_network_contacts_to_dir(
                                    &self.network_contacts_dir,
                                    &network_contacts,
                                )
                                .await?;
                                *genesis_key = Some(*network_contacts.genesis_key());
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
                            if let Ok(network_contacts) =
                                Self::retrieve_remote_network_contacts(&url).await
                            {
                                Self::write_network_contacts_to_dir(
                                    &self.network_contacts_dir,
                                    &network_contacts,
                                )
                                .await?;
                                *genesis_key = Some(*network_contacts.genesis_key());
                            } else {
                                remove_list.push(network_name.clone());
                            }
                        }
                    },
                    None => {
                        let url = Url::parse(url)?;
                        if let Ok(network_contacts) =
                            Self::retrieve_remote_network_contacts(&url).await
                        {
                            Self::write_network_contacts_to_dir(
                                &self.network_contacts_dir,
                                &network_contacts,
                            )
                            .await?;
                            *genesis_key = Some(*network_contacts.genesis_key());
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

        // add unaccounted SectionTree from network_contacts_dir to cli_config
        for (filename, present) in dir_files_checklist.iter() {
            if !present {
                let path = self.network_contacts_dir.join(filename);
                if let Ok(network_contacts) = Self::retrieve_local_network_contacts(&path).await {
                    let genesis_key = *network_contacts.genesis_key();
                    self.settings.networks.insert(
                        format!("{:?}", genesis_key),
                        NetworkInfo::Local(path, Some(genesis_key)),
                    );
                }
                // else remove the network_contacts if not SectionTree type?
            }
        }
        self.write_settings_to_file().await?;
        Ok(())
    }

    pub async fn read_default_network_contacts(&self) -> Result<(SectionTree, String)> {
        let default_path = self
            .network_contacts_dir
            .join(DEFAULT_NETWORK_CONTACTS_FILE_NAME);
        let network_contacts = Self::retrieve_local_network_contacts(&default_path)
            .await
            .wrap_err_with(|| {
                eyre!("There doesn't seem to be any default Network Map").suggestion(
                    "A Network Map will be created if you join a network or launch your own.",
                )
            })?;

        Ok((network_contacts, default_path.display().to_string()))
    }

    pub async fn read_network_contacts(&mut self, name: &str) -> Result<(SectionTree, String)> {
        match self.settings.networks.get(name).cloned() {
            Some(NetworkInfo::Local(ref mut path, _)) => {
                if !path.is_absolute() {
                    *path = fs::canonicalize(&path).await?;
                }
                let network_contacts = Self::retrieve_local_network_contacts(path).await?;
                Ok((network_contacts, path.display().to_string()))
            }
            Some(NetworkInfo::Remote(ref url, _)) => {
                let url = Url::parse(url)?;
                let network_contacts = Self::retrieve_remote_network_contacts(&url).await?;
                Ok((network_contacts, url.to_string()))
            }
            None => Err(eyre!("No network with name '{}' was found in config", name)),
        }
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
                let network_contacts = Self::retrieve_local_network_contacts(path).await?;
                self.write_network_contacts(&network_contacts).await?;
                *genesis_key = Some(*network_contacts.genesis_key());
            }
            NetworkInfo::Remote(ref url, ref mut genesis_key) => {
                let url = Url::parse(url)?;
                let network_contacts = Self::retrieve_remote_network_contacts(&url).await?;
                self.write_network_contacts(&network_contacts).await?;
                *genesis_key = Some(*network_contacts.genesis_key());
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
                    let network_contacts_path = self.network_contacts_dir.join(format!("{:?}", gk));
                    if fs::remove_file(&network_contacts_path).await.is_err() {
                        println!(
                            "Failed to remove network map from {}",
                            network_contacts_path.display()
                        )
                    }
                }
                // if None, then the file is not present, since we sync during config init
            }
            Some(NetworkInfo::Remote(_, genesis_key)) => {
                self.write_settings_to_file().await?;
                if let Some(gk) = genesis_key {
                    let network_contacts_path = self.network_contacts_dir.join(format!("{:?}", gk));
                    if fs::remove_file(&network_contacts_path).await.is_err() {
                        println!(
                            "Failed to remove network map from {}",
                            network_contacts_path.display()
                        )
                    }
                }
            }
            None => println!("No network with name '{}' was found in config", name),
        }
        if fs::remove_file(
            &self
                .network_contacts_dir
                .join(DEFAULT_NETWORK_CONTACTS_FILE_NAME),
        )
        .await
        .is_err()
        {
            debug!("Cannot remove default SectionTree!");
        };
        debug!("Network '{}' removed from config", name);
        println!("Network '{}' was removed from the config", name);

        Ok(())
    }

    pub async fn clear(&mut self) -> Result<()> {
        self.settings = Settings::default();
        self.write_settings_to_file().await?;
        // delete all network contacts files
        let mut network_contacts_dir = fs::read_dir(&self.network_contacts_dir).await?;
        while let Some(entry) = network_contacts_dir.next_entry().await? {
            fs::remove_file(entry.path()).await?;
        }
        Ok(())
    }

    pub async fn switch_to_network(&self, name: &str) -> Result<()> {
        match self.settings.networks.get(name) {
            Some(NetworkInfo::Local(_, genesis_key)) => {
                match genesis_key {
                    Some(gk) => self.set_default_network_contacts(gk).await?,
                    //  can't be none since we sync during get_config
                    None => bail!("Cannot switch to {}, since the network file is not found! Please re-run the same command!", name)
                }
            }
            Some(NetworkInfo::Remote(_, genesis_key)) => {
                match genesis_key {
                    Some(gk) => self.set_default_network_contacts(gk).await?,
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
        table.add_row(&vec![
            "Current",
            "Network name",
            "Genesis Key",
            "Network map info",
        ]);
        let current_network_contacts = self.read_default_network_contacts().await;

        for (network_name, net_info) in self.networks_iter() {
            let mut current = "";
            if let Ok((network_contacts, _)) = &current_network_contacts {
                if net_info.matches(network_contacts.genesis_key()) {
                    current = "*";
                }
            }
            let (simplified_net_info, gk) = match net_info {
                NetworkInfo::Local(path, gk) => (format!("Local: {:?}", path), gk),
                NetworkInfo::Remote(url, gk) => (format!("Remote: {:?}", url), gk),
            };
            let genesis_key = if let Some(key) = gk {
                format!("{:?}", key)
            } else {
                "".to_string()
            };
            table.add_row(&vec![
                current,
                network_name,
                &genesis_key,
                simplified_net_info.as_str(),
            ]);
        }

        println!("{table}");
    }

    pub async fn set_default_network_contacts(&self, genesis_key: &BlsPublicKey) -> Result<()> {
        let network_contacts_file = self.network_contacts_dir.join(format!("{:?}", genesis_key));
        let default_network_contacts = self
            .network_contacts_dir
            .join(DEFAULT_NETWORK_CONTACTS_FILE_NAME);

        if default_network_contacts.exists() {
            fs::remove_file(&default_network_contacts)
                .await
                .wrap_err_with(|| {
                    format!(
                        "Error removing default SectionTree hardlink: {:?}",
                        default_network_contacts.display()
                    )
                })?;
        }
        debug!(
            "Creating hardlink for SectionTree from {:?} to {:?}",
            network_contacts_file.display(),
            default_network_contacts.display()
        );
        fs::hard_link(&network_contacts_file, &default_network_contacts)
            .await
            .wrap_err_with(|| {
                format!(
                    "Error creating hardlink from {:?} to {:?}",
                    network_contacts_file.display(),
                    default_network_contacts.display()
                )
            })?;
        Ok(())
    }

    pub async fn update_default_network_contacts(
        &self,
        network_contacts: &SectionTree,
    ) -> Result<()> {
        self.write_network_contacts(network_contacts).await?;
        self.set_default_network_contacts(network_contacts.genesis_key())
            .await
    }

    pub async fn write_network_contacts(&self, network_contacts: &SectionTree) -> Result<()> {
        Self::write_network_contacts_to_dir(&self.network_contacts_dir, network_contacts).await
    }

    pub async fn retrieve_local_network_contacts(location: &Path) -> Result<SectionTree> {
        let pm = SectionTree::from_disk(location).await?;
        Ok(pm)
    }

    pub async fn retrieve_remote_network_contacts(url: &Url) -> Result<SectionTree> {
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
            Some(b) => {
                let pm = SectionTree::from_bytes(&b[..])?;
                Ok(pm)
            }
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

    // Write the network contacts file within the provided directory path,
    // using the genesis key as the filename.
    async fn write_network_contacts_to_dir(
        dir: &Path,
        network_contacts: &SectionTree,
    ) -> Result<()> {
        let path = dir.join(format!("{:?}", network_contacts.genesis_key()));
        network_contacts.write_to_disk(&path).await?;
        Ok(())
    }

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
}

#[cfg(test)]
pub mod test_utils {
    use super::{Config, NetworkInfo};
    use crate::operations::config::Settings;
    use assert_fs::{prelude::*, TempDir};
    use color_eyre::{eyre::eyre, Result};
    use sn_api::{SectionTree, DEFAULT_NETWORK_CONTACTS_FILE_NAME};
    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};
    use tokio::fs;

    pub async fn store_dummy_network_contacts(
        path: &Path,
        n_network_contacts: usize,
    ) -> Result<Vec<SectionTree>> {
        let mut dummy_network_contacts: Vec<SectionTree> = Vec::new();

        for _ in 0..n_network_contacts {
            let sk = bls::SecretKey::random();
            let network_contacts = SectionTree::new(sk.public_key());
            let filename = format!("{:?}", network_contacts.genesis_key());

            network_contacts.write_to_disk(&path.join(filename)).await?;
            dummy_network_contacts.push(network_contacts);
        }
        Ok(dummy_network_contacts)
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

            let network_contacts_dir = tmp_dir.child(".safe/network_contacts");
            Config::new(
                PathBuf::from(cli_config_file.path()),
                PathBuf::from(network_contacts_dir.path()),
            )
            .await
        }

        pub async fn store_dummy_network_contacts_and_set_default(
            &self,
            n_network_contacts: usize,
        ) -> Result<Vec<SectionTree>> {
            let dummy_network_contacts =
                store_dummy_network_contacts(&self.network_contacts_dir, n_network_contacts)
                    .await?;
            // set one as default
            let default_network_contacts = dummy_network_contacts.clone().pop().unwrap();
            self.set_default_network_contacts(default_network_contacts.genesis_key())
                .await?;
            Ok(dummy_network_contacts)
        }

        /// Compare the network entries by creating a checklist using the settings and marking
        /// them as 'true' if the network is present in the system.
        pub async fn compare_settings_and_network_contacts_dir(&self) -> Result<()> {
            let mut network_contacts_checklist: BTreeMap<String, bool> = BTreeMap::new();

            // get list of all networks from settings
            for (_, net_info) in self.networks_iter() {
                let genesis_key =
                    match net_info {
                        NetworkInfo::Local(_, genesis_key) => genesis_key
                            .ok_or_else(|| eyre!("gk should must be present after sync"))?,
                        NetworkInfo::Remote(_, genesis_key) => genesis_key
                            .ok_or_else(|| eyre!("gk should must be present after sync"))?,
                    };

                // same gk can be present if multiple network entries in settings points
                // to the same network contacts file
                let _ = network_contacts_checklist.insert(format!("{:?}", genesis_key), false);
            }

            // mark them as true if the same entries are found in the network_contacts_dir
            let mut network_contacts_dir = fs::read_dir(&self.network_contacts_dir).await?;
            while let Some(entry) = network_contacts_dir.next_entry().await? {
                if entry.metadata().await?.is_file() {
                    let filename = entry
                        .file_name()
                        .into_string()
                        .map_err(|_| eyre!("Error converting OsString to String"))?;
                    if filename != *DEFAULT_NETWORK_CONTACTS_FILE_NAME {
                        let already_present = network_contacts_checklist.insert(filename, true);
                        // cannot insert new entries. Denotes that an extra network is found in
                        // network_contacts_dir
                        if already_present.is_none() {
                            return Err(eyre!("Extra network found in the system!"));
                        }
                    }
                }
            }

            for (_, present) in network_contacts_checklist.iter() {
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
    use crate::operations::config::{test_utils::store_dummy_network_contacts, Settings};
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
        let network_contacts_dir = tmp_dir.child(".safe/network_contacts");

        let cli_config_file = cli_config_dir.child("config.json");
        let dbc_owner_sk_file = cli_config_dir.child("credentials");
        let sk = SecretKey::random();
        Safe::serialize_bls_key(&sk, dbc_owner_sk_file.path())?;

        let config = Config::new(
            PathBuf::from(cli_config_file.path()),
            PathBuf::from(network_contacts_dir.path()),
        )
        .await?;

        assert_eq!(config.cli_config_path, cli_config_file.path());
        assert_eq!(config.network_contacts_dir, network_contacts_dir.path());
        assert_eq!(config.settings.networks.len(), 0);
        assert!(config.dbc_owner.is_some());
        Ok(())
    }

    #[tokio::test]
    async fn cli_config_directory_should_be_created() -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let cli_config_dir = tmp_dir.child(".safe/cli");
        let cli_config_file = cli_config_dir.child("config.json");
        let network_contacts_dir = tmp_dir.child(".safe/network_contacts");

        let _ = Config::new(
            PathBuf::from(cli_config_file.path()),
            PathBuf::from(network_contacts_dir.path()),
        )
        .await?;

        cli_config_dir.assert(predicate::path::is_dir());
        network_contacts_dir.assert(predicate::path::is_dir());
        Ok(())
    }

    #[tokio::test]
    async fn given_config_file_does_not_exist_then_it_should_be_created() -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let cli_config_file = tmp_dir.child(".safe/cli/config.json");
        let network_contacts_dir = tmp_dir.child(".safe/network_contacts");

        let _ = Config::new(
            PathBuf::from(cli_config_file.path()),
            PathBuf::from(network_contacts_dir.path()),
        )
        .await?;

        cli_config_file.assert(predicate::path::exists());

        Ok(())
    }

    #[tokio::test]
    async fn given_config_file_exists_then_the_settings_should_be_read() -> Result<()> {
        // write cli/config.json file
        let tmp_dir = assert_fs::TempDir::new()?;
        let network_contacts = store_dummy_network_contacts(&tmp_dir, 1)
            .await?
            .pop()
            .unwrap();
        let network_contacts_path = tmp_dir
            .path()
            .join(format!("{:?}", network_contacts.genesis_key()));
        let mut settings = Settings::default();
        settings.networks.insert(
            "network_1".to_string(),
            NetworkInfo::Remote(
                "https://safe-testnet-tool.s3.eu-west-2.amazonaws.com/sn_cli_resources/network_contacts"
                    .to_string(),
                None,
            ),
        );
        settings.networks.insert(
            "network_2".to_string(),
            NetworkInfo::Local(network_contacts_path, Some(*network_contacts.genesis_key())),
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
                NetworkInfo::Local(_, Some(genesis_key)) if genesis_key == network_contacts.genesis_key()
        ));
        Ok(())
    }

    #[tokio::test]
    async fn given_an_empty_config_file_empty_settings_should_be_returned() -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let cli_config_file = tmp_dir.child(".safe/cli/config.json");
        cli_config_file.touch()?;
        let network_contacts_dir = tmp_dir.child(".safe/network_contacts");
        let config = Config::new(
            PathBuf::from(cli_config_file.path()),
            PathBuf::from(network_contacts_dir.path()),
        )
        .await?;

        assert_eq!(0, config.settings.networks.len());
        assert_eq!(cli_config_file.path(), config.cli_config_path.as_path());
        assert_eq!(
            network_contacts_dir.path(),
            config.network_contacts_dir.as_path()
        );

        Ok(())
    }
}

#[cfg(test)]
mod read_network_contacts {
    use super::Config;
    use color_eyre::Result;
    use sn_api::DEFAULT_NETWORK_CONTACTS_FILE_NAME;
    use tokio::fs;

    #[tokio::test]
    async fn given_default_network_contacts_it_should_be_read() -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let config = Config::create_config(&tmp_dir, None).await?;
        let network_contacts = config
            .store_dummy_network_contacts_and_set_default(1)
            .await?
            .pop()
            .unwrap();

        let (retrieved_network_contacts, _) = config.read_default_network_contacts().await?;
        assert_eq!(retrieved_network_contacts, network_contacts);

        Ok(())
    }

    #[tokio::test]
    async fn given_no_default_network_contacts_hardlink_it_should_be_an_error() -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let config = Config::create_config(&tmp_dir, None).await?;
        let _ = config
            .store_dummy_network_contacts_and_set_default(1)
            .await?;
        fs::remove_file(
            &config
                .network_contacts_dir
                .join(DEFAULT_NETWORK_CONTACTS_FILE_NAME),
        )
        .await?;

        let retrieved_network_contacts = config.read_default_network_contacts().await;
        assert!(
            retrieved_network_contacts.is_err(),
            "Hardlink should not exist"
        );

        Ok(())
    }

    #[tokio::test]
    async fn given_no_network_contacts_file_it_should_be_an_error() -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let config = Config::create_config(&tmp_dir, None).await?;
        let retrieved_network_contacts = config.read_default_network_contacts().await;
        assert!(
            retrieved_network_contacts.is_err(),
            "Network contacts file should not exist"
        );

        Ok(())
    }
}

#[cfg(test)]
mod sync_network_contacts_and_settings {
    use super::Config;
    use crate::operations::config::{
        test_utils::store_dummy_network_contacts, NetworkInfo, Settings,
    };
    use color_eyre::eyre::eyre;
    use color_eyre::Result;
    use sn_api::DEFAULT_NETWORK_CONTACTS_FILE_NAME;
    use tokio::fs;

    #[tokio::test]
    async fn empty_cli_config_file_should_be_populated_by_existing_network_contacts() -> Result<()>
    {
        let tmp_dir = assert_fs::TempDir::new()?;
        let mut config = Config::create_config(&tmp_dir, None).await?;
        let _ = config
            .store_dummy_network_contacts_and_set_default(4)
            .await?;

        config.sync().await?;
        // sync should not read "default" file; hence 4 networks as expected
        assert_eq!(config.settings.networks.len(), 4);
        config.compare_settings_and_network_contacts_dir().await?;
        Ok(())
    }

    #[tokio::test]
    async fn network_contacts_should_be_fetched_from_cli_config_file() -> Result<()> {
        // write cli/config.json file
        let tmp_dir = assert_fs::TempDir::new()?;
        let network_contacts = store_dummy_network_contacts(&tmp_dir, 2).await?;
        let mut settings = Settings::default();
        settings.networks.insert(
            "network_1".to_string(),
            NetworkInfo::Remote(
                "https://safe-testnet-tool.s3.eu-west-2.amazonaws.com/sn_cli_resources/network_contacts"
                    .to_string(),
                None,
            ),
        );
        settings.networks.insert(
            "network_2".to_string(),
            NetworkInfo::Remote("https://safe-testnet-tool.s3.eu-west-2.amazonaws.com/sn_cli_resources/network_contacts_1".to_string(), None)
        );
        network_contacts
            .iter()
            .enumerate()
            .for_each(|(idx, network_contacts)| {
                let network_contacts_path = tmp_dir
                    .path()
                    .join(format!("{:?}", network_contacts.genesis_key()));
                settings.networks.insert(
                    format!("local_network_{}", idx + 1),
                    NetworkInfo::Local(
                        network_contacts_path,
                        Some(*network_contacts.genesis_key()),
                    ),
                );
            });
        let mut config = Config::create_config(&tmp_dir, Some(settings)).await?;

        config.sync().await?;
        assert_eq!(config.settings.networks.len(), 4);
        config.compare_settings_and_network_contacts_dir().await?;
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
            NetworkInfo::Remote(
                "https://safe-testnet-tool.s3.eu-west-2.amazonaws.com/sn_cli_resources/error"
                    .to_string(),
                None,
            ),
        );
        settings.networks.insert(
            "network_2".to_string(),
            NetworkInfo::Local(tmp_dir.path().join("PublicKey(0000.0000)"), None),
        );
        let mut config = Config::create_config(&tmp_dir, Some(settings)).await?;

        config.sync().await?;
        assert_eq!(config.settings.networks.len(), 0);
        config.compare_settings_and_network_contacts_dir().await?;
        Ok(())
    }

    #[tokio::test]
    async fn genesis_key_field_should_be_set() -> Result<()> {
        // write cli/config.json file
        let tmp_dir = assert_fs::TempDir::new()?;
        let network_contacts = store_dummy_network_contacts(&tmp_dir, 1)
            .await?
            .pop()
            .unwrap();
        let network_contacts_path = tmp_dir
            .path()
            .join(format!("{:?}", network_contacts.genesis_key()));
        let mut settings = Settings::default();
        settings.networks.insert(
            "network_1".to_string(),
            NetworkInfo::Remote(
                "https://safe-testnet-tool.s3.eu-west-2.amazonaws.com/sn_cli_resources/network_contacts"
                    .to_string(),
                None,
            ),
        );
        settings.networks.insert(
            "network_2".to_string(),
            NetworkInfo::Local(network_contacts_path, None),
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

        config.compare_settings_and_network_contacts_dir().await?;

        Ok(())
    }

    #[tokio::test]
    async fn multiple_networks_with_the_same_network_contacts() -> Result<()> {
        // write cli/config.json file
        let tmp_dir = assert_fs::TempDir::new()?;
        let network_contacts = store_dummy_network_contacts(&tmp_dir, 1)
            .await?
            .pop()
            .unwrap();
        let network_contacts_path = tmp_dir
            .path()
            .join(format!("{:?}", network_contacts.genesis_key()));
        let mut settings = Settings::default();
        settings.networks.insert(
            "network_1".to_string(),
            NetworkInfo::Local(network_contacts_path.clone(), None),
        );
        settings.networks.insert(
            "network_1_copy".to_string(),
            NetworkInfo::Local(network_contacts_path, None),
        );
        let mut config = Config::create_config(&tmp_dir, Some(settings)).await?;

        config.sync().await?;
        assert_eq!(config.settings.networks.len(), 2);
        config.compare_settings_and_network_contacts_dir().await?;
        Ok(())
    }

    #[tokio::test]
    async fn local_variant_with_path_inside_network_contacts_dir() -> Result<()> {
        // This is the case if network contacts file was directly pasted into the dir. It should behave as expected
        let tmp_dir = assert_fs::TempDir::new()?;
        let mut config = Config::create_config(&tmp_dir, None).await?;
        let network_contacts = config
            .store_dummy_network_contacts_and_set_default(1)
            .await?
            .pop()
            .unwrap();

        config.sync().await?;

        assert_eq!(config.settings.networks.len(), 1);
        let (network_name, network_info) = config
            .networks_iter()
            .next()
            .ok_or_else(|| eyre!("failed to obtain item from networks list"))?;
        assert_eq!(
            *network_name,
            format!("{:?}", network_contacts.genesis_key())
        );
        assert!(matches!(
            network_info,
            NetworkInfo::Local(_, Some(genesis_key)) if genesis_key == network_contacts.genesis_key()
        ));

        fs::remove_file(
            config
                .network_contacts_dir
                .join(format!("{:?}", network_contacts.genesis_key())),
        )
        .await?;
        fs::remove_file(
            config
                .network_contacts_dir
                .join(DEFAULT_NETWORK_CONTACTS_FILE_NAME),
        )
        .await?;
        config.sync().await?;
        assert_eq!(config.settings.networks.len(), 0);
        config.compare_settings_and_network_contacts_dir().await?;

        Ok(())
    }

    #[tokio::test]
    async fn original_file_of_the_default_hardlink_should_be_written() -> Result<()> {
        // Switching networks can overwrite the default hardlink, so we make sure we have a copy
        // of the default network contacts file.
        let tmp_dir = assert_fs::TempDir::new()?;
        let mut config = Config::create_config(&tmp_dir, None).await?;
        let network_contacts = config
            .store_dummy_network_contacts_and_set_default(1)
            .await?
            .pop()
            .unwrap();
        fs::remove_file(
            config
                .network_contacts_dir
                .join(format!("{:?}", network_contacts.genesis_key())),
        )
        .await?;

        config.sync().await?;
        assert_eq!(config.settings.networks.len(), 1);
        let (network_name, network_info) = config
            .networks_iter()
            .next()
            .ok_or_else(|| eyre!("failed to obtain item from networks list"))?;
        assert_eq!(
            *network_name,
            format!("{:?}", network_contacts.genesis_key())
        );
        assert!(matches!(
            network_info,
            NetworkInfo::Local(_, Some(genesis_key)) if genesis_key == network_contacts.genesis_key()
        ));

        Ok(())
    }
}

#[cfg(test)]
mod networks {
    use super::{test_utils::store_dummy_network_contacts, Config, NetworkInfo};
    use color_eyre::eyre::eyre;
    use color_eyre::Result;

    #[tokio::test]
    async fn local_and_remote_networks_should_be_added() -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let network_contacts = store_dummy_network_contacts(&tmp_dir, 1)
            .await?
            .pop()
            .unwrap();
        let network_contacts_path = tmp_dir
            .path()
            .join(format!("{:?}", network_contacts.genesis_key()));
        let mut config = Config::create_config(&tmp_dir, None).await?;

        let network_1 = NetworkInfo::Remote(
            "https://safe-testnet-tool.s3.eu-west-2.amazonaws.com/sn_cli_resources/network_contacts"
                .to_string(),
            None,
        );
        let network_2 = NetworkInfo::Local(network_contacts_path, None);
        config.add_network("network_1", network_1).await?;
        config.add_network("network_2", network_2).await?;

        assert_eq!(config.settings.networks.len(), 2);
        config.compare_settings_and_network_contacts_dir().await?;

        Ok(())
    }

    #[tokio::test]
    async fn add_local_network_where_path_lies_inside_network_contacts_dir() -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let mut config = Config::create_config(&tmp_dir, None).await?;
        let network_contacts = config
            .store_dummy_network_contacts_and_set_default(1)
            .await?
            .pop()
            .unwrap();
        let path = config
            .network_contacts_dir
            .join(format!("{:?}", network_contacts.genesis_key()));

        let network_1 = NetworkInfo::Local(path, None);
        config.add_network("network_1", network_1).await?;

        assert_eq!(config.settings.networks.len(), 1);
        config.compare_settings_and_network_contacts_dir().await?;
        Ok(())
    }

    #[tokio::test]
    async fn removing_network_should_give_the_desirable_output() -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let network_contacts = store_dummy_network_contacts(&tmp_dir, 1)
            .await?
            .pop()
            .unwrap();
        let network_contacts_path = tmp_dir
            .path()
            .join(format!("{:?}", network_contacts.genesis_key()));
        let mut config = Config::create_config(&tmp_dir, None).await?;

        let network_1 = NetworkInfo::Local(network_contacts_path, None);
        config.add_network("network_1", network_1).await?;
        assert_eq!(config.settings.networks.len(), 1);

        config.remove_network("a_random_network").await?;
        assert_eq!(config.settings.networks.len(), 1);
        config.compare_settings_and_network_contacts_dir().await?;

        config.remove_network("network_1").await?;
        assert_eq!(config.settings.networks.len(), 0);
        config.compare_settings_and_network_contacts_dir().await?;

        Ok(())
    }

    #[tokio::test]
    async fn switching_network_should_change_the_default_network_contacts() -> Result<()> {
        let tmp_dir = assert_fs::TempDir::new()?;
        let mut network_contacts = store_dummy_network_contacts(&tmp_dir, 2).await?;
        let mut config = Config::create_config(&tmp_dir, None).await?;

        let network_contacts_1 = network_contacts.pop().unwrap();
        let network_1 = NetworkInfo::Local(
            tmp_dir
                .path()
                .join(format!("{:?}", network_contacts_1.genesis_key())),
            None,
        );
        let network_contacts_2 = network_contacts.pop().unwrap();
        let network_2 = NetworkInfo::Local(
            tmp_dir
                .path()
                .join(format!("{:?}", network_contacts_2.genesis_key())),
            None,
        );
        config.add_network("network_1", network_1).await?;
        config.add_network("network_2", network_2).await?;

        config.switch_to_network("network_1").await?;
        let (default, _) = config.read_default_network_contacts().await?;
        let net_info = config
            .settings
            .networks
            .get("network_1")
            .ok_or_else(|| eyre!("network_1 should be present"))?;
        assert_eq!(default, network_contacts_1);
        assert!(matches!(
            net_info,
            NetworkInfo::Local(_, Some(genesis_key)) if genesis_key == default.genesis_key()
        ));

        config.switch_to_network("network_2").await?;
        let (default, _) = config.read_default_network_contacts().await?;
        let net_info = config
            .settings
            .networks
            .get("network_2")
            .ok_or_else(|| eyre!("network_2 should be present"))?;
        assert_eq!(default, network_contacts_2);
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
        assert!(switch_result.is_err());
        let default = config.read_default_network_contacts().await;
        assert!(default.is_err());

        Ok(())
    }
}
