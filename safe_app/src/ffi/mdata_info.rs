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

use errors::AppError;
use ffi_utils::{FFI_RESULT_OK, FfiResult, ReprC, SafePtr, catch_unwind_cb};
use maidsafe_utilities::serialisation::{deserialise, serialise};
use routing::XorName;
use rust_sodium::crypto::secretbox;
use safe_core::MDataInfo;
use safe_core::ffi::{self, SymNonce, SymSecretKey, XorNameArray};
use std::os::raw::c_void;
use std::slice;

/// Create non-encrypted mdata info with explicit data name.
#[no_mangle]
pub unsafe extern "C" fn mdata_info_new_public(
    name: *const XorNameArray,
    type_tag: u64,
    user_data: *mut c_void,
    o_cb: extern "C" fn(*mut c_void, FfiResult, *const ffi::MDataInfo),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<_, AppError> {
        let name = XorName(*name);
        let info = MDataInfo::new_public(name, type_tag);
        let info = info.into_repr_c();

        o_cb(user_data, FFI_RESULT_OK, &info);
        Ok(())
    })
}

/// Create encrypted mdata info with explicit data name and a
/// provided private key.
#[no_mangle]
pub unsafe extern "C" fn mdata_info_new_private(
    name: *const XorNameArray,
    type_tag: u64,
    secret_key: *const SymSecretKey,
    nonce: *const SymNonce,
    user_data: *mut c_void,
    o_cb: extern "C" fn(*mut c_void, FfiResult, *const ffi::MDataInfo),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<_, AppError> {
        let name = XorName(*name);
        let sk = secretbox::Key(*secret_key);
        let nonce = secretbox::Nonce(*nonce);

        let info = MDataInfo::new_private(name, type_tag, (sk, nonce));
        let info = info.into_repr_c();

        o_cb(user_data, FFI_RESULT_OK, &info);
        Ok(())
    })
}

/// Create random, non-encrypted mdata info.
#[no_mangle]
pub unsafe extern "C" fn mdata_info_random_public(
    type_tag: u64,
    user_data: *mut c_void,
    o_cb: extern "C" fn(*mut c_void, FfiResult, *const ffi::MDataInfo),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<_, AppError> {
        let info = MDataInfo::random_public(type_tag)?;
        let info = info.into_repr_c();

        o_cb(user_data, FFI_RESULT_OK, &info);
        Ok(())
    })
}

/// Create random, encrypted mdata info.
#[no_mangle]
pub unsafe extern "C" fn mdata_info_random_private(
    type_tag: u64,
    user_data: *mut c_void,
    o_cb: extern "C" fn(*mut c_void, FfiResult, *const ffi::MDataInfo),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<_, AppError> {
        let info = MDataInfo::random_private(type_tag)?;
        let info = info.into_repr_c();

        o_cb(user_data, FFI_RESULT_OK, &info);
        Ok(())
    })
}

/// Encrypt mdata entry key using the corresponding mdata info.
#[no_mangle]
pub unsafe extern "C" fn mdata_info_encrypt_entry_key(
    info: *const ffi::MDataInfo,
    input_ptr: *const u8,
    input_len: usize,
    user_data: *mut c_void,
    o_cb: extern "C" fn(*mut c_void, FfiResult, *const u8, usize),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<_, AppError> {
        let info = MDataInfo::clone_from_repr_c(info)?;
        let input = slice::from_raw_parts(input_ptr, input_len);
        let encoded = info.enc_entry_key(input).map_err(AppError::from)?;

        o_cb(
            user_data,
            FFI_RESULT_OK,
            encoded.as_safe_ptr(),
            encoded.len(),
        );
        Ok(())
    })
}

/// Encrypt mdata entry value using the corresponding mdata info.
#[no_mangle]
pub unsafe extern "C" fn mdata_info_encrypt_entry_value(
    info: *const ffi::MDataInfo,
    input_ptr: *const u8,
    input_len: usize,
    user_data: *mut c_void,
    o_cb: extern "C" fn(*mut c_void, FfiResult, *const u8, usize),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<_, AppError> {
        let info = MDataInfo::clone_from_repr_c(info)?;
        let input = slice::from_raw_parts(input_ptr, input_len);
        let encoded = info.enc_entry_value(input).map_err(AppError::from)?;

        o_cb(
            user_data,
            FFI_RESULT_OK,
            encoded.as_safe_ptr(),
            encoded.len(),
        );
        Ok(())
    })
}

/// Decrypt mdata entry value or a key using the corresponding mdata info.
#[no_mangle]
pub unsafe extern "C" fn mdata_info_decrypt(
    info: *const ffi::MDataInfo,
    input_ptr: *const u8,
    input_len: usize,
    user_data: *mut c_void,
    o_cb: extern "C" fn(*mut c_void, FfiResult, *const u8, usize),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<_, AppError> {
        let info = MDataInfo::clone_from_repr_c(info)?;
        let encoded = slice::from_raw_parts(input_ptr, input_len);
        let decoded = info.decrypt(encoded).map_err(AppError::from)?;

        o_cb(
            user_data,
            FFI_RESULT_OK,
            decoded.as_safe_ptr(),
            decoded.len(),
        );
        Ok(())
    })
}

/// Serialise `MDataInfo`.
#[no_mangle]
pub unsafe extern "C" fn mdata_info_serialise(
    info: *const ffi::MDataInfo,
    user_data: *mut c_void,
    o_cb: extern "C" fn(*mut c_void, FfiResult, *const u8, usize),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<_, AppError> {
        let info = MDataInfo::clone_from_repr_c(info)?;
        let encoded = serialise(&info).map_err(AppError::from)?;

        o_cb(
            user_data,
            FFI_RESULT_OK,
            encoded.as_safe_ptr(),
            encoded.len(),
        );
        Ok(())
    })
}

/// Deserialise `MDataInfo`.
#[no_mangle]
pub unsafe extern "C" fn mdata_info_deserialise(
    ptr: *const u8,
    len: usize,
    user_data: *mut c_void,
    o_cb: extern "C" fn(*mut c_void, FfiResult, *const ffi::MDataInfo),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<_, AppError> {
        let encoded = slice::from_raw_parts(ptr, len);
        let info: MDataInfo = deserialise(encoded)?;
        let info = info.into_repr_c();

        o_cb(user_data, FFI_RESULT_OK, &info);
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffi_utils::test_utils::{call_1, call_vec_u8};
    use rand;
    use routing::XOR_NAME_LEN;
    use rust_sodium::crypto::secretbox;
    use safe_core::MDataInfo;

    #[test]
    fn create_public() {
        let type_tag: u64 = rand::random();

        let info: MDataInfo =
            unsafe { unwrap!(call_1(|ud, cb| mdata_info_random_public(type_tag, ud, cb))) };

        assert_eq!(info.type_tag, type_tag);
        assert!(info.enc_info.is_none());
    }

    #[test]
    fn create_private() {
        let type_tag: u64 = rand::random();

        let rand_info: MDataInfo =
            unsafe { unwrap!(call_1(|ud, cb| mdata_info_random_private(type_tag, ud, cb))) };

        let key = secretbox::gen_key();
        let nonce = secretbox::gen_nonce();
        let new_info: MDataInfo = unsafe {
            unwrap!(call_1(|ud, cb| {
                mdata_info_new_private(&[2; XOR_NAME_LEN], type_tag, &key.0, &nonce.0, ud, cb)
            }))
        };

        assert_eq!(rand_info.type_tag, type_tag);
        assert!(rand_info.enc_info.is_some());

        assert_eq!(new_info.type_tag, type_tag);
        match new_info.enc_info {
            Some((ref got_key, ref got_nonce)) => {
                assert_eq!(*got_key, key);
                assert_eq!(*got_nonce, nonce);
            }
            None => panic!("Unexpected result: no enc_info in private MDataInfo"),
        }
    }

    #[test]
    fn serialise_deserialise() {
        let info1 = unwrap!(MDataInfo::random_private(1000));
        let info1_ffi = info1.clone().into_repr_c();

        let encoded = unsafe {
            unwrap!(call_vec_u8(
                |ud, cb| mdata_info_serialise(&info1_ffi, ud, cb),
            ))
        };

        let info2 = unsafe {
            let res = call_1(|ud, cb| {
                mdata_info_deserialise(encoded.as_ptr(), encoded.len(), ud, cb)
            });

            unwrap!(res)
        };

        assert_eq!(info1, info2);
    }
}
