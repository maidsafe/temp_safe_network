// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::dbs::{convert_to_error_msg, Error, FileStore, Result};
use crate::UsedSpace;

use sn_interface::{
    messaging::system::NodeQueryResponse,
    types::{log_markers::LogMarker, Chunk, ChunkAddress},
};

use sn_interface::messaging::data::DataCmd;
use sn_interface::types::DataAddress;
use std::{
    fmt::{self, Display, Formatter},
    io::ErrorKind,
    path::Path,
};
use tracing::info;

const CHUNKS_DB_NAME: &str = "chunks";

/// Operations on data chunks.
#[derive(Clone, Debug)]
pub(crate) struct ChunkStorage {
    db: FileStore,
}

impl ChunkStorage {
    pub(crate) fn new(path: &Path, used_space: UsedSpace) -> Result<Self> {
        Ok(Self {
            db: FileStore::new(path.join(CHUNKS_DB_NAME), used_space)?,
        })
    }

    pub(crate) fn keys(&self) -> Result<Vec<DataAddress>> {
        self.db.list_all_data_addresses()
    }

    #[allow(dead_code)]
    pub(crate) async fn remove_chunk(&self, address: &ChunkAddress) -> Result<()> {
        trace!("Removing chunk, {:?}", address);
        self.db.delete_data(&DataAddress::Bytes(*address)).await
    }

    pub(crate) async fn get_chunk(&self, address: &ChunkAddress) -> Result<Chunk> {
        debug!("Getting chunk {:?}", address);

        match self.db.read_data(&DataAddress::Bytes(*address)).await {
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
        NodeQueryResponse::GetChunk(self.get_chunk(address).await.map_err(convert_to_error_msg))
    }

    /// Store a chunk in the local disk store
    /// If that chunk was already in the local store, just overwrites it
    #[instrument(skip_all)]
    pub(super) async fn store(&self, data: DataCmd) -> Result<()> {
        if self.db.data_file_exists(&data.address())? {
            info!(
                "{}: Data already exists, not storing: {:?}",
                self,
                data.address()
            );
            // Nothing more to do here
            return Ok(());
        }

        // cheap extra security check for space (prone to race conditions)
        // just so we don't go too much overboard
        // should not be triggered as chunks should not be sent to full adults
        if let DataCmd::StoreChunk(chunk) = &data {
            if !self.db.can_add(chunk.value().len()) {
                return Err(Error::NotEnoughSpace);
            }
        }

        // store the data
        trace!("{:?}", LogMarker::StoringChunk);
        let _addr = self.db.write_data(data).await?;
        trace!("{:?}", LogMarker::StoredNewChunk);

        Ok(())
    }
}

impl Display for ChunkStorage {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "ChunkStorage")
    }
}
