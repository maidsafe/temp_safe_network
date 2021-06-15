// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use serde::{Deserialize, Serialize};
use sn_data_types::{ChunkAddress, MapAddress, PublicKey, SequenceAddress};
use std::collections::{BTreeMap, BTreeSet};
use xor_name::XorName;

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct ChunkMetadata {
    pub holders: BTreeSet<XorName>,
    pub owner: Option<PublicKey>,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct HolderMetadata {
    pub chunks: BTreeSet<ChunkAddress>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataExchange {
    ///
    pub chunk_data: ChunkDataExchange,
    ///
    pub map_data: MapDataExchange,
    ///
    pub seq_data: SequenceDataExchange,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkDataExchange {
    /// Full Adults register
    pub full_adults: BTreeSet<XorName>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapDataExchange(pub BTreeMap<MapAddress, sn_data_types::Map>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceDataExchange(pub BTreeMap<SequenceAddress, sn_data_types::Sequence>);
