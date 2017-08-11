// Copyright 2016 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement.  This, along with the Licenses can be
// found in the root directory of this project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.
//
// Please review the Licences for the specific language governing permissions and limitations
// relating to use of the SAFE Network Software.

#![allow(unsafe_code)]

/// Ffi module
pub mod ffi;

use self::ffi::Permission;
use ffi_utils::{ReprC, StringError, from_c_str, vec_into_raw_parts};
use ipc::errors::IpcError;
use routing::{Action, PermissionSet, XorName};
use std::{ptr, slice};
use std::collections::{BTreeSet, HashMap};
use std::ffi::{CString, NulError};

/// IPC request
// TODO: `TransOwnership` variant
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum IpcReq {
    /// Authentication request
    Auth(AuthReq),
    /// Containers request
    Containers(ContainersReq),
    /// Unregistered client authenticator request, returning bootstrap config
    Unregistered,
    /// Share mutable data
    ShareMData(ShareMDataReq),
}

/// Represents an authorization request
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AuthReq {
    /// The application identifier for this request
    pub app: AppExchangeInfo,
    /// `true` if the app wants dedicated container for itself. `false`
    /// otherwise.
    pub app_container: bool,
    /// The list of containers it wishes to access (and desired permissions).
    pub containers: HashMap<String, BTreeSet<Permission>>,
}

/// Converts a container name + a set of permissions into an FFI
/// representation `ContainerPermissions`. You're now responsible for
/// freeing this memory once you're done.
pub fn container_perm_into_repr_c(
    cont_name: String,
    access: BTreeSet<Permission>,
) -> Result<ffi::ContainerPermissions, NulError> {
    let access_vec: Vec<_> = access.into_iter().collect();
    let (access_ptr, len, cap) = vec_into_raw_parts(access_vec);

    Ok(ffi::ContainerPermissions {
        cont_name: CString::new(cont_name)?.into_raw(),
        access: access_ptr,
        access_len: len,
        access_cap: cap,
    })
}

/// Consumes the object and returns the wrapped raw pointer
///
/// You're now responsible for freeing this memory once you're done.
pub fn containers_into_vec(
    containers: HashMap<String, BTreeSet<Permission>>,
) -> Result<Vec<ffi::ContainerPermissions>, NulError> {
    let mut container_perms = Vec::new();
    for (key, access) in containers {
        container_perms.push(container_perm_into_repr_c(key, access)?);
    }
    Ok(container_perms)
}

/// Constructs the object from a raw pointer.
///
/// After calling this function, the raw pointer is owned by the resulting
/// object.
#[allow(unsafe_code)]
pub unsafe fn containers_from_repr_c(
    raw: *const ffi::ContainerPermissions,
    len: usize,
) -> Result<HashMap<String, BTreeSet<Permission>>, IpcError> {
    let mut result = HashMap::new();
    let vec = slice::from_raw_parts(raw, len);

    for raw in vec {
        let cont_name = from_c_str(raw.cont_name)?;
        let access = slice::from_raw_parts(raw.access, raw.access_len);
        let _ = result.insert(cont_name, access.iter().cloned().collect());
    }

    Ok(result)
}

impl AuthReq {
    /// Consumes the object and returns the FFI counterpart.
    ///
    /// You're now responsible for freeing the subobjects memory once you're
    /// done.
    pub fn into_repr_c(self) -> Result<ffi::AuthReq, IpcError> {
        let AuthReq {
            app,
            app_container,
            containers,
        } = self;

        let containers = containers_into_vec(containers).map_err(StringError::from)?;
        let (containers_ptr, len, cap) = vec_into_raw_parts(containers);

        Ok(ffi::AuthReq {
            app: app.into_repr_c()?,
            app_container: app_container,
            containers: containers_ptr,
            containers_len: len,
            containers_cap: cap,
        })
    }
}

impl ReprC for AuthReq {
    type C = *const ffi::AuthReq;
    type Error = IpcError;

    /// Constructs the object from the FFI counterpart.
    ///
    /// After calling this function, the subobjects memory is owned by the
    /// resulting object.
    unsafe fn clone_from_repr_c(repr_c: *const ffi::AuthReq) -> Result<Self, IpcError> {
        Ok(AuthReq {
            app: AppExchangeInfo::clone_from_repr_c(&(*repr_c).app)?,
            app_container: (*repr_c).app_container,
            containers: containers_from_repr_c((*repr_c).containers, (*repr_c).containers_len)?,
        })
    }
}

/// Containers request
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct ContainersReq {
    /// Exchange info
    pub app: AppExchangeInfo,
    /// Requested containers
    pub containers: HashMap<String, BTreeSet<Permission>>,
}

impl ContainersReq {
    /// Consumes the object and returns the FFI counterpart.
    ///
    /// You're now responsible for freeing the subobjects memory once you're
    /// done.
    pub fn into_repr_c(self) -> Result<ffi::ContainersReq, IpcError> {
        let ContainersReq { app, containers } = self;

        let containers = containers_into_vec(containers).map_err(StringError::from)?;
        let (containers_ptr, len, cap) = vec_into_raw_parts(containers);

        Ok(ffi::ContainersReq {
            app: app.into_repr_c()?,
            containers: containers_ptr,
            containers_len: len,
            containers_cap: cap,
        })
    }
}

impl ReprC for ContainersReq {
    type C = *const ffi::ContainersReq;
    type Error = IpcError;

    /// Constructs the object from the FFI counterpart.
    ///
    /// After calling this functions, the subobjects memory is owned by the
    /// resulting object.
    unsafe fn clone_from_repr_c(repr_c: *const ffi::ContainersReq) -> Result<Self, IpcError> {
        Ok(ContainersReq {
            app: AppExchangeInfo::clone_from_repr_c(&(*repr_c).app)?,
            containers: containers_from_repr_c((*repr_c).containers, (*repr_c).containers_len)?,
        })
    }
}

/// Represents an application ID in the process of asking permissions
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct AppExchangeInfo {
    /// The ID. It must be unique.
    pub id: String,
    /// Reserved by the frontend.
    pub scope: Option<String>,
    /// The application friendly-name.
    pub name: String,
    /// The application provider/vendor (e.g. MaidSafe)
    pub vendor: String,
}

impl AppExchangeInfo {
    /// Consumes the object and returns the wrapped raw pointer
    ///
    /// You're now responsible for freeing this memory once you're done.
    pub fn into_repr_c(self) -> Result<ffi::AppExchangeInfo, IpcError> {
        let AppExchangeInfo {
            id,
            scope,
            name,
            vendor,
        } = self;

        Ok(ffi::AppExchangeInfo {
            id: CString::new(id).map_err(StringError::from)?.into_raw(),
            scope: if let Some(scope) = scope {
                CString::new(scope).map_err(StringError::from)?.into_raw()
            } else {
                ptr::null()
            },
            name: CString::new(name).map_err(StringError::from)?.into_raw(),
            vendor: CString::new(vendor).map_err(StringError::from)?.into_raw(),
        })
    }
}

impl ReprC for AppExchangeInfo {
    type C = *const ffi::AppExchangeInfo;
    type Error = IpcError;

    /// Constructs the object from a raw pointer.
    ///
    /// After calling this function, the raw pointer is owned by the resulting
    /// object.
    unsafe fn clone_from_repr_c(raw: *const ffi::AppExchangeInfo) -> Result<Self, IpcError> {
        Ok(AppExchangeInfo {
            id: from_c_str((*raw).id).map_err(StringError::from)?,
            scope: if (*raw).scope.is_null() {
                None
            } else {
                Some(from_c_str((*raw).scope).map_err(StringError::from)?)
            },
            name: from_c_str((*raw).name).map_err(StringError::from)?,
            vendor: from_c_str((*raw).vendor).map_err(StringError::from)?,
        })
    }
}

/// Represents a request to share mutable data
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ShareMDataReq {
    /// Info about the app requesting shared access
    pub app: AppExchangeInfo,
    /// List of MD names & type tags and permissions that need to be shared
    pub mdata: Vec<ShareMData>,
}

/// For use in `ShareMDataReq`. Represents a specific `MutableData` that is being shared.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ShareMData {
    /// The mutable data type.
    pub type_tag: u64,
    /// The mutable data name.
    pub name: XorName,
    /// The permissions being requested.
    pub perms: PermissionSet,
}

impl ShareMDataReq {
    /// Convert to it's C representation
    /// The returned `ffi::ShareMDataReq` contains pointers into the returned `Vec`. As such, the
    /// `Vec` *must* be kept alive until the foreign code is finished with the
    /// `ffi::ShareMDataReq`.
    pub fn into_repr_c(self) -> Result<(ffi::ShareMDataReq, Vec<ffi::ShareMData>), IpcError> {
        let mdata_repr_c: Vec<_> = self.mdata
            .into_iter()
            .map(|md| md.into_repr_c())
            .collect::<Result<_, _>>()?;
        Ok((
            ffi::ShareMDataReq {
                app: self.app.into_repr_c()?,
                mdata: mdata_repr_c.as_ptr(),
                mdata_len: mdata_repr_c.len(),
            },
            mdata_repr_c,
        ))
    }
}

impl ReprC for ShareMDataReq {
    type C = *const ffi::ShareMDataReq;
    type Error = IpcError;

    /// Constructs the object from a raw pointer.
    unsafe fn clone_from_repr_c(raw: *const ffi::ShareMDataReq) -> Result<Self, IpcError> {
        Ok(ShareMDataReq {
            app: AppExchangeInfo::clone_from_repr_c(&(*raw).app)?,
            mdata: {
                let mdata = slice::from_raw_parts((*raw).mdata, (*raw).mdata_len);
                mdata
                    .into_iter()
                    .map(|c| ShareMData::clone_from_repr_c(c))
                    .collect::<Result<_, _>>()?
            },
        })
    }
}

impl ShareMData {
    /// Convert to it's C representation
    pub fn into_repr_c(self) -> Result<ffi::ShareMData, IpcError> {
        Ok(ffi::ShareMData {
            type_tag: self.type_tag,
            name: self.name,
            perms: permission_set_into_repr_c(self.perms),
        })
    }
}

impl ReprC for ShareMData {
    type C = *const ffi::ShareMData;
    type Error = IpcError;

    /// Constructs the object from a raw pointer.
    unsafe fn clone_from_repr_c(raw: *const ffi::ShareMData) -> Result<Self, IpcError> {
        Ok(ShareMData {
            type_tag: (*raw).type_tag,
            name: (*raw).name,
            perms: permission_set_clone_from_repr_c(&(*raw).perms),
        })
    }
}

/// Convert a `PermissionSet` into it's C representation.
pub fn permission_set_into_repr_c(perms: PermissionSet) -> ffi::PermissionSet {
    ffi::PermissionSet {
        insert: perms.is_allowed(Action::Insert).unwrap_or(false),
        update: perms.is_allowed(Action::Update).unwrap_or(false),
        delete: perms.is_allowed(Action::Delete).unwrap_or(false),
        manage_permissions: perms.is_allowed(Action::ManagePermissions).unwrap_or(false),
    }
}

/// Create a `PermissionSet` from it's C representation.
pub fn permission_set_clone_from_repr_c(perms: &ffi::PermissionSet) -> PermissionSet {
    let mut pm = PermissionSet::new();

    if perms.insert {
        pm = pm.allow(Action::Insert);
    }

    if perms.update {
        pm = pm.allow(Action::Update);
    }

    if perms.delete {
        pm = pm.allow(Action::Delete);
    }

    if perms.manage_permissions {
        pm = pm.allow(Action::ManagePermissions);
    }

    pm
}

#[cfg(test)]
#[allow(unsafe_code)]
mod tests {
    use super::*;
    use ffi_utils::ReprC;
    use std::collections::HashMap;
    use std::ffi::CStr;

    #[test]
    fn container_permissions() {
        let mut cp = HashMap::new();
        let _ = cp.insert("foobar".to_string(), Default::default());

        let ffi_cp = unwrap!(containers_into_vec(cp));
        assert_eq!(ffi_cp.len(), 1);

        let cp = unsafe { unwrap!(containers_from_repr_c(ffi_cp.as_ptr(), 1)) };

        assert!(cp.contains_key("foobar"));
        assert!(unwrap!(cp.get("foobar")).is_empty());
    }

    #[test]
    fn app_exchange_info() {
        let a = AppExchangeInfo {
            id: "myid".to_string(),
            scope: Some("hi".to_string()),
            name: "bubi".to_string(),
            vendor: "hey girl".to_string(),
        };

        let ffi_a = unwrap!(a.into_repr_c());

        unsafe {
            assert_eq!(unwrap!(CStr::from_ptr(ffi_a.id).to_str()), "myid");
            assert_eq!(unwrap!(CStr::from_ptr(ffi_a.scope).to_str()), "hi");
            assert_eq!(unwrap!(CStr::from_ptr(ffi_a.name).to_str()), "bubi");
            assert_eq!(unwrap!(CStr::from_ptr(ffi_a.vendor).to_str()), "hey girl");
        }

        let mut a = unsafe { unwrap!(AppExchangeInfo::clone_from_repr_c(&ffi_a)) };

        assert_eq!(a.id, "myid");
        assert_eq!(a.scope, Some("hi".to_string()));
        assert_eq!(a.name, "bubi");
        assert_eq!(a.vendor, "hey girl");

        a.scope = None;

        let ffi_a = unwrap!(a.into_repr_c());

        unsafe {
            assert_eq!(unwrap!(CStr::from_ptr(ffi_a.id).to_str()), "myid");
            assert!(ffi_a.scope.is_null());
            assert_eq!(unwrap!(CStr::from_ptr(ffi_a.name).to_str()), "bubi");
            assert_eq!(unwrap!(CStr::from_ptr(ffi_a.vendor).to_str()), "hey girl");
        }
    }

    #[test]
    fn auth_request() {
        let app = AppExchangeInfo {
            id: "1".to_string(),
            scope: Some("2".to_string()),
            name: "3".to_string(),
            vendor: "4".to_string(),
        };

        let a = AuthReq {
            app: app,
            app_container: false,
            containers: HashMap::new(),
        };

        let ffi = unwrap!(a.into_repr_c());

        assert_eq!(ffi.app_container, false);
        assert_eq!(ffi.containers_len, 0);

        let a = unsafe { unwrap!(AuthReq::clone_from_repr_c(&ffi)) };

        assert_eq!(a.app.id, "1");
        assert_eq!(a.app.scope, Some("2".to_string()));
        assert_eq!(a.app.name, "3");
        assert_eq!(a.app.vendor, "4");
        assert_eq!(a.app_container, false);
        assert_eq!(a.containers.len(), 0);
    }

    #[test]
    fn containers_req() {
        let app = AppExchangeInfo {
            id: "1".to_string(),
            scope: Some("2".to_string()),
            name: "3".to_string(),
            vendor: "4".to_string(),
        };

        let a = ContainersReq {
            app: app,
            containers: HashMap::new(),
        };

        let ffi = unwrap!(a.into_repr_c());

        assert_eq!(ffi.containers_len, 0);

        let a = unsafe { unwrap!(ContainersReq::clone_from_repr_c(&ffi)) };

        assert_eq!(a.app.id, "1");
        assert_eq!(a.app.scope, Some("2".to_string()));
        assert_eq!(a.app.name, "3");
        assert_eq!(a.app.vendor, "4");
        assert_eq!(a.containers.len(), 0);
    }
}
