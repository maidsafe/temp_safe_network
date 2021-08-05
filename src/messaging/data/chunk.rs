// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{CmdError, Error, OperationId, QueryResponse};
use crate::types::{Chunk, ChunkAddress, PublicKey};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use xor_name::XorName;

/// [`Chunk`] read operations.
#[derive(Hash, Eq, PartialEq, PartialOrd, Clone, Serialize, Deserialize, Debug)]
pub enum ChunkRead {
    /// Retrieve a [`Chunk`] at the given address.
    ///
    /// This should eventually lead to a [`GetChunk`] response.
    ///
    /// [`GetChunk`]: QueryResponse::GetChunk
    Get(ChunkAddress),
}

/// [`Chunk`] write operations.
#[allow(clippy::large_enum_variant)]
#[derive(Hash, Eq, PartialEq, PartialOrd, Clone, Serialize, Deserialize, Debug)]
pub enum ChunkWrite {
    /// Create a new [`Chunk`] on the network.
    New(Chunk),
    /// Delete a [`PrivateChunk`] from the network.
    ///
    /// [`PrivateChunk`]: crate::types::PrivateChunk
    DeletePrivate(ChunkAddress),
}

impl ChunkRead {
    /// Creates a Response containing an error, with the Response variant corresponding to the
    /// Request variant.
    pub fn error(&self, error: Error) -> QueryResponse {
        QueryResponse::GetChunk(Err(error))
    }

    /// Returns the address of the destination for `request`.
    pub fn dst_address(&self) -> ChunkAddress {
        use ChunkRead::*;
        match self {
            Get(address) => *address,
        }
    }

    /// Returns the address of the destination for `request`.
    pub fn dst_name(&self) -> XorName {
        use ChunkRead::*;
        match self {
            Get(address) => *address.name(),
        }
    }

    /// Return operation Id of the read
    pub fn operation_id(&self) -> OperationId {
        let mut hasher = DefaultHasher::new();

        self.hash(&mut hasher);
        hasher.finish()
    }
}

impl ChunkWrite {
    /// Creates a Response containing an error, with the Response variant corresponding to the
    /// Request variant.
    pub fn error(&self, error: Error) -> CmdError {
        CmdError::Data(error)
    }

    /// Returns the address of the destination for `request`.
    pub fn dst_name(&self) -> XorName {
        use ChunkWrite::*;
        match self {
            New(ref data) => *data.name(),
            DeletePrivate(ref address) => *address.name(),
        }
    }

    /// Returns the address of the destination for `request`.
    pub fn dst_address(&self) -> ChunkAddress {
        use ChunkWrite::*;
        match self {
            New(ref data) => *data.address(),
            DeletePrivate(ref address) => *address,
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
