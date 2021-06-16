// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{CmdError, Error, QueryResponse};
use crate::types::{Chunk, ChunkAddress, PublicKey};
use serde::{Deserialize, Serialize};
use xor_name::XorName;

/// TODO: docs
#[derive(Hash, Eq, PartialEq, PartialOrd, Clone, Serialize, Deserialize, Debug)]
pub enum ChunkRead {
    /// TODO: docs
    Get(ChunkAddress),
}

/// TODO: docs
#[allow(clippy::large_enum_variant)]
#[derive(Hash, Eq, PartialEq, PartialOrd, Clone, Serialize, Deserialize, Debug)]
pub enum ChunkWrite {
    /// TODO: docs
    New(Chunk),
    /// TODO: docs
    DeletePrivate(ChunkAddress),
}

impl ChunkRead {
    /// Creates a Response containing an error, with the Response variant corresponding to the
    /// Request variant.
    pub fn error(&self, error: Error) -> QueryResponse {
        QueryResponse::GetChunk(Err(error))
    }

    /// Returns the address of the destination for `request`.
    pub fn dst_address(&self) -> XorName {
        use ChunkRead::*;
        match self {
            Get(address) => *address.name(),
        }
    }
}

impl ChunkWrite {
    /// Creates a Response containing an error, with the Response variant corresponding to the
    /// Request variant.
    pub fn error(&self, error: Error) -> CmdError {
        CmdError::Data(error)
    }

    /// Returns the address of the destination for `request`.
    pub fn dst_address(&self) -> XorName {
        use ChunkWrite::*;
        match self {
            New(ref data) => *data.name(),
            DeletePrivate(ref address) => *address.name(),
        }
    }

    /// Returns the owner of the chunk on a new chunk write.
    pub fn owner(&self) -> Option<PublicKey> {
        match self {
            Self::New(chunk) => chunk.owner().cloned(),
            Self::DeletePrivate(_) => None,
        }
    }
}
