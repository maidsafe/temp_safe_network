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
#[cfg(feature = "client-mock")]
mod fake_client;
mod helpers;
mod keys;
mod nrs;
mod realpath;
#[cfg(not(feature = "client-mock"))]
mod safe_client;
// mod safe_net;
mod sequence;
#[cfg(test)]
mod test_helpers;
mod xorurl_media_types;

use super::common;
use super::constants;
#[cfg(feature = "client-mock")]
use fake_client::SafeAppFakeClient as SafeAppImpl;
#[cfg(not(feature = "client-mock"))]
use safe_client::SafeAppClient;
// use safe_net::SafeApp;
use xorurl::XorUrlBase;

// The following is what's meant to be the public API

pub mod fetch;
pub mod files;
pub mod nrs_map;
pub mod wallet;
pub mod xorurl;
pub use consts::DEFAULT_XORURL_BASE;
pub use helpers::{parse_coins_amount, xorname_from_pk, KeyPair};
pub use keys::BlsKeyPair;
pub use nrs::ProcessedEntries;
pub use xor_name::{XorName, XOR_NAME_LEN};

pub struct Safe {
    safe_client: SafeAppClient,
    pub xorurl_base: XorUrlBase,
}

impl Default for Safe {
    fn default() -> Self {
        Self::new(Some(DEFAULT_XORURL_BASE))
    }
}

impl Safe {
    pub fn new(xorurl_base: Option<XorUrlBase>) -> Self {
        Self {
            safe_client: SafeAppClient::new(),
            xorurl_base: xorurl_base.unwrap_or_else(|| DEFAULT_XORURL_BASE),
        }
    }
}
