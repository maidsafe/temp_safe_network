// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{client::Error, messaging::data::RegisterWrite};

use dashmap::{DashMap, DashSet};
use futures::future::join_all;
use itertools::Itertools;
use rand::{rngs::OsRng, Rng};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    sync::Arc,
};
use tokio::{
    sync::RwLock,
    task::{self, JoinHandle},
};
use xor_name::XorName;

use super::Stash;

/// Batching ops into pools is a means to not include all chunks of the
/// same file into the same quote, since that quote is then sent around the network.
/// So we avoid that information leak by mixing chunks from different files.
/// Since we pay also for other types of ops than chunk uploads, we can speed up the chunk processing
/// by including those ops in the pools as well.
///
/// Say, our pool count is 4, and the limit is 10 (that means we will start payment step
/// when all pools have reached at least 10 ops). That means we will do 4 payments for 10 ops, initiated once we reach 40 ops.
/// So when we store a file of 20 chunks, we will have 5 chunks in each pool. The limit is 10, and is not reached yet.
/// Say we add another file of 15 chunks, we will then have 3 pools with 9 ops each, and 1 pool with 8 ops. We need another 5 ops.
/// Say we then do some operations on a merkle register; after 5 such ops, the pools are now all filled to their limits, and the
/// payment process starts, while we clear the pools for new ops.

#[allow(unused)]
pub(crate) struct Batch {
    files: BTreeSet<PathBuf>,
    reg_ops: BTreeSet<RegisterWrite>,
    msg_quota: BTreeMap<XorName, u64>,
}

#[derive(Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
struct OpId(pub(crate) XorName);

type UploadJob = DashSet<OpId>;
#[allow(unused)]
type UploadPools = Arc<RwLock<DashMap<XorName, UploadJob>>>;

type Pools = Arc<RwLock<DashMap<u8, UploadJob>>>;

type Db = Arc<sled::Db>;

#[allow(unused)]
pub(crate) struct Dbc {}

#[allow(unused)]
struct UploadBatching<S: Stash> {
    db: Db,
    pools: Pools,
    pool_limit: usize,
    stash: S,
}

impl<S: Stash> UploadBatching<S> {
    // pub fn new(pool_count: u8, pool_limit: usize, stash: S) -> Result<Self, Error> {
    //     let mut pools = DashMap::new();
    //     for i in 0..pool_count {
    //         let _ = pools.insert(i, DashSet::new());
    //     }
    //     let pools = Arc::new(RwLock::new(pools));
    //     let db = sled::open(Path::new("")).map_err(Error::)?;
    //     Ok(Self {
    //         db,
    //         pools,
    //         pool_limit,
    //         stash,
    //     })
    // }

    #[allow(unused)]
    pub(crate) async fn push(&self, batch: Batch) -> Result<(), Error> {
        let files = batch.files.clone();
        let pools = self.pools.clone();
        let _ = task::spawn(process_files(files, pools));
        for op in &batch.reg_ops {
            // pool + store to db
        }
        for (session, quota) in batch.msg_quota {
            // pool + store to db
        }

        self.try_clear_pools().await;

        Ok(())
    }

    // todo: let pools be persisted
    // do not clear pools, but use new pools
    // only clear after all processing is done
    async fn try_clear_pools(&self) {
        // take exclusive lock on the pools
        let pool = self.pools.write().await;
        // if all pools have reached the limit...
        if pool.iter().all(|set| set.value().len() >= self.pool_limit) {
            // ... then kick off payment process, and clear the pools
            let _ = pool
                .iter()
                .map(|set| {
                    set.value()
                        .iter()
                        .map(|s| OpId(s.0))
                        .collect::<BTreeSet<_>>()
                })
                .map(|set| (set, self.db.clone()))
                .map(|(set, db)| task::spawn(pay(set, db)))
                .collect_vec();

            pool.clear();
        }
    }
}

async fn process_files(files: BTreeSet<PathBuf>, pools: Pools) -> Vec<JoinHandle<()>> {
    // get chunks via SE, pool + store to db
    let handles = files
        .into_iter()
        .map(|file| (file, pools.clone()))
        .map(|(file, pools)| task::spawn_blocking(|| get_chunks(file, pools)))
        .collect_vec();
    let chunks = join_all(handles).await.into_iter().flatten().collect_vec();
    chunks
        .into_iter()
        .map(|chunks| (chunks, pools.clone()))
        .map(|(chunks, pools)| task::spawn(add_to_pools(chunks, pools)))
        .collect_vec()
}

fn get_chunks(_file: PathBuf, _pools: Pools) -> Vec<XorName> {
    // get chunks via SE, pool + store to db
    // let chunks = se.chunk(file);
    vec![]
}

async fn add_to_pools(ids: Vec<XorName>, pools: Pools) {
    let pool_ref = pools.read().await;
    let pool_count = pool_ref.len() as u8;
    let mut rng = OsRng;
    for id in ids {
        let index = rng.gen_range(0, pool_count);
        if let Some(pool) = pool_ref.get(&index) {
            let _ = pool.value().insert(OpId(id));
        }
    }
}

async fn pay(_ops: BTreeSet<OpId>, _db: Arc<sled::Db>) {
    // kick off payment process
}

#[allow(unused)]
fn send(_ops: BTreeSet<OpId>, _db: Arc<sled::Db>) {
    // kick off sending process
}
