// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod register;
mod spentbook;

#[allow(unreachable_pub)]
pub use register::RegisterAddress;
#[allow(unreachable_pub)]
pub use spentbook::SpentbookAddress;

use super::{utils, Result};
use serde::{Deserialize, Serialize};
use xor_name::XorName;

/// An address of data on the network
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Debug)]
pub enum DataAddress {
    ///
    SafeKey(XorName),
    ///
    Bytes(ChunkAddress),
    ///
    Register(RegisterAddress),
    ///
    Spentbook(SpentbookAddress),
}

impl DataAddress {
    /// The xorname.
    pub fn name(&self) -> &XorName {
        match self {
            Self::SafeKey(address) => address,
            Self::Bytes(address) => address.name(),
            Self::Register(address) => address.name(),
            Self::Spentbook(address) => address.name(),
        }
    }

    /// Returns the Address serialised and encoded in z-base-32.
    pub fn encode_to_zbase32(&self) -> Result<String> {
        utils::encode(&self)
    }

    /// Creates from z-base-32 encoded string.
    pub fn decode_from_zbase32<T: AsRef<str>>(encoded: T) -> Result<Self> {
        utils::decode(encoded)
    }

    ///
    pub fn register(name: XorName, tag: u64) -> DataAddress {
        DataAddress::Register(RegisterAddress::new(name, tag))
    }

    ///
    pub fn bytes(name: XorName) -> DataAddress {
        DataAddress::Bytes(ChunkAddress(name))
    }

    ///
    pub fn safe_key(name: XorName) -> DataAddress {
        DataAddress::SafeKey(name)
    }

    ///
    pub fn spentbook(name: XorName) -> DataAddress {
        DataAddress::Spentbook(SpentbookAddress::new(name))
    }
}

/// An address of data on the network
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Debug)]
pub enum ReplicatedDataAddress {
    ///
    Chunk(ChunkAddress),
    ///
    Register(RegisterAddress),
    ///
    Spentbook(SpentbookAddress),
}

impl ReplicatedDataAddress {
    /// The xorname.
    pub fn name(&self) -> &XorName {
        match self {
            Self::Chunk(address) => address.name(),
            Self::Register(address) => address.name(),
            Self::Spentbook(address) => address.name(),
        }
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

/// Address of a Chunk.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Debug)]
pub struct ChunkAddress(pub XorName);

impl ChunkAddress {
    /// Returns the name.
    pub fn name(&self) -> &XorName {
        &self.0
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
    use crate::types::{ChunkAddress, DataAddress, Result};
    use xor_name::XorName;

    #[test]
    fn zbase32_encode_decode_chunk_address() -> Result<()> {
        let name: XorName = xor_name::rand::random();
        let chunk_addr = ChunkAddress(name);
        let address = DataAddress::Bytes(chunk_addr);
        let encoded = address.encode_to_zbase32()?;
        let decoded = DataAddress::decode_from_zbase32(&encoded)?;
        assert_eq!(address, decoded);
        Ok(())
    }
}
