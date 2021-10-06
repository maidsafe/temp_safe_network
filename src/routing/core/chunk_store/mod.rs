// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::dbs::{convert_to_error_message, Error, KvStore, Result, Subdir, UsedSpace};
use crate::messaging::{data::StorageLevel, system::NodeQueryResponse};
use crate::types::{Chunk, ChunkAddress};

use std::{
    fmt::{self, Display, Formatter},
    path::Path,
    sync::Arc,
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
        debug!("Getting chunk {:?}", address);

        match self.db.get(address) {
            Ok(res) => Ok(res),
            Err(error) => match error {
                Error::KeyNotFound(_) => Err(Error::ChunkNotFound(*address.name())),
                something_else => Err(something_else),
            },
        }
    }

    // Read chunk from local store and return NodeQueryResponse
    pub(crate) fn get(&self, address: &ChunkAddress) -> NodeQueryResponse {
        NodeQueryResponse::GetChunk(self.get_chunk(address).map_err(convert_to_error_message))
    }

    pub(super) async fn store(&self, chunk: &Chunk) -> Result<Option<StorageLevel>> {
        if self.db.has(chunk.address())? {
            info!(
                "{}: Chunk already exists, not storing: {:?}",
                self,
                chunk.address()
            );
            // Nothing more to do here
            return Ok(None);
        }
        self.db.store(chunk).await?;

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
            chunk.name()
        );
        self.store(&chunk).await
    }
}

impl Display for ChunkStore {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "ChunkStore")
    }
}
