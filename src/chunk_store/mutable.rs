// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::chunk::{Chunk, ChunkId};
use safe_nd::{MData, MDataAddress, XorName};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

impl Chunk for MData {
    type Id = MDataAddress;
    fn id(&self) -> &Self::Id {
        match self {
            MData::Seq(ref chunk) => chunk.address(),
            MData::Unseq(ref chunk) => chunk.address(),
        }
    }
}

impl ChunkId for MDataAddress {}
