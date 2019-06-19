// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

#[cfg(not(feature = "scl-mock"))]
use crate::api::safe_client_libs::SafeApp;
use log::{debug, info};

#[cfg(feature = "scl-mock")]
use crate::api::scl_mock::SafeApp;
use serde::{Deserialize, Serialize};

// The XOR-URL type (in the future in can be a struct with different functions)
pub type XorUrl = String;

// We expose a BLS key pair as two hex encoded strings
// TODO: consider supporting other encodings like base32 or just expose Vec<u8>
#[derive(Clone)]
pub struct BlsKeyPair {
    pub pk: String,
    pub sk: String,
}

// Struct which is serialised and stored in Wallet MD for linking to a spendable balance (Key)
#[derive(Serialize, Deserialize, Debug)]
pub struct WalletSpendableBalance {
    pub xorurl: XorUrl,
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
