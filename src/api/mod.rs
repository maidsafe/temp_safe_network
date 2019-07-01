// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod auth;
mod fetch;
mod files;
mod helpers;
mod keys;
#[cfg(not(feature = "scl-mock"))]
mod safe_client_libs;
#[cfg(feature = "scl-mock")]
mod scl_mock;
mod wallet;
mod xorurl;

pub use fetch::SafeData;
pub use keys::BlsKeyPair;
pub use safe_nd::{XorName, XOR_NAME_LEN};
pub use xorurl::{XorUrl, XorUrlEncoder};

#[cfg(not(feature = "scl-mock"))]
use safe_client_libs::SafeApp;

#[cfg(feature = "scl-mock")]
use scl_mock::SafeApp;

pub struct Safe {
    pub safe_app: SafeApp,
    pub xorurl_base: String,
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
