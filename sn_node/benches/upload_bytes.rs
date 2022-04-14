// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use criterion::{criterion_group, criterion_main, Criterion};
use eyre::Result;
use sn_client::{utils::test_utils::read_network_conn_info, Client, ClientConfig, Error};
use sn_interface::types::utils::random_bytes;
use sn_interface::types::Scope;
use tokio::runtime::Runtime;

/// This bench requires a network already set up
async fn upload_and_read_bytes(size: usize) -> Result<(), Error> {
    let (genesis_key, bootstrap_nodes) = read_network_conn_info().unwrap();
    let bytes = random_bytes(size);
    let config = ClientConfig::new(None, None, genesis_key, None, None, None, None).await;
    let client = Client::new(config, bootstrap_nodes, None).await?;
    let address = client.upload(bytes.clone(), Scope::Public).await?;

    // let's make sure the public chunk is stored
    let received_bytes = client.read_bytes(address).await?;

    assert_eq!(received_bytes, bytes);

    Ok(())
}
/// This bench requires a network already set up
async fn upload_only(size: usize) -> Result<(), Error> {
    let (genesis_key, bootstrap_nodes) = read_network_conn_info().unwrap();
    let bytes = random_bytes(size);
    let config = ClientConfig::new(None, None, genesis_key, None, None, None, None).await;
    let client = Client::new(config, bootstrap_nodes, None).await?;
    let _ = client.upload(bytes.clone(), Scope::Public).await?;

    Ok(())
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("upload-sampling");
    // measure over around 9s in general
    group.measurement_time(std::time::Duration::from_secs(90));

    let runtime = Runtime::new().unwrap();
    group.sample_size(10);

    // upload and read
    group.bench_function("upload and read 3072b", |b| {
        b.to_async(&runtime).iter(|| async {
            match upload_and_read_bytes(3072).await {
                Ok(_) => {}
                Err(error) => println!("3072b upload and read bench failed with {:?}", error),
            }
        });
    });
    group.bench_function("upload and read 1mb", |b| {
        b.to_async(&runtime).iter(|| async {
            match upload_and_read_bytes(1024 * 1024).await {
                Ok(_) => {}
                Err(error) => println!("1mb upload and read bench failed with {:?}", error),
            }
        });
    });
    group.bench_function("upload and read 10mb", |b| {
        b.to_async(&runtime).iter(|| async {
            match upload_and_read_bytes(1024 * 1024 * 10).await {
                Ok(_) => {}
                Err(error) => println!("10mb upload and read bench failed with {:?}", error),
            }
        });
    });

    // only upload
    group.bench_function("upload 3072b", |b| {
        b.to_async(&runtime).iter(|| async {
            match upload_only(3072).await {
                Ok(_) => {}
                Err(error) => println!("3072b upload bench failed with {:?}", error),
            }
        });
    });
    group.bench_function("upload 1mb", |b| {
        b.to_async(&runtime).iter(|| async {
            match upload_only(1024 * 1024).await {
                Ok(_) => {}
                Err(error) => println!("1mb upload bench failed with {:?}", error),
            }
        });
    });
    group.bench_function("upload 10mb", |b| {
        b.to_async(&runtime).iter(|| async {
            match upload_only(1024 * 1024 * 10).await {
                Ok(_) => {}
                Err(error) => println!("10mb upload bench failed with {:?}", error),
            }
        });
    });
    group.finish()
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
