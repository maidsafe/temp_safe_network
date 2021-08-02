// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{chunk::ChunkRead, register::RegisterRead, Error, QueryResponse};
use xor_name::XorName;

use serde::{Deserialize, Serialize};

/// Data queries - retrieving data and inspecting their structure.
///
/// See the [`types`] module documentation for more details of the types supported by the Safe
/// Network, and their semantics.
///
/// [`types`]: crate::types
#[allow(clippy::large_enum_variant)]
#[derive(Hash, Eq, PartialEq, PartialOrd, Clone, Serialize, Deserialize, Debug)]
pub enum DataQuery {
    /// [`Chunk`] read operation.
    ///
    /// [`Chunk`]: crate::types::Chunk
    Chunk(ChunkRead),
    /// [`Register`] read operation.
    ///
    /// [`Register`]: crate::types::register::Register
    Register(RegisterRead),
}

impl DataQuery {
    /// Creates a Response containing an error, with the Response variant corresponding to the
    /// Request variant.
    pub fn error(&self, error: Error) -> QueryResponse {
        use DataQuery::*;
        match self {
            Chunk(q) => q.error(error),
            Register(q) => q.error(error),
        }
    }

    /// Returns the address of the destination for `request`.
    pub fn dst_address(&self) -> XorName {
        use DataQuery::*;
        match self {
            Chunk(q) => q.dst_address(),
            Register(q) => q.dst_address(),
        }
    }

    /// Retrieves the operation identifier for this response, use in tracking node liveness
    /// and responses at clients.
    /// Must be the same as the query response
    /// Right now returning result to fail for anything non-chunk, as that's all we're tracking from other nodes here just now.
    pub fn operation_id(&self) -> XorName {
        match self {
            DataQuery::Chunk(read) => read.dst_address(),
            DataQuery::Register(read) => read.operation_id(),
        }
    }
}
