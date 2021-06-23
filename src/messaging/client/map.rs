// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{CmdError, Error, QueryResponse};
use crate::types::{
    Map, MapAddress as Address, MapEntryActions as Changes, MapPermissionSet as PermissionSet,
    PublicKey,
};
use xor_name::XorName;

use serde::{Deserialize, Serialize};

/// TODO: docs
#[derive(Hash, Eq, PartialEq, PartialOrd, Clone, Serialize, Deserialize, Debug)]
pub enum MapRead {
    /// Get Map.
    Get(Address),
    /// Get Map value.
    GetValue {
        /// Map address.
        address: Address,
        /// Key to get.
        #[serde(with = "serde_bytes")]
        key: Vec<u8>,
    },
    /// Get Map shell.
    GetShell(Address),
    /// Get Map version.
    GetVersion(Address),
    /// List Map entries.
    ListEntries(Address),
    /// List Map keys.
    ListKeys(Address),
    /// List Map values.
    ListValues(Address),
    /// List Map permissions.
    ListPermissions(Address),
    /// Get Map permissions for a user.
    ListUserPermissions {
        /// Map address.
        address: Address,
        /// User to get permissions for.
        user: PublicKey,
    },
}

///
#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct MapCmd {
    ///
    pub write: MapWrite,
    ///
    pub msg_id: crate::messaging::MessageId,
    ///
    pub client_sig: crate::messaging::ClientSigned,
    ///
    pub origin: crate::messaging::EndUser,
}

/// TODO: docs
#[allow(clippy::large_enum_variant)]
#[derive(Hash, Eq, PartialEq, PartialOrd, Clone, Serialize, Deserialize, Debug)]
pub enum MapWrite {
    /// Create new Map.
    New(Map),
    /// Delete instance.
    Delete(Address),
    /// Edit entries.
    Edit {
        /// Map address.
        address: Address,
        /// Changes to apply.
        changes: Changes,
    },
    /// Delete user permissions.
    DelUserPermissions {
        /// Map address.
        address: Address,
        /// User to delete permissions for.
        user: PublicKey,
        /// Version to delete.
        version: u64,
    },
    /// Set user permissions.
    SetUserPermissions {
        /// Map address.
        address: Address,
        /// User to set permissions for.
        user: PublicKey,
        /// New permissions.
        permissions: PermissionSet,
        /// Version to set.
        version: u64,
    },
}

impl MapRead {
    /// Creates a Response containing an error, with the Response variant corresponding to the
    /// Request variant.
    pub fn error(&self, error: Error) -> QueryResponse {
        use MapRead::*;
        match *self {
            Get(_) => QueryResponse::GetMap(Err(error)),
            GetValue { .. } => QueryResponse::GetMapValue(Err(error)),
            GetShell(_) => QueryResponse::GetMapShell(Err(error)),
            GetVersion(_) => QueryResponse::GetMapVersion(Err(error)),
            ListEntries(_) => QueryResponse::ListMapEntries(Err(error)),
            ListKeys(_) => QueryResponse::ListMapKeys(Err(error)),
            ListValues(_) => QueryResponse::ListMapValues(Err(error)),
            ListPermissions(_) => QueryResponse::ListMapPermissions(Err(error)),
            ListUserPermissions { .. } => QueryResponse::ListMapUserPermissions(Err(error)),
        }
    }

    /// Returns the address of the destination for request.
    pub fn dst_address(&self) -> XorName {
        use MapRead::*;
        match self {
            Get(ref address)
            | GetValue { ref address, .. }
            | GetShell(ref address)
            | GetVersion(ref address)
            | ListEntries(ref address)
            | ListKeys(ref address)
            | ListValues(ref address)
            | ListPermissions(ref address)
            | ListUserPermissions { ref address, .. } => *address.name(),
        }
    }
}

impl MapWrite {
    /// Creates a Response containing an error, with the Response variant corresponding to the
    /// Request variant.
    pub fn error(&self, error: Error) -> CmdError {
        CmdError::Data(error)
    }

    /// Returns the address of the destination for request.
    pub fn dst_address(&self) -> XorName {
        use MapWrite::*;
        match self {
            New(ref data) => *data.name(),
            Delete(ref address)
            | SetUserPermissions { ref address, .. }
            | DelUserPermissions { ref address, .. }
            | Edit { ref address, .. } => *address.name(),
        }
    }

    /// Returns the address of the map.
    pub fn address(&self) -> &Address {
        match self {
            Self::New(map) => map.address(),
            Self::Delete(address)
            | Self::Edit { address, .. }
            | Self::DelUserPermissions { address, .. }
            | Self::SetUserPermissions { address, .. } => address,
        }
    }

    /// Returns the owner of the data on a new map write.
    pub fn owner(&self) -> Option<PublicKey> {
        match self {
            Self::New(data) => Some(*data.owner()),
            _ => None,
        }
    }
}
