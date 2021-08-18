// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use criterion::{criterion_group, criterion_main, Criterion};
use eyre::Result;
use safe_network::client::utils::random_bytes;
use safe_network::client::utils::test_utils::{read_network_conn_info, run_w_backoff_delayed};
use safe_network::client::{Client, Config, Error};
use safe_network::url::Scope;
use tokio::runtime::Runtime;

/// This bench requires a network already set up
async fn put_kbs(amount: usize) -> Result<(), Error> {
    let contact_info = read_network_conn_info().unwrap().into_iter().collect();
    let size = 1024 * amount;
    let data = random_bytes(size);
    let config = Config::new(None, Some(contact_info), None, None).await;
    let client = Client::new(None, config).await?;
    let address = client.write_to_network(data.clone(), Scope::Public).await?;

    // let's make sure the public chunk is stored
    let received_data = run_w_backoff_delayed(|| client.read_blob(address), 10).await?;

    assert_eq!(received_data, data);

    Ok(())
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("put-sampling");

    let runtime = Runtime::new().unwrap();
    group.sample_size(10);
    group.bench_function("put 1kb", |b| {
        b.to_async(&runtime).iter(|| async {
            match put_kbs(1).await {
                Ok(_) => {}
                Err(error) => println!("bench failed with {:?}", error),
            }
        });
    });
    group.bench_function("put 1mb", |b| {
        b.to_async(&runtime).iter(|| async {
            match put_kbs(1024).await {
                Ok(_) => {}
                Err(error) => println!("bench failed with {:?}", error),
            }
        });
    });
    group.finish()
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
