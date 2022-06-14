// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use criterion::{BenchmarkId, Criterion};
use eyre::{Result, WrapErr};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use sn_interface::messaging::data::{CreateRegister, SignedRegisterCreate};
use sn_interface::types::{
    register::{Policy, User},
    Chunk, Keypair,
};
use sn_interface::types::{RegisterCmd, ReplicatedData};
use sn_node::node::cfg::config_handler::Config;
use sn_node::node::DataStorage;
use sn_node::UsedSpace;
use std::path::Path;
use tempfile::tempdir;
use tokio::runtime::Runtime;

use std::collections::BTreeMap;

// sample size is _NOT_ the number of times the command is run...
// https://bheisler.github.io/criterion.rs/book/analysis.html#measurement
const SAMPLE_SIZE: usize = 10;

fn main() -> Result<()> {
    let mut criterion = custom_criterion();

    bench_data_storage_writes(&mut criterion)?;
    bench_data_storage_reads(&mut criterion)?;

    Ok(())
}

fn bench_data_storage_writes(c: &mut Criterion) -> Result<()> {
    let mut group = c.benchmark_group("read-sampling");

    let runtime = Runtime::new().unwrap();
    pub const NONSENSE_CHUNK_SIZE: usize = 1024; // data size should not be important for keys() tests

    let size_ranges = [100, 500, 1_000];
    for size in size_ranges.iter() {
        group.bench_with_input(
            BenchmarkId::new("register_writes", size),
            size,
            |b, &size| {
                let storage = get_new_data_store()
                    .context("Could not create a temp data store")
                    .unwrap();

                println!("finished {size} writes");

                b.to_async(&runtime).iter(|| async {
                    for _ in 0..size {
                        let random_data = create_random_register_replicated_data();
                        let _ = storage.store(&random_data).await;
                    }
                })
            },
        );
    }

    for size in size_ranges.iter() {
        group.bench_with_input(BenchmarkId::new("chunk writes", size), size, |b, &size| {
            let storage = get_new_data_store()
                .context("Could not create a temp data store")
                .unwrap();

            b.to_async(&runtime).iter(|| async {
                for _ in 0..size {
                    // println!("writing d:{i}");
                    let file = sn_interface::types::utils::random_bytes(NONSENSE_CHUNK_SIZE);
                    let random_data = ReplicatedData::Chunk(Chunk::new(file));
                    storage
                        .store(&random_data)
                        .await
                        .expect("failed to write chunk {i}");
                }
                println!("finished {size} writes");
            })
        });
    }

    Ok(())
}

fn bench_data_storage_reads(c: &mut Criterion) -> Result<()> {
    let mut group = c.benchmark_group("read-sampling");

    let runtime = Runtime::new().unwrap();
    pub const NONSENSE_CHUNK_SIZE: usize = 1024; // data size should not be important for keys() tests

    let size_ranges = [100, 500, 1_000];
    for size in size_ranges.iter() {
        group.bench_with_input(BenchmarkId::new("register_keys", size), size, |b, &size| {
            let storage = get_new_data_store()
                .context("Could not create a temp data store")
                .unwrap();

            println!("starting writes");
            for _ in 0..size {
                let random_data = create_random_register_replicated_data();

                if runtime
                    .block_on(storage.store(&random_data))
                    .context("could not store register")
                    .is_err()
                {
                    panic!("Error storing register");
                }
            }

            println!("finished {size} writes");

            b.to_async(&runtime).iter(|| async {
                match storage.keys().await {
                    Ok(_) => {
                        let random_filename: String = thread_rng()
                            .sample_iter(&Alphanumeric)
                            .take(7)
                            .map(char::from)
                            .collect();

                        println!("progress? {:?}", random_filename)
                    }
                    Err(error) => println!("Reading store register keys failed with {:?}", error),
                }
            })
        });
    }

    for size in size_ranges.iter() {
        group.bench_with_input(BenchmarkId::new("chunk keys", size), size, |b, &size| {
            let storage = get_new_data_store()
                .context("Could not create a temp data store")
                .unwrap();

            println!("starting writes");
            for _ in 0..size {
                let file = sn_interface::types::utils::random_bytes(NONSENSE_CHUNK_SIZE);
                let random_data = ReplicatedData::Chunk(Chunk::new(file));
                if runtime
                    .block_on(storage.store(&random_data))
                    .context("could not store chunk")
                    .is_err()
                {
                    panic!("Error storing chunk");
                };
            }
            println!("finished {size} writes");

            b.to_async(&runtime).iter(|| async {
                match storage.keys().await {
                    Ok(_keys) => {
                        let random_filename: String = thread_rng()
                            .sample_iter(&Alphanumeric)
                            .take(7)
                            .map(char::from)
                            .collect();

                        println!("progress? {:?}", random_filename)
                    }
                    Err(error) => println!("Reading store register keys failed with {:?}", error),
                }
            })
        });
    }

    Ok(())
}

fn section_auth() -> sn_interface::messaging::SectionAuth {
    use sn_interface::messaging::system::KeyedSig;

    let sk = bls::SecretKey::random();
    let public_key = sk.public_key();
    let data = "hello".to_string();
    let signature = sk.sign(&data);
    let sig = KeyedSig {
        public_key,
        signature,
    };
    sn_interface::messaging::SectionAuth {
        src_name: sn_interface::types::PublicKey::Bls(public_key).into(),
        sig,
    }
}

fn custom_criterion() -> Criterion {
    Criterion::default().sample_size(SAMPLE_SIZE)
}

fn public_policy(owner: User) -> Policy {
    let permissions = BTreeMap::new();
    Policy { owner, permissions }
}

pub fn create_random_register_replicated_data() -> ReplicatedData {
    let keypair = Keypair::new_ed25519();

    let name = xor_name::rand::random();
    let tag = 15000;
    let owner = User::Key(keypair.public_key());
    let policy = public_policy(owner);

    let op = CreateRegister::Empty {
        name,
        tag,
        size: u16::MAX, // TODO: use argument
        policy,
    };
    let signature = keypair.sign(&bincode::serialize(&op).expect("could not serialize op"));
    let reg_cmd = RegisterCmd::Create {
        cmd: SignedRegisterCreate {
            op,
            auth: sn_interface::messaging::ServiceAuth {
                public_key: keypair.public_key(),
                signature,
            },
        },
        section_auth: section_auth(), // obtained after presenting a valid payment to the network
    };

    ReplicatedData::RegisterWrite(reg_cmd)
}

fn get_new_data_store() -> Result<DataStorage> {
    let random_filename: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();

    let root_dir = tempdir().map_err(|e| eyre::eyre!(e.to_string()))?;
    let storage_dir = Path::new(root_dir.path()).join(random_filename);
    let config = Config::default();
    let max_capacity = config.max_capacity();

    let used_space = UsedSpace::new(max_capacity);
    let store = DataStorage::new(&storage_dir, used_space)?;

    Ok(store)
}
