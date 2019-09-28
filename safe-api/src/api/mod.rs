// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod auth;
mod constants;
pub mod errors;
#[cfg(feature = "scl-mock")]
mod fake_scl;
pub mod fetch;
pub mod files;
mod helpers;
pub mod keys;
pub mod nrs;
pub mod nrs_map;
#[cfg(not(feature = "scl-mock"))]
mod safe_client_libs;
mod safe_net;
pub mod wallet;
pub mod xorurl;
mod xorurl_media_types;

pub use errors::{Error, ResultReturn};
pub use fetch::{
    NrsMapContainerInfo, SafeContentType, SafeData, SafeDataType, WalletSpendableBalances,
};
pub use files::ProcessedFiles;
pub use keys::BlsKeyPair;
pub use nrs_map::{NrsMap, SubNamesMap};
pub use safe_nd::XorName;
pub use safe_net::SafeApp;
pub use xorurl::{XorUrl, XorUrlEncoder};

#[cfg(feature = "scl-mock")]
use fake_scl::SafeAppFake as SafeAppImpl;
#[cfg(not(feature = "scl-mock"))]
use safe_client_libs::SafeAppScl as SafeAppImpl;

pub struct Safe {
    safe_app: SafeAppImpl,
    pub xorurl_base: String,
}

#[allow(dead_code)]
impl Safe {
    pub fn new(xorurl_base: &str) -> Self {
        Self {
            safe_app: SafeApp::new(),
            xorurl_base: xorurl_base.to_string(),
        }
    }
}
