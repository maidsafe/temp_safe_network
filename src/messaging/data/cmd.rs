// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{chunk::ChunkWrite, register::RegisterWrite, CmdError, Error};
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
            Register(c) => c.error(error),
        }
    }

    /// Returns the xorname of the data for this command.
    pub fn dst_name(&self) -> XorName {
        use DataCmd::*;
        match self {
            Chunk(c) => c.dst_name(),
            Register(c) => c.dst_name(),
        }
    }

    /// Returns the owner of the data.
    pub fn owner(&self) -> Option<PublicKey> {
        match self {
            Self::Chunk(write) => write.owner(),
            Self::Register(write) => write.owner(),
        }
    }
}
