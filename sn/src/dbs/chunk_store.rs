// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::kv_store::KvStore;
use super::Result;

use crate::types::{Chunk, ChunkAddress};
use crate::UsedSpace;

use self_encryption::MAX_CHUNK_SIZE;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

/// A disk store for chunks
#[derive(Clone)]
pub(crate) struct ChunkStore {
    db: KvStore<Chunk>,
    mem_pool: Arc<RwLock<BTreeMap<ChunkAddress, Chunk>>>,
    old_mem_pool: Arc<RwLock<Vec<Chunk>>>,
    used_space: UsedSpace,
}

impl ChunkStore {
    /// Creates a new `ChunkStore` at location `root/CHUNK_DB_DIR`
    ///
    /// If the location specified already contains a ChunkStore, it is simply used
    ///
    /// Used space of the dir is tracked
    pub(crate) fn new<P: AsRef<Path>>(root: P, used_space: UsedSpace) -> Result<Self> {
        Ok(ChunkStore {
            db: KvStore::new(root, used_space.clone())?,
            mem_pool: Arc::new(RwLock::new(BTreeMap::new())),
            old_mem_pool: Arc::new(RwLock::new(vec![])),
            used_space,
        })
    }

    // ---------------------- api methods ----------------------

    pub(crate) fn keys(&self) -> Result<Vec<ChunkAddress>> {
        self.db.keys()
    }

    pub(crate) fn can_add(&self, size: usize) -> bool {
        self.used_space.can_add(size)
    }

    pub(crate) async fn write_chunk(&self, chunk: Chunk) -> Result<ChunkAddress> {
        let addr = *chunk.address();
        let store_batch = {
            let mut pool = self.mem_pool.write().await;

            self.used_space.increase(chunk.value().len());
            let _ = pool.insert(addr, chunk);

            let pool_len = pool.len();
            let pool_size: usize = pool.values().map(|c| c.value().len()).sum();
            // TODO: randomize the flush
            if pool_len > 1000 || pool_size > 100 * MAX_CHUNK_SIZE {
                self.old_mem_pool
                    .write()
                    .await
                    .extend(pool.values().cloned());
                pool.clear();
                true
            } else {
                false
            }
        };
        if store_batch {
            let batch: Vec<_> = { self.old_mem_pool.write().await.drain(..).collect() };
            self.db.store_batch(&batch).await?;
        }

        Ok(addr)
    }

    pub(crate) fn delete_chunk(&self, addr: &ChunkAddress) -> Result<()> {
        if let Some(size) = self.db.delete(addr)? {
            self.used_space.decrease(size);
        }
        Ok(())
    }

    pub(crate) async fn read_chunk(&self, addr: &ChunkAddress) -> Result<Chunk> {
        let pool = self.mem_pool.read().await;
        match pool.get(addr) {
            Some(chunk) => Ok(chunk.clone()),
            None => self.db.get(addr),
        }
    }

    #[allow(unused)]
    pub(crate) fn exists(&self, addr: &ChunkAddress) -> Result<bool> {
        self.db.has(addr)
    }
}

#[cfg(test)]
mod tests {
    use crate::types::utils::random_bytes;

    use super::*;
    use bytes::Bytes;
    use futures::future::join_all;
    use rayon::prelude::*;
    use tempfile::tempdir;

    fn init_chunk_disk_store() -> ChunkStore {
        let root = tempdir().expect("Failed to create temporary directory for chunk disk store");
        ChunkStore::new(root.path(), UsedSpace::new(usize::MAX))
            .expect("Failed to create chunk disk store")
    }

    #[tokio::test]
    //#[ignore]
    async fn test_write_read_chunk() {
        let store = init_chunk_disk_store();
        // test that a range of different chunks return the written chunk
        let mut keys = vec![];

        // flushing occurs after 1k entries
        for _ in 0..1100 {
            let chunk = Chunk::new(random_bytes(100));

            let addr = store
                .write_chunk(chunk.clone())
                .await
                .expect("Failed to write chunk.");

            let read_chunk = store
                .read_chunk(&addr)
                .await
                .expect("Failed to read chunk.");

            assert_eq!(chunk.value(), read_chunk.value());

            keys.push(addr);
        }

        // check all is available also after flushing
        for addr in keys {
            let _chunk = store
                .read_chunk(&addr)
                .await
                .expect("Failed to read chunk.");
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_write_read_async_multiple_chunks() {
        let store = init_chunk_disk_store();
        let size = 100;
        let chunks: Vec<Chunk> = std::iter::repeat_with(|| Chunk::new(random_bytes(size)))
            .take(7)
            .collect();
        write_and_read_chunks(&chunks, store).await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_write_read_async_multiple_identical_chunks() {
        let store = init_chunk_disk_store();
        let chunks: Vec<Chunk> = std::iter::repeat(Chunk::new(Bytes::from("test_concurrent")))
            .take(7)
            .collect();
        write_and_read_chunks(&chunks, store).await;
    }

    async fn write_and_read_chunks(chunks: &[Chunk], store: ChunkStore) {
        // write all chunks
        let tasks = chunks.iter().map(|c| store.write_chunk(c.clone()));
        let results = join_all(tasks).await;

        // read all chunks
        let tasks = results.iter().flatten().map(|addr| store.read_chunk(addr));
        let results = join_all(tasks).await;
        let read_chunks: Vec<&Chunk> = results.iter().flatten().collect();

        // verify all written were read
        assert!(chunks
            .par_iter()
            .all(|c| read_chunks.iter().any(|r| r.value() == c.value())))
    }
}
