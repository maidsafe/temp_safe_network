// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::client::{Error, Result};
use qp2p::Config as QuicP2pConfig;
use serde::{Deserialize, Serialize};
use std::{
    net::{Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::{
    fs::File,
    io::{self, AsyncReadExt},
};
use tracing::{debug, warn};

const DEFAULT_LOCAL_ADDR: (Ipv4Addr, u16) = (Ipv4Addr::UNSPECIFIED, 0);

/// Defaul amount of time to wait for responses to queries before giving up and returning an error.
pub const DEFAULT_QUERY_TIMEOUT: Duration = Duration::from_secs(90);

const DEFAULT_ROOT_DIR_NAME: &str = "root_dir";

/// Configuration for sn_client.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    /// The local address to bind to.
    pub local_addr: SocketAddr,
    /// Path to local storage.
    pub root_dir: PathBuf,
    /// Network's genesis key
    pub genesis_key: bls::PublicKey,
    /// QuicP2p options.
    pub qp2p: QuicP2pConfig,
    /// The amount of time to wait for responses to queries before giving up and returning an error.
    pub query_timeout: Duration,
}

impl Config {
    /// Returns a new `Config` instance.
    ///
    /// This will try to read QuicP2P configuration from `config_file_path`, or else use the default
    /// QuicP2P config. In either case, `bootstrap_nodes` will be used to override the initial
    /// network contacts.
    ///
    /// If `local_addr` is not specified, `127.0.0.1:0` will be used (e.g. localhost with a random
    /// port).
    ///
    /// If `query_timeout` is not specified, [`DEFAULT_QUERY_TIMEOUT`] will be used.
    pub async fn new(
        root_dir: Option<&Path>,
        local_addr: Option<SocketAddr>,
        genesis_key: bls::PublicKey,
        config_file_path: Option<&Path>,
        query_timeout: Option<Duration>,
    ) -> Self {
        let root_dir = root_dir
            .map(|p| p.to_path_buf())
            .unwrap_or_else(default_dir);
        // If a config file path was provided we try to read it,
        // otherwise we use default qp2p config.
        let mut qp2p = match &config_file_path {
            None => QuicP2pConfig::default(),
            Some(path) => read_config_file(path).await.unwrap_or_default(),
        };

        qp2p.idle_timeout = Some(Duration::from_secs(5));
        qp2p.keep_alive_interval = Some(Duration::from_secs(1));

        Self {
            local_addr: local_addr.unwrap_or_else(|| SocketAddr::from(DEFAULT_LOCAL_ADDR)),
            root_dir: root_dir.clone(),
            genesis_key,
            qp2p,
            query_timeout: query_timeout.unwrap_or(DEFAULT_QUERY_TIMEOUT),
        }
    }
}

async fn read_config_file(filepath: &Path) -> Result<QuicP2pConfig, Error> {
    debug!("Reading config file '{}' ...", filepath.display());
    let mut file = File::open(filepath).await?;

    let mut contents = vec![];
    let _ = file.read_to_end(&mut contents).await?;

    serde_json::from_slice(&contents).map_err(|err| {
        warn!(
            "Could not parse content of config file '{}': {}",
            filepath.display(),
            err
        );
        err.into()
    })
}

/// Root directory for dbs and cached state. If not set, it defaults to
/// `DEFAULT_ROOT_DIR_NAME` within the project's data directory (see `Config::root_dir` for the
/// directories on each platform).
fn default_dir() -> PathBuf {
    project_dirs()
        .unwrap_or_default()
        .join(DEFAULT_ROOT_DIR_NAME)
}

fn project_dirs() -> Result<PathBuf> {
    let mut home_dir = dirs_next::home_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Home directory not found"))?;

    home_dir.push(".safe");
    home_dir.push("client");

    Ok(home_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::utils::test_utils::init_logger;
    use bincode::serialize;
    use eyre::Result;
    use rand::{distributions::Alphanumeric, thread_rng, Rng};
    use std::fs::File;
    use tempfile::tempdir;
    use tokio::fs::create_dir_all;

    // 1. Verify that `Config::new()` generates the correct default config
    //    when the file is not found. The default config shall have the provided
    //    config path in the `boostrap_cache_dir` field.
    // 2. Write the default config file to temp directory.
    // 3. Assert that `Config::new()` reads the default config written to disk.
    // 4. Verify that `Config::new()` returns the correct default config when no path is provided.
    #[tokio::test(flavor = "multi_thread")]
    async fn custom_config_path() -> Result<()> {
        init_logger();

        let temp_dir = tempdir().map_err(|e| Error::Generic(e.to_string()))?;
        let root_dir = temp_dir.path().to_path_buf();
        let cfg_filename: String = thread_rng().sample_iter(&Alphanumeric).take(15).collect();
        let config_filepath = root_dir.join(&cfg_filename);
        let genesis_key = bls::SecretKey::random().public_key();

        // In the absence of a config file, the config handler
        // should initialize bootstrap_cache_dir only
        let config = Config::new(
            Some(&root_dir),
            None,
            genesis_key,
            Some(&config_filepath),
            None,
        )
        .await;
        // convert to string for assert
        let mut str_path = root_dir
            .to_str()
            .ok_or(eyre::eyre!("No path for to_str".to_string()))?
            .to_string();
        // normalise for mac
        if str_path.ends_with('/') {
            let _ = str_path.pop();
        }

        let expected_config = Config {
            local_addr: (Ipv4Addr::UNSPECIFIED, 0).into(),
            root_dir: root_dir.clone(),
            genesis_key,
            qp2p: QuicP2pConfig::default(),
            query_timeout: DEFAULT_QUERY_TIMEOUT,
        };
        assert_eq!(serialize(&config)?, serialize(&expected_config)?);

        create_dir_all(&root_dir).await?;
        let mut file = File::create(&config_filepath)?;

        let config_on_disk =
            Config::new(None, None, genesis_key, Some(&config_filepath), None).await;
        serde_json::to_writer_pretty(&mut file, &config_on_disk)?;
        file.sync_all()?;

        let read_cfg = Config::new(None, None, genesis_key, None, None).await;
        assert_eq!(serialize(&config_on_disk)?, serialize(&read_cfg)?);

        Ok(())
    }
}
