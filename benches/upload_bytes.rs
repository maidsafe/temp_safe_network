// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use criterion::{criterion_group, criterion_main, Criterion};
use eyre::Result;
use safe_network::url::Scope;
use safe_network::{
    client::{
        utils::test_utils::{read_network_conn_info, run_w_backoff_delayed},
        Client, Config, Error,
    },
    types::utils::random_bytes,
};
use tokio::runtime::Runtime;

/// This bench requires a network already set up
async fn upload_bytes(size: usize) -> Result<(), Error> {
    let bootstrap_nodes = read_network_conn_info().unwrap();
    let data = random_bytes(size);
    let config = Config::new(None, None, None, None).await;
    let client = Client::new(config, bootstrap_nodes, None).await?;
    let address = client.write_to_network(data.clone(), Scope::Public).await?;

    // let's make sure the public chunk is stored
    let received_data = run_w_backoff_delayed(|| client.read_blob(address), 10).await?;

    assert_eq!(received_data, data);

    Ok(())
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("upload-sampling");

    let runtime = Runtime::new().unwrap();
    group.sample_size(10);
    group.bench_function("upload 3072b", |b| {
        b.to_async(&runtime).iter(|| async {
            match upload_bytes(3072).await {
                Ok(_) => {}
                Err(error) => println!("bench failed with {:?}", error),
            }
        });
    });
    group.bench_function("upload 1mb", |b| {
        b.to_async(&runtime).iter(|| async {
            match upload_bytes(1024 * 1024).await {
                Ok(_) => {}
                Err(error) => println!("bench failed with {:?}", error),
            }
        });
    });
    group.finish()
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
