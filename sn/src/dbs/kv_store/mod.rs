// Copyright 2022 MaidSafe.net limited.
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

pub(crate) use kv::{Key, Value};

use super::{
    deserialise,
    used_space::UsedSpace,
    {encoding::serialise, Error, Result},
};
use serde::de::DeserializeOwned;
use sled::Db;
use std::{marker::PhantomData, path::Path};
use to_db_key::ToDbKey;

const CHUNK_DB_DIR: &str = "chunks";

/// `KvStore` is a store of keys and values into a Sled db, while maintaining a maximum disk
/// usage to restrict storage.
#[derive(Clone, Debug)]
pub(crate) struct KvStore<V> {
    // tracks space used.
    used_space: UsedSpace,
    db: Db,
    _v: PhantomData<V>,
}

impl<V: Value> KvStore<V> {
    /// Creates a new `KvStore` at location `root/STORE_DIR/<type>`.
    ///
    /// If the location specified already exists, the previous KvStore there is opened, otherwise
    /// the required folder structure is created.
    ///
    /// Used space of the dir is tracked.
    pub(crate) fn new<P: AsRef<Path>>(root: P, used_space: UsedSpace) -> Result<Self> {
        let dir = root.as_ref().join("db").join(CHUNK_DB_DIR);

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
    #[cfg(test)]
    pub(crate) async fn total_used_space(&self) -> usize {
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
    pub(crate) fn delete(&self, key: &V::Key) -> Result<Option<usize>> {
        let key = key.to_db_key()?;
        self.db
            .remove(key)
            .map_err(Error::from)
            .map(|option| option.map(|data| data.len()))
    }

    /// Stores a new value.
    ///
    /// If there is not enough storage space available, returns `Error::NotEnoughSpace`.  In case of
    /// an IO error, it returns `Error::Io`.
    ///
    /// If a value with the same id already exists, it will not be overwritten.
    #[cfg(test)]
    pub(crate) async fn store(&self, value: &V) -> Result<()> {
        debug!("Writing value to KV store");

        let key = value.key().to_db_key()?;
        let serialised_value = serialise(value)?.to_vec();

        let exists = self.db.contains_key(key.clone()).map_err(Error::from)?;
        if !exists && !self.used_space.can_add(serialised_value.len()) {
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

    /// Stores a batch of values.
    ///
    /// If there is not enough storage space available, returns `Error::NotEnoughSpace`.  In case of
    /// an IO error, it returns `Error::Io`.
    ///
    /// If a value with the same id already exists, it will be overwritten.
    pub(crate) async fn store_batch(&self, values: &[V]) -> Result<()> {
        info!("Writing batch");
        use rayon::prelude::*;
        type KvPair = (String, Vec<u8>);
        type KvPairResults = Vec<Result<KvPair>>;

        let mut batch = sled::Batch::default();
        let (ok, err_results): (KvPairResults, KvPairResults) = values
            .par_iter()
            .map(|value| {
                let serialised_value = serialise(value)?.to_vec();
                let key = value.key().to_db_key()?;
                Ok((key, serialised_value))
            })
            .partition(|r| r.is_ok());

        if !err_results.is_empty() {
            for e in err_results {
                error!("{:?}", e);
            }

            return Err(Error::SledBatching);
        }

        let consumed_space = ok
            .into_iter()
            .flatten()
            .map(|(key, value)| {
                let consumed_space = value.len();
                // FIXME: this will write value redundantly if it is already set, upsetting used
                // space calculation and wasting resources. This is quite likely to occur in
                // practice since many operations are applied redundantly from multiple nodes.
                batch.insert(key.as_bytes(), value);
                consumed_space
            })
            .sum();

        if !self.used_space.can_add(consumed_space) {
            return Err(Error::NotEnoughSpace);
        }

        let res = self.db.apply_batch(batch);

        match res {
            Ok(_) => {
                info!("Writing batch succeeded!");
                self.used_space.increase(consumed_space);
                Ok(())
            }
            Err(e) => {
                warn!("Writing batch failed!");
                Err(Error::Sled(e))
            }
        }
    }

    /// Returns a value previously stored under `key`.
    ///
    /// If the value can't be accessed, it returns `Error::KeyNotFound`.
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

    /// Lists all keys of currently stored data.
    pub(crate) fn keys(&self) -> Result<Vec<V::Key>> {
        let keys = self
            .db
            .iter()
            .flatten()
            .map(|(key, _)| key)
            .flat_map(convert)
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
