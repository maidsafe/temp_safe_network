// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::RegisterCmd;
use crate::types::{ChunkAddress, PublicKey};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use xor_name::XorName;

/// Information about a chunk.
#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct ChunkMetadata {
    /// `XorName`s of the holders of the chunk.
    pub holders: BTreeSet<XorName>,

    /// Chunk owner.
    pub owner: Option<PublicKey>,
}

/// Information about a holder.
#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct HolderMetadata {
    /// Held chunks.
    pub chunks: BTreeSet<ChunkAddress>,
}

/// Metadata (register and chunk holders) replication.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataExchange {
    /// Chunk data exchange.
    pub chunk_data: ChunkDataExchange,
    /// Register data exchange.
    pub reg_data: RegisterDataExchange,
}

/// Chunk data exchange.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkDataExchange {
    /// Full adults.
    pub full_adults: BTreeSet<XorName>,
}

/// Register data exchange.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisterDataExchange(pub BTreeMap<XorName, Vec<RegisterCmd>>);
