// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{
    common::errors::Result,
    common::helpers::{c_str_str_to_string_vec, from_c_str_to_str_option},
    ffi_structs::{safe_url_into_repr_c, SafeUrl, XorNameArray},
};
use sn_ffi_utils::{catch_unwind_cb, FfiResult, OpaqueCtx, ReprC, FFI_RESULT_OK};
use safe_api::xorurl::{SafeContentType, SafeDataType, SafeUrl as NativeSafeUrl, XorUrlBase};
use std::{
    ffi::CString,
    os::raw::{c_char, c_void},
};
use xor_name::XorName;

#[no_mangle]
pub unsafe extern "C" fn safe_url_encode(
    name: *const XorNameArray,
    nrs_name: *const c_char,
    type_tag: u64,
    data_type: u64,
    content_type: u16,
    path: *const c_char,
    sub_names: *const *const c_char,
    sub_names_len: usize,
    query_string: *const c_char,
    fragment: *const c_char,
    content_version: u64,
    base_encoding: u16,
    user_data: *mut c_void,
    o_cb: extern "C" fn(
        user_data: *mut c_void,
        result: *const FfiResult,
        encoded_safe_url: *const c_char,
    ),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let xor_name = XorName(*name);
        let data_type_enum = SafeDataType::from_u64(data_type)?;
        let content_type_enum = SafeContentType::from_u16(content_type)?;
        let nrs_name = from_c_str_to_str_option(nrs_name);
        let url_path = from_c_str_to_str_option(path);
        let query_string = from_c_str_to_str_option(query_string);
        let fragment = from_c_str_to_str_option(fragment);
        let sub_names = if sub_names_len == 0 {
            None
        } else {
            Some(c_str_str_to_string_vec(sub_names, sub_names_len)?)
        };
        let encoding_base = XorUrlBase::from_u16(base_encoding)?;
        let encoded_safe_url = NativeSafeUrl::encode(
            xor_name,
            nrs_name,
            type_tag,
            data_type_enum,
            content_type_enum,
            url_path,
            sub_names,
            query_string,
            fragment,
            Some(content_version),
            encoding_base,
        )?;
        let encoded_string = CString::new(encoded_safe_url)?;
        o_cb(user_data.0, FFI_RESULT_OK, encoded_string.as_ptr());
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn new_safe_url(
    name: *const XorNameArray,
    nrs_name: *const c_char,
    type_tag: u64,
    data_type: u64,
    content_type: u16,
    path: *const c_char,
    sub_names: *const *const c_char,
    sub_names_len: usize,
    query_string: *const c_char,
    fragment: *const c_char,
    content_version: u64,
    user_data: *mut c_void,
    o_cb: extern "C" fn(user_data: *mut c_void, result: *const FfiResult, safe_url: *const SafeUrl),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let nrs_name = from_c_str_to_str_option(nrs_name);
        let xor_name = XorName(*name);
        let data_type_enum = SafeDataType::from_u64(data_type)?;
        let content_type_enum = SafeContentType::from_u16(content_type)?;
        let url_path = from_c_str_to_str_option(path);
        let query_string = from_c_str_to_str_option(query_string);
        let fragment = from_c_str_to_str_option(fragment);
        let sub_names = if sub_names_len == 0 {
            None
        } else {
            Some(c_str_str_to_string_vec(sub_names, sub_names_len)?)
        };
        let encoder = NativeSafeUrl::new(
            xor_name,
            nrs_name,
            type_tag,
            data_type_enum,
            content_type_enum,
            url_path,
            sub_names,
            query_string,
            fragment,
            Some(content_version),
        )?;
        let ffi_encoder = safe_url_into_repr_c(encoder)?;
        o_cb(user_data.0, FFI_RESULT_OK, &ffi_encoder);
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn safe_url_from_url(
    safe_url: *const c_char,
    user_data: *mut c_void,
    o_cb: extern "C" fn(user_data: *mut c_void, result: *const FfiResult, safe_url: *const SafeUrl),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let safe_url_str = String::clone_from_repr_c(safe_url)?;
        let safe_url = NativeSafeUrl::from_url(&safe_url_str)?;
        let ffi_encoder = safe_url_into_repr_c(safe_url)?;
        o_cb(user_data.0, FFI_RESULT_OK, &ffi_encoder);
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn encode_safekey(
    name: *const XorNameArray,
    base_encoding: u16,
    user_data: *mut c_void,
    o_cb: extern "C" fn(
        user_data: *mut c_void,
        result: *const FfiResult,
        encoded_safe_url: *const c_char,
    ),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let xor_name = XorName(*name);
        let encoding_base = XorUrlBase::from_u16(base_encoding)?;
        let encoded_safe_url = NativeSafeUrl::encode_safekey(xor_name, encoding_base)?;
        let encoded_string = CString::new(encoded_safe_url)?;
        o_cb(user_data.0, FFI_RESULT_OK, encoded_string.as_ptr());
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn encode_immutable_data(
    name: *const XorNameArray,
    content_type: u16,
    base_encoding: u16,
    user_data: *mut c_void,
    o_cb: extern "C" fn(
        user_data: *mut c_void,
        result: *const FfiResult,
        encoded_safe_url: *const c_char,
    ),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let xor_name = XorName(*name);
        let content_type_enum = SafeContentType::from_u16(content_type)?;
        let encoding_base = XorUrlBase::from_u16(base_encoding)?;
        let encoded_safe_url =
            NativeSafeUrl::encode_immutable_data(xor_name, content_type_enum, encoding_base)?;
        let encoded_string = CString::new(encoded_safe_url)?;
        o_cb(user_data.0, FFI_RESULT_OK, encoded_string.as_ptr());
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn encode_mutable_data(
    name: *const XorNameArray,
    type_tag: u64,
    content_type: u16,
    base_encoding: u16,
    user_data: *mut c_void,
    o_cb: extern "C" fn(
        user_data: *mut c_void,
        result: *const FfiResult,
        encoded_safe_url: *const c_char,
    ),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let xor_name = XorName(*name);
        let content_type_enum = SafeContentType::from_u16(content_type)?;
        let encoding_base = XorUrlBase::from_u16(base_encoding)?;
        let encoded_safe_url = NativeSafeUrl::encode_mutable_data(
            xor_name,
            type_tag,
            content_type_enum,
            encoding_base,
        )?;
        let encoded_string = CString::new(encoded_safe_url)?;
        o_cb(user_data.0, FFI_RESULT_OK, encoded_string.as_ptr());
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn encode_sequence_data(
    name: *const XorNameArray,
    type_tag: u64,
    content_type: u16,
    base_encoding: u16,
    private: bool,
    user_data: *mut c_void,
    o_cb: extern "C" fn(
        user_data: *mut c_void,
        result: *const FfiResult,
        encoded_safe_url: *const c_char,
    ),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let xor_name = XorName(*name);
        let content_type_enum = SafeContentType::from_u16(content_type)?;
        let encoding_base = XorUrlBase::from_u16(base_encoding)?;
        let encoded_safe_url = NativeSafeUrl::encode_sequence_data(
            xor_name,
            type_tag,
            content_type_enum,
            encoding_base,
            private,
        )?;
        let encoded_string = CString::new(encoded_safe_url)?;
        o_cb(user_data.0, FFI_RESULT_OK, encoded_string.as_ptr());
        Ok(())
    })
}
