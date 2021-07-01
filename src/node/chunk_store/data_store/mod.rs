// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! A simple, persistent, disk-based key-value store.

mod data;
#[cfg(test)]
mod tests;
mod used_space;

use crate::node::utils;
use crate::node::{Error, Result};
pub use data::{Data, DataId};
use std::{
    fs::Metadata,
    marker::PhantomData,
    path::{Path, PathBuf},
};
use tokio::fs::{self, DirEntry, File};
use tokio::io::AsyncWriteExt; // for write_all()
use tracing::info;
pub use used_space::UsedSpace;

const DB_DIR: &str = "database";

/// The max name length for a chunk file.
const MAX_CHUNK_FILE_NAME_LENGTH: usize = 104;

/// `DataStore` is a store of data held as serialised files on disk, implementing a maximum disk
/// usage to restrict storage.
pub(crate) struct DataStore<T: Data> {
    dir: PathBuf,
    // Maximum space allowed for all `DataStore`s to consume.
    used_space: UsedSpace,
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
    pub async fn new<P: AsRef<Path>>(root: P, used_space: UsedSpace) -> Result<Self> {
        let dir = root.as_ref().join(DB_DIR).join(Self::subdir());

        if fs::read(&dir).await.is_err() {
            Self::create_new_root(&dir).await?
        }

        used_space.add_dir(&dir);

        Ok(DataStore {
            dir,
            used_space,
            _phantom: PhantomData,
        })
    }
}

impl<T: Data> DataStore<T> {
    async fn create_new_root(root: &Path) -> Result<()> {
        fs::create_dir_all(root).await?;

        // Verify that chunk files can be created.
        let temp_file_path = root.join("0".repeat(MAX_CHUNK_FILE_NAME_LENGTH));
        let _ = File::create(&temp_file_path).await?;
        fs::remove_file(temp_file_path).await?;

        Ok(())
    }

    /// Stores a new data chunk.
    ///
    /// If there is not enough storage space available, returns `Error::NotEnoughSpace`.  In case of
    /// an IO error, it returns `Error::Io`.
    ///
    /// If a chunk with the same id already exists, it will be overwritten.
    pub async fn put(&self, chunk: &T) -> Result<()> {
        info!("Writing chunk");
        let serialised_chunk = utils::serialise(chunk)?;
        let consumed_space = serialised_chunk.len() as u64;

        info!("consumed space: {:?}", consumed_space);

        let file_path = self.file_path(chunk.id())?;
        self.do_delete(&file_path).await?;

        if !self.used_space.can_consume(consumed_space).await {
            return Err(Error::NotEnoughSpace);
        }

        let mut file = File::create(&file_path).await?;
        let res = file.write_all(&serialised_chunk).await;

        match res {
            Ok(_) => {
                info!("Writing chunk succeeded!");
                Ok(())
            }
            Err(e) => {
                info!("Writing chunk failed!");
                Err(e.into())
            }
        }
    }

    /// Deletes the data chunk stored under `id`.
    ///
    /// If the data doesn't exist, it does nothing and returns `Ok`.  In the case of an IO error, it
    /// returns `Error::Io`.
    pub async fn delete(&self, id: &T::Id) -> Result<()> {
        self.do_delete(&self.file_path(id)?).await
    }

    /// Used space to max capacity ratio.
    pub async fn used_space_ratio(&self) -> f64 {
        let used = self.total_used_space().await;
        let max_capacity = self.used_space.max_capacity();
        let used_space_ratio = used as f64 / max_capacity as f64;
        info!("Used space: {:?}", used);
        info!("Max capacity: {:?}", max_capacity);
        info!("Used space ratio: {:?}", used_space_ratio);
        used_space_ratio
    }

    /// Returns a data chunk previously stored under `id`.
    ///
    /// If the data file can't be accessed, it returns `Error::NoSuchData`.
    pub async fn get(&self, id: &T::Id) -> Result<T> {
        let contents = fs::read(self.file_path(id)?)
            .await
            .map_err(|_| Error::NoSuchData(id.to_data_address()))?;

        let chunk = bincode::deserialize::<T>(&contents)?;
        // Check it's the requested chunk variant.
        if chunk.id() == id {
            Ok(chunk)
        } else {
            Err(Error::NoSuchData(id.to_data_address()))
        }
    }

    pub async fn total_used_space(&self) -> u64 {
        self.used_space.total().await
    }

    /// Tests if a data chunk has been previously stored under `id`.
    pub async fn has(&self, id: &T::Id) -> bool {
        if let Ok(path) = self.file_path(id) {
            fs::metadata(path)
                .await
                .as_ref()
                .map(Metadata::is_file)
                .unwrap_or(false)
        } else {
            false
        }
    }

    /// Lists all keys of currently stored data.
    #[cfg_attr(not(test), allow(unused))]
    pub async fn keys(&self) -> Result<Vec<T::Id>> {
        let mut read_dir = fs::read_dir(&self.dir).await?;
        let mut keys = vec![];
        while let Some(entry) = read_dir.next_entry().await? {
            let chunk_id = to_chunk_id(&entry);
            if let Some(id) = chunk_id {
                keys.push(id);
            }
        }
        Ok(keys)
    }

    async fn do_delete(&self, file_path: &Path) -> Result<()> {
        if let Ok(_metadata) = fs::metadata(file_path).await {
            fs::remove_file(file_path).await.map_err(From::from)
        } else {
            Ok(())
        }
    }

    fn file_path(&self, id: &T::Id) -> Result<PathBuf> {
        Ok(self.dir.join(&hex::encode(utils::serialise(id)?)))
    }
}

pub(crate) trait Subdir {
    fn subdir() -> &'static Path;
}

fn to_chunk_id<T: DataId>(entry: &DirEntry) -> Option<T> {
    let file_name = entry.file_name();
    let file_name = file_name.into_string().ok()?;
    let bytes = hex::decode(file_name).ok()?;
    bincode::deserialize(&bytes).ok()
}
