use super::errors::Result;
use super::ffi_structs::{
    processed_entries_into_repr_c, xorurl_encoder_into_repr_c, ProcessedEntries, XorUrlEncoder,
};
use ffi_utils::{catch_unwind_cb, FfiResult, OpaqueCtx, ReprC, FFI_RESULT_OK};
use safe_api::Safe;
use std::ffi::CString;
use std::os::raw::{c_char, c_void};

#[no_mangle]
pub unsafe extern "C" fn parse_url(
    url: *const c_char,
    user_data: *mut c_void,
    o_cb: extern "C" fn(
        user_data: *mut c_void,
        result: *const FfiResult,
        xorurl_encoder: *const XorUrlEncoder,
    ),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let url_string = String::clone_from_repr_c(url)?;
        let encoder = Safe::parse_url(&url_string)?;
        let ffi_xorurl_encoder = xorurl_encoder_into_repr_c(encoder)?;
        o_cb(user_data.0, FFI_RESULT_OK, &ffi_xorurl_encoder);
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn parse_and_resolve_url(
    app: *mut Safe,
    url: *const c_char,
    user_data: *mut c_void,
    o_cb: extern "C" fn(
        user_data: *mut c_void,
        result: *const FfiResult,
        xorurl_encoder: *const XorUrlEncoder,
        resolved_from: *const XorUrlEncoder,
    ),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let url_string = String::clone_from_repr_c(url)?;
        let (encoder, resolved_from) = (*app).parse_and_resolve_url(&url_string)?;
        let ffi_xorurl_encoder = xorurl_encoder_into_repr_c(encoder)?;
        let ffi_nrs_xorurl_encoder = if let Some(nrs_xorurl_encoder) = resolved_from {
            xorurl_encoder_into_repr_c(nrs_xorurl_encoder)?
        } else {
            XorUrlEncoder::new()?
        };

        o_cb(
            user_data.0,
            FFI_RESULT_OK,
            &ffi_xorurl_encoder,
            &ffi_nrs_xorurl_encoder,
        );
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn nrs_map_container_create(
    app: *mut Safe,
    name: *const c_char,
    link: *const c_char,
    direct_link: bool,
    dry_run: bool,
    set_default: bool,
    user_data: *mut c_void,
    o_cb: extern "C" fn(
        user_data: *mut c_void,
        result: *const FfiResult,
        nrs_map: *const c_char,
        processed_entries: *const ProcessedEntries,
        xorurl: *const c_char,
    ),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let nrs_str = String::clone_from_repr_c(name)?;
        let link_str = String::clone_from_repr_c(link)?;
        let (nrs_map_container_xorurl, processed_entries, nrs_map) = (*app)
            .nrs_map_container_create(&nrs_str, &link_str, set_default, direct_link, dry_run)?;
        let xorurl_string = CString::new(nrs_map_container_xorurl)?;
        let nrs_map_json = CString::new(serde_json::to_string(&nrs_map)?)?;
        let ffi_processed_entries = processed_entries_into_repr_c(&processed_entries)?;
        o_cb(
            user_data.0,
            FFI_RESULT_OK,
            nrs_map_json.as_ptr(),
            &ffi_processed_entries,
            xorurl_string.as_ptr(),
        );
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn nrs_map_container_add(
    app: *mut Safe,
    name: *const c_char,
    link: *const c_char,
    set_default: bool,
    direct_link: bool,
    dry_run: bool,
    user_data: *mut c_void,
    o_cb: extern "C" fn(
        user_data: *mut c_void,
        result: *const FfiResult,
        nrs_map: *const c_char,
        xorurl: *const c_char,
        version: u64, // todo: add processed entries
    ),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let name_str = String::clone_from_repr_c(name)?;
        let link_str = String::clone_from_repr_c(link)?;
        let (version, xorurl, _processed_entries, nrs_map) = (*app).nrs_map_container_add(
            &name_str,
            &link_str,
            set_default,
            direct_link,
            dry_run,
        )?;
        let xorurl_string = CString::new(xorurl)?;
        let nrs_map_json = CString::new(serde_json::to_string(&nrs_map)?)?;
        o_cb(
            user_data.0,
            FFI_RESULT_OK,
            nrs_map_json.as_ptr(),
            xorurl_string.as_ptr(),
            version,
        );
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn nrs_map_container_remove(
    app: *mut Safe,
    name: *const c_char,
    dry_run: bool,
    user_data: *mut c_void,
    o_cb: extern "C" fn(
        user_data: *mut c_void,
        result: *const FfiResult,
        nrs_map: *const c_char,
        xorurl: *const c_char,
        version: u64, // todo: add processed entries
    ),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let name_str = String::clone_from_repr_c(name)?;
        let (version, xorurl, _processed_entries, nrs_map) =
            (*app).nrs_map_container_remove(&name_str, dry_run)?;
        let xorurl_string = CString::new(xorurl)?;
        let nrs_map_json = CString::new(serde_json::to_string(&nrs_map)?)?;
        o_cb(
            user_data.0,
            FFI_RESULT_OK,
            nrs_map_json.as_ptr(),
            xorurl_string.as_ptr(),
            version,
        );
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn nrs_map_container_get(
    app: *mut Safe,
    url: *const c_char,
    user_data: *mut c_void,
    o_cb: extern "C" fn(
        user_data: *mut c_void,
        result: *const FfiResult,
        nrs_map: *const c_char,
        version: u64,
    ),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let url_string = String::clone_from_repr_c(url)?;
        let (version, nrs_map) = (*app).nrs_map_container_get(&url_string)?;
        let nrs_map_json = CString::new(serde_json::to_string(&nrs_map)?)?;
        o_cb(user_data.0, FFI_RESULT_OK, nrs_map_json.as_ptr(), version);
        Ok(())
    })
}
