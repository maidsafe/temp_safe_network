// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use serde::{Deserialize, Serialize};
use sn_dbc::Hash;
use xor_name::XorName;

use super::Error;

/// DBC reason for being spent
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct DbcReason(Hash);

impl From<XorName> for DbcReason {
    // A XorName's bytes representation is identical to Hash [u8, 32]
    fn from(value: XorName) -> Self {
        let bytes = value.0;
        DbcReason(Hash::from(bytes))
    }
}

impl From<Hash> for DbcReason {
    fn from(value: Hash) -> Self {
        DbcReason(value)
    }
}

impl From<DbcReason> for Hash {
    fn from(value: DbcReason) -> Hash {
        value.0
    }
}

impl std::str::FromStr for DbcReason {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(DbcReason::from(Hash::from_hex(s)?))
    }
}

impl DbcReason {
    pub fn is_empty(&self) -> bool {
        self == &Default::default()
    }

    pub fn none() -> Self {
        Default::default()
    }
}
