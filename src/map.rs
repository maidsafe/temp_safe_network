// Copyright 2021MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{AuthorisationKind, CmdError, DataAuthKind, QueryResponse};
use sn_data_types::{
    Map, MapAddress as Address, MapEntryActions as Changes, MapPermissionSet as PermissionSet,
    PublicKey,
};
use xor_name::XorName;

use crate::Error;
use serde::{Deserialize, Serialize};
use std::fmt;

/// TODO: docs
#[derive(Hash, Eq, PartialEq, PartialOrd, Clone, Serialize, Deserialize)]
pub enum MapRead {
    /// Get Map.
    Get(Address),
    /// Get Map value.
    GetValue {
        /// Map address.
        address: Address,
        /// Key to get.
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

/// TODO: docs
#[allow(clippy::large_enum_variant)]
#[derive(Hash, Eq, PartialEq, PartialOrd, Clone, Serialize, Deserialize)]
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

    /// Returns the type of authorisation needed for the request.
    pub fn authorisation_kind(&self) -> AuthorisationKind {
        use MapRead::*;
        match *self {
            Get(_)
            | GetValue { .. }
            | GetShell(_)
            | GetVersion(_)
            | ListEntries(_)
            | ListKeys(_)
            | ListValues(_)
            | ListPermissions(_)
            | ListUserPermissions { .. } => AuthorisationKind::Data(DataAuthKind::PrivateRead),
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

impl fmt::Debug for MapRead {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use MapRead::*;
        write!(
            formatter,
            "Request::{}",
            match *self {
                Get(_) => "GetMap",
                GetValue { .. } => "GetMapValue",
                GetShell(_) => "GetMapShell",
                GetVersion(_) => "GetMapVersion",
                ListEntries(_) => "ListMapEntries",
                ListKeys(_) => "ListMapKeys",
                ListValues(_) => "ListMapValues",
                ListPermissions(_) => "ListMapPermissions",
                ListUserPermissions { .. } => "ListMapUserPermissions",
            }
        )
    }
}

impl MapWrite {
    /// Creates a Response containing an error, with the Response variant corresponding to the
    /// Request variant.
    pub fn error(&self, error: Error) -> CmdError {
        CmdError::Data(error)
    }

    /// Returns the type of authorisation needed for the request.
    pub fn authorisation_kind(&self) -> AuthorisationKind {
        AuthorisationKind::Data(DataAuthKind::Write)
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

    /// Returns the owner of the data on a New map write.
    pub fn owner(&self) -> Option<PublicKey> {
        match self {
            Self::New(data) => Some(data.owner()),
            _ => None,
        }
    }
}

impl fmt::Debug for MapWrite {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use MapWrite::*;
        write!(
            formatter,
            "Request::{}",
            match *self {
                New(_) => "NewMap",
                Delete(_) => "DeleteMap",
                SetUserPermissions { .. } => "SetMapUserPermissions",
                DelUserPermissions { .. } => "DelMapUserPermissions",
                Edit { .. } => "EditMap",
            }
        )
    }
}
