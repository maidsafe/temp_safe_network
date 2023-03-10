// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! `sn_node` provides the interface to Safe routing.  The resulting executable is the node
//! for the Safe network.

use sn_client::Client;
use sn_interface::types::utils::random_bytes;

use eyre::{eyre, Result};
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

const INTERVAL_IN_MS: u64 = 10000;
const ADDITIONAL_NODE_COUNT: usize = 12;
const FILES_TO_PUT: i32 = 40;
const FILE_SIZE_LENGTH: usize = 1024 * 1024 * 10; // 10mb

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_thread_names(true)
        .with_ansi(false)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .try_init()
        .unwrap_or_else(|_| println!("Error initializing logger"));
    build_node().await?;
    run_split(std::env::var("NODE_COUNT")?.parse()?).await?;
    Ok(())
}

async fn build_node() -> Result<()> {
    println!("Building safenode");
    let args: Vec<&str> = vec!["build", "--release", "--features", "statemap"];
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
    println!("safenode built successfully");
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
        .node_launch_interval(INTERVAL_IN_MS)
        .clear_nodes_dir()
        .build()?;
    testnet.launch_genesis(None, vec![])?;
    testnet.launch_nodes(node_count, &network_contacts_path, vec![])?;
    testnet.configure_network_contacts(&network_contacts_path)?;

    let mut all_data_put = vec![];
    for _i in 0..FILES_TO_PUT {
        let (address, hash) = upload_data().await?;
        all_data_put.push((address, hash));
    }

    testnet.launch_nodes(ADDITIONAL_NODE_COUNT, &network_contacts_path, vec![])?;
    testnet.configure_network_contacts(&network_contacts_path)?;

    let post_churn_interval_to_allow_replication = Duration::from_secs(30);
    sleep(post_churn_interval_to_allow_replication).await;

    let client = Client::builder().build().await?;

    for (address, hash) in all_data_put {
        println!("...reading bytes at address {address:?} ...");
        let mut bytes = client.read_bytes(address).await;

        let mut attempts = 0;
        while bytes.is_err() && attempts < 10 {
            attempts += 1;
            println!("another attempt {attempts} ...reading bytes at address {address:?} ...",);
            // do some retries to ensure we're not just timing out by chance
            sleep(Duration::from_millis(100)).await;
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
    // Retry connecting up to 5 times, with 1.5s in between.
    let mut retries = 5;
    let client = loop {
        match Client::builder().build().await {
            Ok(client) => break client,
            Err(sn_client::Error::NetworkContacts(e)) => {
                retries -= 1;
                if retries == 0 {
                    return Err(sn_client::Error::NetworkContacts(e).into());
                }
                println!("Client connection failed, retrying in 1.5s");
                tokio::time::sleep(Duration::from_millis(1500)).await;
            }
            Err(e) => return Err(e.into()),
        }
    };

    let bytes = random_bytes(FILE_SIZE_LENGTH);

    let mut hasher = Sha3::v256();
    let mut output = [0; 32];
    hasher.update(&bytes);
    hasher.finalize(&mut output);

    println!("Storing bytes w/ hash {output:?}");

    let address = client.upload(bytes).await?;
    println!("Bytes stored at address: {address:?}");

    let delay = 300;
    println!("Reading bytes from the network in {delay} millisecs...");
    sleep(Duration::from_millis(delay)).await;

    println!("...reading bytes from the network now...");
    let mut bytes = client.read_bytes(address).await;

    let mut attempts = 0;
    while bytes.is_err() && attempts < 10 {
        attempts += 1;
        // do some retries to ensure we're not just timing out by chance
        sleep(Duration::from_millis(100)).await;
        bytes = client.read_bytes(address).await;
    }

    let _bytes = bytes?;

    println!("Bytes successfully read from {address:?}:");

    Ok((address, output))
}
