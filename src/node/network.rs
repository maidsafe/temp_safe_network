// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::{node::NodeMsg, DstLocation, WireMsg};
use crate::node::{state_db::store_network_keypair, Config as NodeConfig, Error, Result};
use crate::routing::{
    Config as RoutingConfig, Error as RoutingError, EventStream, PeerUtils, Routing as RoutingNode,
    SectionAuthorityProviderUtils,
};
use crate::types::PublicKey;
use bls::PublicKeySet;
use ed25519_dalek::PublicKey as Ed25519PublicKey;
use secured_linked_list::SecuredLinkedList;
use std::{collections::BTreeSet, net::SocketAddr, path::Path, sync::Arc};
use xor_name::{Prefix, XorName};

///
#[derive(Clone, Debug)]
pub struct Network {
    routing: Arc<RoutingNode>,
}

#[allow(missing_docs)]
impl Network {
    pub async fn new(root_dir: &Path, config: &NodeConfig) -> Result<(Self, EventStream)> {
        let routing_config = RoutingConfig {
            first: config.is_first(),
            transport_config: config.network_config().clone(),
            keypair: None,
        };
        let (routing, event_stream) = RoutingNode::new(routing_config).await?;

        // Network keypair may have to be changed due to naming criteria or network requirements.
        store_network_keypair(root_dir, routing.keypair_as_bytes().await).await?;

        Ok((
            Self {
                routing: Arc::new(routing),
            },
            event_stream,
        ))
    }

    pub async fn age(&self) -> u8 {
        self.routing.age().await
    }

    pub async fn public_key(&self) -> Ed25519PublicKey {
        self.routing.public_key().await
    }

    pub async fn propose_offline(&self, name: XorName) -> Result<()> {
        // Notify the entire section to test connectivity to this node
        self.routing.start_connectivity_test(name).await?;
        self.routing
            .propose_offline(name)
            .await
            .map_err(|_| Error::NotAnElder)
    }

    pub async fn section_public_key(&self) -> Result<PublicKey> {
        Ok(PublicKey::Bls(
            self.routing
                .public_key_set()
                .await
                .map_err(|_| Error::NoSectionPublicKey)?
                .public_key(),
        ))
    }

    pub async fn our_public_key_set(&self) -> Result<PublicKeySet> {
        self.routing.public_key_set().await.map_err(Error::Routing)
    }

    pub async fn get_section_pk_by_name(&self, name: &XorName) -> Result<PublicKey> {
        self.routing
            .matching_section(name)
            .await
            .map(|provider| PublicKey::from(provider.section_key()))
            .map_err(From::from)
    }

    pub async fn our_name(&self) -> XorName {
        self.routing.name().await
    }

    pub async fn our_connection_info(&self) -> SocketAddr {
        self.routing.our_connection_info().await
    }

    pub async fn our_prefix(&self) -> Prefix {
        self.routing.our_prefix().await
    }

    pub async fn section_chain(&self) -> SecuredLinkedList {
        self.routing.section_chain().await
    }

    pub async fn send_message(&self, wire_msg: WireMsg) -> Result<(), RoutingError> {
        self.routing.send_message(wire_msg).await
    }

    pub async fn set_joins_allowed(&mut self, joins_allowed: bool) -> Result<()> {
        self.routing
            .set_joins_allowed(joins_allowed)
            .await
            .map_err(Error::Routing)
    }

    // Returns whether the node is Elder.
    pub async fn is_elder(&self) -> bool {
        self.routing.is_elder().await
    }

    pub async fn our_elder_names(&self) -> BTreeSet<XorName> {
        self.routing
            .our_elders()
            .await
            .iter()
            .map(|p2p_node| *p2p_node.name())
            .collect::<BTreeSet<_>>()
    }

    pub async fn our_adults(&self) -> BTreeSet<XorName> {
        self.routing
            .our_adults()
            .await
            .into_iter()
            .map(|p2p_node| *p2p_node.name())
            .collect::<BTreeSet<_>>()
    }

    pub async fn our_adults_sorted_by_distance_to(&self, name: &XorName) -> Vec<XorName> {
        self.routing
            .our_adults_sorted_by_distance_to(name)
            .await
            .into_iter()
            .map(|p2p_node| *p2p_node.name())
            .collect::<Vec<_>>()
    }

    pub async fn sign_msg_for_dst_accumulation(
        &self,
        node_msg: NodeMsg,
        dst: DstLocation,
    ) -> Result<WireMsg> {
        self.routing
            .sign_msg_for_dst_accumulation(node_msg, dst)
            .await
            .map_err(Error::Routing)
    }

    pub async fn sign_single_src_msg(
        &self,
        node_msg: NodeMsg,
        dst: DstLocation,
    ) -> Result<WireMsg> {
        self.routing
            .sign_single_src_msg(node_msg, dst)
            .await
            .map_err(Error::Routing)
    }
}
