// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

macro_rules! assert_err {
    ($cond:expr, $error:pat) => {
        match $cond {
            Err($error) => (),
            result => panic!(
                concat!("Expecting ", stringify!($error), " got {:?}"),
                result
            ),
        }
    };
}

use chunk_store::{Chunk, ChunkId, ChunkStore, Error};
use maidsafe_utilities::{serialisation, SeededRng};
use rand::Rng;
use tempdir::TempDir;

struct Chunks {
    data_and_sizes: Vec<(Vec<u8>, u64)>,
    total_size: u64,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
struct Data(Vec<u8>);

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
struct Id(u64);

impl Chunk<Id> for Data {
    type Id = Id;
}

impl ChunkId<Id> for Id {
    type Chunk = Data;
    fn to_key(&self) -> Id {
        *self
    }
}

// Construct random amount of randomly-sized chunks, keeping track of the total size of all
// chunks when serialised.
fn generate_random_chunks() -> Chunks {
    let mut rng = SeededRng::thread_rng();
    let mut chunks = Chunks {
        data_and_sizes: vec![],
        total_size: 0,
    };
    let chunk_count: u8 = rng.gen();
    for _ in 0..chunk_count {
        let size: u8 = rng.gen();
        let data = rng.gen_iter().take(size as usize).collect();
        let serialised_size = unwrap!(serialisation::serialise(&data)).len() as u64;
        chunks.total_size += serialised_size;
        chunks.data_and_sizes.push((data, serialised_size));
    }
    chunks
}

#[test]
fn create_multiple_instances_in_the_same_root() {
    let test_dir = "safe_vault_test";
    // root already exists
    {
        let temp_dir = unwrap!(TempDir::new(test_dir));
        let root = unwrap!(temp_dir.path().to_str()).to_string();
        let chunk_store1 = unwrap!(ChunkStore::<Id>::new(Some(root.clone()), Some(64)));
        let chunk_store2 = unwrap!(ChunkStore::<Id>::new(Some(root), Some(64)));
        assert_ne!(chunk_store1.root_dir.path(), chunk_store2.root_dir.path());
    }
    // root doesn't exist yet
    {
        let temp_dir = unwrap!(TempDir::new(test_dir));
        let root = unwrap!(temp_dir.path().join("foo").join("bar").to_str()).to_string();
        let chunk_store1 = unwrap!(ChunkStore::<Id>::new(Some(root.clone()), Some(64)));
        let chunk_store2 = unwrap!(ChunkStore::<Id>::new(Some(root), Some(64)));
        assert_ne!(chunk_store1.root_dir.path(), chunk_store2.root_dir.path());
    }
}

#[test]
fn store_dir_should_cleanup() {
    let store_dir;
    {
        let mut chunk_store = unwrap!(ChunkStore::new(None, Some(64)));
        store_dir = chunk_store.root_dir.path().to_path_buf();
        unwrap!(chunk_store.put(&Id(0), &Data(vec![0; 10])));
        assert!(store_dir.exists());
    }
    assert!(!store_dir.exists());
}

#[test]
fn successful_put() {
    let chunks = generate_random_chunks();
    let mut chunk_store = unwrap!(ChunkStore::new(None, Some(chunks.total_size)));
    {
        let mut put = |id, data, size| {
            let size_before_insert = chunk_store.used_space();
            assert!(!chunk_store.has(&id));
            unwrap!(chunk_store.put(&id, &data));
            assert_eq!(chunk_store.used_space(), size + size_before_insert);
            assert!(chunk_store.has(&id));
            assert!(chunk_store.used_space() <= chunks.total_size);
        };

        for (index, &(ref data, ref size)) in chunks.data_and_sizes.iter().enumerate().rev() {
            put(Id(index as u64), Data(data.clone()), size);
        }
    }
    assert_eq!(chunk_store.used_space(), chunks.total_size);

    let mut keys = chunk_store.keys();
    keys.sort();
    assert_eq!(
        (0..chunks.data_and_sizes.len())
            .map(|i| Id(i as u64))
            .collect::<Vec<_>>(),
        keys
    );
}

#[test]
fn failed_put_when_not_enough_space() {
    let disk_size = 32;
    let mut chunk_store = unwrap!(ChunkStore::new(None, Some(disk_size)));
    let mut rng = SeededRng::thread_rng();
    let id = Id(rng.gen());
    let data = Data(rng.gen_iter().take((disk_size + 1) as usize).collect());
    assert_err!(chunk_store.put(&id, &data), Error::NotEnoughSpace);
}

#[test]
fn delete() {
    let chunks = generate_random_chunks();
    let mut chunk_store = unwrap!(ChunkStore::new(None, Some(chunks.total_size)));
    let mut put_and_delete = |id, value, size| {
        unwrap!(chunk_store.put(&id, &value));
        assert_eq!(chunk_store.used_space(), size);
        assert!(chunk_store.has(&id));
        unwrap!(chunk_store.delete(&id));
        assert!(!chunk_store.has(&id));
        assert_eq!(chunk_store.used_space(), 0);
    };

    for (index, &(ref data, ref size)) in chunks.data_and_sizes.iter().enumerate() {
        put_and_delete(Id(index as u64), Data(data.clone()), *size);
    }
}

#[test]
fn put_and_get_value_should_be_same() {
    let chunks = generate_random_chunks();
    let mut chunk_store = unwrap!(ChunkStore::new(None, Some(chunks.total_size)));
    for (index, &(ref data, _)) in chunks.data_and_sizes.iter().enumerate() {
        unwrap!(chunk_store.put(&Id(index as u64), &Data(data.clone())));
    }
    for (index, &(ref data, _)) in chunks.data_and_sizes.iter().enumerate() {
        let retrieved_value = unwrap!(chunk_store.get(&Id(index as u64)));
        assert_eq!(*data, retrieved_value.0);
    }
}

#[test]
fn overwrite_value() {
    let chunks = generate_random_chunks();
    let mut chunk_store = unwrap!(ChunkStore::new(None, Some(chunks.total_size)));
    for (ref data, ref size) in chunks.data_and_sizes {
        unwrap!(chunk_store.put(&Id(0), &Data(data.clone())));
        assert_eq!(chunk_store.used_space(), *size);
        let retrieved_value = unwrap!(chunk_store.get(&Id(0)));
        assert_eq!(*data, retrieved_value.0);
    }
}

#[test]
fn get_fails_when_key_does_not_exist() {
    let chunk_store = unwrap!(ChunkStore::new(None, Some(64)));
    let id = Id(SeededRng::thread_rng().gen());
    assert_err!(chunk_store.get(&id), Error::NotFound);
}

#[test]
fn keys() {
    let chunks = generate_random_chunks();
    let mut chunk_store = unwrap!(ChunkStore::new(None, Some(chunks.total_size)));

    for (index, &(ref data, _)) in chunks.data_and_sizes.iter().enumerate() {
        let id = Id(index as u64);
        assert!(!chunk_store.keys().contains(&id));
        unwrap!(chunk_store.put(&id, &Data(data.clone())));
        assert!(chunk_store.keys().contains(&id));
        assert_eq!(chunk_store.keys().len(), index + 1);
    }

    for (index, _) in chunks.data_and_sizes.iter().enumerate() {
        let id = Id(index as u64);

        assert!(chunk_store.keys().contains(&id));
        unwrap!(chunk_store.delete(&id));
        assert!(!chunk_store.keys().contains(&id));
        assert_eq!(
            chunk_store.keys().len(),
            chunks.data_and_sizes.len() - index - 1
        );
    }
}
