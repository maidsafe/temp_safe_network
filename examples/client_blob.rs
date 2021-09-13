// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use eyre::Result;
use safe_network::{
    client::{utils::test_utils::read_network_conn_info, Client, Config},
    types::utils::random_bytes,
    url::{ContentType, Scope, Url, DEFAULT_XORURL_BASE},
};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    println!("Reading network bootstrap information...");
    let (genesis_key, bootstrap_nodes) = read_network_conn_info()?;

    println!("Creating a Client to connect to {:?}", bootstrap_nodes);
    println!(
        "Network's genesis key: {}",
        hex::encode(genesis_key.to_bytes())
    );
    let config = Config::new(None, None, genesis_key, None, None).await;
    let client = Client::new(config, bootstrap_nodes, None).await?;

    let pk = client.public_key();
    println!("Client Public Key: {}", pk);

    let random_bytes = random_bytes(self_encryption::MIN_ENCRYPTABLE_BYTES);
    println!("Storing data.. ({} bytes)", random_bytes.len());

    let address = client.write_to_network(random_bytes, Scope::Public).await?;
    let xorurl = Url::encode_blob(
        *address.name(),
        Scope::Public,
        ContentType::Raw,
        DEFAULT_XORURL_BASE,
    )?;
    println!("Blob stored at xorurl: {}", xorurl);

    let delay = 5;
    println!("Fetching Blob from the network in {} secs...", delay);
    sleep(Duration::from_secs(delay)).await;

    println!("...fetching Blob from the network now...");
    let _ = client.read_blob(address).await?;
    println!("Blob read from {:?}", address);

    println!();

    Ok(())
}
