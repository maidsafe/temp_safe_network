// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{
    super::Subdir, Error, Key, KvStore, Result as KvStoreResult, ToDbKey, UsedSpace, Value,
};
use eyre::{eyre, Result};
use rand::{distributions::Standard, rngs::ThreadRng, Rng};
use serde::{Deserialize, Serialize};
use std::{path::Path, u64};
use tempfile::{tempdir, TempDir};
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
struct TestData {
    id: Id,
    value: Vec<u8>,
}

impl Value for TestData {
    type Key = Id;
    fn key(&self) -> &Self::Key {
        &self.id
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
struct Id(u64);

impl ToDbKey for Id {}
impl Key for Id {}

impl Subdir for KvStore<TestData> {
    fn subdir() -> &'static Path {
        Path::new("test")
    }
}

// TODO: use seedable rng
fn new_rng() -> ThreadRng {
    rand::thread_rng()
}

fn temp_dir() -> KvStoreResult<TempDir> {
    tempdir().map_err(|e| Error::TempDirCreationFailed(e.to_string()))
}

struct Chunks {
    data_and_sizes: Vec<(Vec<u8>, usize)>,
    total_size: usize,
}

impl Chunks {
    // Construct random amount of randomly-sized chunks, keeping track of the total size of all
    // chunks when serialised.
    fn gen<R: Rng>(rng: &mut R) -> Result<Self> {
        let mut chunks = Self {
            data_and_sizes: vec![],
            total_size: 0,
        };
        let chunk_count: u8 = rng.gen();
        for _ in 0..chunk_count {
            let size: u8 = rng.gen();
            let data = TestData {
                id: Id(0),
                value: rng.sample_iter(&Standard).take(size as usize).collect(),
            };
            let serialised_size = bincode::serialized_size(&data).map_err(Error::Bincode)? as usize;

            chunks.total_size += serialised_size;
            chunks.data_and_sizes.push((data.value, serialised_size));
        }
        Ok(chunks)
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs refactor"]
async fn used_space_increases() -> Result<()> {
    let mut rng = new_rng();
    let chunks = Chunks::gen(&mut rng)?;

    let root = temp_dir()?;
    let used_space = UsedSpace::new(u64::MAX as usize);
    let db = KvStore::<TestData>::new(root.path(), used_space)?;

    let used_space_before = db.total_used_space().await;

    for (index, (data, _size)) in chunks.data_and_sizes.iter().enumerate().rev() {
        let the_data = &TestData {
            id: Id(index as u64),
            value: data.clone(),
        };

        assert!(!db.has(&the_data.id)?);
        db.store(the_data).await?;
        assert!(db.has(&the_data.id)?);
    }

    db.flush().await?;
    let used_space_after = db.total_used_space().await;
    assert!(used_space_after > used_space_before);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "it doesn't decrease.."]
async fn used_space_decreases() -> Result<()> {
    let mut rng = new_rng();
    let chunks = Chunks::gen(&mut rng)?;

    let root = temp_dir()?;
    let used_space = UsedSpace::new(u64::MAX as usize);
    let db = KvStore::<TestData>::new(root.path(), used_space)?;

    for (index, (data, _size)) in chunks.data_and_sizes.iter().enumerate().rev() {
        let the_data = &TestData {
            id: Id(index as u64),
            value: data.clone(),
        };

        assert!(!db.has(&the_data.id)?);
        db.store(the_data).await?;
        assert!(db.has(&the_data.id)?);
    }

    let used_space_before = db.total_used_space().await;

    for key in db.keys()? {
        let _ = db.delete(&key)?;
    }

    db.flush().await?;
    let used_space_after = db.total_used_space().await;
    assert!(used_space_after < used_space_before);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn successful_put() -> Result<()> {
    let mut rng = new_rng();
    let chunks = Chunks::gen(&mut rng)?;

    let root = temp_dir()?;
    let used_space = UsedSpace::new(u64::MAX as usize);
    let db = KvStore::<TestData>::new(root.path(), used_space)?;

    for (index, (data, _size)) in chunks.data_and_sizes.iter().enumerate().rev() {
        let the_data = &TestData {
            id: Id(index as u64),
            value: data.clone(),
        };
        assert!(!db.has(&the_data.id)?);
        db.store(the_data).await?;
        assert!(db.has(&the_data.id)?);
    }

    let mut keys = db.keys()?;
    keys.sort();
    assert_eq!(
        (0..chunks.data_and_sizes.len())
            .map(|i| Id(i as u64))
            .collect::<Vec<_>>(),
        keys
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn failed_put_when_not_enough_space() -> Result<()> {
    let mut rng = new_rng();
    let root = temp_dir()?;
    let capacity = 32;
    let used_space = UsedSpace::new(capacity);
    let db = KvStore::new(root.path(), used_space)?;

    let data = TestData {
        id: Id(rng.gen()),
        value: rng.sample_iter(&Standard).take(capacity + 1).collect(),
    };

    match db.store(&data).await {
        Err(Error::NotEnoughSpace) => (),
        x => return Err(eyre!(format!("Unexpected: {:?}", x))),
    }

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn delete() -> Result<()> {
    let mut rng = new_rng();
    let chunks = Chunks::gen(&mut rng)?;

    let root = temp_dir()?;
    let used_space = UsedSpace::new(u64::MAX as usize);
    let db = KvStore::new(root.path(), used_space)?;

    for (index, (data, _size)) in chunks.data_and_sizes.iter().enumerate() {
        let the_data = &TestData {
            id: Id(index as u64),
            value: data.clone(),
        };
        db.store(the_data).await?;

        while !db.has(&the_data.id)? {
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }

        let _ = db.delete(&the_data.id)?;

        while db.has(&the_data.id)? {
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }
    }

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn put_and_get_value_should_be_same() -> Result<()> {
    let mut rng = new_rng();
    let chunks = Chunks::gen(&mut rng)?;

    let root = temp_dir()?;
    let used_space = UsedSpace::new(u64::MAX as usize);
    let db = KvStore::new(root.path(), used_space)?;

    for (index, (data, _)) in chunks.data_and_sizes.iter().enumerate() {
        db.store(&TestData {
            id: Id(index as u64),
            value: data.clone(),
        })
        .await?
    }

    for (index, (data, _)) in chunks.data_and_sizes.iter().enumerate() {
        let retrieved_value = db.get(&Id(index as u64))?;
        assert_eq!(*data, retrieved_value.value);
    }

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs refactor"]
async fn no_overwrite_value() -> Result<()> {
    let mut rng = new_rng();
    let chunks = Chunks::gen(&mut rng)?;
    let (first_chunk, remaining_chunks) = chunks
        .data_and_sizes
        .split_first()
        .expect("generated empty chunks");

    let root = temp_dir()?;
    let used_space = UsedSpace::new(u64::MAX as usize);
    let db = KvStore::new(root.path(), used_space)?;

    let initial_used_space = db.total_used_space().await;

    let key = &Id(0);
    db.store(&TestData {
        id: *key,
        value: first_chunk.0.clone(),
    })
    .await?;
    db.flush().await?;

    let total_used_space = db.total_used_space().await;
    assert!(total_used_space >= initial_used_space + first_chunk.1);

    for (data, _) in remaining_chunks {
        db.store(&TestData {
            id: *key,
            value: data.clone(),
        })
        .await?;

        assert_eq!(db.total_used_space().await, total_used_space);
        assert_eq!(db.get(key)?.value, first_chunk.0);
    }

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn get_fails_when_key_does_not_exist() -> Result<()> {
    let root = temp_dir()?;
    let used_space = UsedSpace::new(u64::MAX as usize);
    let db: KvStore<TestData> = KvStore::new(root.path(), used_space)?;

    let id = Id(new_rng().gen());
    match db.get(&id) {
        Err(Error::KeyNotFound(_)) => (),
        x => return Err(eyre!(format!("Unexpected {:?}", x))),
    }

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn keys() -> Result<()> {
    let mut rng = new_rng();
    let chunks = Chunks::gen(&mut rng)?;

    let root = temp_dir()?;
    let used_space = UsedSpace::new(u64::MAX as usize);
    let db = KvStore::new(root.path(), used_space)?;

    for (index, (data, _)) in chunks.data_and_sizes.iter().enumerate() {
        let id = Id(index as u64);
        assert!(!db.keys()?.contains(&id));
        db.store(&TestData {
            id,
            value: data.clone(),
        })
        .await?;

        let keys = db.keys()?;
        assert!(keys.contains(&id));
        assert_eq!(keys.len(), index + 1);
    }

    for (index, _) in chunks.data_and_sizes.iter().enumerate() {
        let id = Id(index as u64);

        assert!(db.keys()?.contains(&id));
        let _ = db.delete(&id)?;

        let keys = db.keys()?;
        assert!(!keys.contains(&id));
        assert_eq!(keys.len(), chunks.data_and_sizes.len() - index - 1);
    }

    Ok(())
}
