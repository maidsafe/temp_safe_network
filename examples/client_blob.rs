// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use bytes::Bytes;
use eyre::{Context, Result};
use safe_network::{
    client::{utils::test_utils::read_network_conn_info, Client, Config},
    url::{ContentType, Scope, Url, DEFAULT_XORURL_BASE},
};
use std::{
    io::{stdout, Write},
    time::Duration,
};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    println!("Reading network bootstrap information...");
    let bootstrap_nodes = read_network_conn_info()?;

    println!("Creating a Client to connect to {:?}", bootstrap_nodes);
    let config = Config::new(None, None, None, None).await;
    let client = Client::new(config, bootstrap_nodes, None).await?;

    let pk = client.public_key();
    println!("Client Public Key: {}", pk);

    let random_num: u64 = rand::random();
    let raw_data = format!("Hello Safe World #{}", random_num);
    println!("Storing data: {}", raw_data);

    let address = client
        .write_to_network(Bytes::from(raw_data), Scope::Public)
        .await?;
    let xorurl = Url::encode_blob(
        *address.name(),
        Scope::Public,
        ContentType::Raw,
        DEFAULT_XORURL_BASE,
    )?;
    println!("Blob stored at xorurl: {}", xorurl);

    let delay = 10;
    println!("Fetching Blob from the network in {} secs...", delay);
    sleep(Duration::from_secs(delay)).await;

    println!("...fetching Blob from the network now...");
    let data = client.read_blob(address).await?;
    println!("Blob read from {:?}:", address);
    stdout()
        .write_all(&data)
        .context("Failed to print out the content of the file")?;

    println!();

    Ok(())
}
