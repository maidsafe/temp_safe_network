// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! `safenode` provides the interface to Safe routing.  The resulting executable is the node
//! for the Safe network.

use eyre::{eyre, Result};
use sn_client::Client;
use sn_interface::types::utils::random_bytes;
use sn_testnet::Testnet;
use std::{
    path::PathBuf,
    process::{Command, Stdio},
};
use tiny_keccak::{Hasher, Sha3};
use tokio::time::{sleep, Duration};
use tracing::info;
use xor_name::XorName;

#[cfg(not(target_os = "windows"))]
const SAFE_NODE_EXECUTABLE: &str = "safenode";

#[cfg(target_os = "windows")]
const SAFE_NODE_EXECUTABLE: &str = "safenode.exe";

const INTERVAL: u64 = 5000; // milliseconds
const ADDITIONAL_NODES_TO_SPLIT: usize = 15;

#[tokio::main]
async fn main() -> Result<()> {
    build_node().await?;
    run_split(std::env::var("NODE_COUNT")?.parse()?).await?;

    Ok(())
}

async fn build_node() -> Result<()> {
    println!("Building safenode");
    let args: Vec<&str> = vec!["build", "--release", "--features=test-utils"];
    let _child = Command::new("cargo")
        .args(args.clone())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|err| {
            eyre!(
                "Failed to run build command with args '{:?}': {}",
                args,
                err
            )
        })?;
    println!("safenode bins built successfully");
    Ok(())
}

pub async fn run_split(node_count: usize) -> Result<()> {
    info!("Starting local network");
    let node_bin_path = PathBuf::from(".")
        .join("target")
        .join("release")
        .join(SAFE_NODE_EXECUTABLE);
    let (mut testnet, network_contacts_path) = Testnet::configure()
        .node_bin_path(node_bin_path)
        .node_launch_interval(INTERVAL)
        .clear_nodes_dir()
        .build()?;
    testnet.launch_genesis(None, vec![])?;
    testnet.launch_nodes(node_count, &network_contacts_path, vec![])?;
    testnet.configure_network_contacts(&network_contacts_path)?;

    // leave a longer interval with more nodes to allow for splits if using split amounts
    let interval_duration = Duration::from_millis(2 * INTERVAL);
    sleep(interval_duration).await;
    println!("Done sleeping....");

    let mut all_data_put = vec![];

    let files_to_put: i32 = 40;
    for _i in 0..files_to_put {
        let (address, hash) = upload_data().await?;
        all_data_put.push((address, hash));
    }

    testnet.launch_nodes(ADDITIONAL_NODES_TO_SPLIT, &network_contacts_path, vec![])?;
    testnet.configure_network_contacts(&network_contacts_path)?;

    // leave a longer interval with more nodes to allow for splits if using split amounts
    let interval_duration = Duration::from_millis(INTERVAL * ADDITIONAL_NODES_TO_SPLIT as u64 / 10);
    sleep(interval_duration).await;

    let client = Client::builder().build().await?;

    for (address, hash) in all_data_put {
        println!("...reading bytes at address {address:?} ...");
        let mut bytes = client.read_bytes(address).await;

        let mut attempts = 0;
        while bytes.is_err() && attempts < 10 {
            attempts += 1;
            // do some retries to ensure we're not just timing out by chance
            sleep(Duration::from_secs(attempts)).await;
            bytes = client.read_bytes(address).await;
        }

        let bytes = bytes?;
        println!("Bytes read from {address:?}:");

        let mut hasher = Sha3::v256();
        let mut output = [0; 32];
        hasher.update(&bytes);
        hasher.finalize(&mut output);

        assert_eq!(output, hash);
    }

    println!("All okay");

    Ok(())
}

async fn upload_data() -> Result<(XorName, [u8; 32])> {
    let client = Client::builder().build().await?;

    let bytes = random_bytes(1024 * 1024 * 10);

    let mut hasher = Sha3::v256();
    let mut output = [0; 32];
    hasher.update(&bytes);
    hasher.finalize(&mut output);

    println!("Storing bytes w/ hash {output:?}");

    let address = client.upload(bytes).await?;
    println!("Bytes stored at address: {address:?}");

    let delay = 2;
    println!("Reading bytes from the network in {delay} secs...");
    sleep(Duration::from_secs(delay)).await;

    println!("...reading bytes from the network now...");
    let mut bytes = client.read_bytes(address).await;

    let mut attempts = 0;
    while bytes.is_err() && attempts < 10 {
        attempts += 1;
        // do some retries to ensure we're not just timing out by chance
        sleep(Duration::from_secs(attempts)).await;
        bytes = client.read_bytes(address).await;
    }

    let _bytes = bytes?;

    println!("Bytes successfully read from {address:?}:");

    Ok((address, output))
}
