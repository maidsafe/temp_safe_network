// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::dbs::UsedSpace;

use crate::node::logging::log_ctx::LogCtx;
use crate::node::logging::run_system_logger;
use crate::node::{
    network::Network,
    state_db::{get_reward_pk, store_new_reward_keypair},
    Config, Error, Result,
};
use crate::routing::{
    EventStream, {Prefix, XorName},
};
use crate::types::PublicKey;
use rand::rngs::OsRng;
use std::{net::SocketAddr, path::PathBuf};
use tokio::time::Duration;

/// Main node struct.
#[derive(custom_debug::Debug)]
pub struct Node {
    #[debug(skip)]
    network_api: Network,
    used_space: UsedSpace,
}

impl Node {
    /// Initialize a new node.
    pub async fn new(config: &Config, joining_timeout: Duration) -> Result<(Self, EventStream)> {
        let root_dir_buf = config.root_dir()?;
        let root_dir = root_dir_buf.as_path();
        tokio::fs::create_dir_all(root_dir).await?;

        let reward_key = match get_reward_pk(root_dir).await? {
            Some(public_key) => PublicKey::Ed25519(public_key),
            None => {
                let mut rng = OsRng;
                let keypair = ed25519_dalek::Keypair::generate(&mut rng);
                store_new_reward_keypair(root_dir, &keypair).await?;
                PublicKey::Ed25519(keypair.public)
            }
        };

        let used_space = UsedSpace::new(config.max_capacity());

        let joining_timeout = if cfg!(feature = "always-joinable") {
            debug!(
                ">>>Feature \"always-joinable\" is set. Running with join timeout: {:?}",
                joining_timeout * 10
            );
            // arbitrarily long time, the join process should just loop w/ backoff until then
            joining_timeout * 10
        } else {
            joining_timeout
        };

        let (network_api, network_events) = tokio::time::timeout(
            joining_timeout,
            Network::new(root_dir_buf.as_path(), config, used_space.clone()),
        )
        .await
        .map_err(|_| Error::JoinTimeout)??;

        let node = Self {
            used_space,
            network_api: network_api.clone(),
        };

        let our_pid = std::process::id();
        let node_prefix = node.our_prefix().await;
        let node_name = node.our_name().await;
        let node_age = node.our_age().await;
        let our_conn_info = node.our_connection_info().await;
        let our_conn_info_json = serde_json::to_string(&our_conn_info)
            .unwrap_or_else(|_| "Failed to serialize connection info".into());
        println!(
            "Node PID: {:?}, prefix: {:?}, name: {:?}, age: {}, connection info:\n{}",
            our_pid, node_prefix, node_name, node_age, our_conn_info_json,
        );
        info!(
            "Node PID: {:?}, prefix: {:?}, name: {:?}, age: {}, connection info: {}",
            our_pid, node_prefix, node_name, node_age, our_conn_info_json,
        );

        run_system_logger(LogCtx::new(network_api), config.resource_logs).await;

        Ok((node, network_events))
    }

    /// Returns our connection info.
    pub async fn our_connection_info(&self) -> SocketAddr {
        self.network_api.our_connection_info().await
    }

    /// Returns our name.
    pub async fn our_name(&self) -> XorName {
        self.network_api.our_name().await
    }

    /// Returns our age.
    pub async fn our_age(&self) -> u8 {
        self.network_api.age().await
    }

    /// Returns our prefix.
    pub async fn our_prefix(&self) -> Prefix {
        self.network_api.our_prefix().await
    }

    /// Returns the network's genesis key.
    pub async fn genesis_key(&self) -> bls::PublicKey {
        self.network_api.genesis_key().await
    }
}
