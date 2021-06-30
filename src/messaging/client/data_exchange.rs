// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{MapCmd, RegisterCmd, SequenceCmd};
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

/// Aggregate of chunk, map, and sequence data exchanges.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataExchange {
    ///
    pub chunk_data: ChunkDataExchange,
    ///
    pub map_data: MapDataExchange,
    ///
    pub reg_data: RegisterDataExchange,
    ///
    pub seq_data: SequenceDataExchange,
}

/// Chunk data exchange.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkDataExchange {
    /// Full Adults register
    pub full_adults: BTreeSet<XorName>,
}

/// Map data exchange.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapDataExchange(pub BTreeMap<XorName, Vec<MapCmd>>);

/// Map data exchange.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisterDataExchange(pub BTreeMap<XorName, Vec<RegisterCmd>>);

/// Sequence data exchange.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceDataExchange(pub BTreeMap<XorName, Vec<SequenceCmd>>);
