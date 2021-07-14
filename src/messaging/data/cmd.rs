// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{
    chunk::ChunkWrite, map::MapWrite, register::RegisterWrite, sequence::SequenceWrite, CmdError,
    Error,
};
use crate::types::PublicKey;
use xor_name::XorName;

use serde::{Deserialize, Serialize};

/// Data commands - creating, updating, or removing data.
///
/// See the [`types`] module documentation for more details of the types supported by the Safe
/// Network, and their semantics.
///
/// [`types`]: crate::types
#[allow(clippy::large_enum_variant)]
#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Debug)]
pub enum DataCmd {
    /// [`Chunk`] write operation.
    ///
    /// [`Chunk`]: crate::types::Chunk
    Chunk(ChunkWrite),
    /// [`Map`] write operation.
    ///
    /// [`Map`]: crate::types::Map
    Map(MapWrite),
    /// [`Sequence`] write operation.
    ///
    /// [`Sequence`]: crate::types::Sequence
    Sequence(SequenceWrite),
    /// [`Register`] write operation.
    ///
    /// [`Register`]: crate::types::register::Register
    Register(RegisterWrite),
}

impl DataCmd {
    /// Creates a Response containing an error, with the Response variant corresponding to the
    /// command variant.
    pub fn error(&self, error: Error) -> CmdError {
        use DataCmd::*;
        match self {
            Chunk(c) => c.error(error),
            Map(c) => c.error(error),
            Sequence(c) => c.error(error),
            Register(c) => c.error(error),
        }
    }

    /// Returns the address of the destination for command.
    pub fn dst_address(&self) -> XorName {
        use DataCmd::*;
        match self {
            Chunk(c) => c.dst_address(),
            Map(c) => c.dst_address(),
            Sequence(c) => c.dst_address(),
            Register(c) => c.dst_address(),
        }
    }

    /// Returns the owner of the data.
    pub fn owner(&self) -> Option<PublicKey> {
        match self {
            Self::Chunk(write) => write.owner(),
            Self::Map(write) => write.owner(),
            Self::Sequence(write) => write.owner(),
            Self::Register(write) => write.owner(),
        }
    }
}
