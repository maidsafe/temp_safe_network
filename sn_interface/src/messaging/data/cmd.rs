// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{Error, RegisterCmd, SpentbookCmd};
use crate::{
    messaging::data::CmdResponse,
    types::{Chunk, DataAddress},
};
use serde::{Deserialize, Serialize};
use xor_name::XorName;

/// Data cmds - creating, updating, or removing data.
///
/// See the [`types`] module documentation for more details of the types supported by the Safe
/// Network, and their semantics.
///
/// [`types`]: crate::types
#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Debug)]
pub enum DataCmd {
    /// [`Chunk`] write operation.
    ///
    /// [`Chunk`]: crate::types::Chunk
    StoreChunk(Chunk),
    /// [`Register`] write operation.
    ///
    /// [`Register`]: crate::types::register::Register
    Register(RegisterCmd),
    /// Spentbook write operation.
    Spentbook(SpentbookCmd),
}

impl DataCmd {
    /// Returns the address of the corresponding variant.
    pub fn address(&self) -> DataAddress {
        match self {
            Self::StoreChunk(chunk) => DataAddress::Bytes(*chunk.address()),
            Self::Register(register_cmd) => DataAddress::Register(register_cmd.dst_address()),
            Self::Spentbook(spentbook_cmd) => DataAddress::Spentbook(spentbook_cmd.dst_address()),
        }
    }

    /// Returns the xorname of the data for this cmd.
    pub fn dst_name(&self) -> XorName {
        use DataCmd::*;
        match self {
            StoreChunk(c) => *c.name(),
            Register(c) => c.name(), // TODO: c.dst_id(), as to not co-locate private and public and different tags of same name.
            Spentbook(c) => c.name(),
        }
    }

    /// Creates a Response containing an error, with the Response variant corresponding to the
    /// Request.
    pub fn to_error_response(&self, error: Error) -> CmdResponse {
        use DataCmd::*;
        match self {
            StoreChunk(_) => CmdResponse::StoreChunk(Err(error)),
            Register(c) => c.to_error_response(error),
            Spentbook(c) => c.to_error_response(error),
        }
    }
}
