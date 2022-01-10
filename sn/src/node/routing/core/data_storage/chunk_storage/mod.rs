// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod chunk_disk_store;

use crate::dbs::{convert_to_error_message, Error, Result};
use crate::messaging::{data::StorageLevel, system::NodeQueryResponse};
use crate::types::{log_markers::LogMarker, Chunk, ChunkAddress};
use crate::UsedSpace;

use chunk_disk_store::ChunkDiskStore;
use std::{
    fmt::{self, Display, Formatter},
    io::ErrorKind,
    path::Path,
    sync::Arc,
};
use tokio::sync::RwLock;
use tracing::info;

/// Operations on data chunks.
#[derive(Clone)]
pub(crate) struct ChunkStorage {
    disk_store: ChunkDiskStore,
    last_recorded_level: Arc<RwLock<StorageLevel>>,
}

impl ChunkStorage {
    pub(crate) fn new(path: &Path, used_space: UsedSpace) -> Result<Self> {
        Ok(Self {
            disk_store: ChunkDiskStore::new(path, used_space)?,
            last_recorded_level: Arc::new(RwLock::new(StorageLevel::zero())),
        })
    }

    pub(crate) fn keys(&self) -> Result<Vec<ChunkAddress>> {
        self.disk_store.list_all_chunk_addresses()
    }

    pub(crate) async fn remove_chunk(&self, address: &ChunkAddress) -> Result<()> {
        trace!("Removing chunk, {:?}", address);
        self.disk_store.delete_chunk(address).await
    }

    pub(crate) async fn get_chunk(&self, address: &ChunkAddress) -> Result<Chunk> {
        debug!("Getting chunk {:?}", address);

        match self.disk_store.read_chunk(address).await {
            Ok(res) => Ok(res),
            Err(error) => match error {
                Error::Io(io_error) if io_error.kind() == ErrorKind::NotFound => {
                    Err(Error::ChunkNotFound(*address.name()))
                }
                something_else => Err(something_else),
            },
        }
    }

    // Read chunk from local store and return NodeQueryResponse
    pub(crate) async fn get(&self, address: &ChunkAddress) -> NodeQueryResponse {
        trace!("{:?}", LogMarker::ChunkQueryReceviedAtAdult);
        NodeQueryResponse::GetChunk(
            self.get_chunk(address)
                .await
                .map_err(convert_to_error_message),
        )
    }

    /// Store a chunk in the local disk store
    /// If that chunk was already in the local store, just overwrites it
    #[instrument(skip_all)]
    pub(super) async fn store(&self, data: &Chunk) -> Result<Option<StorageLevel>> {
        if self.disk_store.chunk_file_exists(data.address())? {
            info!(
                "{}: Chunk already exists, not storing: {:?}",
                self,
                data.address()
            );
            // Nothing more to do here
            return Ok(None);
        }

        // cheap extra security check for space (prone to race conditions)
        // just so we don't go too much overboard
        // should not be triggered as chunks should not be sent to full adults
        if !self.disk_store.can_add(data.value().len()) {
            return Err(Error::NotEnoughSpace);
        }

        // store the data
        trace!("{:?}", LogMarker::StoringChunk);
        let _addr = self.disk_store.write_chunk(data).await?;
        trace!("{:?}", LogMarker::StoredNewChunk);

        // check if we've filled another apprx. 10%-points of our storage
        // if so, update the recorded level
        let last_recorded_level = { *self.last_recorded_level.read().await };
        if let Ok(next_level) = last_recorded_level.next() {
            // used_space_ratio is a heavy task that's why we don't do it all the time
            let used_space_ratio = self.disk_store.used_space_ratio();
            let used_space_level = 10.0 * used_space_ratio;
            // every level represents 10 percentage points
            if used_space_level as u8 >= next_level.value() {
                debug!("Next level for storage has been reached");
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

impl Display for ChunkStorage {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "ChunkStorage")
    }
}
