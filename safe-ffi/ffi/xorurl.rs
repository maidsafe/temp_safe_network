use super::ffi_structs::{xorurl_encoder_into_repr_c, XorNameArray, XorUrlEncoder};
use super::helpers::{from_c_str_to_str_option, to_c_str};
use ffi_utils::{catch_unwind_cb, from_c_str, FfiResult, OpaqueCtx, FFI_RESULT_OK};
use safe_api::xorurl::{SafeContentType, SafeDataType, XorUrlEncoder as NativeXorUrlEncoder};
use safe_api::ResultReturn;
use safe_nd::XorName;
use std::os::raw::{c_char, c_void};

// todo: Can be convertered to a struct
#[no_mangle]
pub unsafe extern "C" fn xorurl_encode(
    name: *const XorNameArray,
    type_tag: u64,
    data_type: u64,
    content_type: u16,
    path: *const c_char,
    _sub_names: *const c_char, // todo: update this later
    content_version: u64,
    base_encoding: *const c_char,
    user_data: *mut c_void,
    o_cb: extern "C" fn(
        user_data: *mut c_void,
        result: *const FfiResult,
        encoded_xor_url: *const c_char,
    ),
) {
    catch_unwind_cb(user_data, o_cb, || -> ResultReturn<()> {
        let user_data = OpaqueCtx(user_data);
        let xor_name = XorName(*name);
        let data_type_enum = SafeDataType::from_u64(data_type)?;
        let content_type_enum = SafeContentType::from_u16(content_type)?;
        let url_path = from_c_str_to_str_option(path);
        let encoding_base = from_c_str(base_encoding)?;
        let encoded_xor_url = NativeXorUrlEncoder::encode(
            xor_name,
            type_tag,
            data_type_enum,
            content_type_enum,
            url_path,
            Some(vec![]),
            Some(content_version),
            &encoding_base,
        )?; //todo: update sub_names parameter
        let encoded_string = to_c_str(encoded_xor_url)?;
        o_cb(user_data.0, FFI_RESULT_OK, encoded_string.as_ptr());
        Ok(())
    })
}

// todo: Can be convertered to a struct
#[no_mangle]
pub unsafe extern "C" fn xorurl_encoder(
    name: *const XorNameArray,
    type_tag: u64,
    data_type: u64,
    content_type: u16,
    path: *const c_char,
    _sub_names: *const c_char, // todo: update this later
    content_version: u64,
    user_data: *mut c_void,
    o_cb: extern "C" fn(
        user_data: *mut c_void,
        result: *const FfiResult,
        xor_url_encoder: *const XorUrlEncoder,
    ),
) {
    catch_unwind_cb(user_data, o_cb, || -> ResultReturn<()> {
        let user_data = OpaqueCtx(user_data);
        let xor_name = XorName(*name);
        let data_type_enum = SafeDataType::from_u64(data_type)?;
        let content_type_enum = SafeContentType::from_u16(content_type)?;
        let url_path = from_c_str_to_str_option(path);
        let encoder = NativeXorUrlEncoder::new(
            xor_name,
            type_tag,
            data_type_enum,
            content_type_enum,
            url_path,
            Some(vec![]),
            Some(content_version),
        )?; //todo: update sub_names parameter
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
    catch_unwind_cb(user_data, o_cb, || -> ResultReturn<()> {
        let user_data = OpaqueCtx(user_data);
        let xor_url = from_c_str(xor_url)?;
        let xor_url_encoder = NativeXorUrlEncoder::from_url(&xor_url)?;
        let ffi_encoder = xorurl_encoder_into_repr_c(xor_url_encoder)?;
        o_cb(user_data.0, FFI_RESULT_OK, &ffi_encoder);
        Ok(())
    })
}
