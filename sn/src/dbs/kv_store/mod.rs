// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! A simple, persistent, disk-based key-value store.

mod kv;
#[cfg(test)]
mod tests;

pub(super) mod to_db_key;
pub(super) mod used_space;

pub(crate) use kv::{Key, Value};

use super::{
    deserialise, Subdir,
    {encoding::serialise, Error, Result},
};
use serde::de::DeserializeOwned;
use sled::Db;
use std::{marker::PhantomData, path::Path};
use to_db_key::ToDbKey;
use used_space::UsedSpace;

const DB_DIR: &str = "db";

/// `KvStore` is a store of keys and values into a Sled db, while maintaining a maximum disk
/// usage to restrict storage.
#[derive(Clone, Debug)]
pub(crate) struct KvStore<V> {
    // tracks space used.
    used_space: UsedSpace,
    db: Db,
    _v: PhantomData<V>,
}

impl<V: Value> KvStore<V>
where
    Self: Subdir,
{
    /// Creates a new `KvStore` at location `root/STORE_DIR/<type>`.
    ///
    /// If the location specified already exists, the previous KvStore there is opened, otherwise
    /// the required folder structure is created.
    ///
    /// Used space of the dir is tracked.
    pub(crate) fn new<P: AsRef<Path>>(root: P, used_space: UsedSpace) -> Result<Self> {
        let dir = root.as_ref().join(DB_DIR).join(Self::subdir());

        used_space.add_dir(&dir);

        let db = sled::Config::default()
            .path(&dir)
            .flush_every_ms(Some(10000))
            .open()
            .map_err(Error::from)?;

        Ok(KvStore {
            used_space,
            db,
            _v: PhantomData,
        })
    }
}

impl<V: Value + Send + Sync> KvStore<V> {
    ///
    pub(crate) async fn total_used_space(&self) -> u64 {
        self.used_space.total().await
    }

    /// Tests if a value has been previously stored under `key`.
    pub(crate) fn has(&self, key: &V::Key) -> Result<bool> {
        let key = key.to_db_key()?;
        self.db.contains_key(key).map_err(Error::from)
    }

    /// Deletes the value stored under `key`.
    ///
    /// If the data doesn't exist, it does nothing and returns `Ok`.  In the case of an IO error, it
    /// returns `Error::Io`.
    pub(crate) fn delete(&self, key: &V::Key) -> Result<()> {
        let key = key.to_db_key()?;
        self.db.remove(key).map_err(Error::from).map(|_| ())
    }

    /// Stores a new value.
    ///
    /// If there is not enough storage space available, returns `Error::NotEnoughSpace`.  In case of
    /// an IO error, it returns `Error::Io`.
    ///
    /// If a value with the same id already exists, it will not be overwritten.
    pub(crate) async fn store(&self, value: &V) -> Result<()> {
        debug!("Writing value to KV store");

        let key = value.key().to_db_key()?;
        let serialised_value = serialise(value)?.to_vec();

        let exists = self.db.contains_key(key.clone()).map_err(Error::from)?;
        if !exists
            && !self
                .used_space
                .can_consume(serialised_value.len() as u64)
                .await
        {
            return Err(Error::NotEnoughSpace);
        }

        // Atomically write the value if it's new - this prevents multiple concurrent writes from
        // consuming extra space (since sled is backed by a log).
        match self
            .db
            .compare_and_swap::<_, &[u8], _>(key, None, Some(serialised_value))?
        {
            Ok(()) => debug!("Successfully wrote new value"),
            Err(sled::CompareAndSwapError { .. }) => {
                // We throw away the value if the compare_and_swap failed since current use-cases do
                // not require updates. In future we may want to return the preexisting value, or
                // have a separate API for overwriting stores.
                debug!("Value already existed, so we didn't have to write")
            }
        }

        Ok(())
    }

    /// Returns a value previously stored under `key`.
    ///
    /// If the value can't be accessed, it returns `Error::NoSuchData`.
    pub(crate) fn get(&self, key: &V::Key) -> Result<V> {
        let db_key = key.to_db_key()?;
        let res = self
            .db
            .get(db_key.clone())
            .map_err(|_| Error::KeyNotFound(db_key.clone()))?;

        if let Some(data) = res {
            let value: V = deserialise(&data)?;
            // Check it's the requested value.
            if value.key() == key {
                return Ok(value);
            }
        }

        Err(Error::KeyNotFound(db_key))
    }

    /// Used space to max capacity ratio.
    pub(crate) async fn used_space_ratio(&self) -> f64 {
        self.used_space.ratio().await
    }

    /// Lists all keys of currently stored data.
    pub(crate) fn keys(&self) -> Result<Vec<V::Key>> {
        let keys = self
            .db
            .iter()
            .flatten()
            .map(|(key, _)| key)
            .map(convert)
            .flatten()
            .collect();
        Ok(keys)
    }

    // We should avoid manually flushing in real code, since it can take a lot of time - tuning
    // `flush_every_ms` would be preferrable. This is useful in tests, however.
    #[cfg(test)]
    pub(crate) async fn flush(&self) -> Result<()> {
        let _bytes_flushed = self.db.flush_async().await?;
        Ok(())
    }
}

pub(crate) fn convert<T: DeserializeOwned>(key: sled::IVec) -> Result<T> {
    let db_key = &String::from_utf8(key.to_vec()).map_err(|_| Error::CouldNotConvertDbKey)?;
    to_db_key::from_db_key(db_key)
}
