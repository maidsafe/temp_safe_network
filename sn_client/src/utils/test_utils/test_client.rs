// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{Client, ClientConfig};
use bls::SecretKey;
use eyre::{eyre, Result};
use sn_dbc::{Dbc, Owner};
use sn_interface::types::Keypair;
use std::{env, fs::read_to_string, path::PathBuf, time::Duration};
use tempfile::tempdir;

const TEST_ENV_GENESIS_DBC_PATH: &str = "TEST_ENV_GENESIS_DBC_PATH";
const DEFAULT_TEST_GENESIS_DBC_PATH: &str =
    ".safe/node/local-test-network/sn-node-genesis/genesis_dbc";

/// Create a test client without providing any specific keypair, DBC owner, `bootstrap_config`, or
/// timeout.
pub async fn create_test_client() -> Result<Client> {
    create_test_client_with(None, None, None).await
}

/// Create a test client optionally providing keypair and/or `bootstrap_config`
/// If no keypair is provided, a check is run that a balance has been generated for the client
pub async fn create_test_client_with(
    optional_keypair: Option<Keypair>,
    dbc_owner: Option<Owner>,
    timeout: Option<u64>,
) -> Result<Client> {
    let root_dir = tempdir().map_err(|e| eyre::eyre!(e.to_string()))?;
    let timeout = timeout.map(Duration::from_secs);
    // use standard wait
    let cmd_ack_wait = None;

    let config = ClientConfig::new(
        Some(root_dir.path()),
        None,
        None,
        timeout,
        timeout,
        cmd_ack_wait,
    )
    .await;
    let client = Client::new(config, optional_keypair.clone(), dbc_owner.clone()).await?;

    Ok(client)
}

/// Given a BLS secret key as a hex string, it will be deserialised to a `SecretKey`, which will
/// then be used as the basis for an `Owner`.
///
/// The conversion method was copied from the `mint-repl` example in `sn_dbc`.
pub fn get_dbc_owner_from_secret_key_hex(secret_key_hex: &str) -> Result<Owner> {
    let mut decoded_bytes = hex::decode(secret_key_hex).map_err(|e| eyre!(e))?;
    decoded_bytes.reverse(); // convert from big endian to little endian
    let sk: SecretKey = bincode::deserialize(&decoded_bytes).map_err(|e| eyre!(e))?;
    Ok(Owner::from(sk))
}

pub fn read_genesis_dbc_from_first_node() -> Result<Dbc> {
    let path = match env::var(TEST_ENV_GENESIS_DBC_PATH) {
        Ok(dir) => PathBuf::from(&dir),
        Err(_) => {
            let mut path = dirs_next::home_dir()
                .ok_or_else(|| eyre!("Failed to obtain user's home path".to_string()))?;

            path.push(DEFAULT_TEST_GENESIS_DBC_PATH);
            path
        }
    };

    let dbc_data = read_to_string(path.clone()).map_err(|err| {
        eyre!(
            "Failed to read genesis DBC file from '{}'.\
            Please set TEST_ENV_GENESIS_DBC_PATH env var with the path to the genesis dbc file: {}",
            path.display(),
            err
        )
    })?;

    Dbc::from_hex(dbc_data.trim()).map_err(|err| {
        eyre!(
            "The file read from '{}' does not appear to have genesis DBC data.\
            Please set TEST_ENV_GENESIS_DBC_PATH env var with the path to the genesis dbc file: {}",
            path.display(),
            err
        )
    })
}
