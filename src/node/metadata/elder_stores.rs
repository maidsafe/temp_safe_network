// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{
    chunk_records::ChunkRecords, map_storage::MapStorage, register_storage::RegisterStorage,
    sequence_storage::SequenceStorage,
};
use crate::messaging::{
    client::{ClientSig, DataCmd, DataExchange, DataQuery},
    EndUser, MessageId,
};
use crate::node::{node_ops::NodeDuty, Error, Result};
use crate::routing::Prefix;
use crate::types::PublicKey;
use log::info;
use std::sync::Arc;
use tokio::sync::RwLock;
/// The various data type stores,
/// that are only managed at Elders.
pub(super) struct ElderStores {
    chunk_records: Arc<RwLock<ChunkRecords>>,
    map_storage: Arc<RwLock<MapStorage>>,
    sequence_storage: Arc<RwLock<SequenceStorage>>,
    register_storage: Arc<RwLock<RegisterStorage>>,
}

impl ElderStores {
    pub fn new(
        chunk_records: ChunkRecords,
        map_storage: MapStorage,
        sequence_storage: SequenceStorage,
        register_storage: RegisterStorage,
    ) -> Self {
        Self {
            chunk_records: Arc::new(RwLock::new(chunk_records)),
            map_storage: Arc::new(RwLock::new(map_storage)),
            sequence_storage: Arc::new(RwLock::new(sequence_storage)),
            register_storage: Arc::new(RwLock::new(register_storage)),
        }
    }

    pub async fn read(
        &self,
        query: DataQuery,
        msg_id: MessageId,
        requester: PublicKey,
        origin: EndUser,
    ) -> Result<NodeDuty> {
        match &query {
            DataQuery::Blob(read) => {
                self.chunk_records
                    .write()
                    .await
                    .read(read, msg_id, origin)
                    .await
            }
            DataQuery::Map(read) => {
                self.map_storage
                    .read()
                    .await
                    .read(read, msg_id, requester, origin)
                    .await
            }
            DataQuery::Sequence(read) => {
                self.sequence_storage
                    .read()
                    .await
                    .read(read, msg_id, requester, origin)
                    .await
            }
            DataQuery::Register(read) => {
                self.register_storage
                    .read()
                    .await
                    .read(read, msg_id, requester, origin)
                    .await
            }
        }
    }

    pub async fn write(
        &self,
        cmd: DataCmd,
        msg_id: MessageId,
        client_sig: ClientSig,
        origin: EndUser,
    ) -> Result<NodeDuty> {
        info!("Writing Data");
        match cmd {
            DataCmd::Blob(write) => {
                info!("Writing Blob");
                self.chunk_records
                    .write()
                    .await
                    .write(write, msg_id, client_sig, origin)
                    .await
            }
            DataCmd::Map(write) => {
                info!("Writing Map");
                self.map_storage
                    .write()
                    .await
                    .write(write, msg_id, client_sig.public_key, origin)
                    .await
            }
            DataCmd::Sequence(write) => {
                info!("Writing Sequence");
                self.sequence_storage
                    .write()
                    .await
                    .write(write, msg_id, client_sig.public_key, origin)
                    .await
            }
            DataCmd::Register(write) => {
                info!("Writing Register");
                self.register_storage
                    .write()
                    .await
                    .write(write, msg_id, client_sig.public_key, origin)
                    .await
            }
        }
    }

    pub fn chunk_records(&self) -> Arc<RwLock<ChunkRecords>> {
        self.chunk_records.clone()
    }

    // NB: Not yet including Register metadata.
    pub async fn get_data_of(&self, prefix: Prefix) -> Result<DataExchange> {
        // Prepare chunk_records, map and sequence data
        let chunk_data = self.chunk_records.read().await.get_data_of(prefix).await;
        let map_data = self.map_storage.read().await.get_data_of(prefix).await?;
        let seq_data = self
            .sequence_storage
            .read()
            .await
            .get_data_of(prefix)
            .await?;

        Ok(DataExchange {
            chunk_data,
            map_data,
            seq_data,
        })
    }

    pub async fn update(&self, data: DataExchange) -> Result<(), Error> {
        self.map_storage.write().await.update(data.map_data).await?;
        self.sequence_storage
            .write()
            .await
            .update(data.seq_data)
            .await?;
        self.chunk_records
            .write()
            .await
            .update(data.chunk_data)
            .await;

        Ok(())
    }
}
