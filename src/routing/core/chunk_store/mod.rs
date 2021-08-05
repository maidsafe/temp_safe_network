// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::dbs::{convert_to_error_message, Error, Key, KvStore, Result, Subdir, UsedSpace, Value};
use crate::types::{Chunk, ChunkAddress, ChunkKind, PublicKey};
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

// #[derive(Clone)]
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
        self.address()
    }
}

impl ChunkStore {
    pub(crate) fn new(path: &Path, used_space: UsedSpace) -> Result<Self> {
        Ok(Self {
            db: Db::new(path, used_space)?,
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
                        Err(Error::InvalidOperation(
                            "Public chunk retrieved for Private read...".to_string(),
                        ))
                    }
                }
            },
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
                if !self.db.has(head_address)? {
                    info!(
                        "{}: Immutable chunk doesn't exist: {:?}",
                        self, head_address
                    );
                    return Ok(());
                }

                match self.db.get(head_address) {
                    Ok(Chunk::Private(data)) => {
                        if data.owner() == &requester {
                            self.db.delete(head_address)
                        } else {
                            Err(Error::InvalidOwner(requester))
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
                            write.dst_name()
                        )))
                    }
                    _ => Ok(()),
                }?;

                Ok(())
            }
        }
    }

    async fn try_store(&self, data: &Chunk) -> Result<()> {
        if self.db.has(data.address())? {
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

    // TODO: this is redundant, see if it can be omitted
    pub(crate) async fn is_storage_getting_full(&self) -> bool {
        info!("Checking used storage");
        self.db.used_space_ratio().await > MAX_STORAGE_USAGE_RATIO
    }

    /// Stores a chunk that Elders sent to it for replication.
    /// Chunk should already have network authority
    /// TODO: define what authority is needed here...
    pub(crate) async fn store_for_replication(&self, chunk: Chunk) -> Result<()> {
        debug!(
            "Trying to store for replication of chunk: {:?}",
            chunk.address()
        );
        if self.db.has(chunk.address())? {
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
