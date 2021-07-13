// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{CmdError, Error, QueryResponse};
use crate::types::{
    register::{Address, Entry, Register, RegisterOp, User},
    PublicKey,
};
use serde::{Deserialize, Serialize};
use std::fmt;
use xor_name::XorName;

/// Register reading queries
#[derive(Hash, Eq, PartialEq, PartialOrd, Clone, Serialize, Deserialize, Debug)]
pub enum RegisterRead {
    /// Get Register from the network.
    Get(Address),
    /// Read last entry, or entries if there are branches, from the Register.
    Read(Address),
    /// List current policy
    GetPolicy(Address),
    /// Get current permissions for a specified user(s).
    GetUserPermissions {
        /// Register address.
        address: Address,
        /// User to get permissions for.
        user: User,
    },
    /// Get current owner.
    GetOwner(Address),
}

///
#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct RegisterCmd {
    ///
    pub write: RegisterWrite,
    ///
    pub msg_id: crate::messaging::MessageId,
    ///
    pub client_sig: crate::messaging::ClientSigned,
    ///
    pub origin: crate::messaging::EndUser,
}

/// Register writing commands
#[allow(clippy::large_enum_variant)]
#[derive(Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum RegisterWrite {
    /// Create a new Register on the network.
    New(Register),
    /// Edit the Register (insert/remove entry).
    Edit(RegisterOp<Entry>),
    /// Delete a private Register.
    ///
    /// This operation MUST return an error if applied to public Register. Only the current
    /// owner(s) can perform this action.
    Delete(Address),
}

impl RegisterRead {
    /// Creates a Response containing an error, with the Response variant corresponding to the
    /// Request variant.
    pub fn error(&self, error: Error) -> QueryResponse {
        match *self {
            RegisterRead::Get(_) => QueryResponse::GetRegister(Err(error)),
            RegisterRead::Read(_) => QueryResponse::ReadRegister(Err(error)),
            RegisterRead::GetPolicy(_) => QueryResponse::GetRegisterPolicy(Err(error)),
            RegisterRead::GetUserPermissions { .. } => {
                QueryResponse::GetRegisterUserPermissions(Err(error))
            }
            RegisterRead::GetOwner(_) => QueryResponse::GetRegisterOwner(Err(error)),
        }
    }

    /// Returns the address of the destination for request.
    pub fn dst_address(&self) -> XorName {
        match self {
            RegisterRead::Get(ref address)
            | RegisterRead::Read(ref address)
            | RegisterRead::GetPolicy(ref address)
            | RegisterRead::GetUserPermissions { ref address, .. }
            | RegisterRead::GetOwner(ref address) => *address.name(),
        }
    }
}

impl RegisterWrite {
    /// Creates a Response containing an error, with the Response variant corresponding to the
    /// Request variant.
    pub fn error(&self, error: Error) -> CmdError {
        CmdError::Data(error)
    }

    /// Returns the address of the destination for request.
    pub fn dst_address(&self) -> XorName {
        match self {
            RegisterWrite::New(ref data) => *data.name(),
            RegisterWrite::Delete(ref address) => *address.name(),
            RegisterWrite::Edit(ref op) => *op.address.name(),
        }
    }

    /// Returns the address of the map.
    pub fn address(&self) -> &Address {
        match self {
            Self::New(map) => map.address(),
            Self::Delete(address) => address,
            Self::Edit(ref op) => &op.address,
        }
    }

    /// Owner of the RegisterWrite
    pub fn owner(&self) -> Option<PublicKey> {
        match self {
            Self::New(data) => Some(data.owner()),
            _ => None,
        }
    }
}

impl fmt::Debug for RegisterWrite {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "RegisterWrite::{}",
            match self {
                RegisterWrite::New(register) => format!("New({:?})", register.address()),
                RegisterWrite::Delete(address) => format!("Delete({:?})", address),
                RegisterWrite::Edit(op) => format!("Edit({:?})", op),
            }
        )
    }
}
