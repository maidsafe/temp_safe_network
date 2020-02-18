// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

mod auth;
mod authd_client_api;
mod authenticator;
mod constants;
pub mod errors;
#[cfg(feature = "scl-mock")]
mod fake_scl;
pub mod fetch;
pub mod files;
mod helpers;
pub mod keys;
mod notifs_endpoint;
pub mod nrs;
pub mod nrs_map;
#[cfg(not(feature = "scl-mock"))]
mod safe_client_libs;
mod safe_net;
pub mod wallet;
pub mod xorurl;
mod xorurl_media_types;

pub use authd_client_api::{
    AuthAllowPrompt, AuthReq, AuthdStatus, PendingAuthReqs, SafeAuthdClient,
};
pub use authenticator::{AuthedApp, AuthedAppsList, SafeAuthReq, SafeAuthReqId, SafeAuthenticator};
use constants::DEFAULT_XORURL_BASE;
pub use errors::{Error, Result};
pub use fetch::{
    NrsMapContainerInfo, SafeContentType, SafeData, SafeDataType, WalletSpendableBalances,
};
pub use files::{FileItem, FilesMap, ProcessedFiles};
pub use keys::BlsKeyPair;
pub use nrs::ProcessedEntries;
pub use nrs_map::{NrsMap, SubNamesMap};
pub use safe_nd::XorName;
pub use safe_net::SafeApp;
pub use xorurl::{XorUrl, XorUrlBase, XorUrlEncoder};

#[cfg(feature = "scl-mock")]
use fake_scl::SafeAppFake as SafeAppImpl;
#[cfg(not(feature = "scl-mock"))]
use safe_client_libs::SafeAppScl as SafeAppImpl;

pub struct Safe {
    safe_app: SafeAppImpl,
    pub xorurl_base: XorUrlBase,
}

impl Default for Safe {
    fn default() -> Self {
        Self::new(Some(DEFAULT_XORURL_BASE))
    }
}

#[allow(dead_code)]
impl Safe {
    pub fn new(xorurl_base: Option<XorUrlBase>) -> Self {
        Self {
            safe_app: SafeApp::new(),
            xorurl_base: xorurl_base.unwrap_or_else(|| DEFAULT_XORURL_BASE),
        }
    }
}
