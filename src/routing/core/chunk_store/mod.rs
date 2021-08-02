// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::node_ops::{MsgType, OutgoingMsg};
use crate::dbs::{Key, KvStore, Subdir, UsedSpace, Value};
use crate::node::{
    node_ops::{NodeDuties, NodeDuty},
    Result,
};

use crate::dbs::Error as DbError;
use crate::types::{Chunk, ChunkAddress, PublicKey};
use crate::{
    messaging::{
        data::{ChunkRead, ChunkWrite},
        node::NodeQueryResponse,
    },
    types::DataAddress,
};
use std::{
    fmt::{self, Display, Formatter},
    path::Path,
};
use tracing::info;

/// At 50% full, the node will report that it's reaching full capacity.
pub(super) const MAX_STORAGE_USAGE_RATIO: f64 = 0.5;

type Db = KvStore<ChunkAddress, Chunk>;

impl Subdir for Db {
    fn subdir() -> &'static Path {
        Path::new("chunks")
    }
}

/// Operations on data chunks.
#[derive(Clone)]
pub(crate) struct ChunkStore {
    db: Db,
}

impl Key for ChunkAddress {}

impl Value for Chunk {
    type Key = ChunkAddress;

    fn key(&self) -> &Self::Key {
        match self {
            Chunk::Public(ref chunk) => chunk.address(),
            Chunk::Private(ref chunk) => chunk.address(),
        }
    }
}

impl ChunkStore {
    pub(crate) fn new(path: &Path, used_space: UsedSpace) -> Result<Self> {
        Ok(Self {
            db: Db::new(path, used_space).await?,
        })
    }

    pub(crate) async fn keys(&self) -> Result<Vec<ChunkAddress>> {
        self.db.keys().await.map_err(Error::from)
    }

    pub(crate) async fn remove_chunk(&self, address: &ChunkAddress) -> Result<()> {
        self.db.delete(address).await.map_err(Error::from)
    }

    pub(crate) async fn get_chunk(&self, address: &ChunkAddress) -> Result<Chunk> {
        self.db
            .get(address)
            .await
            .map_err(|_| Error::NoSuchData(DataAddress::Chunk(*address)))
    }

    // Read chunk from local store and return NodeQueryResponse
    pub(crate) fn read(&self, read: &ChunkRead, requester: PublicKey) -> Result<NodeQueryResponse> {
        let ChunkRead::Get(address) = read;
        let result = match self.get_chunk(address) {
            Ok(Chunk::Private(data)) => {
                if data.owner() == &requester {
                    Ok(Chunk::Private(data))
                } else {
                    Err(Error::InvalidOwner(requester))
                }
            }
            Ok(chunk) => Ok(chunk),
            error => error,
        };

        Ok(NodeQueryResponse::GetChunk(
            result.map_err(convert_to_error_message),
        ))
    }

    pub(super) async fn write(&self, write: &ChunkWrite, requester: PublicKey) -> Result<()> {
        match &write {
            ChunkWrite::New(data) => self.try_store(data).await,
            ChunkWrite::DeletePrivate(head_address) => {
                if !self.db.has(head_address).await? {
                    info!(
                        "{}: Immutable chunk doesn't exist: {:?}",
                        self, head_address
                    );
                    return Ok(());
                }

                match self.db.get(head_address).await {
                    Ok(Chunk::Private(data)) => {
                        if data.owner() == &requester {
                            self.db
                                .delete(head_address)
                                .await
                                .map_err(|_error| ErrorMessage::FailedToDelete)
                        } else {
                            Err(Error::InvalidOwner(requester))
                            // Err(ErrorMessage::InvalidOwners(requester))
                        }
                    }
                    Ok(_) => {
                        error!(
                            "{}: Invalid DeletePrivate(Chunk::Public) encountered: {:?}",
                            self,
                            write.dst_address()
                        );

                        Err(Error::InvalidOperation(format!(
                            "{}: Invalid DeletePrivate(Chunk::Public) encountered: {:?}",
                            self,
                            write.dst_address()
                        )))
                    }
                    _ => Ok(()),
                    // _ => Err(Error::NoSuchKey),
                }?;

                Ok(())
            }
        }
    }

    async fn try_store(&self, data: &Chunk) -> Result<NodeDuty> {
        if self.db.has(data.address()).await? {
            info!(
                "{}: Immutable chunk already exists, not storing: {:?}",
                self,
                data.address()
            );
            return Err(Error::DataExists);
        }
        self.db.store(data).await?;

        Ok(())
    }

    // // TODO: this is redundant, see if it can be omitted
    // pub(crate) async fn check_storage(&self) -> Result<NodeDuties> {
    //     info!("Checking used storage");
    //     if self.db.used_space_ratio().await > MAX_STORAGE_USAGE_RATIO {
    //         Ok(NodeDuties::from(NodeDuty::ReachingMaxCapacity))
    //     } else {
    //         Ok(vec![])
    //     }
    // }

    /// Stores a chunk that Elders sent to it for replication.
    /// Chunk should already have network authority
    /// TODO: define what authority is needed here...
    pub(crate) async fn store_for_replication(&self, chunk: Chunk) -> Result<()> {
        trace!(
            "Trying to store for replication of chunk: {:?}",
            chunk.address()
        );
        if self.db.has(chunk.address()).await? {
            info!(
                "{}: Immutable chunk already exists, not storing: {:?}",
                self,
                chunk.address()
            );
            return Ok(());
        }

        self.db.store(&chunk).await?;

        Ok(())
    }
}

impl Display for ChunkStore {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "ChunkStore")
    }
}
