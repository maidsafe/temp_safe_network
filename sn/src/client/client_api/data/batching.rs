// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::pac_man::{encrypt_blob, encrypt_from_path};
use super::{ItemKey, Stash};

use crate::client::{utils::encryption, Error, Result};
use crate::dbs::{ChunkDiskStore, ToDbKey};
use crate::messaging::data::RegisterWrite;
use crate::types::{Chunk, Scope};
use crate::UsedSpace;

use bytes::Bytes;
use dashmap::{DashMap, DashSet};
use futures::{future::join_all, stream::FuturesUnordered};
use itertools::Itertools;
use rand::{rngs::OsRng, Rng};
use rayon::prelude::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    sync::Arc,
};
use tokio::{
    sync::RwLock,
    task::{self, JoinHandle},
};

/// Batching ops into pools is a means to not include all chunks of the
/// same file into the same quote, since that quote is then sent around the network.
/// So we avoid that information leak, by mixing chunks from different files.
/// Since we pay also for other types of ops than chunk uploads, we can speed up the chunk processing
/// by including those ops in the pools as well.
///
/// Say, our pool count is 4, and the limit is 10 (that means we will start payment step
/// when all pools have reached at least 10 ops). That means we will do 4 payments for 10 ops each, initiated once we have all of them (i.e 40 ops).
/// So when we store a file of 20 chunks, we will have 5 chunks in each pool. The limit is 10, and is not reached yet.
/// Say we add another file of 15 chunks, we will then have 3 pools with 9 ops each, and 1 pool with 8 ops. We need another 5 ops.
/// Say we then do some operations on a merkle register; after 5 such ops, the pools are now all filled to their limits, and the
/// payment process starts, while we clear the pools for new ops.

/// It is up to client how many
/// entries it wants to batch at a time.
/// It could push a single entry, or thousands of them, in every batch.
#[derive(Debug, Default)]
pub struct Batch {
    /// files on disk
    pub files: BTreeMap<ItemKey, (PathBuf, Scope)>, // becomes chunks
    /// values from memory
    pub values: BTreeMap<ItemKey, (Bytes, Scope)>, // becomes chunks
    /// register ops
    pub reg_ops: BTreeMap<ItemKey, RegisterWrite>, // reg entry
    /// quotas for ephemeral messaging
    pub msg_quotas: BTreeMap<ItemKey, u64>, // ..
}

type BatchTasks = FuturesUnordered<JoinHandle<()>>;
type BatchResult = Result<BatchTasks>;

type PaymentJob = DashSet<ItemKey>;
type Pools = Arc<RwLock<DashMap<u8, PaymentJob>>>;
type ChunkDb = ChunkDiskStore;

#[derive(Clone, Debug)]
struct Dbs {
    chunks: ChunkDb,
}

///
#[derive(Clone, Debug)]
pub(crate) struct Batching<S: Stash> {
    dbs: Dbs,
    pools: Pools,
    pool_limit: usize,
    stash: S,
}

///
pub(crate) struct BatchingConfig {
    ///
    pub(crate) pool_count: u8,
    ///
    pub(crate) pool_limit: usize,
    ///
    pub(crate) root_dir: PathBuf,
    ///
    pub(crate) used_space: UsedSpace,
}

impl<S: Stash> Batching<S> {
    pub(crate) fn new(config: BatchingConfig, stash: S) -> Result<Self> {
        let pools = DashMap::new();
        for i in 0..config.pool_count {
            let _ = pools.insert(i, DashSet::new());
        }
        let pools = Arc::new(RwLock::new(pools));
        let dbs = Dbs {
            chunks: ChunkDb::new(config.root_dir, config.used_space)?,
        };
        Ok(Self {
            dbs,
            pools,
            pool_limit: config.pool_limit,
            stash,
        })
    }

    #[allow(unused)]
    pub(crate) fn push(&self, batch: Batch) {
        let dbs = self.dbs.clone();
        let pools = self.pools.clone();
        let pool_limit = self.pool_limit;
        let _ = task::spawn(push_task(batch, dbs, pools, pool_limit));
    }
}

async fn push_task(batch: Batch, dbs: Dbs, pools: Pools, pool_limit: usize) {
    let Batch {
        files,
        values,
        reg_ops,
        msg_quotas,
    } = batch;

    let values_task = task::spawn(process_values(values, dbs.clone(), pools.clone()));
    let files_task = task::spawn(process_files(files, dbs.clone(), pools.clone()));
    let reg_task = task::spawn(process_reg_ops(reg_ops, pools.clone()));
    let msg_task = task::spawn(process_msg_quotas(msg_quotas, pools.clone()));

    let res = join_all([values_task, files_task, reg_task, msg_task])
        .await
        .into_iter()
        .flatten() // drops errors
        .flatten() // drops errors
        .flatten()
        .collect_vec();

    let _ = join_all(res).await;

    for refm in pools.read().await.iter() {
        println!("Pool {}: {} ops", refm.key(), refm.value().len());
    }

    try_clear_pools(dbs, pools, pool_limit).await;
}

// todo: let pools be persisted
// do not clear pools, but use new pools
// only clear after all processing is done
async fn try_clear_pools(db: Dbs, pools: Pools, pool_limit: usize) {
    // take exclusive lock on the pools, this makes the transition between pools clean
    let pool = pools.write().await;
    // if all pools have reached the limit...
    if pool.iter().all(|set| set.value().len() >= pool_limit) {
        // ... then kick off payment process, and clear the pools
        let _ = pool
            .iter()
            .map(|set| {
                set.value()
                    .iter()
                    .map(|s| s.clone())
                    .collect::<BTreeSet<_>>()
            })
            .map(|set| (set, db.clone()))
            .map(|(set, dbs)| task::spawn(pay(set, dbs)))
            .collect_vec();

        pool.iter().for_each(|s| s.value().clear());
    }
}

async fn process_reg_ops(_reg_ops: BTreeMap<ItemKey, RegisterWrite>, _pools: Pools) -> BatchResult {
    Ok(BatchTasks::new())
}

async fn process_msg_quotas(_msg_quotas: BTreeMap<ItemKey, u64>, _pools: Pools) -> BatchResult {
    Ok(BatchTasks::new())
}

async fn process_values(
    values: BTreeMap<ItemKey, (Bytes, Scope)>,
    dbs: Dbs,
    pools: Pools,
) -> BatchResult {
    // get chunks via SE (+ store to db), then pool them
    let batches: Vec<_> = values
        .par_iter()
        .map(|(key, (data, scope))| {
            let owner = encryption(*scope, get_random_pk());
            let (_root_address, chunks) = encrypt_blob(data.clone(), owner.as_ref())?;
            Ok::<_, Error>((key.clone(), chunks))
        })
        .flatten()  // drops errors
        .collect();

    store_batches(dbs, batches, pools).await
}

async fn store_batches(dbs: Dbs, batches: Vec<(String, Vec<Chunk>)>, pools: Pools) -> BatchResult {
    let tasks: Vec<_> = batches
        .into_iter()
        .map(|(key, chunks)| {
            let dbs = dbs.clone();
            task::spawn(async move {
                // store all chunks to local db
                match dbs.chunks.store_batch(&chunks).await {
                    Ok(_) => (),
                    Err(e) => println!("Store batch error: {}", e),
                } // fix, onlything async in there is used_space..
                  //TODO: register/multimap op: [ItemKey => Address], i.e. [key => root_address]
                let ids = chunks
                    .par_iter()
                    .map(|c| c.address().to_db_key())
                    .flatten()
                    .collect();
                (key, ids)
            })
        })
        .collect();

    let to_return = BatchTasks::new();

    join_all(tasks)
        .await
        .into_iter()
        .flatten() // drops errors
        .for_each(|(key, ids)| to_return.push(task::spawn(add_to_pools(key, ids, pools.clone()))));

    Ok(to_return)
}

async fn process_files(
    files: BTreeMap<ItemKey, (PathBuf, Scope)>,
    dbs: Dbs,
    pools: Pools,
) -> BatchResult {
    // get chunks via SE (+ store to db), then pool them
    let batches: Vec<_> = files
        .par_iter()
        .map(|(key, (path, scope))| {
            let owner = encryption(*scope, get_random_pk());
            let (_root_address, chunks) = encrypt_from_path(path, owner.as_ref())?;
            Ok::<_, Error>((key.clone(), chunks))
        })
        .flatten()  // drops errors
        .collect();

    store_batches(dbs, batches, pools).await
}

// /// Takes a chunk and fetches the data map from it.
// /// If the data map is not the root data map of the user's contents,
// /// the process repeats itself until it obtains the root data map.
// async fn unpack(&self, mut chunk: Chunk) -> Result<DataMap> {
//     loop {
//         let public = chunk.is_public();
//         match deserialize(chunk.value())? {
//             SecretKeyLevel::Root(data_map) => {
//                 return Ok(data_map);
//             }
//             SecretKeyLevel::Child(data_map) => {
//                 let serialized_chunk = self
//                     .read_all(data_map, public)
//                     .await?;
//                 chunk = deserialize(&serialized_chunk)?;
//             }
//         }
//     }
// }

async fn add_to_pools(_key: ItemKey, ids: Vec<ItemKey>, pools: Pools) {
    let pool_ref = pools.read().await;
    let pool_count = pool_ref.len() as u8;
    //let mut rng = OsRng;
    ids.par_iter().for_each(|id| {
        let mut rng = OsRng;
        let index = rng.gen_range(0, pool_count);
        if let Some(pool) = pool_ref.get(&index) {
            let _ = pool.value().insert(id.clone());
        }
    });
    // for id in ids {
    //     let index = rng.gen_range(0, pool_count);
    //     if let Some(pool) = pool_ref.get(&index) {
    //         let _ = pool.value().insert(id);
    //     }
    // }
}

async fn pay(ops: BTreeSet<ItemKey>, _dbs: Dbs) {
    // kick off payment process
    // get quote
    println!("Paying for {} ops", ops.len());
}

#[allow(unused)]
fn send(_ops: BTreeSet<ItemKey>, _dbs: Dbs) {
    // kick off sending process
}

fn get_random_pk() -> crate::types::PublicKey {
    crate::types::PublicKey::from(bls::SecretKey::random().public_key())
}
