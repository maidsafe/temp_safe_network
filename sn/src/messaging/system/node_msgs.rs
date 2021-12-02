// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::types::{Chunk, PublicKey};
use crate::{
    messaging::{
        data::{DataCmd, DataExchange, DataQuery, Result, StorageLevel},
        EndUser, ServiceAuth,
    },
    types::ChunkAddress,
};
use serde::{Deserialize, Serialize};
use xor_name::XorName;

/// Command message sent among nodes
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeCmd {
    /// Metadata is handled by Elders
    Metadata {
        /// The contianed command
        cmd: DataCmd,
        /// Requester pk and signature
        auth: ServiceAuth,
        /// Message source
        origin: EndUser,
    },
    /// Chunks are stored by Adults
    StoreChunk {
        /// The chunk
        chunk: Chunk,
        /// Requester pk and signature
        auth: ServiceAuth,
        /// Message source
        origin: EndUser,
    },
    /// Notify Elders on nearing max capacity
    RecordStorageLevel {
        /// Node Id
        node_id: PublicKey,
        /// Section to which the message needs to be sent to. (NB: this is the section of the node id).
        section: XorName,
        /// The storage level reported by the node.
        level: StorageLevel,
    },
    /// Replicate a given chunk at an Adult (sent from elders on receipt of RepublishChunk)
    ReplicateChunk(Chunk),
    /// Tells the Elders to re-publish a chunk in the data section
    RepublishChunk(Chunk),
    /// Sent to all promoted nodes (also sibling if any) after
    /// a completed transition to a new constellation.
    ReceiveExistingData {
        /// Metadata
        metadata: DataExchange,
    },
}

/// Query originating at a node
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub enum NodeQuery {
    /// Metadata is handled by Elders
    Metadata {
        /// The actual query message
        query: DataQuery,
        /// Client signature
        auth: ServiceAuth,
        /// The user that has initiated this query
        origin: EndUser,
    },
    /// Chunks are handled by Adults
    GetChunk {
        /// The chunk address
        address: ChunkAddress,
        /// The user that has initiated this query
        origin: EndUser,
    },
}

///
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NodeQueryResponse {
    /// Elder to Adult Get.
    GetChunk(Result<Chunk>),
}
