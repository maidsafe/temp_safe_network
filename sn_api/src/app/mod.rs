// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

// --------------------------------------------------------------------
// ------ The following is what's meant to be the public API -------

pub mod files;
pub mod multimap;
pub mod nrs;
pub mod register;
pub mod resolver;

pub use crate::safeurl::*;
pub use consts::DEFAULT_XORURL_BASE;
pub use helpers::parse_tokens_amount;
pub use xor_name::{XorName, XOR_NAME_LEN};

// --------------------------------------------------------------------

mod auth;
mod consts;
mod helpers;
mod keys;

#[cfg(test)]
mod test_helpers;

use super::{common, constants, Error, Result};

use crate::NodeConfig;

use rand::rngs::OsRng;
use safe_network::client::{Client, ClientConfig, DEFAULT_QUERY_TIMEOUT};
use safe_network::types::Keypair;
use tracing::debug;

use std::path::Path;
use std::time::Duration;

#[derive(Clone)]
pub struct Safe {
    client: Client,
    pub xorurl_base: XorUrlBase,
    pub dry_run_mode: bool,
}

impl Safe {
    pub async fn dry_runner(
        bootstrap_config: NodeConfig,
        app_keypair: Option<Keypair>,
        config_path: Option<&Path>,
        xorurl_base: Option<XorUrlBase>,
        timeout: Option<Duration>,
    ) -> Result<Self> {
        let mut safe = Safe::connect(
            bootstrap_config,
            app_keypair,
            config_path,
            xorurl_base,
            timeout,
        )
        .await?;
        safe.dry_run_mode = true;
        Ok(safe)
    }

    /// Connect to the SAFE Network using the provided auth credentials
    pub async fn connect(
        bootstrap_config: NodeConfig,
        app_keypair: Option<Keypair>,
        config_path: Option<&Path>,
        xorurl_base: Option<XorUrlBase>,
        timeout: Option<Duration>,
    ) -> Result<Self> {
        debug!("Connecting to SAFE Network...");

        let config_path = config_path.map(|p| p.to_path_buf());

        debug!(
            "Client to be instantiated with specific pk?: {:?}",
            app_keypair
        );
        debug!("Bootstrap contacts list set to: {:?}", bootstrap_config);

        let config = ClientConfig::new(
            None,
            None,
            bootstrap_config.0,
            config_path.as_deref(),
            timeout.or(Some(DEFAULT_QUERY_TIMEOUT)),
            None,
        )
        .await;

        let safe = Self {
            client: Client::new(config, bootstrap_config.1, app_keypair)
                .await
                .map_err(|err| {
                    Error::ConnectionError(format!(
                        "Failed to connect to the SAFE Network: {:?}",
                        err
                    ))
                })?,
            xorurl_base: xorurl_base.unwrap_or(DEFAULT_XORURL_BASE),
            dry_run_mode: false,
        };

        debug!("Successfully connected to the Network!!!");

        Ok(safe)
    }

    /// Generate a new random Ed25519 keypair
    pub fn new_keypair(&self) -> Keypair {
        let mut rng = OsRng;
        Keypair::new_ed25519(&mut rng)
    }

    /// Retrieve the keypair this instance was instantiated with, i.e. the
    /// keypair this instance uses by default to sign each outgoing message
    pub fn get_my_keypair(&self) -> Keypair {
        self.client.keypair()
    }
}
