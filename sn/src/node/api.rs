// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{
    keypair_storage::store_network_keypair,
    keypair_storage::{get_reward_pk, store_new_reward_keypair},
    logging::{log_ctx::LogCtx, run_system_logger},
    routing::{Config as RoutingConfig, EventStream, Routing},
    Config as NodeConfig, Error, Result,
};
use crate::types::PublicKey;
use crate::UsedSpace;

use rand::rngs::OsRng;
use std::{net::SocketAddr, sync::Arc};
use tokio::time::Duration;
use xor_name::{Prefix, XorName};

type Network = Arc<Routing>;

/// Main node struct.
#[derive(custom_debug::Debug)]
pub struct Node {
    #[debug(skip)]
    network_api: Network,
}

impl Node {
    /// Initialize a new node.
    pub async fn new(
        config: &NodeConfig,
        joining_timeout: Duration,
    ) -> Result<(Self, EventStream)> {
        let root_dir_buf = config.root_dir()?;
        let root_dir = root_dir_buf.as_path();
        tokio::fs::create_dir_all(root_dir).await?;

        let _reward_key = match get_reward_pk(root_dir).await? {
            Some(public_key) => PublicKey::Ed25519(public_key),
            None => {
                let mut rng = OsRng;
                let keypair = ed25519_dalek::Keypair::generate(&mut rng);
                store_new_reward_keypair(root_dir, &keypair).await?;
                PublicKey::Ed25519(keypair.public)
            }
        };

        let joining_timeout = if cfg!(feature = "always-joinable") {
            debug!(
                "Feature \"always-joinable\" is set. Running with join timeout: {:?}",
                joining_timeout * 10
            );
            // arbitrarily long time, the join process should just loop w/ backoff until then
            joining_timeout * 10
        } else {
            joining_timeout
        };

        let mut routing_config = RoutingConfig {
            first: config.is_first(),
            bootstrap_nodes: config.hard_coded_contacts.clone(),
            genesis_key: config.genesis_key.clone(),
            network_config: config.network_config().clone(),
            ..Default::default()
        };
        if let Some(local_addr) = config.local_addr {
            routing_config.local_addr = local_addr;
        }

        let used_space = UsedSpace::new(config.max_capacity());

        let (routing, network_events) = tokio::time::timeout(
            joining_timeout,
            Routing::new(routing_config, used_space, root_dir.to_path_buf()),
        )
        .await
        .map_err(|_| Error::JoinTimeout)??;

        // Network keypair may have to be changed due to naming criteria or network requirements.
        store_network_keypair(root_dir, routing.keypair_as_bytes().await).await?;

        let network_api = Arc::new(routing);

        let node = Self {
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
        self.network_api.name().await
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
