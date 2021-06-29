// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::{
    client::Error as ErrorMessage,
    node::{NodeDataQueryResponse, NodeMsg, NodeQueryResponse},
    Aggregation, DstLocation, MessageId,
};
use crate::node::{
    data_store::ChunkDataStore,
    node_ops::MsgType,
    node_ops::{NodeDuty, OutgoingMsg},
    Error, Result,
};
use crate::types::{Chunk, ChunkAddress, DataAddress, PublicKey};
use std::{
    fmt::{self, Display, Formatter},
    path::Path,
};
use tracing::{error, info};

/// Storage of data chunks.
pub(crate) struct ChunkStorage {
    chunks: ChunkDataStore,
}

impl ChunkStorage {
    pub(crate) async fn new(path: &Path, max_capacity: u64) -> Result<Self> {
        let chunks = ChunkDataStore::new(path, max_capacity).await?;
        Ok(Self { chunks })
    }

    pub async fn keys(&self) -> Result<Vec<ChunkAddress>> {
        self.chunks.keys().await
    }

    pub(crate) async fn store(&mut self, data: &Chunk) -> Result<NodeDuty> {
        self.try_store(data).await?;

        Ok(NodeDuty::NoOp)
    }

    async fn try_store(&mut self, data: &Chunk) -> Result<()> {
        if self.chunks.has(data.address()).await {
            info!(
                "{}: Immutable chunk already exists, not storing: {:?}",
                self,
                data.address()
            );
            return Err(Error::DataExists);
        }
        self.chunks.put(&data).await
    }

    pub(crate) async fn get_chunk(&self, address: &ChunkAddress) -> Result<Chunk> {
        self.chunks.get(address).await
    }

    pub(crate) async fn delete_chunk(&mut self, address: &ChunkAddress) -> Result<()> {
        self.chunks.delete(&address).await
    }

    pub(crate) async fn get(&self, address: &ChunkAddress, msg_id: MessageId) -> NodeDuty {
        let result = self
            .get_chunk(address)
            .await
            .map_err(|_| ErrorMessage::DataNotFound(DataAddress::Chunk(*address)));

        NodeDuty::Send(OutgoingMsg {
            msg: MsgType::Node(NodeMsg::NodeQueryResponse {
                response: NodeQueryResponse::Data(NodeDataQueryResponse::GetChunk(result)),
                id: MessageId::in_response_to(&msg_id),
                correlation_id: msg_id,
            }),
            section_source: false, // sent as single node
            dst: DstLocation::Section(*address.name()),
            aggregation: Aggregation::None,
        })
    }

    /// Stores a chunk that Elders sent to it for replication.
    pub async fn store_for_replication(&mut self, chunk: Chunk) -> Result<()> {
        if self.chunks.has(chunk.address()).await {
            info!(
                "{}: Immutable chunk already exists, not storing: {:?}",
                self,
                chunk.address()
            );
            return Ok(());
        }

        self.chunks.put(&chunk).await?;

        Ok(())
    }

    pub async fn used_space_ratio(&self) -> f64 {
        self.chunks.used_space_ratio().await
    }

    pub(crate) async fn delete(
        &mut self,
        head_address: ChunkAddress,
        msg_id: MessageId,
        requester: PublicKey,
    ) -> Result<NodeDuty> {
        if !self.chunks.has(&head_address).await {
            info!(
                "{}: Immutable chunk doesn't exist: {:?}",
                self, head_address
            );
            return Ok(NodeDuty::NoOp);
        }

        match self.chunks.get(&head_address).await {
            Ok(Chunk::Private(data)) => {
                if data.owner() == &requester {
                    self.delete_chunk(&head_address)
                        .await
                        .map_err(|_error| ErrorMessage::FailedToDelete)
                } else {
                    Err(ErrorMessage::InvalidOwners(requester))
                }
            }
            Ok(_) => {
                error!(
                    "{}: Invalid DeletePrivate(Chunk::Public) encountered: {:?}",
                    self, msg_id
                );
                Err(ErrorMessage::InvalidOperation(format!(
                    "{}: Invalid DeletePrivate(Chunk::Public) encountered: {:?}",
                    self, msg_id
                )))
            }
            _ => Err(ErrorMessage::NoSuchKey),
        }?;

        Ok(NodeDuty::NoOp)
    }
}

impl Display for ChunkStorage {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "ChunkStorage")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::Result;
    use crate::types::{PrivateChunk, PublicChunk, PublicKey};
    use bls::SecretKey;
    use std::path::PathBuf;
    use tempdir::TempDir;

    fn temp_dir() -> Result<TempDir> {
        TempDir::new("test").map_err(|e| Error::TempDirCreationFailed(e.to_string()))
    }

    fn get_random_pk() -> PublicKey {
        PublicKey::from(SecretKey::random().public_key())
    }

    #[tokio::test]
    pub async fn try_store_stores_public_chunk() -> Result<()> {
        let path = PathBuf::from(temp_dir()?.path());
        let mut storage = ChunkStorage::new(&path, u64::MAX).await?;
        let value = "immutable data value".to_owned().into_bytes();
        let chunk = Chunk::Public(PublicChunk::new(value));
        assert!(storage.try_store(&chunk).await.is_ok());
        assert!(storage.chunks.has(chunk.address()).await);

        Ok(())
    }

    #[tokio::test]
    pub async fn try_store_stores_private_chunk() -> Result<()> {
        let path = PathBuf::from(temp_dir()?.path());
        let mut storage = ChunkStorage::new(&path, u64::MAX).await?;
        let value = "immutable data value".to_owned().into_bytes();
        let key = get_random_pk();
        let chunk = Chunk::Private(PrivateChunk::new(value, key));
        assert!(storage.try_store(&chunk).await.is_ok());
        assert!(storage.chunks.has(chunk.address()).await);

        Ok(())
    }
}
