// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! A simple, persistent, disk-based key-value store.

pub(super) mod data;
#[cfg(test)]
mod tests;
pub(super) mod to_db_key;
pub(super) mod used_space;

use to_db_key::ToDbKey;
use used_space::UsedSpace;

use super::{encoding::serialise, Error, Result}; // TODO: FIX
use data::{Data, DataId};
use sled::Db;
use std::{marker::PhantomData, path::Path};
use tracing::info;

const DB_DIR: &str = "db";

/// `DataStore` is a store of data held as serialised files on disk, implementing a maximum disk
/// usage to restrict storage.
#[derive(Clone)]
pub(crate) struct DataStore<T: Data> {
    // Maximum space allowed for all `DataStore`s to consume.
    used_space: UsedSpace,
    sled: Db,
    _phantom: PhantomData<T>,
}

impl<T> DataStore<T>
where
    T: Data,
    Self: Subdir,
{
    /// Creates a new `DataStore` at location `root/CHUNK_STORE_DIR/<chunk type>`.
    ///
    /// If the location specified already exists, the previous DataStore there is opened, otherwise
    /// the required folder structure is created.
    ///
    /// The maximum storage space is defined by `max_capacity`.  This specifies the max usable by
    /// _all_ `DataStores`, not per `DataStore`.
    pub(crate) fn new<P: AsRef<Path>>(root: P, used_space: UsedSpace) -> Result<Self> {
        let dir = root.as_ref().join(DB_DIR).join(Self::subdir());

        used_space.add_dir(&dir);

        let sled = sled::open(&dir).map_err(Error::from)?;

        Ok(DataStore {
            used_space,
            sled,
            _phantom: PhantomData,
        })
    }
}

impl<T: Data> DataStore<T> {
    ///
    pub(crate) async fn total_used_space(&self) -> u64 {
        self.used_space.total().await
    }

    /// Tests if a data chunk has been previously stored under `id`.
    pub(crate) fn has(&self, id: &T::Id) -> Result<bool> {
        let key = id.to_db_key()?;
        self.sled.contains_key(key).map_err(Error::from)
    }

    /// Deletes the data chunk stored under `id`.
    ///
    /// If the data doesn't exist, it does nothing and returns `Ok`.  In the case of an IO error, it
    /// returns `Error::Io`.
    pub(crate) fn delete(&self, id: &T::Id) -> Result<()> {
        let key = id.to_db_key()?;
        self.sled.remove(key).map_err(Error::from).map(|_| ())
    }

    /// Stores a new data chunk.
    ///
    /// If there is not enough storage space available, returns `Error::NotEnoughSpace`.  In case of
    /// an IO error, it returns `Error::Io`.
    ///
    /// If a chunk with the same id already exists, it will be overwritten.
    pub(crate) async fn put(&self, chunk: &T) -> Result<()> {
        let serialised_chunk = serialise(chunk)?.to_vec();
        let consumed_space = serialised_chunk.len() as u64;
        info!("consumed space: {:?}", consumed_space);
        if !self.used_space.can_consume(consumed_space).await {
            return Err(Error::NotEnoughSpace);
        }

        let key = chunk.id().to_db_key()?;
        let res = self.sled.insert(key, serialised_chunk);

        match res {
            Ok(_) => {
                info!("Writing chunk succeeded!");
                Ok(())
            }
            Err(e) => {
                info!("Writing chunk failed!");
                Err(Error::Sled(e))
            }
        }
    }

    /// Returns a data chunk previously stored under `id`.
    ///
    /// If the data file can't be accessed, it returns `Error::NoSuchData`.
    pub(crate) fn get(&self, id: &T::Id) -> Result<T> {
        let key = id.to_db_key()?;
        let res = self
            .sled
            .get(key)
            .map_err(|_| Error::KeyNotFound(id.to_data_address()))?;

        if let Some(data) = res {
            let chunk = bincode::deserialize::<T>(&data)?;
            // Check it's the requested chunk variant.
            if chunk.id() == id {
                return Ok(chunk);
            }
        }

        Err(Error::KeyNotFound(id.to_data_address()))
    }

    /// Used space to max capacity ratio.
    pub(crate) async fn used_space_ratio(&self) -> f64 {
        let used = self.total_used_space().await;
        let max_capacity = self.used_space.max_capacity();
        let used_space_ratio = used as f64 / max_capacity as f64;
        info!("Used space: {:?}", used);
        info!("Max capacity: {:?}", max_capacity);
        info!("Used space ratio: {:?}", used_space_ratio);
        used_space_ratio
    }

    /// Lists all keys of currently stored data.
    #[cfg_attr(not(test), allow(unused))]
    pub(crate) fn keys(&self) -> Result<Vec<T::Id>> {
        let keys = self
            .sled
            .iter()
            .flatten()
            .map(|(key, _)| {
                let db_key = &String::from_utf8(key.to_vec())
                    .map_err(|e| Error::InvalidOperation(e.to_string()))?;
                to_db_key::from_db_key(db_key)
            })
            .flatten()
            .collect();
        Ok(keys)
    }
}

pub(crate) trait Subdir {
    fn subdir() -> &'static Path;
}
