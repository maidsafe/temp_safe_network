// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::chunk::{Chunk, ChunkId};
use safe_nd::{
    ADataAddress, AppendOnlyData, PubSeqAppendOnlyData, PubUnseqAppendOnlyData,
    UnpubSeqAppendOnlyData, UnpubUnseqAppendOnlyData, XorName,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Hash, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub(crate) enum AppendOnlyChunk {
    PubSeq(PubSeqAppendOnlyData),
    PubUnseq(PubUnseqAppendOnlyData),
    UnpubSeq(UnpubSeqAppendOnlyData),
    UnpubUnseq(UnpubUnseqAppendOnlyData),
}

impl Chunk for AppendOnlyChunk {
    type Id = ADataAddress;
    fn id(&self) -> &Self::Id {
        match self {
            AppendOnlyChunk::PubSeq(ref chunk) => chunk.address(),
            AppendOnlyChunk::PubUnseq(ref chunk) => chunk.address(),
            AppendOnlyChunk::UnpubSeq(ref chunk) => chunk.address(),
            AppendOnlyChunk::UnpubUnseq(ref chunk) => chunk.address(),
        }
    }
}

impl ChunkId for ADataAddress {
    fn raw_name(&self) -> &XorName {
        self.name()
    }
}
