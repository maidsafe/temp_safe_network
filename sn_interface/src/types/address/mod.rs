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

/// Address of a Chunk.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Debug)]
pub struct ChunkAddress(pub XorName);

impl ChunkAddress {
    /// Returns the name.
    pub fn name(&self) -> &XorName {
        &self.0
    }
}
