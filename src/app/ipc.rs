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

//! App-related IPC utilities.

use ipc::{AuthReq, ContainersReq, IpcError, IpcMsg, IpcReq, IpcResp};
use ipc::req::ffi::AuthReq as FfiAuthReq;
use ipc::req::ffi::ContainersReq as FfiContainersReq;
use ipc::resp::ffi::AuthGranted as FfiAuthGranted;
use maidsafe_utilities::serialisation::{deserialise, serialise};
use rand::{self, Rng};
use std::os::raw::c_void;
use std::u32;
use util;
use util::ffi::{FfiString, catch_unwind_cb, catch_unwind_error_code};

/// Generate unique request ID.
pub fn gen_req_id() -> u32 {
    // Generate the number in range 1..MAX inclusive.
    rand::thread_rng().gen_range(0, u32::MAX) + 1
}

/// Encode `AuthReq`.
#[no_mangle]
pub unsafe extern "C" fn encode_auth_req(req: FfiAuthReq,
                                         o_req_id: *mut u32,
                                         o_encoded: *mut FfiString)
                                         -> i32 {
    catch_unwind_error_code(|| -> Result<_, IpcError> {
        let req = AuthReq::from_repr_c(req)?;
        let req_id = gen_req_id();

        let msg = IpcMsg::Req {
            req_id: req_id,
            req: IpcReq::Auth(req),
        };

        let encoded = serialise(&msg)?;
        let encoded = util::base64_encode(&encoded);

        *o_req_id = req_id;
        *o_encoded = FfiString::from_string(format!("safe-auth:{}", encoded));

        Ok(())
    })
}

/// Encode `ContainersReq`.
#[no_mangle]
pub unsafe extern "C" fn encode_containers_req(req: FfiContainersReq,
                                               o_req_id: *mut u32,
                                               o_encoded: *mut FfiString)
                                               -> i32 {
    catch_unwind_error_code(|| -> Result<_, IpcError> {
        let req = ContainersReq::from_repr_c(req)?;
        let req_id = gen_req_id();

        let msg = IpcMsg::Req {
            req_id: req_id,
            req: IpcReq::Containers(req),
        };

        let encoded = serialise(&msg)?;
        let encoded = util::base64_encode(&encoded);

        *o_req_id = req_id;
        *o_encoded = FfiString::from_string(format!("safe-auth:{}", encoded));

        Ok(())
    })
}

/// Decode IPC message.
#[no_mangle]
pub unsafe extern "C" fn decode_ipc_msg(msg: FfiString,
                                        user_data: *mut c_void,
                                        o_auth: extern "C" fn(*mut c_void, u32, FfiAuthGranted),
                                        _o_containers: extern "C" fn(*mut c_void, u32),
                                        o_revoked: extern "C" fn(*mut c_void),
                                        o_err: extern "C" fn(*mut c_void, i32, u32)) {
    catch_unwind_cb(user_data, o_err, || -> Result<_, IpcError> {
        let msg = msg.as_str()?;
        let msg = util::base64_decode(msg)?;
        let msg: IpcMsg = deserialise(&msg)?;

        match msg {
            IpcMsg::Resp { resp: IpcResp::Auth(res), req_id } => {
                match res {
                    Ok(auth_granted) => {
                        let auth_granted = auth_granted.into_repr_c();
                        o_auth(user_data, req_id, auth_granted);
                    }
                    Err(err) => {
                        o_err(user_data, ffi_error_code!(err), req_id);
                    }
                }
            }
            IpcMsg::Resp { resp: IpcResp::Containers, .. } => unimplemented!(),
            IpcMsg::Revoked { .. } => o_revoked(user_data),
            _ => {
                return Err(IpcError::InvalidMsg.into());
            }
        }

        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use core::utility;
    use ipc::{AppExchangeInfo, AuthReq, IpcMsg, IpcReq};
    use maidsafe_utilities::serialisation::deserialise;
    use super::*;
    use util;
    use util::ffi::FfiString;
    use util::ffi::string::ffi_string_free;

    #[test]
    fn encode_auth_req_basics() {
        let app = AppExchangeInfo {
            id: unwrap!(utility::generate_random_string(10)),
            scope: None,
            name: "Test app".to_string(),
            vendor: "Test vendor".to_string(),
        };

        let req = AuthReq {
            app: app,
            app_container: false,
            containers: Vec::new(),
        };

        let req_c = req.clone().into_repr_c();

        let mut req_id = 0u32;
        let mut output = FfiString::default();

        let error_code = unsafe { encode_auth_req(req_c, &mut req_id, &mut output) };
        assert_eq!(error_code, 0);

        // Decode it and verify it's the same we encoded.
        let encoded = unsafe {
            let s = unwrap!(output.to_string());
            ffi_string_free(output);
            s
        };

        assert!(encoded.starts_with("safe-auth:"));
        let encoded = encoded.trim_left_matches("safe-auth:");
        let encoded = unwrap!(util::base64_decode(encoded));
        let msg: IpcMsg = unwrap!(deserialise(&encoded));

        let (decoded_req_id, decoded_req) = match msg {
            IpcMsg::Req { req_id, req: IpcReq::Auth(req) } => (req_id, req),
            x => panic!("Unexpected {:?}", x),
        };

        assert_eq!(decoded_req_id, req_id);
        assert_eq!(decoded_req, req);
    }
}
