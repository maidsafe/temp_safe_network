// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use criterion::Criterion;

use eyre::{Result, WrapErr};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use sn_interface::messaging::data::{CreateRegister, SignedRegisterCreate};
use sn_interface::types::{
    register::{Policy, PublicPolicy, User},
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

fn bench_data_storage(c: &mut Criterion) -> Result<()> {
    let mut group = c.benchmark_group("read-sampling");

    let runtime = Runtime::new().unwrap();

    group.bench_function("single registers", |b| {
        let storage = get_new_data_store()
            .context("Could not create a temp data store")
            .unwrap();
        let random_data = create_random_register_replicated_data();
        runtime
            .block_on(storage.store(&random_data))
            .context("could not store register")
            .unwrap();

        b.to_async(&runtime).iter(|| async {
            match storage.keys().await {
                Ok(_) => {}
                Err(error) => println!("Reading store register keys failed with {:?}", error),
            }
        })
    });

    group.bench_function("single chunks", |b| {
        pub const NONSENSE_CHUNK_SIZE: usize = 1024; // data size should not be important for keys() tests
        let storage = get_new_data_store()
            .context("Could not create a temp data store")
            .unwrap();

        let file = sn_interface::types::utils::random_bytes(NONSENSE_CHUNK_SIZE);
        let random_data = ReplicatedData::Chunk(Chunk::new(file));

        // store the data
        runtime
            .block_on(storage.store(&random_data))
            .context("could not store chunk")
            .unwrap();

        b.to_async(&runtime).iter(|| async {
            match storage.keys().await {
                Ok(_) => {}
                Err(error) => println!("Reading store for chunk keys failed with {:?}", error),
            }
        })
    });

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

fn main() -> Result<()> {
    let mut criterion = custom_criterion();

    bench_data_storage(&mut criterion)?;

    Ok(())
}

fn public_policy(owner: User) -> Policy {
    let permissions = BTreeMap::new();
    Policy::Public(PublicPolicy { owner, permissions })
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
