// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use anyhow::{anyhow, Result};
use bytes::Buf;
use sn_api::{fetch::SafeData, BootstrapConfig, Safe};
use std::{env::args, net::SocketAddr};

// To be executed passing Safe network contact address and Blob Safe URL, e.g.:
// $ cargo run --release --example fetch_blob 127.0.0.1:12000 safe://hy8oyeyqhd1e8keggcjyb9zjyje1m7ihod1pyru6h5y6jkmmihdnym4ngdf
#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    // Skip executable name form args
    let mut args_received = args();
    args_received.next();

    // Read the network contact socket address from first arg passed
    let network_contact = args_received
        .next()
        .ok_or_else(|| anyhow!("No Safe network contact socket address provided"))?;
    let network_addr: SocketAddr = network_contact
        .parse()
        .map_err(|err| anyhow!("Invalid Safe network contact socket address: {}", err))?;
    println!("Safe network to be contacted at {}", network_addr);

    // Read URL from second argument passed
    let url = args_received
        .next()
        .ok_or_else(|| anyhow!("No Safe URL provided as argument"))?;
    println!("Fetching Blob from Safe with URL: {}", url);

    // The Safe instance is what will give us access to the API.
    let mut safe = Safe::default();

    // We assume there is a local network running which we can
    // bootstrap to using the provided contact address.
    let bootstrap_contacts: BootstrapConfig = vec![network_addr].into_iter().collect();

    // Using our safe instance we connect to the network
    safe.connect(None, None, bootstrap_contacts).await?;

    println!("Connected to Safe!");

    // Now we can simply fetch the file using `fetch` API,
    // it will return not only the content of the file
    // but its metadata too, so we can distinguish what has
    // been fetched from the provided Safe-URL.
    match safe.fetch(&url, None).await {
        Ok(SafeData::PublicBlob { data, .. }) => {
            let data = String::from_utf8(data.chunk().to_vec())?;
            println!("Blob content retrieved:\n{}", data);
        }
        Ok(other) => println!("Failed to retrieve Blob, instead obtained: {:?}", other),
        Err(err) => println!("Failed to retrieve Blob: {:?}", err),
    }

    Ok(())
}
