// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{Safe, SafeUrl};

use sn_dbc::Owner;
use sn_interface::types::Keypair;

use anyhow::{anyhow, Context, Result};
use bls::SecretKey;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::{collections::HashMap, env::var, ops::Index, sync::Once};
use tracing_subscriber::{fmt, EnvFilter};

// Environment variable which can be set with the auth credentials
// to be used for all sn_api tests
const TEST_AUTH_CREDENTIALS: &str = "TEST_AUTH_CREDENTIALS";

static INIT: Once = Once::new();

// Initialise logger for tests, this is run only once, even if called multiple times.
fn init_logger() {
    INIT.call_once(|| {
        fmt()
            // NOTE: comment out this line for more compact (but less readable) log output.
            // .pretty()
            .with_ansi(false)
            .with_env_filter(EnvFilter::from_default_env())
            .with_target(false)
            .init()
    });
}

pub struct TestDataFilesContainer {
    pub url: SafeUrl,
    pub files_map: HashMap<String, SafeUrl>,
}

impl TestDataFilesContainer {
    pub async fn get_container<'a>(
        files: impl IntoIterator<Item = &'a str>,
    ) -> Result<TestDataFilesContainer> {
        let mut map: HashMap<String, SafeUrl> = HashMap::new();
        let safe = new_safe_instance().await?;
        let (container_xorurl, _, files_map) = safe
            .files_container_create_from("./testdata", None, false, false)
            .await?;
        let container_url = SafeUrl::from_url(&container_xorurl)?;
        for file in files {
            let file_info = files_map
                .get(file)
                .ok_or_else(|| anyhow!(format!("could not retrieve {file} from files map")))?;
            let file_link = file_info
                .get("link")
                .ok_or_else(|| anyhow!("could not retrieve file link"))?;
            let file_url = SafeUrl::from_url(file_link)?;
            map.insert(file.to_string(), file_url);
        }
        Ok(TestDataFilesContainer {
            url: container_url,
            files_map: map,
        })
    }
}

impl Index<&str> for TestDataFilesContainer {
    type Output = SafeUrl;

    fn index(&self, file_path: &str) -> &Self::Output {
        match self.files_map.get(file_path) {
            Some(url) => url,
            None => panic!("cannot find file in files map"),
        }
    }
}

// Instantiate a Safe instance
pub async fn new_safe_instance() -> Result<Safe> {
    init_logger();
    let credentials = match var(TEST_AUTH_CREDENTIALS) {
        Ok(val) => serde_json::from_str(&val).with_context(|| {
            format!(
                "Failed to parse credentials read from {} env var",
                TEST_AUTH_CREDENTIALS
            )
        })?,
        Err(_) => Keypair::new_ed25519(),
    };
    let safe = Safe::connected(Some(credentials), None, None, None, None).await?;
    Ok(safe)
}

pub async fn new_safe_instance_with_dbc_owner(secret_key: &str) -> Result<(Safe, Owner)> {
    init_logger();
    let credentials = match var(TEST_AUTH_CREDENTIALS) {
        Ok(val) => serde_json::from_str(&val).with_context(|| {
            format!(
                "Failed to parse credentials read from {} env var",
                TEST_AUTH_CREDENTIALS
            )
        })?,
        Err(_) => Keypair::new_ed25519(),
    };

    let sk: SecretKey = bincode::deserialize(secret_key.as_bytes())
        .with_context(|| "Failed to deserialize secret key for DBC owner")?;
    let dbc_owner = Owner::from(sk);
    let safe =
        Safe::connected(Some(credentials), None, None, None, Some(dbc_owner.clone())).await?;

    Ok((safe, dbc_owner))
}

// Instantiate a Safe instance with read-only access
pub async fn new_read_only_safe_instance() -> Result<Safe> {
    init_logger();
    let safe = Safe::connected(None, None, None, None, None).await?;
    Ok(safe)
}

// Create a random NRS name
pub fn random_nrs_name() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(15)
        .map(char::from)
        .collect()
}

#[macro_export]
macro_rules! retry_loop {
    ($n:literal, $async_func:expr) => {{
        let mut retries: u64 = $n;
        loop {
            match $async_func.await {
                Ok(val) => break val,
                Err(_) if retries > 0 => {
                    retries -= 1;
                    tokio::time::sleep(std::time::Duration::from_secs(20)).await;
                }
                Err(e) => anyhow::bail!("Failed after {} retries: {:?}", $n, e),
            }
        }
    }};
    // Defaults to 10 retries if n is not provided
    ($async_func:expr) => {{
        retry_loop!(10, $async_func)
    }};
}

#[macro_export]
macro_rules! retry_loop_for_pattern {
    ($n:literal, $async_func:expr, $pattern:pat $(if $cond:expr)?) => {{
        let mut retries: u64 = $n;
        loop {
            let result = $async_func.await;
            match &result {
                $pattern $(if $cond)? => break result,
                Ok(_) | Err(_) if retries > 0 => {
                    retries -= 1;
                    tokio::time::sleep(std::time::Duration::from_secs(20)).await;
                },
                Err(e) => anyhow::bail!("Failed after {} retries: {:?}", $n, e),
                Ok(_) => anyhow::bail!("Failed to match pattern after {} retries", $n),
            }
        }
    }};
    // Defaults to 10 retries if n is not provided
    ($async_func:expr, $pattern:pat $(if $cond:expr)?) => {{
        retry_loop_for_pattern!(10, $async_func, $pattern $(if $cond)?)
    }};
}
