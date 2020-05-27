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
#[cfg(feature = "scl-mock")]
mod fake_scl;
mod helpers;
mod keys;
mod nrs;
mod realpath;
#[cfg(not(feature = "scl-mock"))]
mod safe_client_libs;
mod safe_net;
#[cfg(test)]
mod test_helpers;
mod xorurl_media_types;

use super::common;
use super::constants;
#[cfg(feature = "scl-mock")]
use fake_scl::SafeAppFake as SafeAppImpl;
#[cfg(not(feature = "scl-mock"))]
use safe_client_libs::SafeAppScl as SafeAppImpl;
use safe_net::SafeApp;
use xorurl::XorUrlBase;

// The following is what's meant to be the public API

pub mod fetch;
pub mod files;
pub mod nrs_map;
pub mod wallet;
pub mod xorurl;
pub use consts::DEFAULT_XORURL_BASE;
pub use keys::BlsKeyPair;
pub use nrs::ProcessedEntries;
pub use safe_nd::XorName;

pub struct Safe {
    safe_app: SafeAppImpl,
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
            safe_app: SafeApp::new(),
            xorurl_base: xorurl_base.unwrap_or_else(|| DEFAULT_XORURL_BASE),
        }
    }
}
