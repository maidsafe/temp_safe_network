// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod capacity;
mod liveness_tracking;

pub(crate) use self::capacity::{Capacity, MIN_LEVEL_WHEN_FULL};
pub(crate) use self::liveness_tracking::Liveness;

use crate::data_copy_count;
use crate::messaging::{
    data::{CmdError, DataQuery, MetadataExchange, OperationId, ServiceMsg, StorageLevel},
    system::{NodeCmd, NodeQuery, SystemMsg},
    AuthorityProof, DstLocation, EndUser, MsgId, MsgKind, ServiceAuth, WireMsg,
};
use crate::node::{
    core::{
        messaging::MsgSender, Cmd, Prefix, DATA_QUERY_LIMIT, DATA_QUERY_TIMEOUT,
        MAX_WAITING_PEERS_PER_QUERY,
    },
    error::convert_to_error_msg,
    messages::WireMsgUtils,
    network_knowledge::NetworkKnowledge,
    Error, NodeInfo, Result,
};
use crate::types::{log_markers::LogMarker, Cache, Peer, PublicKey, ReplicatedData};

use bytes::Bytes;
use itertools::Itertools;
use std::sync::Arc;
use std::{cmp::Ordering, collections::BTreeSet};
use tokio::sync::RwLock;
use tracing::info;
use xor_name::XorName;

/// Elders only
pub(crate) struct DataRecords {
    liveness: Liveness,
    capacity: Capacity,
    pending_data_queries: Arc<Cache<OperationId, Vec<Peer>>>,
    network_knowledge: NetworkKnowledge,
    our_info: Arc<RwLock<NodeInfo>>,
    sender: MsgSender,
}

impl DataRecords {
    pub(crate) async fn new(
        network_knowledge: NetworkKnowledge,
        our_info: Arc<RwLock<NodeInfo>>,
    ) -> Self {
        let liveness = Liveness::new(
            network_knowledge
                .adults()
                .await
                .iter()
                .map(|peer| peer.name())
                .collect::<Vec<XorName>>(),
        );

        let sender = MsgSender::new(network_knowledge.clone(), our_info.clone()).await;

        Self {
            liveness,
            capacity: Capacity::default(),
            pending_data_queries: Arc::new(Cache::with_expiry_duration(DATA_QUERY_TIMEOUT)),
            network_knowledge,
            our_info,
            sender,
        }
    }

    /// see if a client is waiting in the pending queries for a response
    /// If not, we could remove its link eg.
    pub(crate) async fn pending_data_queries_contains_client(&self, peer: &Peer) -> bool {
        // now we check if our peer is still waiting...
        for (_op_id, peer_vec) in self.pending_data_queries.get_items().await {
            if peer_vec.object.contains(peer) {
                return true;
            }
        }

        false
    }

    pub(crate) async fn remove_pending_query(&self, op_id: &OperationId) -> Option<Vec<Peer>> {
        self.pending_data_queries.remove(op_id).await
    }

    pub(crate) async fn remove_expired_queries(&self) {
        self.pending_data_queries.remove_expired().await
    }

    pub(crate) async fn pending_queries_exceeded(&self) -> bool {
        let pending_query_length = self.pending_data_queries.len().await;
        pending_query_length > DATA_QUERY_LIMIT
    }

    pub(crate) async fn remove_op_id(&self, node_id: &XorName, op_id: &OperationId) -> bool {
        self.liveness.remove_op_id(node_id, op_id).await
    }

    // Locate ideal holders for this data, line up wiremsgs for those to instruct them to store the data
    pub(crate) async fn replicate(&self, data: ReplicatedData) -> Result<Vec<Cmd>> {
        trace!("{:?}: {:?}", LogMarker::DataStoreReceivedAtElder, data);
        if self.is_elder().await {
            let targets = self.get_adults_who_should_store_data(data.name()).await;

            info!(
                "Replicating data {:?} to holders {:?}",
                data.name(),
                &targets,
            );

            let msg = SystemMsg::NodeCmd(NodeCmd::ReplicateData(data));
            self.send_to_nodes(msg, targets).await
        } else {
            Err(Error::InvalidState)
        }
    }

    pub(crate) async fn read_data_from_adults(
        &self,
        query: DataQuery,
        msg_id: MsgId,
        auth: AuthorityProof<ServiceAuth>,
        origin: Peer,
    ) -> Result<Vec<Cmd>> {
        let address = query.address();
        let operation_id = query.operation_id()?;
        trace!(
            "{:?} preparing to query adults for data at {:?} with op_id: {:?}",
            LogMarker::DataQueryReceviedAtElder,
            address,
            operation_id
        );

        let targets = self.get_adults_holding_data(address.name()).await;

        if targets.is_empty() {
            let error =
                convert_to_error_msg(Error::NoAdults(self.network_knowledge.prefix().await));

            return self
                .sender
                .send_cmd_error_response(CmdError::Data(error), origin, msg_id)
                .await;
        }

        let mut already_waiting_on_response = false;
        let mut this_peer_already_waiting_on_response = false;
        let waiting_peers = if let Some(peers) = self.pending_data_queries.get(&operation_id).await
        {
            already_waiting_on_response = true;
            this_peer_already_waiting_on_response = peers.contains(&origin.clone());
            peers
        } else {
            vec![origin.clone()]
        };

        if this_peer_already_waiting_on_response {
            // no need to add to pending queue then
            return Ok(vec![]);
        }

        // drop if we exceed
        if waiting_peers.len() > MAX_WAITING_PEERS_PER_QUERY {
            warn!("Dropping query from {origin:?}, there are more than {MAX_WAITING_PEERS_PER_QUERY} waiting already");
            return Ok(vec![]);
        }

        // ensure we only add a pending request when we're actually sending out requests.
        for target in &targets {
            self.liveness
                .add_a_pending_request_operation(*target, operation_id)
                .await;
        }

        let _prior_value = self
            .pending_data_queries
            .set(operation_id, waiting_peers, None)
            .await;

        if already_waiting_on_response {
            // no need to send query again.
            return Ok(vec![]);
        }

        let msg = SystemMsg::NodeQuery(NodeQuery::Data {
            query,
            auth: auth.into_inner(),
            origin: EndUser(origin.name()),
            correlation_id: MsgId::from_xor_name(*address.name()),
        });

        self.send_to_nodes(msg, targets).await
    }

    pub(crate) async fn get_metadata_of(&self, prefix: &Prefix) -> MetadataExchange {
        // Load tracked adult_levels
        let adult_levels = self.capacity.levels_matching(*prefix).await;
        MetadataExchange { adult_levels }
    }

    pub(crate) async fn set_adult_levels(&self, adult_levels: MetadataExchange) {
        let MetadataExchange { adult_levels } = adult_levels;
        self.capacity.set_adult_levels(adult_levels).await
    }

    /// Registered holders not present in current list of adults,
    /// will be removed from capacity and liveness tracking.
    pub(crate) async fn update_member_tracking(&self) -> Result<()> {
        // current list of adults
        let members = self
            .network_knowledge
            .adults()
            .await
            .iter()
            .map(|peer| peer.name())
            .collect();

        // full adults
        self.capacity.retain_members_only(&members).await;

        // stop tracking liveness of absent holders
        self.liveness.retain_members_only(members);

        Ok(())
    }

    /// Adds an adult to the Capacity and Liveness trackers.
    pub(crate) async fn track_adult(&self, adult: XorName) {
        info!("Adding new Adult: {adult} to trackers");
        self.capacity.add_new_adult(adult).await;
        self.liveness.add_new_adult(adult);
    }

    /// Set storage level of a given node.
    /// Returns whether the level changed or not.
    pub(crate) async fn set_storage_level(&self, node_id: &PublicKey, level: StorageLevel) -> bool {
        info!("Setting new storage level..");
        let changed = self
            .capacity
            .set_adult_level(XorName::from(*node_id), level)
            .await;
        let avg_usage = self.capacity.avg_usage().await;
        info!(
            "Avg storage usage among Adults is between {}-{} %",
            avg_usage * 10,
            (avg_usage + 1) * 10
        );
        changed
    }

    pub(crate) async fn find_unresponsive_and_deviant_nodes(
        &self,
    ) -> (Vec<(XorName, usize)>, BTreeSet<XorName>) {
        self.liveness.find_unresponsive_and_deviant_nodes().await
    }

    pub(crate) async fn is_full(&self, adult: &XorName) -> Option<bool> {
        self.capacity.is_full(adult).await
    }

    async fn is_elder(&self) -> bool {
        self.network_knowledge
            .is_elder(&self.our_info.read().await.name())
            .await
    }

    async fn full_adults(&self) -> BTreeSet<XorName> {
        self.capacity.full_adults().await
    }

    // Used to fetch the list of holders for given data name.
    async fn get_adults_holding_data(&self, target: &XorName) -> BTreeSet<XorName> {
        let full_adults = self.full_adults().await;
        // TODO: reuse our_adults_sorted_by_distance_to API when core is merged into upper layer
        let adults = self.network_knowledge.adults().await;

        let adults_names = adults.iter().map(|p2p_node| p2p_node.name());

        let mut candidates = adults_names
            .into_iter()
            .sorted_by(|lhs, rhs| target.cmp_distance(lhs, rhs))
            .filter(|peer| !full_adults.contains(peer))
            .take(data_copy_count())
            .collect::<BTreeSet<_>>();

        trace!(
            "Chunk holders of {:?} are empty adults: {:?} and full adults: {:?}",
            target,
            candidates,
            full_adults
        );

        // Full adults that are close to the chunk, shall still be considered as candidates
        // to allow chunks stored to empty adults can be queried when nodes become full.
        let close_full_adults = if let Some(closest_empty) = candidates.iter().next() {
            full_adults
                .iter()
                .filter_map(|name| {
                    if target.cmp_distance(name, closest_empty) == Ordering::Less {
                        Some(*name)
                    } else {
                        None
                    }
                })
                .collect::<BTreeSet<_>>()
        } else {
            // In case there is no empty candidates, query all full_adults
            full_adults
        };

        candidates.extend(close_full_adults);
        candidates
    }

    // Used to fetch the list of holders for given name of data.
    async fn get_adults_who_should_store_data(&self, target: XorName) -> BTreeSet<XorName> {
        let full_adults = self.full_adults().await;
        // TODO: reuse our_adults_sorted_by_distance_to API when core is merged into upper layer
        let adults = self.network_knowledge.adults().await;

        trace!("Total adults known about: {:?}", adults.len());

        let adults_names = adults.iter().map(|p2p_node| p2p_node.name());

        let candidates = adults_names
            .into_iter()
            .sorted_by(|lhs, rhs| target.cmp_distance(lhs, rhs))
            .filter(|peer| !full_adults.contains(peer))
            .take(data_copy_count())
            .collect::<BTreeSet<_>>();

        trace!(
               "Target chunk holders of {:?} are empty adults: {:?} and full adults that were ignored: {:?}",
               target,
               candidates,
               full_adults
           );

        candidates
    }

    // Takes a message for specified targets, and builds internal send cmds
    // for sending to each of the targets.
    // Targets are XorName specified so must be within the section
    async fn send_to_nodes(&self, msg: SystemMsg, targets: BTreeSet<XorName>) -> Result<Vec<Cmd>> {
        // we create a dummy/random dst location,
        // we will set it correctly for each msg and target
        let section_pk = self.network_knowledge.section_key().await;
        let our_name = self.our_info.read().await.name();
        let dummy_dst_location = DstLocation::Node {
            name: our_name,
            section_pk,
        };

        // separate this into form_wire_msg based on agg
        let wire_msg = WireMsg::single_src(
            &self.our_info.read().await.clone(),
            dummy_dst_location,
            msg,
            section_pk,
        )?;

        let mut cmds = vec![];

        for target in targets {
            debug!("Sending {:?} to {:?}", wire_msg, target);
            let mut wire_msg = wire_msg.clone();
            let dst_section_pk = self.network_knowledge.section_key_by_name(&target).await;
            wire_msg.set_dst_section_pk(dst_section_pk);
            wire_msg.set_dst_xorname(target);

            cmds.extend(self.sender.send_msg_to_nodes(wire_msg).await?);
        }

        Ok(cmds)
    }

    pub(crate) async fn ed_sign_service_msg(&self, msg: &ServiceMsg) -> Result<(MsgKind, Bytes)> {
        self.sender.ed_sign_service_msg(msg).await
    }
}
