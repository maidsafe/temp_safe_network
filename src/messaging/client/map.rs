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

/// [`Map`] read operation.
#[derive(Hash, Eq, PartialEq, PartialOrd, Clone, Serialize, Deserialize, Debug)]
pub enum MapRead {
    /// Retrieve the [`Map`] at the given address.
    ///
    /// This should eventually lead to a [`GetMap`] response.
    ///
    /// Note that alternative map queries may be more efficient and convenient if you do not need
    /// the full value of the map.
    ///
    /// [`GetMap`]: QueryResponse::GetMap
    Get(Address),
    /// Retrieve the value at a given key from the [`Map`] at the given address.
    ///
    /// This should eventially lead to a [`GetMapValue`] response.
    ///
    /// [`GetMapValue`]: QueryResponse::GetMapValue
    GetValue {
        /// Map address.
        address: Address,
        /// Key to get.
        #[serde(with = "serde_bytes")]
        key: Vec<u8>,
    },
    /// Retrieve the 'shell' of the [`Map`] at the given address.
    ///
    /// This should eventually lead to a [`GetMapShell`] response. The [`Map`] contained in the
    /// response will have all its metadata fields set, but not the data itself.
    ///
    /// [`GetMapShell`]: QueryResponse::GetMapShell
    GetShell(Address),
    /// Retrieve the version of the [`Map`] at the given address.
    ///
    /// This should eventually lead to a [`GetMapVersion`] response.
    ///
    /// [`GetMapVersion`]: QueryResponse::GetMapVersion
    GetVersion(Address),
    /// Retrieve the data in the [`Map`] at the given address.
    ///
    /// This should eventually lead to a [`ListMapEntries`] response.
    ///
    /// [`ListMapEntries`]: QueryResponse::ListMapEntries
    ListEntries(Address),
    /// Retrieve the list of keys in the [`Map`] at the given address.
    ///
    /// This should eventually lead to a [`ListMapKeys`] response.
    ///
    /// [`ListMapKeys`]: QueryResponse::ListMapKeys
    ListKeys(Address),
    /// Retrieve the list of values in the [`Map`] at the given address.
    ///
    /// This should eventually lead to a [`ListMapValues`] response.
    ///
    /// [`ListMapValues`]: QueryResponse::ListMapValues
    ListValues(Address),
    /// Retrieve the permissions for the [`Map`] at the given address.
    ///
    /// This should eventually lead to a [`ListMapPermissions`] response.
    ///
    /// [`ListMapPermissions`]: QueryResponse::ListMapPermissions
    ListPermissions(Address),
    /// Retrieve the permissions of a given user for the [`Map`] at the given address.
    ///
    /// This should eventually lead to a [`ListMapUserPermissions`] response.
    ///
    /// [`ListMapUserPermissions`]: QueryResponse::ListMapUserPermissions
    ListUserPermissions {
        /// Map address.
        address: Address,
        /// User to get permissions for.
        user: PublicKey,
    },
}

/// A [`Map`] write operation.
#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct MapCmd {
    /// The operation to perform.
    pub write: MapWrite,
    /// The ID of the message from which the operation originated, used to send error responses.
    pub msg_id: crate::messaging::MessageId,
    /// A signature carrying authority to perform the operation.
    ///
    /// This will be verified against the map's owner and permissions.
    pub client_sig: crate::messaging::ClientSigned,
    /// The origin of the request, used to send error responses.
    pub origin: crate::messaging::EndUser,
}

/// [`Map`] write operations.
#[allow(clippy::large_enum_variant)]
#[derive(Hash, Eq, PartialEq, PartialOrd, Clone, Serialize, Deserialize, Debug)]
pub enum MapWrite {
    /// Create a new [`Map`] on the network.
    New(Map),
    /// Delete the [`Map`] at the given address.
    ///
    /// Maps can only be deleted by their owners.
    Delete(Address),
    /// Edit the entries of the [`Map`]'s at the given address.
    ///
    /// The requester must have the necessary permissions for the operation to succeed.
    Edit {
        /// Map address.
        address: Address,
        /// Changes to apply.
        changes: Changes,
    },
    /// Delete a given user's permissions from the [`Map`] at the given address.
    DelUserPermissions {
        /// Map address.
        address: Address,
        /// User to delete permissions for.
        user: PublicKey,
        /// Version to delete.
        version: u64,
    },
    /// Set a given user's permissions for the [`Map`] at the given address.
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
