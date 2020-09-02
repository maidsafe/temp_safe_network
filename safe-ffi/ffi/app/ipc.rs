// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::common::errors::{Error, Result};
use bincode::serialize;
use sn_ffi_utils::ffi_error;
use sn_ffi_utils::{
    catch_unwind_cb, vec_clone_from_raw_parts, FfiResult, NativeResult, ReprC, FFI_RESULT_OK,
};
use safe_core::ffi::ipc::req::{AuthReq, ContainersReq, ShareMDataRequest};
use safe_core::ffi::ipc::resp::AuthGranted;
use safe_core::ipc::{
    self, AuthReq as NativeAuthReq, ContainersReq as NativeContainersReq, IpcError, IpcMsg, IpcReq,
    IpcResp, ShareMDataReq as NativeShareMDataReq,
};
use std::ffi::CString;
use std::os::raw::{c_char, c_void};

/// Encode `AuthReq`.
#[no_mangle]
pub unsafe extern "C" fn encode_auth_req(
    req: *const AuthReq,
    user_data: *mut c_void,
    o_cb: extern "C" fn(
        user_data: *mut c_void,
        result: *const FfiResult,
        req_id: u32,
        encoded: *const c_char,
    ),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<_> {
        let req_id = ipc::gen_req_id();
        let req = NativeAuthReq::clone_from_repr_c(req)?;

        let encoded = encode_ipc(req_id, IpcReq::Auth(req))?;
        o_cb(user_data, FFI_RESULT_OK, req_id, encoded.as_ptr());
        Ok(())
    })
}

/// Encode `ContainersReq`.
#[no_mangle]
pub unsafe extern "C" fn encode_containers_req(
    req: *const ContainersReq,
    user_data: *mut c_void,
    o_cb: extern "C" fn(
        user_data: *mut c_void,
        result: *const FfiResult,
        req_id: u32,
        encoded: *const c_char,
    ),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<_> {
        let req_id = ipc::gen_req_id();
        let req = NativeContainersReq::clone_from_repr_c(req)?;

        let encoded = encode_ipc(req_id, IpcReq::Containers(req))?;
        o_cb(user_data, FFI_RESULT_OK, req_id, encoded.as_ptr());
        Ok(())
    })
}

/// Encode `AuthReq` for an unregistered client.
#[no_mangle]
pub unsafe extern "C" fn encode_unregistered_req(
    extra_data: *const u8,
    extra_data_len: usize,
    user_data: *mut c_void,
    o_cb: extern "C" fn(
        user_data: *mut c_void,
        result: *const FfiResult,
        req_id: u32,
        encoded: *const c_char,
    ),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<_> {
        let data = vec_clone_from_raw_parts(extra_data, extra_data_len);

        let req_id = ipc::gen_req_id();
        let encoded = encode_ipc(req_id, IpcReq::Unregistered(data))?;
        o_cb(user_data, FFI_RESULT_OK, req_id, encoded.as_ptr());
        Ok(())
    })
}

/// Encode `ShareMDataReq`.
#[no_mangle]
pub unsafe extern "C" fn encode_share_mdata_req(
    req: *const ShareMDataRequest,
    user_data: *mut c_void,
    o_cb: extern "C" fn(
        user_data: *mut c_void,
        result: *const FfiResult,
        req_id: u32,
        encoded: *const c_char,
    ),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<_> {
        let req_id = ipc::gen_req_id();
        let req = NativeShareMDataReq::clone_from_repr_c(req)?;

        let encoded = encode_ipc(req_id, IpcReq::ShareMData(req))?;
        o_cb(user_data, FFI_RESULT_OK, req_id, encoded.as_ptr());
        Ok(())
    })
}

fn encode_ipc(req_id: u32, req: IpcReq) -> Result<CString> {
    let encoded = ipc::encode_msg(&IpcMsg::Req {
        req_id,
        request: req,
    })?;
    Ok(CString::new(encoded)?)
}

/// Decode IPC message.
#[no_mangle]
pub unsafe extern "C" fn decode_ipc_msg(
    msg: *const c_char,
    user_data: *mut c_void,
    o_auth: extern "C" fn(user_data: *mut c_void, req_id: u32, auth_granted: *const AuthGranted),
    o_unregistered: extern "C" fn(
        user_data: *mut c_void,
        req_id: u32,
        serialised_cfg: *const u8,
        serialised_cfg_len: usize,
    ),
    o_containers: extern "C" fn(user_data: *mut c_void, req_id: u32),
    o_share_mdata: extern "C" fn(user_data: *mut c_void, req_id: u32),
    o_revoked: extern "C" fn(user_data: *mut c_void),
    o_err: extern "C" fn(user_data: *mut c_void, result: *const FfiResult, req_id: u32),
) {
    catch_unwind_cb(user_data, o_err, || -> Result<_> {
        let msg = String::clone_from_repr_c(msg)?;
        let msg = ipc::decode_msg(&msg)?;

        decode_ipc_msg_impl(
            msg,
            user_data,
            o_auth,
            o_unregistered,
            o_containers,
            o_share_mdata,
            o_revoked,
            o_err,
        )?;

        Ok(())
    })
}

#[allow(clippy::too_many_arguments)]
fn decode_ipc_msg_impl(
    msg: IpcMsg,
    user_data: *mut c_void,
    o_auth: extern "C" fn(user_data: *mut c_void, req_id: u32, auth_granted: *const AuthGranted),
    o_unregistered: extern "C" fn(
        user_data: *mut c_void,
        req_id: u32,
        serialised_cfg: *const u8,
        serialised_cfg_len: usize,
    ),
    o_containers: extern "C" fn(user_data: *mut c_void, req_id: u32),
    o_share_mdata: extern "C" fn(user_data: *mut c_void, req_id: u32),
    o_revoked: extern "C" fn(user_data: *mut c_void),
    o_err: extern "C" fn(user_data: *mut c_void, result: *const FfiResult, req_id: u32),
) -> Result<()> {
    match msg {
        IpcMsg::Resp {
            response: IpcResp::Auth(res),
            req_id,
        } => match res {
            Ok(auth_granted) => match auth_granted.into_repr_c() {
                Ok(auth_granted) => {
                    o_auth(user_data, req_id, &auth_granted);
                }
                Err(err) => {
                    let e = Error::from(err);
                    let (error_code, description) = ffi_error!(e);
                    let res = NativeResult {
                        error_code,
                        description: Some(description),
                    }
                    .into_repr_c()?;
                    o_err(user_data, &res, req_id);
                }
            },
            Err(err) => {
                let e = Error::from(err);
                let (error_code, description) = ffi_error!(e);
                let res = NativeResult {
                    error_code,
                    description: Some(description),
                }
                .into_repr_c()?;
                o_err(user_data, &res, req_id);
            }
        },
        IpcMsg::Resp {
            response: IpcResp::Containers(res),
            req_id,
        } => match res {
            Ok(()) => o_containers(user_data, req_id),
            Err(err) => {
                let e = Error::from(err);
                let (error_code, description) = ffi_error!(e);
                let res = NativeResult {
                    error_code,
                    description: Some(description),
                }
                .into_repr_c()?;
                o_err(user_data, &res, req_id);
            }
        },
        IpcMsg::Resp {
            response: IpcResp::Unregistered(res),
            req_id,
        } => match res {
            Ok(bootstrap_cfg) => {
                let serialised_cfg = serialize(&bootstrap_cfg)?;
                o_unregistered(
                    user_data,
                    req_id,
                    serialised_cfg.as_ptr(),
                    serialised_cfg.len(),
                );
            }
            Err(err) => {
                let e = Error::from(err);
                let (error_code, description) = ffi_error!(e);
                let res = NativeResult {
                    error_code,
                    description: Some(description),
                }
                .into_repr_c()?;
                o_err(user_data, &res, req_id);
            }
        },
        IpcMsg::Resp {
            response: IpcResp::ShareMData(res),
            req_id,
        } => match res {
            Ok(()) => o_share_mdata(user_data, req_id),
            Err(err) => {
                let e = Error::from(err);
                let (error_code, description) = ffi_error!(e);
                let res = NativeResult {
                    error_code,
                    description: Some(description),
                }
                .into_repr_c()?;
                o_err(user_data, &res, req_id);
            }
        },
        IpcMsg::Revoked { .. } => o_revoked(user_data),
        _ => {
            return Err(IpcError::InvalidMsg.into());
        }
    };

    Ok(())
}
