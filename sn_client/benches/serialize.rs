// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use criterion::Throughput;
use criterion::{criterion_group, criterion_main, Criterion};

use eyre::Result;

use sn_client::{Client, Error};
use sn_interface::{
    messaging::{data::ServiceMsg, AuthKind, Dst, MsgId, ServiceAuth, WireMsg},
    types::register::{Policy, User},
};
use tokio::runtime::Runtime;

use std::collections::BTreeMap;

fn public_policy(owner: User) -> Policy {
    let permissions = BTreeMap::new();
    Policy { owner, permissions }
}

/// Generates a random vector of Dsts using provided `length`.
fn random_vectorof_dsts(length: usize) -> Vec<Dst> {
    let mut dsts = vec![];
    for _i in 0..length {
        dsts.push(Dst {
            name: xor_name::rand::random(),
            section_key: bls::SecretKey::random().public_key(),
        });
    }
    dsts
}

async fn create_client() -> Result<Client, Error> {
    let client = Client::builder().build().await?;

    Ok(client)
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialize");
    let runtime = Runtime::new().unwrap();

    let client = match runtime.block_on(create_client()) {
        Ok(client) => client,
        Err(err) => {
            println!("Failed to create client with {:?}", err);
            return;
        }
    };

    let (auth, payload, msg_id) = runtime.block_on(async {
        let name = xor_name::rand::random();
        let tag = 15000;
        let owner = User::Key(client.public_key());
        let policy = public_policy(owner);

        let (_address, batch) = match client.create_register(name, tag, policy).await {
            Ok(x) => x,
            Err(error) => panic!("error creating register {error:?}"),
        };

        let client_pk = client.public_key();

        let msg_id = MsgId::new();

        let payload = {
            let msg = ServiceMsg::Cmd(batch[0].clone());
            match WireMsg::serialize_msg_payload(&msg) {
                Ok(payload) => payload,
                Err(error) => panic!("failed to serialise msg payload: {error:?}"),
            }
        };

        let auth = ServiceAuth {
            public_key: client_pk,
            signature: client.sign(&payload),
        };

        let auth = AuthKind::Service(auth);

        // wire_msg parts
        (auth, payload, msg_id)
    });

    let dsts = random_vectorof_dsts(1024);

    // here we determine the size for our throughput measurement.

    let dst = Dst {
        name: xor_name::rand::random(),
        section_key: bls::SecretKey::random().public_key(),
    };
    let mut the_wire_msg = WireMsg::new_msg(msg_id, payload, auth, dst);
    let (header, dst, payload) = match the_wire_msg.serialize_and_cache_bytes() {
        Ok(bytes) => bytes,
        Err(_erorr) => {
            panic!("Could not form initial WireMsg");
        }
    };

    let bytes_size = header.len() + dst.len() + payload.len();

    group.throughput(Throughput::Bytes((bytes_size * dsts.len()) as u64));
    // upload and read
    group.bench_with_input(
        "serialize for sending",
        &(dsts, the_wire_msg),
        |b, (dsts, the_wire_msg)| {
            b.to_async(&runtime).iter(|| async {
                for dst in dsts.iter() {
                    if let Err(error) = the_wire_msg.serialize_with_new_dst(dst) {
                        panic!("failed to serialise next dst {error:?}");
                    }
                }

                Ok::<(), Box<dyn std::error::Error>>(())
            });
        },
    );

    group.finish()
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
