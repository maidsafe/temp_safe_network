// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use bytes::Buf;
use color_eyre::{eyre::eyre, Result};
use sn_api::{resolver::SafeData, PublicKey, Safe, Url};
use std::{
    collections::BTreeSet, env::temp_dir, fs::File, io::Write, net::SocketAddr, path::PathBuf,
};

const FILE_TO_UPLOAD: &str = "file_to_upload.rs";

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    // We first create a temporary file with some content,
    // which is the file we'll then upload to the network.
    let file_path = create_tmp_file()?;

    // The Safe instance is what will give us access to the API.
    let mut safe = Safe::default();

    // We assume there is a local network running which we can
    // bootstrap to using 127.0.0.1:12000 contact address.
    // We would also need to be supplied the BLS public key, or the 'genesis key' from the running
    // network. Here we just provide an example.
    let genesis_key = PublicKey::bls_from_hex("8640e62cc44e75cf4fadc8ee91b74b4cf0fd2c0984fb0e3ab40f026806857d8c41f01d3725223c55b1ef87d669f5e2cc")?
        .bls()
        .ok_or_else(|| eyre!("Unexpectedly failed to obtain (BLS) genesis key."))?;
    let mut nodes: BTreeSet<SocketAddr> = BTreeSet::new();
    nodes.insert("127.0.0.1:12000".parse()?);
    let node_config = (genesis_key, nodes);
    // Using our afe instance we connect to the network
    safe.connect(None, None, node_config).await?;

    // We can now upload the file to the network, using the following information
    let location = file_path.display().to_string();
    let dest = None; // root path at destination container
    let recursive = false; // do not do a recursive look up of files on local path
    let follow_links = false; // do not attempt to follow local links
    let dry_run = false; // commit the operation on the network

    println!("Uploading '{}' to Safe ...", location);
    let (xorurl, _, _) = safe
        .files_container_create(Some(&location), dest, recursive, follow_links, dry_run)
        .await?;
    // The 'files_container_create' API returns (among other information) the
    // XOR-URL of the FilesContainer where the file was uplaoded to
    println!("\nFile '{}' uploaded to Safe at {}", location, xorurl);

    // We give the network a moment to make sure nodes get in sync
    std::thread::sleep(std::time::Duration::from_millis(300));

    // Using the FilesContainer XOR-URL we can construct the Safe-URL of
    // the file by post fixing it with its path name,
    // i.e. safe://<FilesContainer XOR-URL>/<file name>
    let mut url = Url::from_url(&xorurl)?;
    url.set_path(FILE_TO_UPLOAD);
    println!("\nRetrieving file from {} ...\n", url);

    // Now we can simly fetch the file using `fetch` API,
    // it will return not only thee content of the file
    // but its metadata too so we can distinguish what has
    // been fetched from the provided Safe-URL.
    let fetched = safe.fetch(&url.to_string(), None).await;
    if let Ok(SafeData::PublicBlob { data, .. }) = fetched {
        println!(
            "Content retrieved:\n{}",
            String::from_utf8(data.chunk().to_vec())?
        );
    } else {
        println!("Failed to retrieve Blob, obtained: {:?}", fetched);
    }

    Ok(())
}

// Creates a temporary file
fn create_tmp_file() -> Result<PathBuf> {
    let tmp_dir = temp_dir();
    let file_path = tmp_dir.join(FILE_TO_UPLOAD);
    let mut file = File::create(&file_path)?;
    writeln!(file, "Hello Safe!")?;

    Ok(file_path)
}
