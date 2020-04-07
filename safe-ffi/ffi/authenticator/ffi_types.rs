// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::common::errors::Result;
use ffi_utils::vec_into_raw_parts;
use safe_api::AuthedAppsList as NativeAuthedAppsList;
use safe_core::{ffi::ipc::req::ContainerPermissions, ipc::req::containers_into_vec};
use std::ffi::CString;
use std::os::raw::c_char;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct AppPermissions {
    pub transfer_coins: bool,
    pub perform_mutations: bool,
    pub get_balance: bool,
}

#[repr(C)]
pub struct AuthedApp {
    pub id: *const c_char,
    pub name: *const c_char,
    pub vendor: *const c_char,
    pub app_permissions: AppPermissions,
    pub containers: *const ContainerPermissions,
    pub containers_len: usize,
    pub own_container: bool,
}

pub fn authed_apps_into_repr_c(
    authd_apps: NativeAuthedAppsList,
) -> Result<(*const AuthedApp, usize)> {
    let mut vec = Vec::with_capacity(authd_apps.len());

    for app in authd_apps {
        let container_permissions_vec = containers_into_vec(app.containers.clone().into_iter())?;
        let (containers_ptr, containers_len) = vec_into_raw_parts(container_permissions_vec);
        vec.push(AuthedApp {
            id: CString::new(app.id.to_string())?.into_raw(),
            name: CString::new(app.name.to_string())?.into_raw(),
            vendor: CString::new(app.vendor.to_string())?.into_raw(),
            app_permissions: AppPermissions {
                transfer_coins: app.app_permissions.transfer_coins,
                get_balance: app.app_permissions.get_balance,
                perform_mutations: app.app_permissions.perform_mutations,
            },
            containers: containers_ptr,
            containers_len,
            own_container: false,
        })
    }

    let (apps, apps_len) = vec_into_raw_parts(vec);
    Ok((apps, apps_len))
}
