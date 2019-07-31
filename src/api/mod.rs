// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod auth;
mod constants;
mod errors;
#[cfg(feature = "scl-mock")]
mod fake_scl;
mod fetch;
mod files;
mod helpers;
mod keys;
mod nrs;
mod nrs_map;
#[cfg(not(feature = "scl-mock"))]
mod safe_client_libs;
mod safe_net;
mod wallet;
mod xorurl;

pub use errors::{Error, ResultReturn};
pub use fetch::{NrsMapContainerInfo, SafeData, SafeDataType};
pub use keys::BlsKeyPair;
pub use safe_nd::XorName;
pub use safe_net::SafeApp;
pub use xorurl::{XorUrl, XorUrlEncoder};

#[cfg(feature = "scl-mock")]
use fake_scl::SafeAppFake as SafeAppImpl;
#[cfg(not(feature = "scl-mock"))]
use safe_client_libs::SafeAppScl as SafeAppImpl;

pub struct Safe {
    safe_app: SafeAppImpl,
    xorurl_base: String,
}

#[allow(dead_code)]
impl Safe {
    pub fn new(xorurl_base: String) -> Self {
        Self {
            safe_app: SafeApp::new(),
            xorurl_base,
        }
    }
}
