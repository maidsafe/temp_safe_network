// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::dbs::{convert_to_error_message, Error, Key, KvStore, Result, Subdir, UsedSpace, Value};
use crate::messaging::data::StorageLevel;
use crate::types::{Chunk, ChunkAddress, ChunkKind, PublicKey};
use crate::{
    messaging::{
        data::{ChunkRead, ChunkWrite},
        node::NodeQueryResponse,
    },
    types::DataAddress,
};
use std::sync::Arc;
use std::{
    fmt::{self, Display, Formatter},
    path::Path,
};
use tokio::sync::RwLock;
use tracing::info;

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
    last_recorded_level: Arc<RwLock<StorageLevel>>,
}

impl Key for ChunkAddress {}

impl Value for Chunk {
    type Key = ChunkAddress;

    fn key(&self) -> &Self::Key {
        self.address()
    }
}

impl ChunkStore {
    pub(crate) fn new(path: &Path, used_space: UsedSpace) -> Result<Self> {
        Ok(Self {
            db: Db::new(path, used_space)?,
            last_recorded_level: Arc::new(RwLock::new(StorageLevel::zero())),
        })
    }

    pub(crate) fn keys(&self) -> Result<Vec<ChunkAddress>> {
        self.db.keys()
    }

    pub(crate) fn remove_chunk(&self, address: &ChunkAddress) -> Result<()> {
        trace!("Removing chunk, {:?}", address);
        self.db.delete(address)
    }

    pub(crate) fn get_chunk(&self, address: &ChunkAddress) -> Result<Chunk> {
        debug!("Getting chunk at address {:?}", address);

        match self.db.get(address) {
            Ok(res) => Ok(res),
            Err(error) => match error {
                Error::KeyNotFound(_) => Err(Error::NoSuchData(DataAddress::Chunk(*address))),
                something_else => Err(something_else),
            },
        }
    }

    // Read chunk from local store and return NodeQueryResponse
    pub(crate) fn read(&self, read: &ChunkRead, requester: PublicKey) -> Result<NodeQueryResponse> {
        let ChunkRead::Get(address) = read;
        let req_kind = address.kind();

        let result = match self.get_chunk(address) {
            Ok(chunk) => match chunk.kind() {
                ChunkKind::Private => {
                    if let Some(owner) = chunk.owner() {
                        if req_kind.is_private() && owner == &requester {
                            Ok(chunk)
                        } else {
                            Err(Error::InvalidOwner(requester))
                        }
                    } else {
                        Err(Error::InvalidOwner(requester))
                    }
                }
                ChunkKind::Public => {
                    if req_kind.is_public() {
                        Ok(chunk)
                    } else {
                        Err(Error::DataIdNotFound(DataAddress::Chunk(*address)))
                    }
                }
            },
            error => error,
        };

        Ok(NodeQueryResponse::GetChunk(
            result.map_err(convert_to_error_message),
        ))
    }

    pub(super) async fn write(
        &self,
        write: &ChunkWrite,
        requester: PublicKey,
    ) -> Result<Option<StorageLevel>> {
        match &write {
            ChunkWrite::New(data) => self.try_store(data).await,
            ChunkWrite::DeletePrivate(head_address) => {
                if !self.db.has(head_address)? {
                    info!(
                        "{}: Immutable chunk doesn't exist: {:?}",
                        self, head_address
                    );
                    return Ok(None);
                }

                match self.db.get(head_address) {
                    Ok(Chunk::Private(data)) => {
                        if data.owner() == &requester {
                            self.db.delete(head_address)?;
                            // if we have dropped 10 %-points in usage, then we'll report that to Elders
                            let last_recorded_level = { *self.last_recorded_level.read().await };
                            if let Ok(previous_level) = last_recorded_level.previous() {
                                let used_space = self.db.used_space_ratio().await;
                                // every level represents 10 percentage points
                                if previous_level.value() > (10.0 * used_space) as u8 {
                                    *self.last_recorded_level.write().await = previous_level;
                                    return Ok(Some(previous_level));
                                }
                            }
                        } else {
                            return Err(Error::InvalidOwner(requester));
                        }
                    }
                    Ok(_) => {
                        error!(
                            "{}: Invalid DeletePrivate(Chunk::Public) encountered: {:?}",
                            self,
                            write.dst_address()
                        );
                        return Err(Error::NoSuchData(DataAddress::Chunk(*head_address)));
                    }
                    _ => (),
                };

                Ok(None)
            }
        }
    }

    async fn try_store(&self, data: &Chunk) -> Result<Option<StorageLevel>> {
        if self.db.has(data.address())? {
            info!(
                "{}: Immutable chunk already exists, not storing: {:?}",
                self,
                data.address()
            );
            // Nothing more to do here
            return Ok(None);
        }
        self.db.store(data).await?;

        let last_recorded_level = { *self.last_recorded_level.read().await };
        if let Ok(next_level) = last_recorded_level.next() {
            let used_space = self.db.used_space_ratio().await;
            // every level represents 10 percentage points
            if (10.0 * used_space) as u8 >= next_level.value() {
                *self.last_recorded_level.write().await = next_level;
                return Ok(Some(next_level));
            }
        }

        Ok(None)
    }

    /// Stores a chunk that Elders sent to it for replication.
    /// Chunk should already have network authority
    /// TODO: define what authority is needed here...
    pub(crate) async fn store_for_replication(&self, chunk: Chunk) -> Result<Option<StorageLevel>> {
        debug!(
            "Trying to store for replication of chunk: {:?}",
            chunk.address()
        );
        self.try_store(&chunk).await
    }
}

impl Display for ChunkStore {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "ChunkStore")
    }
}
