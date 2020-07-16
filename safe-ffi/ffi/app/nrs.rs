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
    ffi_structs::{processed_entries_into_repr_c, safe_url_into_repr_c, ProcessedEntries, SafeUrl},
};
use ffi_utils::{catch_unwind_cb, FfiResult, OpaqueCtx, ReprC, FFI_RESULT_OK};
use safe_api::Safe;
use std::{
    ffi::CString,
    os::raw::{c_char, c_void},
    time::Duration,
};
use tokio::runtime::Runtime;

#[no_mangle]
pub unsafe extern "C" fn parse_url(
    url: *const c_char,
    user_data: *mut c_void,
    o_cb: extern "C" fn(user_data: *mut c_void, result: *const FfiResult, safe_url: *const SafeUrl),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let url_string = String::clone_from_repr_c(url)?;
        let safe_url = Safe::parse_url(&url_string)?;
        let ffi_safe_url = safe_url_into_repr_c(safe_url)?;
        o_cb(user_data.0, FFI_RESULT_OK, &ffi_safe_url);
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
        let mut runtime = Runtime::new().expect("Failed to create runtime");
        let (nrs_map_container_xorurl, processed_entries, nrs_map) = runtime.block_on(
            (*app).nrs_map_container_create(&nrs_str, &link_str, set_default, direct_link, dry_run),
        )?;
        runtime.shutdown_timeout(Duration::from_millis(1));
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
        let mut runtime = Runtime::new().expect("Failed to create runtime");
        let (version, xorurl, _processed_entries, nrs_map) = runtime.block_on(
            (*app).nrs_map_container_add(&name_str, &link_str, set_default, direct_link, dry_run),
        )?;
        runtime.shutdown_timeout(Duration::from_millis(1));
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
        let mut runtime = Runtime::new().expect("Failed to create runtime");
        let (version, xorurl, _processed_entries, nrs_map) =
            runtime.block_on((*app).nrs_map_container_remove(&name_str, dry_run))?;
        runtime.shutdown_timeout(Duration::from_millis(1));
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
        let mut runtime = Runtime::new().expect("Failed to create runtime");
        let (version, nrs_map) = runtime.block_on((*app).nrs_map_container_get(&url_string))?;
        runtime.shutdown_timeout(Duration::from_millis(1));
        let nrs_map_json = CString::new(serde_json::to_string(&nrs_map)?)?;
        o_cb(user_data.0, FFI_RESULT_OK, nrs_map_json.as_ptr(), version);
        Ok(())
    })
}
