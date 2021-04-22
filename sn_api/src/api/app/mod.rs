// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

mod auth;
mod consts;
mod helpers;
mod keys;
mod safe_client;
mod sequence;
#[cfg(test)]
mod test_helpers;

use super::{common, constants, Result};
use rand::rngs::OsRng;
use safe_client::SafeAppClient;
use safeurl::XorUrlBase;
use sn_data_types::Keypair;
use std::time::Duration;

static DEFAULT_TIMEOUT_SECS: u64 = 20;

// The following is what's meant to be the public API

pub mod fetch;
pub mod files;
pub mod multimap;
pub mod nrs;
pub mod register;
pub mod safeurl;
pub mod wallet;
pub use consts::DEFAULT_XORURL_BASE;
pub use helpers::parse_coins_amount;
pub use xor_name::{XorName, XOR_NAME_LEN};

#[derive(Clone)]
pub struct Safe {
    safe_client: SafeAppClient,
    pub xorurl_base: XorUrlBase,
    #[allow(dead_code)]
    timeout: Duration,
}

impl Default for Safe {
    fn default() -> Self {
        Self::new(
            Some(DEFAULT_XORURL_BASE),
            Duration::from_secs(DEFAULT_TIMEOUT_SECS),
        )
    }
}

impl Safe {
    pub fn new(xorurl_base: Option<XorUrlBase>, timeout: Duration) -> Self {
        Self {
            safe_client: SafeAppClient::new(),
            xorurl_base: xorurl_base.unwrap_or(DEFAULT_XORURL_BASE),
            timeout,
        }
    }

    /// Generate a new random Ed25519 keypair
    pub fn keypair(&self) -> Keypair {
        let mut rng = OsRng;
        Keypair::new_ed25519(&mut rng)
    }

    /// Retrieve the keypair this instance was instantiated with, i.e. the
    /// keypair this instance uses by default to sign each outgoing message
    pub async fn get_my_keypair(&self) -> Result<Keypair> {
        self.safe_client.keypair().await
    }
}
