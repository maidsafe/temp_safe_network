// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::read_network_conn_info;
use crate::client::Client;
use crate::types::{Keypair, Token};
use crate::{retry_loop, retry_loop_for_pattern};
use anyhow::Result;
use std::str::FromStr;
use std::sync::Once;
use tracing_subscriber::{fmt, EnvFilter};

static INIT: Once = Once::new();

/// Initialise logger for tests, this is run only once, even if called multiple times.
pub fn init_logger() {
    INIT.call_once(|| {
        fmt()
            // NOTE: uncomment this line for pretty printed log output.
            //.pretty()
            .with_thread_names(true)
            .with_env_filter(EnvFilter::from_default_env())
            .with_target(false)
            .init()
    });
}

/// Create a test client without providing any specific keypair or bootstrap_config
pub async fn create_test_client() -> Result<Client> {
    create_test_client_with(None).await
}

/// Create a test client optionally providing keypair and/or bootstrap_config
/// If no keypair is provided, a check is run that a balance has been generated for the client
pub async fn create_test_client_with(optional_keypair: Option<Keypair>) -> Result<Client> {
    init_logger();
    let contact_info = read_network_conn_info()?;
    let query_timeout: u64 = 20; // 20 seconds
    let client = Client::new(
        optional_keypair.clone(),
        None,
        Some(contact_info),
        query_timeout,
    )
    .await?;

    if optional_keypair.is_none() {
        // get history, will only be Ok when we have _some_ history, aka test tokens
        retry_loop!(client.get_history());
        // check we have some balance, 10 test coins
        let _ = retry_loop_for_pattern!(client.get_balance(),
            Ok(balance) if *balance == Token::from_str("10")?)?;
    }

    Ok(client)
}
