// Copyright 2016 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net
// Commercial License, version 1.0 or later, or (2) The General Public License
// (GPL), version 3, depending on which licence you accepted on initial access
// to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project
// generally, you agree to be bound by the terms of the MaidSafe Contributor
// Agreement, version 1.0.
// This, along with the Licenses can be found in the root directory of this
// project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network
// Software distributed under the GPL Licence is distributed on an "AS IS"
// BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied.
//
// Please review the Licences for the specific language governing permissions
// and limitations relating to use of the SAFE Network Software.

use routing::XorName;
use rust_sodium::crypto::{box_, secretbox, sign};
use std::mem;

/// Ffi module
pub mod ffi;

/// Errors module
mod errors;

pub use self::errors::IpcError;

use self::ffi::PermissionAccess;

// TODO: replace with `crust::Config`
/// Placeholder for `crust::Config`
#[derive(RustcEncodable, RustcDecodable, Debug, Eq, PartialEq)]
pub struct Config;

/// Represents the set of permissions for a given container
#[derive(RustcEncodable, RustcDecodable, Debug)]
pub struct ContainerPermission {
    /// The id
    pub container_key: String,
    /// The permissions
    pub access: Vec<PermissionAccess>,
}

impl ContainerPermission {
    /// Consumes the object and returns the wrapped raw pointer
    ///
    /// You're now responsible for freeing this memory once you're done.
    pub fn into_raw(self) -> *mut ffi::ContainerPermission {
        let ContainerPermission { container_key, mut access } = self;

        let ck_ptr = container_key.as_ptr();
        let ck_len = container_key.len();
        let ck_cap = container_key.capacity();

        mem::forget(container_key);

        let a_ptr = access.as_mut_ptr();
        let a_len = access.len();
        let a_cap = access.capacity();

        mem::forget(access);

        Box::into_raw(Box::new(ffi::ContainerPermission {
            container_key: ck_ptr,
            container_key_len: ck_len,
            container_key_cap: ck_cap,
            access: a_ptr,
            access_len: a_len,
            access_cap: a_cap,
        }))
    }

    /// Constructs the object from a raw pointer.
    ///
    /// After calling this function, the raw pointer is owned by the resulting
    /// object.
    #[allow(unsafe_code)]
    pub unsafe fn from_raw(raw: *mut ffi::ContainerPermission) -> Self {
        let raw = Box::from_raw(raw);
        let ck = String::from_raw_parts(raw.container_key as *mut u8,
                                        raw.container_key_len,
                                        raw.container_key_cap);
        ContainerPermission {
            container_key: ck,
            access: Vec::from_raw_parts(raw.access, raw.access_len, raw.access_cap),
        }
    }
}

/// Represents an application ID in the process of asking permissions
#[derive(RustcEncodable, RustcDecodable, Debug)]
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
    pub fn into_raw(self) -> *mut ffi::AppExchangeInfo {
        let AppExchangeInfo { id, scope, name, vendor } = self;

        let id_ptr = id.as_ptr();
        let id_len = id.len();
        let id_cap = id.capacity();

        mem::forget(id);

        let (s_ptr, s_len, s_cap) = match scope {
            Some(ref s) => (s.as_ptr(), s.len(), s.capacity()),
            None => (0 as *const u8, 0, 0),
        };

        mem::forget(scope);

        let n_ptr = name.as_ptr();
        let n_len = name.len();
        let n_cap = name.capacity();

        mem::forget(name);

        let v_ptr = vendor.as_ptr();
        let v_len = vendor.len();
        let v_cap = vendor.capacity();

        mem::forget(vendor);

        Box::into_raw(Box::new(ffi::AppExchangeInfo {
            id: id_ptr,
            id_len: id_len,
            id_cap: id_cap,
            scope: s_ptr,
            scope_len: s_len,
            scope_cap: s_cap,
            name: n_ptr,
            name_len: n_len,
            name_cap: n_cap,
            vendor: v_ptr,
            vendor_len: v_len,
            vendor_cap: v_cap,
        }))
    }

    /// Constructs the object from a raw pointer.
    ///
    /// After calling this function, the raw pointer is owned by the resulting
    /// object.
    #[allow(unsafe_code)]
    pub unsafe fn from_raw(raw: *mut ffi::AppExchangeInfo) -> Self {
        let raw = Box::from_raw(raw);
        let id = String::from_raw_parts(raw.id as *mut u8, raw.id_len, raw.id_cap);
        let scope = match (raw.scope, raw.scope_len, raw.scope_cap) {
            (p, _, _) if p.is_null() => None,
            (p, l, c) => Some(String::from_raw_parts(p as *mut u8, l, c)),
        };
        let name = String::from_raw_parts(raw.name as *mut u8, raw.name_len, raw.name_cap);
        let vendor = String::from_raw_parts(raw.vendor as *mut u8, raw.vendor_len, raw.vendor_cap);

        AppExchangeInfo {
            id: id,
            scope: scope,
            name: name,
            vendor: vendor,
        }
    }
}

/// Represents an authorization request
#[derive(RustcEncodable, RustcDecodable, Debug)]
pub struct AuthReq {
    /// The application identifier for this request
    pub app: AppExchangeInfo,
    /// `true` if the app wants dedicated container for itself. `false`
    /// otherwise.
    pub app_container: bool,
    /// The list of containers it wishes to access (and desired permissions).
    pub containers: Vec<ContainerPermission>,
}

impl AuthReq {
    /// Consumes the object and returns the FFI counterpart.
    ///
    /// You're now responsible for freeing the subobjects memory once you're
    /// done.
    pub fn into_ffi(self) -> ffi::AuthReq {
        let AuthReq { app, app_container, containers } = self;

        let mut containers: Vec<_> = containers.into_iter()
            .map(|c| c.into_raw())
            .collect();

        let c_ptr = containers.as_mut_ptr();
        let c_len = containers.len();
        let c_cap = containers.capacity();

        mem::forget(containers);

        ffi::AuthReq {
            app: app.into_raw(),
            app_container: app_container,
            containers: c_ptr,
            containers_len: c_len,
            containers_cap: c_cap,
        }
    }

    /// Constructs the object from the FFI counterpart.
    ///
    /// After calling this function, the subobjects memory is owned by the
    /// resulting object.
    #[allow(unsafe_code)]
    pub unsafe fn from_ffi(raw: ffi::AuthReq) -> Self {
        let app = AppExchangeInfo::from_raw(raw.app);
        let containers =
            Vec::from_raw_parts(raw.containers, raw.containers_len, raw.containers_cap)
                .into_iter()
                .map(|c| ContainerPermission::from_raw(c))
                .collect();
        AuthReq {
            app: app,
            app_container: raw.app_container,
            containers: containers,
        }
    }
}

/// Represents the needed keys to work with the data
#[derive(RustcEncodable, RustcDecodable, Debug, Eq, PartialEq)]
pub struct AppKeys {
    /// Owner signing public key.
    pub owner_key: sign::PublicKey,
    /// Data symmetric encryption key
    pub enc_key: secretbox::Key,
    /// Asymmetric sign public key.
    ///
    /// This is the identity of the App in the Network.
    pub sign_pk: sign::PublicKey,
    /// Asymmetric sign private key.
    pub sign_sk: sign::SecretKey,
    /// Asymmetric enc public key.
    pub enc_pk: box_::PublicKey,
    /// Asymmetric enc private key.
    pub enc_sk: box_::SecretKey,
}

impl AppKeys {
    /// Consumes the object and returns the wrapped raw pointer
    ///
    /// You're now responsible for freeing this memory once you're done.
    pub fn into_raw(self) -> *mut ffi::AppKeys {
        let AppKeys { owner_key, enc_key, sign_pk, sign_sk, enc_pk, enc_sk } = self;
        Box::into_raw(Box::new(ffi::AppKeys {
            owner_key: owner_key.0,
            enc_key: enc_key.0,
            sign_pk: sign_pk.0,
            sign_sk: sign_sk.0,
            enc_pk: enc_pk.0,
            enc_sk: enc_sk.0,
        }))
    }

    /// Constructs the object from a raw pointer.
    ///
    /// After calling this function, the raw pointer is owned by the resulting
    /// object.
    #[allow(unsafe_code)]
    pub unsafe fn from_raw(raw: *mut ffi::AppKeys) -> Self {
        let raw = Box::from_raw(raw);
        AppKeys {
            owner_key: sign::PublicKey(raw.owner_key),
            enc_key: secretbox::Key(raw.enc_key),
            sign_pk: sign::PublicKey(raw.sign_pk),
            sign_sk: sign::SecretKey(raw.sign_sk),
            enc_pk: box_::PublicKey(raw.enc_pk),
            enc_sk: box_::SecretKey(raw.enc_sk),
        }
    }
}

/// It represents the authentication response.
#[derive(RustcEncodable, RustcDecodable, Debug, PartialEq, Eq)]
pub struct AuthGranted {
    /// The access keys.
    pub access_token: AppAccessToken,
    /// The crust config.
    ///
    /// Useful to reuse bootstrap nodes and speed up access.
    pub bootstrap_config: Config,
    /// Access container
    pub access_container: Option<(XorName, u64, secretbox::Nonce)>,
}

/// Containers request
#[derive(RustcEncodable, RustcDecodable, Debug)]
pub struct ContainersReq;

/// Containers response
#[derive(RustcEncodable, RustcDecodable, Debug, Eq, PartialEq)]
pub struct ContainersGranted;

#[derive(RustcEncodable, RustcDecodable, Debug)]
/// IPC message
pub enum IpcMsg {
    /// Request
    Req {
        /// Application ID
        app_id: String,
        /// Request ID
        req_id: u32,
        /// Request
        req: IpcReq,
    },
    /// Response
    Resp {
        /// Request ID
        req_id: u32,
        /// Response
        resp: IpcResp,
    },
    /// Revoked
    Revoked {
        /// Application ID
        app_id: String,
    },
    /// Generic error like couldn't parse IpcMsg etc.
    Err(IpcError),
}

#[derive(RustcEncodable, RustcDecodable, Debug)]
/// IPC request
// TODO: `TransOwnership` variant
pub enum IpcReq {
    /// Authentication request
    Auth(AuthReq),
    /// Containers request
    Containers(ContainersReq),
}

#[derive(Debug, Eq, PartialEq, RustcEncodable, RustcDecodable)]
/// IPC response
// TODO: `TransOwnership` variant
pub enum IpcResp {
    /// Authentication
    Auth(Result<AuthGranted, IpcError>),
    /// Containers
    Containers(Result<ContainersGranted, IpcError>),
}

#[cfg(test)]
#[allow(unsafe_code)]
mod tests {
    use super::*;

    #[test]
    fn container_permission() {
        let cp = ContainerPermission {
            container_key: "foobar".to_string(),
            access: vec![],
        };

        let raw_cp = cp.into_raw();

        unsafe {
            assert_eq!((*raw_cp).container_key_len, 6);
            assert!((*raw_cp).container_key_cap >= 6);
            assert_eq!(*(*raw_cp).container_key, 'f' as u8);
            assert_eq!(*(*raw_cp).container_key.offset(1), 'o' as u8);
            assert_eq!(*(*raw_cp).container_key.offset(2), 'o' as u8);
            assert_eq!(*(*raw_cp).container_key.offset(3), 'b' as u8);
            assert_eq!(*(*raw_cp).container_key.offset(4), 'a' as u8);
            assert_eq!(*(*raw_cp).container_key.offset(5), 'r' as u8);

            assert_eq!((*raw_cp).access_len, 0);
        }

        let cp = unsafe { ContainerPermission::from_raw(raw_cp) };

        assert_eq!(cp.container_key, "foobar");
        assert_eq!(cp.access, vec![]);

        // If test runs under special mode (e.g. Valgrind) we can detect memory
        // leaks
        unsafe {
            ffi::container_permission_free(cp.into_raw());
        }
    }

    #[test]
    fn app_exchange_info() {
        let a = AppExchangeInfo {
            id: "myid".to_string(),
            scope: Some("hi".to_string()),
            name: "bubi".to_string(),
            vendor: "hey girl".to_string(),
        };

        let raw = a.into_raw();

        unsafe {
            assert_eq!((*raw).id_len, 4);
            assert_eq!((*raw).scope_len, 2);
            assert_eq!((*raw).name_len, 4);
            assert_eq!((*raw).vendor_len, 8);
        }

        let mut a = unsafe { AppExchangeInfo::from_raw(raw) };

        assert_eq!(a.id, "myid");
        assert_eq!(a.scope, Some("hi".to_string()));
        assert_eq!(a.name, "bubi");
        assert_eq!(a.vendor, "hey girl");

        a.scope = None;

        let raw = a.into_raw();

        unsafe {
            assert_eq!((*raw).id_len, 4);
            assert_eq!((*raw).scope, 0 as *const u8);
            assert_eq!((*raw).scope_len, 0);
            assert_eq!((*raw).scope_cap, 0);
            assert_eq!((*raw).name_len, 4);
            assert_eq!((*raw).vendor_len, 8);
        }

        unsafe { ffi::app_exchange_info_free(raw) };
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
            containers: vec![],
        };

        let ffi = a.into_ffi();

        assert_eq!(ffi.app_container, false);
        assert_eq!(ffi.containers_len, 0);

        let a = unsafe { AuthReq::from_ffi(ffi) };

        assert_eq!(a.app.id, "1");
        assert_eq!(a.app.scope, Some("2".to_string()));
        assert_eq!(a.app.name, "3");
        assert_eq!(a.app.vendor, "4");
        assert_eq!(a.app_container, false);
        assert_eq!(a.containers.len(), 0);

        unsafe { ffi::auth_request_drop(a.into_ffi()) };
    }
}
