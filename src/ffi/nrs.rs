use crate::api::{ResultReturn};
use crate::api::nrs_map::{NrsMap};
use crate::api::{Safe};
use super::helpers::{to_c_str};
use ffi_utils::{catch_unwind_cb, from_c_str, FfiResult, OpaqueCtx, FFI_RESULT_OK};
use std::os::raw::{c_char, c_void};

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
        nrs_map: *const NrsMap,
        xor_url: *const c_char)
) {
    catch_unwind_cb(user_data, o_cb,  || -> ResultReturn<()> {
        let user_data = OpaqueCtx(user_data);
        let nrs_str = from_c_str(name)?;
        let link_str = from_c_str(link)?;
        let (nrs_map_container_xorurl, _processed_entries, nrs_map) =
            (*app).nrs_map_container_create(&nrs_str, &link_str, set_default, direct_link, dry_run)?;
        let xor_url_string = to_c_str(nrs_map_container_xorurl)?;
        o_cb(user_data.0, FFI_RESULT_OK, &nrs_map, xor_url_string.as_ptr());
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
        nrs_map: *const NrsMap,
        xor_url: *const c_char,
        version: u64)
) {
    catch_unwind_cb(user_data, o_cb,  || -> ResultReturn<()> {
        let user_data = OpaqueCtx(user_data);
        let name_str = from_c_str(name)?;
        let link_str = from_c_str(link)?;
        let (version, xorurl, _processed_entries, nrs_map) =
            (*app).nrs_map_container_add(&name_str, &link_str, set_default, direct_link, dry_run)?;
        let xor_url_string = to_c_str(xorurl)?;
        o_cb(user_data.0, FFI_RESULT_OK, &nrs_map, xor_url_string.as_ptr(), version);
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
        nrs_map: *const NrsMap,
        xor_url: *const c_char,
        version: u64)
) {
    catch_unwind_cb(user_data, o_cb,  || -> ResultReturn<()> {
        let user_data = OpaqueCtx(user_data);
        let name_str = from_c_str(name)?;
        let (version, xorurl, _processed_entries, nrs_map) =
            (*app).nrs_map_container_remove(&name_str, dry_run)?;
        let xor_url_string = to_c_str(xorurl)?;
        o_cb(user_data.0, FFI_RESULT_OK, &nrs_map, xor_url_string.as_ptr(), version);
        Ok(())
    })
}
