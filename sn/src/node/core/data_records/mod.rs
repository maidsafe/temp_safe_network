// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{Command, Core, Prefix};

use crate::{
    data_copy_count,
    messaging::{
        data::{CmdError, DataQuery, MetadataExchange, StorageLevel},
        system::{NodeCmd, NodeQuery, SystemMsg},
        AuthorityProof, EndUser, MessageId, ServiceAuth,
    },
    node::{core::MAX_WAITING_PEERS_PER_QUERY, error::convert_to_error_message, Error, Result},
    peer::Peer,
    types::{log_markers::LogMarker, PublicKey, ReplicatedData, ReplicatedDataAddress},
};

use itertools::Itertools;
use std::collections::BTreeSet;
use tracing::info;
use xor_name::XorName;

impl Core {
    // Locate ideal holders for this data, line up wiremsgs for those to instruct them to store the data
    pub(crate) async fn replicate_data(&self, data: ReplicatedData) -> Result<Vec<Command>> {
        trace!("{:?}: {:?}", LogMarker::DataStoreReceivedAtElder, data);
        if self.is_elder().await {
            let targets = self.get_adults_who_should_store_data(data.name()).await;

            info!(
                "Replicating data {:?} to holders {:?}",
                data.name(),
                &targets,
            );

            let msg = SystemMsg::NodeCmd(NodeCmd::ReplicateData(data));
            self.send_node_msg_to_nodes(msg, targets, false).await
        } else {
            Err(Error::InvalidState)
        }
    }

    pub(crate) async fn read_data_from_adults(
        &self,
        query: DataQuery,
        msg_id: MessageId,
        auth: AuthorityProof<ServiceAuth>,
        origin: Peer,
    ) -> Result<Vec<Command>> {
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
            return self
                .send_error(
                    Error::NoAdults(self.network_knowledge().prefix().await),
                    msg_id,
                    origin,
                )
                .await;
        }

        let mut fresh_targets = BTreeSet::new();
        for target in targets {
            self.liveness
                .add_a_pending_request_operation(target, operation_id)
                .await;
            let _existed = fresh_targets.insert(target);
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
            correlation_id: MessageId::from_xor_name(*address.name()),
        });
        let aggregation = false;

        self.send_node_msg_to_nodes(msg, fresh_targets, aggregation)
            .await
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

    /// Registered holders not present in provided list of members
    /// will be removed from adult_storage_info and no longer tracked for liveness.
    pub(crate) async fn liveness_retain_only(&self, members: BTreeSet<XorName>) -> Result<()> {
        // full adults
        self.capacity.retain_members_only(&members).await;

        // stop tracking liveness of absent holders
        self.liveness.retain_members_only(members);

        Ok(())
    }

    /// Adds the new adult to the Capacity and Liveness trackers.
    pub(crate) async fn add_new_adult_to_trackers(&self, adult: XorName) {
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

    pub(crate) async fn full_adults(&self) -> BTreeSet<XorName> {
        self.capacity.full_adults().await
    }

    pub(crate) async fn send_error(
        &self,
        error: Error,
        msg_id: MessageId,
        origin: Peer,
    ) -> Result<Vec<Command>> {
        let error = convert_to_error_message(error);
        let error = CmdError::Data(error);

        self.send_cmd_error_response(error, origin, msg_id).await
    }

    pub(crate) fn compute_holders(
        &self,
        addr: &ReplicatedDataAddress,
        adult_list: &BTreeSet<XorName>,
    ) -> BTreeSet<XorName> {
        adult_list
            .iter()
            .sorted_by(|lhs, rhs| addr.name().cmp_distance(lhs, rhs))
            .take(data_copy_count())
            .cloned()
            .collect()
    }
}
