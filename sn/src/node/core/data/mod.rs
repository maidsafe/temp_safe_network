// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod records;
mod storage;

pub(crate) use self::records::MIN_LEVEL_WHEN_FULL;

use super::Result;

use crate::dbs::{Error as DbError, Result as DbResult};
use crate::messaging::{
    data::{DataQuery, MetadataExchange, OperationId, ServiceMsg, StorageLevel},
    system::NodeQueryResponse,
    AuthorityProof, DstLocation, EndUser, MsgId, ServiceAuth, WireMsg,
};
use crate::node::{api::cmds::Cmd, network_knowledge::NetworkKnowledge, NodeInfo};
use crate::types::{
    register::User, Peer, PublicKey, ReplicatedData, ReplicatedDataAddress as DataAddress,
};
use crate::UsedSpace;

use records::DataRecords;
use std::{collections::BTreeSet, path::Path, sync::Arc};
use storage::DataStorage;
use tokio::sync::RwLock;
use xor_name::{Prefix, XorName};

pub(super) struct Data {
    records: DataRecords,
    storage: DataStorage,
}

impl Data {
    pub(crate) async fn new(
        path: &Path,
        used_space: UsedSpace,
        network_knowledge: NetworkKnowledge,
        our_info: Arc<RwLock<NodeInfo>>,
    ) -> Result<Self> {
        let storage = DataStorage::new(path, used_space)?;
        let records = DataRecords::new(network_knowledge, our_info).await;
        Ok(Self { records, storage })
    }

    pub(crate) async fn pending_queries_exceeded(&self) -> bool {
        self.records.pending_queries_exceeded().await
    }

    pub(crate) async fn replicate(&self, data: ReplicatedData) -> Result<Vec<Cmd>> {
        // first try to cache the data
        let cache_full = match self.storage.store(&data).await {
            Ok(None) => false,
            Ok(Some(level)) => {
                info!("Elder storage level: {:?}", level);
                level.value() >= MIN_LEVEL_WHEN_FULL // db nearing full
            }
            Err(error) => {
                match error {
                    DbError::NotEnoughSpace => true, // db full
                    _ => {
                        error!("Problem caching data at Elder, but it was ignored: {error}");
                        false
                    }
                }
            }
        };

        // if the cache is full, we start dropping data
        if cache_full {
            info!("Elder cache limit exceeded. Dropping data..");
            // TODO: drop data..
        }

        // then send to adults
        self.records.replicate(data).await
    }

    /// Registered holders not present in current list of adults,
    /// will be removed from capacity and liveness tracking.
    pub(crate) async fn update_member_tracking(&self) -> Result<()> {
        self.records.update_member_tracking().await
    }

    pub(crate) async fn set_adult_levels(&self, metadata: MetadataExchange) {
        self.records.set_adult_levels(metadata).await
    }

    /// Set storage level of a given node.
    /// Returns whether the level changed or not.
    pub(crate) async fn set_storage_level(&self, node_id: &PublicKey, level: StorageLevel) -> bool {
        self.records.set_storage_level(node_id, level).await
    }

    pub(crate) async fn get_metadata_of(&self, prefix: &Prefix) -> MetadataExchange {
        self.records.get_metadata_of(prefix).await
    }

    /// Starts tracking capacity and liveness of an adult.
    pub(crate) async fn track_adult(&self, name: XorName) {
        self.records.track_adult(name).await
    }

    pub(crate) async fn pending_data_queries_contains_client(&self, peer: &Peer) -> bool {
        self.records
            .pending_data_queries_contains_client(peer)
            .await
    }

    pub(crate) async fn remove_pending_query(&self, op_id: &OperationId) -> Option<Vec<Peer>> {
        self.records.remove_pending_query(op_id).await
    }

    pub(crate) async fn remove_expired_queries(&self) {
        self.records.remove_expired_queries().await
    }

    pub(crate) async fn remove_op_id(&self, node_id: &XorName, op_id: &OperationId) -> bool {
        self.records.remove_op_id(node_id, op_id).await
    }

    pub(crate) async fn find_unresponsive_and_deviant_nodes(
        &self,
    ) -> (Vec<(XorName, usize)>, BTreeSet<XorName>) {
        self.records.find_unresponsive_and_deviant_nodes().await
    }

    pub(crate) async fn is_full(&self, adult: &XorName) -> Option<bool> {
        self.records.is_full(adult).await
    }

    pub(crate) async fn read_data_from_adults(
        &self,
        query: DataQuery,
        msg_id: MsgId,
        auth: AuthorityProof<ServiceAuth>,
        origin: Peer,
    ) -> Result<Vec<Cmd>> {
        // first check cache:
        let response = self
            .query(&query, User::Key(auth.public_key))
            .await
            .convert();

        if response.is_success() {
            // on cache hit, return it to client
            let (msg_kind, payload) = self
                .records
                .ed_sign_service_msg(&ServiceMsg::QueryResponse {
                    response,
                    correlation_id: msg_id,
                })
                .await?;

            let dst = DstLocation::EndUser(EndUser(origin.name()));
            let wire_msg = WireMsg::new_msg(msg_id, payload, msg_kind, dst)?;

            trace!(
                "Data query cache hit at Elder for query {:?}, sending response to {:?}",
                query,
                dst
            );

            Ok(vec![Cmd::SendMsg {
                recipients: vec![origin],
                wire_msg,
            }])
        } else {
            // else send query to adults
            self.records
                .read_data_from_adults(query, msg_id, auth, origin)
                .await
        }
    }

    /// =====================================================================
    /// =========================== STORAGE =================================
    /// =====================================================================

    #[instrument(skip_all)]
    pub(crate) async fn store(&self, data: &ReplicatedData) -> DbResult<Option<StorageLevel>> {
        self.storage.store(data).await
    }

    pub(crate) async fn get_for_replication(
        &self,
        address: &DataAddress,
    ) -> DbResult<ReplicatedData> {
        self.storage.get_for_replication(address).await
    }

    #[allow(dead_code)]
    pub(crate) async fn remove(&self, address: &DataAddress) -> DbResult<()> {
        self.storage.remove(address).await
    }

    pub(crate) async fn keys(&self) -> DbResult<Vec<DataAddress>> {
        self.storage.keys().await
    }

    pub(crate) async fn query(&self, query: &DataQuery, requester: User) -> NodeQueryResponse {
        self.storage.query(query, requester).await
    }
}
