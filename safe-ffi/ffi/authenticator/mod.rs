// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::common;
use safe_authenticator::ipc::encode_response;

pub mod ffi_types;

use super::common::errors::{Error, Result};
use ffi_types::{authed_apps_into_repr_c, AuthedApp};
use ffi_utils::{
    catch_unwind_cb, ffi_error, FfiResult, NativeResult, OpaqueCtx, ReprC, FFI_RESULT_OK,
};
use safe_api::{Error as NativeError, SafeAuthReq, SafeAuthenticator};
use safe_core::{
    client,
    ffi::ipc::req::{AuthReq, ContainersReq},
    ipc::req::IpcReq,
    ipc::resp::IpcResp,
    ipc::{decode_msg, IpcMsg},
};
use std::{
    ffi::CString,
    os::raw::{c_char, c_void},
    time::Duration,
};
use tokio::runtime::Runtime;

#[no_mangle]
pub extern "C" fn auth_is_mock() -> bool {
    cfg!(feature = "scl-mock")
}

#[no_mangle]
pub unsafe extern "C" fn log_in(
    passphrase: *const c_char,
    password: *const c_char,
    user_data: *mut c_void,
    o_cb: extern "C" fn(
        user_data: *mut c_void,
        result: *const FfiResult,
        auth: *mut SafeAuthenticator,
    ),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let passphrase = String::clone_from_repr_c(passphrase)?;
        let password = String::clone_from_repr_c(password)?;
        let mut authenticator = SafeAuthenticator::default();
        let mut runtime = Runtime::new().expect("Failed to create runtime");
        runtime.block_on(authenticator.log_in(&passphrase, &password))?;
        runtime.shutdown_timeout(Duration::from_millis(1));
        o_cb(
            user_data.0,
            FFI_RESULT_OK,
            Box::into_raw(Box::new(authenticator)),
        );
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn log_out(
    app: *mut SafeAuthenticator,
    user_data: *mut c_void,
    o_cb: extern "C" fn(user_data: *mut c_void, result: *const FfiResult),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        (*app).log_out()?;
        o_cb(user_data.0, FFI_RESULT_OK);
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn is_logged_in(
    app: *mut SafeAuthenticator,
    user_data: *mut c_void,
    o_cb: extern "C" fn(user_data: *mut c_void, result: *const FfiResult, logged_in: bool),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let is_logged_in = (*app).is_logged_in();
        o_cb(user_data.0, FFI_RESULT_OK, is_logged_in);
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn create_acc(
    secret_key: *const c_char,
    passphrase: *const c_char,
    password: *const c_char,
    user_data: *mut c_void,
    o_cb: extern "C" fn(
        user_data: *mut c_void,
        result: *const FfiResult,
        auth: *mut SafeAuthenticator,
    ),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let secret_key = String::clone_from_repr_c(secret_key)?;
        let passphrase = String::clone_from_repr_c(passphrase)?;
        let password = String::clone_from_repr_c(password)?;
        let mut authenticator = SafeAuthenticator::default();
        let mut runtime = Runtime::new().expect("Failed to create runtime");
        runtime.block_on(authenticator.create_acc(&secret_key, &passphrase, &password))?;
        runtime.shutdown_timeout(Duration::from_millis(1));
        o_cb(
            user_data.0,
            FFI_RESULT_OK,
            Box::into_raw(Box::new(authenticator)),
        );
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn autherise_app(
    app: *mut SafeAuthenticator,
    request: *const c_char,
    is_granted: bool,
    user_data: *mut c_void,
    o_cb: extern "C" fn(user_data: *mut c_void, result: *const FfiResult, response: *const c_char),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        if is_granted {
            let request = String::clone_from_repr_c(request)?;
            let mut runtime = Runtime::new().expect("Failed to create runtime");
            let response = runtime.block_on((*app).authorise_app(&request))?;
            runtime.shutdown_timeout(Duration::from_millis(1));
            o_cb(user_data.0, FFI_RESULT_OK, CString::new(response)?.as_ptr());
        } else {
            let error = Error::from(NativeError::AuthdError("Auth denied".to_string()));
            let (error_code, description) = ffi_error!(error);
            let ffi_result = NativeResult {
                error_code,
                description: Some(description),
            }
            .into_repr_c()?;
            o_cb(user_data.0, &ffi_result, std::ptr::null());
        }
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn revoke_app(
    app: *mut SafeAuthenticator,
    app_id: *const c_char,
    user_data: *mut c_void,
    o_cb: extern "C" fn(user_data: *mut c_void, result: *const FfiResult),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let app_id = String::clone_from_repr_c(app_id)?;
        let mut runtime = Runtime::new().expect("Failed to create runtime");
        runtime.block_on((*app).revoke_app(&app_id))?;
        runtime.shutdown_timeout(Duration::from_millis(1));
        o_cb(user_data.0, FFI_RESULT_OK);
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn authd_app(
    app: *mut SafeAuthenticator,
    user_data: *mut c_void,
    o_cb: extern "C" fn(
        user_data: *mut c_void,
        result: *const FfiResult,
        apps: *const AuthedApp,
        apps_len: usize,
    ),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let mut runtime = Runtime::new().expect("Failed to create runtime");
        let authd_app_list = runtime.block_on((*app).authed_apps())?;
        runtime.shutdown_timeout(Duration::from_millis(1));
        let (apps, apps_len) = authed_apps_into_repr_c(authd_app_list)?;
        o_cb(user_data.0, FFI_RESULT_OK, apps, apps_len);
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn decode_req(
    app: *const SafeAuthenticator,
    msg: *const c_char,
    user_data: *mut c_void,
    o_auth: extern "C" fn(user_data: *mut c_void, req_id: u32, req: *const AuthReq),
    o_containers: extern "C" fn(user_data: *mut c_void, req_id: u32, req: *const ContainersReq),
    o_unregistered: extern "C" fn(
        user_data: *mut c_void,
        req_id: u32,
        extra_data: *const u8,
        extra_data_len: usize,
    ),
    o_err: extern "C" fn(user_data: *mut c_void, result: *const FfiResult),
) {
    let user_data = OpaqueCtx(user_data);

    catch_unwind_cb(user_data.0, o_err, || -> Result<_> {
        let msg = String::clone_from_repr_c(msg)?;
        let mut runtime = Runtime::new().expect("Failed to create runtime");
        let decode_result = runtime.block_on((*app).decode_req(&msg));
        runtime.shutdown_timeout(Duration::from_millis(1));
        match decode_result {
            Ok((req_id, auth_req)) => match auth_req {
                SafeAuthReq::Auth(auth_req) => {
                    let repr_c = auth_req.into_repr_c()?;
                    o_auth(user_data.0, req_id, &repr_c);
                }
                SafeAuthReq::Containers(cont_req) => {
                    let repr_c = cont_req.into_repr_c()?;
                    o_containers(user_data.0, req_id, &repr_c);
                }
                SafeAuthReq::Unregistered(extra_data) => {
                    o_unregistered(user_data.0, req_id, extra_data.as_ptr(), extra_data.len());
                }
                SafeAuthReq::ShareMData(_) => {
                    let (error_code, description) = ffi_error!(Error::from("Not supported"));
                    let ffi_result = NativeResult {
                        error_code,
                        description: Some(description),
                    };
                    o_err(user_data.0, &ffi_result.into_repr_c()?);
                }
            },
            Err(err) => {
                let error = Error::from(err);
                let (error_code, description) = ffi_error!(error);
                let ffi_result = NativeResult {
                    error_code,
                    description: Some(description),
                };
                o_err(user_data.0, &ffi_result.into_repr_c()?);
            }
        }
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn decode_auth_unregistered_req(
    msg: *const c_char,
    user_data: *mut c_void,
    o_unregistered: extern "C" fn(
        user_data: *mut c_void,
        req_id: u32,
        extra_data: *const u8,
        extra_data_len: usize,
    ),
    o_err: extern "C" fn(user_data: *mut c_void, result: *const FfiResult),
) {
    let user_data = OpaqueCtx(user_data);
    catch_unwind_cb(user_data.0, o_err, || -> Result<_> {
        let msg_str = String::clone_from_repr_c(msg)?;
        let msg = decode_msg(&msg_str)?;

        match msg {
            IpcMsg::Req {
                request: IpcReq::Unregistered(extra_data),
                req_id,
            } => {
                o_unregistered(user_data.0, req_id, extra_data.as_ptr(), extra_data.len());
            }
            _ => {
                let (error_code, description) = ffi_error!(Error::from("Invalid unregistered req"));
                let ffi_result = NativeResult {
                    error_code,
                    description: Some(description),
                };
                o_err(user_data.0, &ffi_result.into_repr_c()?);
            }
        }
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn encode_connection_info(
    req_id: u32,
    is_granted: bool,
    user_data: *mut c_void,
    o_cb: extern "C" fn(user_data: *mut c_void, result: *const FfiResult, response: *const c_char),
) {
    let user_data = OpaqueCtx(user_data);
    catch_unwind_cb(user_data.0, o_cb, || -> Result<()> {
        if is_granted {
            let bootstrap_cfg = client::bootstrap_config().map_err(|err| {
                NativeError::AuthenticatorError(format!(
                    "Failed to obtain bootstrap info for response: {}",
                    err
                ))
            })?;
            let resp = encode_response(&IpcMsg::Resp {
                req_id,
                response: IpcResp::Unregistered(Ok(bootstrap_cfg)),
            })?;
            o_cb(user_data.0, FFI_RESULT_OK, resp.as_ptr());
        } else {
            let error = Error::from(NativeError::AuthdError("Auth denied".to_string()));
            let (error_code, description) = ffi_error!(error);
            let ffi_result = NativeResult {
                error_code,
                description: Some(description),
            };
            o_cb(user_data.0, &ffi_result.into_repr_c()?, std::ptr::null());
        }
        Ok(())
    })
}
