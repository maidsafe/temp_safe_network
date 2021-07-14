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
use safe_network::client::DEFAULT_QUERY_TIMEOUT;
use safe_network::types::Keypair;

use std::time::Duration;

// The following is what's meant to be the public API

pub mod fetch;
pub mod files;
pub mod multimap;
pub mod nrs;
pub mod nrs_multimap;
pub mod register;
pub use consts::DEFAULT_XORURL_BASE;
pub use helpers::parse_tokens_amount;
pub use safe_network::url::*;
pub use xor_name::{XorName, XOR_NAME_LEN};

#[derive(Clone)]
pub struct Safe {
    safe_client: SafeAppClient,
    pub xorurl_base: XorUrlBase,
}

impl Default for Safe {
    fn default() -> Self {
        Self::new(
            Some(DEFAULT_XORURL_BASE),
            Duration::from_secs(DEFAULT_QUERY_TIMEOUT),
        )
    }
}

impl Safe {
    pub fn new(xorurl_base: Option<XorUrlBase>, timeout: Duration) -> Self {
        Self {
            safe_client: SafeAppClient::new(timeout),
            xorurl_base: xorurl_base.unwrap_or(DEFAULT_XORURL_BASE),
        }
    }

    /// Generate a new random Ed25519 keypair
    pub fn keypair(&self) -> Keypair {
        let mut rng = OsRng;
        Keypair::new_ed25519(&mut rng)
    }

    /// Retrieve the keypair this instance was instantiated with, i.e. the
    /// keypair this instance uses by default to sign each outgoing message
    pub fn get_my_keypair(&self) -> Result<Keypair> {
        self.safe_client.keypair()
    }
}
