// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::read_network_conn_info;
use crate::{Client, ClientConfig};
use eyre::Result;
use sn_interface::types::Keypair;
use std::time::Duration;
use tempfile::tempdir;

/// Create a test client without providing any specific keypair, bootstrap_config, or timeout.
pub async fn create_test_client() -> Result<Client> {
    create_test_client_with(None, None, false).await
}

/// Create a test client optionally providing keypair and/or bootstrap_config
/// If no keypair is provided, a check is run that a balance has been generated for the client
pub async fn create_test_client_with(
    optional_keypair: Option<Keypair>,
    timeout: Option<u64>,
    read_prefix_map: bool,
) -> Result<Client> {
    let root_dir = tempdir().map_err(|e| eyre::eyre!(e.to_string()))?;
    let timeout = timeout.map(Duration::from_secs);
    let (genesis_key, bootstrap_nodes) = read_network_conn_info()?;

    // use standard wait
    let cmd_ack_wait = None;

    let config = ClientConfig::new(
        Some(root_dir.path()),
        None,
        genesis_key,
        None,
        timeout,
        timeout,
        cmd_ack_wait,
    )
    .await;
    let client = Client::create_with(
        config,
        bootstrap_nodes,
        optional_keypair.clone(),
        read_prefix_map,
    )
    .await?;

    Ok(client)
}
