// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{permission_set_clone_from_repr_c, permission_set_into_repr_c, AppExchangeInfo};
use crate::ffi::ipc::req as ffi;
use crate::ipc::errors::IpcError;
use ffi_utils::{vec_into_raw_parts, ReprC};
use safe_nd::MapPermissionSet;
use serde::{Deserialize, Serialize};
use std::slice;
use xor_name::XorName;

/// Represents a request to share mutable data.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ShareMapReq {
    /// Info about the app requesting shared access.
    pub app: AppExchangeInfo,
    /// List of MD names & type tags and permissions that need to be shared.
    pub map: Vec<ShareMap>,
}

/// For use in `ShareMapReq`. Represents a specific `Map` that is being shared.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ShareMap {
    /// The mutable data type.
    pub type_tag: u64,
    /// The mutable data name.
    pub name: XorName,
    /// The permissions being requested.
    pub perms: MapPermissionSet,
}

impl ShareMapReq {
    /// Construct FFI wrapper for the native Rust object, consuming self.
    pub fn into_repr_c(self) -> Result<ffi::ShareMapRequest, IpcError> {
        let map_repr_c: Vec<_> = self
            .map
            .into_iter()
            .map(ShareMap::into_repr_c)
            .collect::<Result<_, _>>()?;

        let (map, map_len) = vec_into_raw_parts(map_repr_c);

        Ok(ffi::ShareMapRequest {
            app: self.app.into_repr_c()?,
            map,
            map_len,
        })
    }
}

impl ReprC for ShareMapReq {
    type C = *const ffi::ShareMapRequest;
    type Error = IpcError;

    unsafe fn clone_from_repr_c(repr_c: Self::C) -> Result<Self, Self::Error> {
        Ok(Self {
            app: AppExchangeInfo::clone_from_repr_c(&(*repr_c).app)?,
            map: {
                let map = slice::from_raw_parts((*repr_c).map, (*repr_c).map_len);
                map.iter()
                    .map(|c| ShareMap::clone_from_repr_c(c))
                    .collect::<Result<_, _>>()?
            },
        })
    }
}

impl ShareMap {
    /// Construct FFI wrapper for the native Rust object, consuming self.
    pub fn into_repr_c(self) -> Result<ffi::ShareMap, IpcError> {
        Ok(ffi::ShareMap {
            type_tag: self.type_tag,
            name: self.name.0,
            perms: permission_set_into_repr_c(self.perms),
        })
    }
}

impl ReprC for ShareMap {
    type C = *const ffi::ShareMap;
    type Error = IpcError;

    unsafe fn clone_from_repr_c(repr_c: Self::C) -> Result<Self, Self::Error> {
        Ok(Self {
            type_tag: (*repr_c).type_tag,
            name: XorName((*repr_c).name),
            perms: permission_set_clone_from_repr_c((*repr_c).perms)?,
        })
    }
}
