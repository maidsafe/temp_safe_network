// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::dbs::UsedSpace;
use crate::messaging::{system::SystemMsg, DstLocation, WireMsg};
use crate::node::{state_db::store_network_keypair, Config as NodeConfig, Result};
use crate::routing::{
    Config as RoutingConfig, Error as RoutingError, EventStream, Routing as RoutingNode,
};
use crate::types::PublicKey;
use bls::PublicKey as BlsPublicKey;
use std::{net::SocketAddr, path::Path, sync::Arc};
use xor_name::{Prefix, XorName};

///
#[derive(Clone)]
pub(crate) struct Network {
    routing: Arc<RoutingNode>,
}

#[allow(missing_docs)]
impl Network {
    pub(crate) async fn new(
        root_dir: &Path,
        config: &NodeConfig,
        used_space: UsedSpace,
    ) -> Result<(Self, EventStream)> {
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

        let (routing, event_stream) =
            RoutingNode::new(routing_config, used_space, root_dir.to_path_buf()).await?;

        // Network keypair may have to be changed due to naming criteria or network requirements.
        store_network_keypair(root_dir, routing.keypair_as_bytes().await).await?;

        Ok((
            Self {
                routing: Arc::new(routing),
            },
            event_stream,
        ))
    }

    pub(crate) async fn age(&self) -> u8 {
        self.routing.age().await
    }

    /// Returns our section's public key.
    pub(crate) async fn our_section_public_key(&self) -> BlsPublicKey {
        self.routing.our_section_auth().await.section_key()
    }

    pub(crate) async fn get_section_pk_by_name(&self, name: &XorName) -> Result<PublicKey> {
        self.routing
            .matching_section(name)
            .await
            .map(|provider| PublicKey::from(provider.section_key()))
            .map_err(From::from)
    }

    pub(crate) async fn our_name(&self) -> XorName {
        self.routing.name().await
    }

    pub(crate) async fn our_connection_info(&self) -> SocketAddr {
        self.routing.our_connection_info().await
    }

    pub(crate) async fn our_prefix(&self) -> Prefix {
        self.routing.our_prefix().await
    }

    pub(crate) async fn genesis_key(&self) -> BlsPublicKey {
        self.routing.genesis_key().await
    }

    pub(crate) async fn send_message(&self, wire_msg: WireMsg) -> Result<(), RoutingError> {
        self.routing.send_message(wire_msg).await
    }

    pub(crate) async fn set_joins_allowed(&mut self, joins_allowed: bool) -> Result<()> {
        self.routing.set_joins_allowed(joins_allowed).await?;
        Ok(())
    }

    pub(crate) async fn sign_msg_for_dst_accumulation(
        &self,
        node_msg: SystemMsg,
        dst: DstLocation,
    ) -> Result<WireMsg> {
        let wire_msg = self
            .routing
            .sign_msg_for_dst_accumulation(node_msg, dst)
            .await?;
        Ok(wire_msg)
    }

    pub(crate) async fn sign_single_src_msg(
        &self,
        node_msg: SystemMsg,
        dst: DstLocation,
    ) -> Result<WireMsg> {
        let wire_msg = self.routing.sign_single_src_msg(node_msg, dst).await?;
        Ok(wire_msg)
    }
}
