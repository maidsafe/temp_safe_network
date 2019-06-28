// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

pub mod auth;
pub mod fetch;
pub mod helpers;
pub mod keys;
#[cfg(not(feature = "scl-mock"))]
pub mod safe_client_libs;
pub mod xorurl;
// #[cfg(feature = "scl-mock")]
pub mod files;
pub mod scl_mock;
pub mod wallet;

pub use fetch::SafeData;
pub use safe_nd::{XorName, XOR_NAME_LEN};
pub use xorurl::{XorUrl, XorUrlEncoder};

#[cfg(not(feature = "scl-mock"))]
use safe_client_libs::SafeApp;

#[cfg(feature = "scl-mock")]
use scl_mock::SafeApp;

// We expose a BLS key pair as two hex encoded strings
// TODO: consider supporting other encodings like base32 or just expose Vec<u8>
#[derive(Clone)]
pub struct BlsKeyPair {
    pub pk: String,
    pub sk: String,
}

pub struct Safe {
    pub safe_app: SafeApp,
    pub xorurl_base: String,
}

impl Safe {
    pub fn new(xorurl_base: String) -> Self {
        Self {
            safe_app: SafeApp::new(),
            xorurl_base,
        }
    }
}
