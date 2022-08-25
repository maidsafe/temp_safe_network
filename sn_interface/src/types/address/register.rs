// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::super::{utils, Result, XorName};
use serde::{Deserialize, Serialize};
use std::hash::Hash;

/// Address of a Register, different from
/// a `ChunkAddress` in that it also includes a tag.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Debug)]
pub struct RegisterAddress {
    /// Name.
    pub name: XorName,
    /// Tag.
    pub tag: u64,
}

impl RegisterAddress {
    /// Constructs a new `RegisterAddress` given `name` and `tag`.
    pub fn new(name: XorName, tag: u64) -> Self {
        Self { name, tag }
    }

    /// Returns the name.
    /// This is not a unique identifier.
    pub fn name(&self) -> &XorName {
        &self.name
    }

    /// Returns the tag.
    pub fn tag(&self) -> u64 {
        self.tag
    }

    /// Returns the Address serialised and encoded in z-base-32.
    pub fn encode_to_zbase32(&self) -> Result<String> {
        utils::encode(&self)
    }

    /// Creates from z-base-32 encoded string.
    pub fn decode_from_zbase32<T: AsRef<str>>(encoded: T) -> Result<Self> {
        utils::decode(encoded)
    }
}

#[cfg(test)]
mod tests {
    use super::RegisterAddress;
    use crate::types::Result;
    use xor_name::XorName;

    #[test]
    fn zbase32_encode_decode_register_address() -> Result<()> {
        let name: XorName = xor_name::rand::random();
        let address = RegisterAddress::new(name, rand::random());
        let encoded = address.encode_to_zbase32()?;
        let decoded = RegisterAddress::decode_from_zbase32(&encoded)?;
        assert_eq!(address, decoded);
        Ok(())
    }
}
