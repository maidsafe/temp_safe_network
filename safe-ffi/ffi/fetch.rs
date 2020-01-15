// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{
    constants::{FILE_READ_FROM_START, FILE_READ_TO_END},
    errors::{Error, Result},
    ffi_structs::{
        files_map_into_repr_c, nrs_map_container_info_into_repr_c,
        wallet_spendable_balances_into_repr_c, FilesContainer, NrsMapContainerInfo,
        PublishedImmutableData, SafeKey, Wallet,
    },
};
use ffi_utils::{catch_unwind_cb, vec_into_raw_parts, FfiResult, NativeResult, OpaqueCtx, ReprC};
use safe_api::{fetch::SafeData, Safe};
use std::ffi::CString;
use std::os::raw::{c_char, c_void};

#[no_mangle]
pub unsafe extern "C" fn fetch(
    app: *mut Safe,
    url: *const c_char,
    user_data: *mut c_void,
    start: u64,
    end: u64,
    o_published: extern "C" fn(user_data: *mut c_void, data: *const PublishedImmutableData),
    o_wallet: extern "C" fn(user_data: *mut c_void, data: *const Wallet),
    o_keys: extern "C" fn(user_data: *mut c_void, data: *const SafeKey),
    o_container: extern "C" fn(user_data: *mut c_void, data: *const FilesContainer),
    o_err: extern "C" fn(user_data: *mut c_void, result: *const FfiResult),
) {
    catch_unwind_cb(user_data, o_err, || -> Result<()> {
        let url = String::clone_from_repr_c(url)?;
        let start = if start == FILE_READ_FROM_START {
            None
        } else {
            Some(start)
        };

        let end = if end == FILE_READ_TO_END {
            None
        } else {
            Some(end)
        };
        let content = (*app).fetch(&url, Some((start, end))).await;
        invoke_callback(
            content,
            user_data,
            o_published,
            o_wallet,
            o_keys,
            o_container,
            o_err,
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn inspect(
    app: *mut Safe,
    url: *const c_char,
    user_data: *mut c_void,
    o_published: extern "C" fn(user_data: *mut c_void, data: *const PublishedImmutableData),
    o_wallet: extern "C" fn(user_data: *mut c_void, data: *const Wallet),
    o_keys: extern "C" fn(user_data: *mut c_void, data: *const SafeKey),
    o_container: extern "C" fn(user_data: *mut c_void, data: *const FilesContainer),
    o_err: extern "C" fn(user_data: *mut c_void, result: *const FfiResult),
) {
    catch_unwind_cb(user_data, o_err, || -> Result<()> {
        let url = String::clone_from_repr_c(url)?;
        let content = (*app).inspect(&url).await;
        invoke_callback(
            content,
            user_data,
            o_published,
            o_wallet,
            o_keys,
            o_container,
            o_err,
        )
    })
}

unsafe fn invoke_callback(
    content: safe_api::Result<SafeData>,
    user_data: *mut c_void,
    o_published: extern "C" fn(user_data: *mut c_void, data: *const PublishedImmutableData),
    o_wallet: extern "C" fn(user_data: *mut c_void, data: *const Wallet),
    o_keys: extern "C" fn(user_data: *mut c_void, data: *const SafeKey),
    o_container: extern "C" fn(user_data: *mut c_void, data: *const FilesContainer),
    o_err: extern "C" fn(user_data: *mut c_void, result: *const FfiResult),
) -> Result<()> {
    let user_data = OpaqueCtx(user_data);
    match &content {
        Ok(SafeData::PublishedImmutableData {
            xorurl,
            data,
            xorname,
            resolved_from,
            media_type,
        }) => {
            let (data, data_len) = vec_into_raw_parts(data.to_vec());
            let published_data = PublishedImmutableData {
                xorurl: CString::new(xorurl.clone())?.into_raw(),
                xorname: xorname.0,
                data,
                data_len,
                resolved_from: match resolved_from {
                    Some(nrs_container_map) => {
                        nrs_map_container_info_into_repr_c(&nrs_container_map)?
                    }
                    None => NrsMapContainerInfo::new()?,
                },
                media_type: CString::new(media_type.clone().unwrap())?.into_raw(),
            };
            o_published(user_data.0, &published_data);
        }
        Ok(SafeData::FilesContainer {
            xorurl,
            version,
            files_map,
            type_tag,
            xorname,
            data_type,
            resolved_from,
        }) => {
            let container = FilesContainer {
                xorurl: CString::new(xorurl.clone())?.into_raw(),
                version: *version,
                files_map: files_map_into_repr_c(&files_map)?,
                type_tag: *type_tag,
                xorname: xorname.0,
                data_type: (*data_type).clone() as u64,
                resolved_from: match resolved_from {
                    Some(nrs_container_map) => {
                        nrs_map_container_info_into_repr_c(&nrs_container_map)?
                    }
                    None => NrsMapContainerInfo::new()?,
                },
            };
            o_container(user_data.0, &container);
        }
        Ok(SafeData::Wallet {
            xorurl,
            xorname,
            type_tag,
            balances,
            data_type,
            resolved_from,
        }) => {
            let wallet = Wallet {
                xorurl: CString::new(xorurl.clone())?.into_raw(),
                xorname: xorname.0,
                type_tag: *type_tag,
                balances: wallet_spendable_balances_into_repr_c(balances)?,
                data_type: (*data_type).clone() as u64,
                resolved_from: match resolved_from {
                    Some(nrs_container_map) => {
                        nrs_map_container_info_into_repr_c(&nrs_container_map)?
                    }
                    None => NrsMapContainerInfo::new()?,
                },
            };
            o_wallet(user_data.0, &wallet);
        }
        Ok(SafeData::SafeKey {
            xorurl,
            xorname,
            resolved_from,
        }) => {
            let keys = SafeKey {
                xorurl: CString::new(xorurl.clone())?.into_raw(),
                xorname: xorname.0,
                resolved_from: match resolved_from {
                    Some(nrs_container_map) => {
                        nrs_map_container_info_into_repr_c(&nrs_container_map)?
                    }
                    None => NrsMapContainerInfo::new()?,
                },
            };
            o_keys(user_data.0, &keys);
        }
        Err(err) => {
            let (error_code, description) = ffi_error!(Error::from(err.clone()));
            let ffi_result = NativeResult {
                error_code,
                description: Some(description),
            };
            o_err(user_data.0, &ffi_result.into_repr_c()?);
        }
    };
    Ok(())
}
