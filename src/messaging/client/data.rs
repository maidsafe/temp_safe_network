// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{
    chunk::{ChunkRead, ChunkWrite},
    map::{MapRead, MapWrite},
    register::{RegisterRead, RegisterWrite},
    sequence::{SequenceRead, SequenceWrite},
    CmdError, Error, QueryResponse,
};
use crate::types::PublicKey;
use xor_name::XorName;

use serde::{Deserialize, Serialize};

/// Data command operations. Creating, updating or removing data
#[allow(clippy::large_enum_variant)]
#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Debug)]
pub enum DataCmd {
    /// Blob write operation
    Blob(ChunkWrite),
    /// Map write operation
    Map(MapWrite),
    /// Sequence write operation
    Sequence(SequenceWrite),
    /// Register write operation
    Register(RegisterWrite),
}

/// Data command operations. Creating, updating or removing data
#[allow(clippy::large_enum_variant)]
#[derive(Eq, PartialEq, PartialOrd, Ord, Clone, Serialize, Deserialize, Debug)]
pub enum PointerEdit {
    /// Map write operation
    Map,
    /// Sequence write operation
    Sequence,
    /// Register write operation
    Register,
}

/// Data command operations. Creating, updating or removing data
#[allow(clippy::large_enum_variant)]
#[derive(Eq, PartialEq, PartialOrd, Ord, Clone, Serialize, Deserialize, Debug)]
pub enum PointerEditKind {
    /// Map write operation
    Map,
    /// Sequence write operation
    Sequence,
    /// Register write operation
    Register,
}

impl DataCmd {
    /// Creates a Response containing an error, with the Response variant corresponding to the
    /// cuest variant.
    pub fn error(&self, error: Error) -> CmdError {
        use DataCmd::*;
        match self {
            Blob(c) => c.error(error),
            Map(c) => c.error(error),
            Sequence(c) => c.error(error),
            Register(c) => c.error(error),
        }
    }

    /// Returns the address of the destination for `cuest`.
    pub fn dst_address(&self) -> XorName {
        use DataCmd::*;
        match self {
            Blob(c) => c.dst_address(),
            Map(c) => c.dst_address(),
            Sequence(c) => c.dst_address(),
            Register(c) => c.dst_address(),
        }
    }

    /// Returns the owner of the data.
    pub fn owner(&self) -> Option<PublicKey> {
        match self {
            Self::Blob(write) => write.owner(),
            Self::Map(write) => write.owner(),
            Self::Sequence(write) => write.owner(),
            Self::Register(write) => write.owner(),
        }
    }
}

/// TODO: docs
#[allow(clippy::large_enum_variant)]
#[derive(Hash, Eq, PartialEq, PartialOrd, Clone, Serialize, Deserialize, Debug)]
pub enum DataQuery {
    /// TODO: docs
    Blob(ChunkRead),
    /// TODO: docs
    Map(MapRead),
    /// TODO: docs
    Sequence(SequenceRead),
    /// TODO: docs
    Register(RegisterRead),
}

impl DataQuery {
    /// Creates a Response containing an error, with the Response variant corresponding to the
    /// Request variant.
    pub fn error(&self, error: Error) -> QueryResponse {
        use DataQuery::*;
        match self {
            Blob(q) => q.error(error),
            Map(q) => q.error(error),
            Sequence(q) => q.error(error),
            Register(q) => q.error(error),
        }
    }

    /// Returns the address of the destination for `request`.
    pub fn dst_address(&self) -> XorName {
        use DataQuery::*;
        match self {
            Blob(q) => q.dst_address(),
            Map(q) => q.dst_address(),
            Sequence(q) => q.dst_address(),
            Register(q) => q.dst_address(),
        }
    }
}
