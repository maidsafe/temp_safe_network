// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use serde::{Deserialize, Serialize};
use sn_data_types::{BlobAddress, MapAddress, PublicKey, SequenceAddress};
use std::collections::{BTreeMap, BTreeSet};
use xor_name::XorName;

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct ChunkMetadata {
    pub holders: BTreeSet<XorName>,
    pub owner: Option<PublicKey>,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct HolderMetadata {
    pub chunks: BTreeSet<BlobAddress>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataExchange {
    ///
    pub blob_data: BlobDataExchange,
    ///
    pub map_data: MapDataExchange,
    ///
    pub seq_data: SequenceDataExchange,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlobDataExchange {
    /// Full Adults register
    pub full_adults: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapDataExchange(pub BTreeMap<MapAddress, sn_data_types::Map>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceDataExchange(pub BTreeMap<SequenceAddress, sn_data_types::Sequence>);
