// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{
    errors::Result,
    ffi_structs::{xorurl_encoder_into_repr_c, XorNameArray, XorUrlEncoder},
    helpers::{c_str_str_to_string_vec, from_c_str_to_str_option},
};
use ffi_utils::{catch_unwind_cb, FfiResult, OpaqueCtx, ReprC, FFI_RESULT_OK};
use safe_api::xorurl::{
    SafeContentType, SafeDataType, XorUrlBase, XorUrlEncoder as NativeXorUrlEncoder,
};
use safe_nd::XorName;
use std::{
    ffi::CString,
    os::raw::{c_char, c_void},
    str::FromStr,
};

#[no_mangle]
pub unsafe extern "C" fn xorurl_encode(
    name: *const XorNameArray,
    type_tag: u64,
    data_type: u64,
    content_type: u16,
    path: *const c_char,
    sub_names: *const *const c_char,
    sub_names_len: usize,
    content_version: u64,
    base_encoding: *const c_char,
    user_data: *mut c_void,
    o_cb: extern "C" fn(
        user_data: *mut c_void,
        result: *const FfiResult,
        encoded_xor_url: *const c_char,
    ),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let xor_name = XorName(*name);
        let data_type_enum = SafeDataType::from_u64(data_type)?;
        let content_type_enum = SafeContentType::from_u16(content_type)?;
        let url_path = from_c_str_to_str_option(path);
        let sub_names = if sub_names_len == 0 {
            None
        } else {
            Some(c_str_str_to_string_vec(sub_names, sub_names_len)?)
        };
        let encoding_base = XorUrlBase::from_str(&String::clone_from_repr_c(base_encoding)?)?;
        let encoded_xor_url = NativeXorUrlEncoder::encode(
            xor_name,
            None,
            type_tag,
            data_type_enum,
            content_type_enum,
            url_path,
            sub_names,
            None,
            None,
            Some(content_version),
            encoding_base,
        )?;
        let encoded_string = CString::new(encoded_xor_url)?;
        o_cb(user_data.0, FFI_RESULT_OK, encoded_string.as_ptr());
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn xorurl_encoder(
    name: *const XorNameArray,
    type_tag: u64,
    data_type: u64,
    content_type: u16,
    path: *const c_char,
    sub_names: *const *const c_char,
    sub_names_len: usize,
    content_version: u64,
    user_data: *mut c_void,
    o_cb: extern "C" fn(
        user_data: *mut c_void,
        result: *const FfiResult,
        xor_url_encoder: *const XorUrlEncoder,
    ),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let xor_name = XorName(*name);
        let data_type_enum = SafeDataType::from_u64(data_type)?;
        let content_type_enum = SafeContentType::from_u16(content_type)?;
        let url_path = from_c_str_to_str_option(path);
        let sub_names = if sub_names_len == 0 {
            None
        } else {
            Some(c_str_str_to_string_vec(sub_names, sub_names_len)?)
        };
        let encoder = NativeXorUrlEncoder::new(
            xor_name,
            None,
            type_tag,
            data_type_enum,
            content_type_enum,
            url_path,
            sub_names,
            None,
            None,
            Some(content_version),
        )?;
        let ffi_encoder = xorurl_encoder_into_repr_c(encoder)?;
        o_cb(user_data.0, FFI_RESULT_OK, &ffi_encoder);
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn xorurl_encoder_from_url(
    xor_url: *const c_char,
    user_data: *mut c_void,
    o_cb: extern "C" fn(
        user_data: *mut c_void,
        result: *const FfiResult,
        xor_url_encoder: *const XorUrlEncoder,
    ),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let xor_url = String::clone_from_repr_c(xor_url)?;
        let xor_url_encoder = NativeXorUrlEncoder::from_url(&xor_url)?;
        let ffi_encoder = xorurl_encoder_into_repr_c(xor_url_encoder)?;
        o_cb(user_data.0, FFI_RESULT_OK, &ffi_encoder);
        Ok(())
    })
}
