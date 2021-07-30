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
