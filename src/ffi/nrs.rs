use crate::api::{ResultReturn, Error};
use crate::api::nrs_map::{NrsMap};
use crate::api::{Safe};
use ffi_utils::{catch_unwind_cb, from_c_str, FfiResult, OpaqueCtx, FFI_RESULT_OK};
use std::os::raw::{c_char, c_void};
use std::ffi::{CString};

#[no_mangle]
pub unsafe extern "C" fn create_new_public_name(
    app: *mut Safe,
    name: *const c_char,
    link: *const c_char,
    direct_link: bool,
    dry_run: bool,
    default: bool,
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
            (*app).nrs_map_container_create(&nrs_str, &link_str, default, direct_link, dry_run)?;
        let xor_url_string = CString::new(nrs_map_container_xorurl).map_err(|_| Error::Unexpected("Couldn't convert to string".to_string()))?;
        o_cb(user_data.0, FFI_RESULT_OK, &nrs_map, xor_url_string.as_ptr());
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn add_update_sub_name(
    app: *mut Safe,
    name: *const c_char,
    link: *const c_char,
    default: bool,
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
            (*app).nrs_map_container_add(&name_str, &link_str, default, direct_link, dry_run)?;
        let xor_url_string = CString::new(xorurl).map_err(|_| Error::Unexpected("Couldn't convert to string".to_string()))?;
        o_cb(user_data.0, FFI_RESULT_OK, &nrs_map, xor_url_string.as_ptr(), version);
        Ok(())
    })
}

#[no_mangle]
pub unsafe extern "C" fn remove_sub_name(
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
        let xor_url_string = CString::new(xorurl).map_err(|_| Error::Unexpected("Couldn't convert to string".to_string()))?;
        o_cb(user_data.0, FFI_RESULT_OK, &nrs_map, xor_url_string.as_ptr(), version);
        Ok(())
    })
}
