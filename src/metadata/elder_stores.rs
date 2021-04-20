// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{
    blob_register::BlobRegister, map_storage::MapStorage, sequence_storage::SequenceStorage,
};
use crate::node::{MapDataExchange, SequenceDataExchange};
use crate::Error;

/// The various data type stores,
/// that are only managed at Elders.
pub(super) struct ElderStores {
    blob_register: BlobRegister,
    map_storage: MapStorage,
    sequence_storage: SequenceStorage,
}

impl ElderStores {
    pub fn new(
        blob_register: BlobRegister,
        map_storage: MapStorage,
        sequence_storage: SequenceStorage,
    ) -> Self {
        Self {
            blob_register,
            map_storage,
            sequence_storage,
        }
    }

    pub fn map_storage(&self) -> &MapStorage {
        &self.map_storage
    }

    pub fn sequence_storage(&self) -> &SequenceStorage {
        &self.sequence_storage
    }

    pub fn blob_register_mut(&mut self) -> &mut BlobRegister {
        &mut self.blob_register
    }

    pub fn map_storage_mut(&mut self) -> &mut MapStorage {
        &mut self.map_storage
    }

    pub fn sequence_storage_mut(&mut self) -> &mut SequenceStorage {
        &mut self.sequence_storage
    }

    pub fn fetch_map_and_sequence(&self) -> Result<(MapDataExchange, SequenceDataExchange), Error> {
        let map_data = self.map_storage.fetch_map_data()?;
        let seq_data = self.sequence_storage.fetch_seq_data()?;
        Ok((map_data, seq_data))
    }

    pub async fn update_map_and_sequence(
        &mut self,
        data: (MapDataExchange, SequenceDataExchange),
    ) -> Result<(), Error> {
        self.map_storage.update_map_data(data.0).await?;
        self.sequence_storage.update_seq_data(data.1).await?;
        Ok(())
    }
}
