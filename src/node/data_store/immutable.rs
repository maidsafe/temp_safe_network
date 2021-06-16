// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::data::{Data, DataId};
use crate::types::{Chunk, ChunkAddress, DataAddress};

impl Data for Chunk {
    type Id = ChunkAddress;
    fn id(&self) -> &Self::Id {
        match self {
            Chunk::Public(ref chunk) => chunk.address(),
            Chunk::Private(ref chunk) => chunk.address(),
        }
    }
}

impl DataId for ChunkAddress {
    fn to_data_address(&self) -> DataAddress {
        DataAddress::Chunk(*self)
    }
}
