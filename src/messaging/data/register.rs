// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{CmdError, Error, QueryResponse};
use crate::messaging::data::OperationId;
use crate::types::{
    register::{Address, Entry, Register, RegisterOp, User},
    PublicKey,
};
use serde::{Deserialize, Serialize};
use xor_name::XorName;

/// [`Register`] read operations.
#[derive(Hash, Eq, PartialEq, PartialOrd, Clone, Serialize, Deserialize, Debug)]
pub enum RegisterRead {
    /// Retrieve the [`Register`] at the given address.
    ///
    /// This should eventually lead to a [`GetRegister`] response.
    ///
    /// [`GetRegister`]: QueryResponse::GetRegister
    Get(Address),
    /// Retrieve the current entries from the [`Register`] at the given address.
    ///
    /// Multiple entries occur on concurrent writes. This should eventually lead to a
    /// [`ReadRegister`] response.
    ///
    /// [`ReadRegister`]: QueryResponse::ReadRegister
    Read(Address),
    /// Retrieve the policy of the [`Register`] at the given address.
    ///
    /// This should eventually lead to a [`GetRegisterPolicy`] response.
    ///
    /// [`GetRegisterPolicy`]: QueryResponse::GetRegisterPolicy
    GetPolicy(Address),
    /// Retrieve the permissions of a given user for the [`Register`] at the given address.
    ///
    /// This should eventually lead to a [`GetRegisterUserPermissions`] response.
    ///
    /// [`GetRegisterUserPermissions`]: QueryResponse::GetRegisterUserPermissions
    GetUserPermissions {
        /// Register address.
        address: Address,
        /// User to get permissions for.
        user: User,
    },
    /// Retrieve the owner of the [`Register`] at the given address.
    ///
    /// This should eventually lead to a [`GetRegisterOwner`] response.
    ///
    /// [`GetRegisterOwner`]: QueryResponse::GetRegisterOwner
    GetOwner(Address),
}

/// A [`Register`] write operation.
#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct RegisterCmd {
    /// The operation to perform.
    pub write: RegisterWrite,
    /// A signature carrying authority to perform the operation.
    ///
    /// This will be verified against the register's owner and permissions.
    pub auth: crate::messaging::ServiceAuth,
}

/// [`Register`] write operations.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum RegisterWrite {
    /// Create a new [`Register`] on the network.
    New(Register),
    /// Edit a [`Register`].
    Edit(RegisterOp<Entry>),
    /// Delete a private [`Register`].
    ///
    /// This operation will result in an error if applied to a public register. Only private
    /// registers can be deleted, and only by their current owner(s).
    Delete(Address),
}

impl RegisterRead {
    /// Creates a Response containing an error, with the Response variant corresponding to the
    /// Request variant.
    pub fn error(&self, error: Error) -> QueryResponse {
        match *self {
            RegisterRead::Get(_) => QueryResponse::GetRegister((Err(error), self.operation_id())),
            RegisterRead::Read(_) => QueryResponse::ReadRegister((Err(error), self.operation_id())),
            RegisterRead::GetPolicy(_) => {
                QueryResponse::GetRegisterPolicy((Err(error), self.operation_id()))
            }
            RegisterRead::GetUserPermissions { .. } => {
                QueryResponse::GetRegisterUserPermissions((Err(error), self.operation_id()))
            }
            RegisterRead::GetOwner(_) => {
                QueryResponse::GetRegisterOwner((Err(error), self.operation_id()))
            }
        }
    }

    /// Returns the address of the data for request. (Scoped to Private/Public)
    pub fn dst_address(&self) -> Address {
        match self {
            RegisterRead::Get(ref address)
            | RegisterRead::Read(ref address)
            | RegisterRead::GetPolicy(ref address)
            | RegisterRead::GetUserPermissions { ref address, .. }
            | RegisterRead::GetOwner(ref address) => *address,
        }
    }

    /// Returns the xorname of the data for request.
    pub fn dst_name(&self) -> XorName {
        match self {
            RegisterRead::Get(ref address)
            | RegisterRead::Read(ref address)
            | RegisterRead::GetPolicy(ref address)
            | RegisterRead::GetUserPermissions { ref address, .. }
            | RegisterRead::GetOwner(ref address) => *address.name(),
        }
    }

    /// Retrieves the operation identifier for this response, use in tracking node liveness
    /// and responses at clients.
    /// Must be the same as the query response
    /// Right now returning result to fail for anything non-chunk, as that's all we're tracking from other nodes here just now.
    pub fn operation_id(&self) -> OperationId {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hasher;

        let mut hasher = DefaultHasher::new();

        let op_id_string = match self {
            RegisterRead::Get(ref address) => format!("Get-{:?}", address),

            RegisterRead::Read(ref address) => format!("Read-{:?}", address),

            RegisterRead::GetPolicy(ref address) => format!("GetPolicy-{:?}", address),

            RegisterRead::GetUserPermissions { ref address, .. } => {
                format!("GetUserPermissions-{:?}", address)
            }

            RegisterRead::GetOwner(ref address) => format!("GetOwner-{:?}", address),
        };

        hasher.write(op_id_string.as_bytes());

        hasher.finish()
    }
}

impl RegisterWrite {
    /// Creates a Response containing an error, with the Response variant corresponding to the
    /// Request variant.
    pub fn error(&self, error: Error) -> CmdError {
        CmdError::Data(error)
    }

    /// Returns the address of the destination for request.
    pub fn dst_name(&self) -> XorName {
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
