// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::client::Error;
use qp2p::Config as QuicP2pConfig;
use serde::{Deserialize, Serialize};
use std::{
    //collections::BTreeSet,
    net::{Ipv4Addr, SocketAddr},
    path::Path,
    time::Duration,
};
use tokio::{fs::File, io::AsyncReadExt};
use tracing::{debug, warn};

const DEFAULT_LOCAL_ADDR: (Ipv4Addr, u16) = (Ipv4Addr::LOCALHOST, 0);

/// Defaul amount of time to wait for responses to queries before giving up and returning an error.
pub const DEFAULT_QUERY_TIMEOUT: Duration = Duration::from_secs(90);

/// Configuration for sn_client.
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct Config {
    /// The local address to bind to.
    pub local_addr: SocketAddr,
    // /// Initial network contacts.
    // pub bootstrap_nodes: BTreeSet<SocketAddr>,
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
        local_addr: Option<SocketAddr>,
        // bootstrap_nodes: Option<BTreeSet<SocketAddr>>,
        config_file_path: Option<&Path>,
        query_timeout: Option<Duration>,
    ) -> Self {
        // If a config file path was provided we try to read it,
        // otherwise we use default qp2p config.
        let qp2p = match &config_file_path {
            None => QuicP2pConfig::default(),
            Some(path) => read_config_file(path).await.unwrap_or_default(),
        };

        Self {
            local_addr: local_addr.unwrap_or_else(|| SocketAddr::from(DEFAULT_LOCAL_ADDR)),
            // bootstrap_nodes: bootstrap_nodes.unwrap_or_default(),
            qp2p,
            query_timeout: query_timeout.unwrap_or(DEFAULT_QUERY_TIMEOUT),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            local_addr: SocketAddr::from(DEFAULT_LOCAL_ADDR),
            // bootstrap_nodes: Default::default(),
            qp2p: Default::default(),
            query_timeout: Default::default(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::utils::test_utils::init_logger;
    use eyre::Result;
    use rand::{distributions::Alphanumeric, thread_rng, Rng};
    use std::env::temp_dir;
    use std::fs::File;
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

        let path = temp_dir();
        let random_filename: String = thread_rng().sample_iter(&Alphanumeric).take(15).collect();
        let config_filepath = path.join(random_filename);

        // In the absence of a config file, the config handler
        // should initialize bootstrap_cache_dir only
        let config = Config::new(None, Some(&config_filepath), None).await;
        // convert to string for assert
        let mut str_path = path
            .to_str()
            .ok_or(eyre::eyre!("No path for to_str".to_string()))?
            .to_string();
        // normalise for mac
        if str_path.ends_with('/') {
            let _ = str_path.pop();
        }

        let expected_config = Config {
            local_addr: (Ipv4Addr::LOCALHOST, 0).into(),
            // bootstrap_nodes: BTreeSet::new(),
            qp2p: QuicP2pConfig::default(),
            query_timeout: DEFAULT_QUERY_TIMEOUT,
        };
        assert_eq!(config, expected_config);

        create_dir_all(path).await?;
        let mut file = File::create(&config_filepath)?;

        let config_on_disk = Config::default();
        serde_json::to_writer_pretty(&mut file, &config_on_disk)?;
        file.sync_all()?;

        let read_cfg = Config::new(None, Some(&config_filepath), None).await;
        assert_eq!(config_on_disk, read_cfg);

        let default_cfg = Config::new(None, None, None).await;
        assert_eq!(Config::default(), default_cfg);

        Ok(())
    }
}
