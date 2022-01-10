// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod chunk_storage;
//mod errors;
mod register_storage;

use super::{Command, Core};

use crate::{
    dbs::Result,
    messaging::{
        data::{DataQuery, StorageLevel},
        system::{NodeCmd, NodeQueryResponse, SystemMsg},
    },
    types::{ReplicatedData, ReplicatedDataAddress as DataAddress},
    UsedSpace,
};

pub(crate) use chunk_storage::ChunkStorage;
//pub(crate) use errors::{Error as DataStorageError, Result};
pub(crate) use register_storage::RegisterStorage;

use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
use tracing::info;
use xor_name::XorName;

/// Operations on data.
#[derive(Clone)]
pub(crate) struct DataStorage {
    chunks: ChunkStorage,
    registers: RegisterStorage,
}

impl DataStorage {
    pub(crate) fn new(path: &Path, used_space: UsedSpace) -> Result<Self> {
        Ok(Self {
            chunks: ChunkStorage::new(path, used_space.clone())?,
            registers: RegisterStorage::new(path, used_space)?,
        })
    }

    /// Store data in the local store
    #[instrument(skip_all)]
    pub(super) async fn store(&self, data: &ReplicatedData) -> Result<Option<StorageLevel>> {
        match data {
            ReplicatedData::Chunk(chunk) => self.chunks.store(chunk).await,
            ReplicatedData::Register(register) => unimplemented!(), //self.registers.write(write, auth)
            ReplicatedData::RegisterWrite(write) => unimplemented!(), //self.registers.write(write, ),
        }
    }

    // Query the local store and return NodeQueryResponse
    pub(crate) async fn query(&self, query: &DataQuery) -> NodeQueryResponse {
        match query {
            DataQuery::GetChunk(addr) => self.chunks.get(addr).await,
            DataQuery::Register(read) => unimplemented!(), // self.registers.read(read, requester_pk).await,
        }
    }

    /// --- System calls ---

    /// Stores data that Elders sent to it for replication.
    /// Chunk should already have network authority
    /// TODO: define what authority is needed here...
    pub(crate) async fn store_for_replication(
        &self,
        data: &ReplicatedData,
    ) -> Result<Option<StorageLevel>> {
        debug!("Trying to store for replication: {:?}", data.name());
        self.store(data).await
    }

    // Read data from local store
    pub(crate) async fn get_for_replication(
        &self,
        address: &DataAddress,
    ) -> Result<ReplicatedData> {
        match address {
            DataAddress::Chunk(addr) => {
                self.chunks.get_chunk(addr).await.map(ReplicatedData::Chunk)
            }
            DataAddress::Register(addr) => self
                .registers
                .get_register_for_replication(addr)
                .await
                .map(ReplicatedData::Register),
        }
    }

    pub(crate) async fn remove(&self, address: &DataAddress) -> Result<()> {
        match address {
            DataAddress::Chunk(addr) => self.chunks.remove_chunk(addr).await,
            DataAddress::Register(addr) => self.registers.remove_register(addr).await,
        }
    }

    async fn keys(&self) -> Result<Vec<DataAddress>> {
        let chunk_keys = self.chunks.keys()?.into_iter().map(DataAddress::Chunk);
        let reg_keys = self
            .registers
            .keys()
            .await?
            .into_iter()
            .map(DataAddress::Register);
        Ok(reg_keys.chain(chunk_keys).collect())
    }
}

impl Core {
    #[allow(clippy::mutable_key_type)]
    pub(crate) async fn reorganize_data(
        &self,
        our_name: XorName,
        new_adults: BTreeSet<XorName>,
        lost_adults: BTreeSet<XorName>,
        remaining: BTreeSet<XorName>,
    ) -> Result<Vec<Command>> {
        let data = self.data_storage.clone();
        let keys = data.keys().await?;
        let mut data_for_replication = BTreeMap::new();
        for addr in keys.iter() {
            if let Some((data, holders)) = self
                .republish_and_cache(addr, &our_name, &new_adults, &lost_adults, &remaining)
                .await
            {
                let _prev = data_for_replication.insert(*data.name(), (data, holders));
            }
        }

        let mut commands = vec![];
        let section_pk = self.network_knowledge.section_key().await;
        for (_, (data, targets)) in data_for_replication {
            for name in targets {
                commands.push(Command::PrepareNodeMsgToSend {
                    msg: SystemMsg::NodeCmd(NodeCmd::ReplicateData(data.clone())),
                    dst: crate::messaging::DstLocation::Node { name, section_pk },
                })
            }
        }

        Ok(commands)
    }

    // on adults
    async fn republish_and_cache(
        &self,
        address: &DataAddress,
        our_name: &XorName,
        new_adults: &BTreeSet<XorName>,
        lost_adults: &BTreeSet<XorName>,
        remaining: &BTreeSet<XorName>,
    ) -> Option<(ReplicatedData, BTreeSet<XorName>)> {
        let storage = self.data_storage.clone();

        let old_adult_list = remaining.union(lost_adults).copied().collect();
        let new_adult_list = remaining.union(new_adults).copied().collect();
        let new_holders = self.compute_holders(address, &new_adult_list);
        let old_holders = self.compute_holders(address, &old_adult_list);

        let we_are_not_holder_anymore = !new_holders.contains(our_name);
        let new_adult_is_holder = !new_holders.is_disjoint(new_adults);
        let lost_old_holder = !old_holders.is_disjoint(lost_adults);

        if we_are_not_holder_anymore || new_adult_is_holder || lost_old_holder {
            info!("Republishing data at {:?}", address);
            trace!("We are not a holder anymore? {}, New Adult is Holder? {}, Lost Adult was holder? {}", we_are_not_holder_anymore, new_adult_is_holder, lost_old_holder);
            let data = storage.get_for_replication(address).await.ok()?;
            if we_are_not_holder_anymore {
                if let Err(err) = storage.remove(address).await {
                    warn!("Error deleting data during republish: {:?}", err);
                }
            }
            // TODO: Push to LRU cache
            Some((data, new_holders))
        } else {
            None
        }
    }
}
