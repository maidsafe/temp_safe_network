// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{
    common::errors::{Error, Result},
    constants::{FILE_READ_FROM_START, FILE_READ_TO_END},
    ffi_structs::{
        files_map_into_repr_c, wallet_spendable_balances_into_repr_c, FilesContainer,
        NrsMapContainer, PublicImmutableData, SafeKey, SequenceData, Wallet,
    },
};
use sn_ffi_utils::{
    catch_unwind_cb, vec_into_raw_parts, FfiResult, NativeResult, OpaqueCtx, ReprC, FFI_RESULT_OK,
};
use safe_api::{fetch::SafeData, Safe};
use std::ffi::CString;
use std::{
    os::raw::{c_char, c_void},
    time::Duration,
};
use tokio::runtime::Runtime;

#[no_mangle]
pub unsafe extern "C" fn fetch(
    app: *mut Safe,
    url: *const c_char,
    user_data: *mut c_void,
    start: u64,
    end: u64,
    public_immutable_cb: extern "C" fn(user_data: *mut c_void, data: *const PublicImmutableData),
    sequence_cb: extern "C" fn(user_data: *mut c_void, data: *const SequenceData),
    wallet_cb: extern "C" fn(user_data: *mut c_void, data: *const Wallet),
    keys_cb: extern "C" fn(user_data: *mut c_void, data: *const SafeKey),
    container_cb: extern "C" fn(user_data: *mut c_void, data: *const FilesContainer),
    nrs_map_container_cb: extern "C" fn(user_data: *mut c_void, data: *const NrsMapContainer),
    err_cb: extern "C" fn(user_data: *mut c_void, result: *const FfiResult),
) {
    catch_unwind_cb(user_data, err_cb, || -> Result<()> {
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
        let mut runtime = Runtime::new().expect("Failed to create runtime");
        let content = runtime.block_on((*app).fetch(&url, Some((start, end))));
        runtime.shutdown_timeout(Duration::from_millis(1));
        invoke_callback(
            content,
            user_data,
            public_immutable_cb,
            sequence_cb,
            wallet_cb,
            keys_cb,
            container_cb,
            nrs_map_container_cb,
            err_cb,
        )
    })
}

#[no_mangle]
pub unsafe extern "C" fn inspect(
    app: *mut Safe,
    url: *const c_char,
    user_data: *mut c_void,
    o_cb: extern "C" fn(
        user_data: *mut c_void,
        result: *const FfiResult,
        inspect_result: *const c_char,
    ),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let user_data = OpaqueCtx(user_data);
        let url = String::clone_from_repr_c(url)?;
        let mut runtime = Runtime::new().expect("Failed to create runtime");
        let content = runtime.block_on((*app).inspect(&url))?;
        runtime.shutdown_timeout(Duration::from_millis(1));
        let content_json = CString::new(serde_json::to_string(&content)?)?;
        o_cb(user_data.0, FFI_RESULT_OK, content_json.as_ptr());
        Ok(())
    })
}

#[allow(clippy::too_many_arguments)]
unsafe fn invoke_callback(
    content: safe_api::Result<SafeData>,
    user_data: *mut c_void,
    public_immutable_cb: extern "C" fn(user_data: *mut c_void, data: *const PublicImmutableData),
    sequence_cb: extern "C" fn(user_data: *mut c_void, data: *const SequenceData),
    wallet_cb: extern "C" fn(user_data: *mut c_void, data: *const Wallet),
    keys_cb: extern "C" fn(user_data: *mut c_void, data: *const SafeKey),
    container_cb: extern "C" fn(user_data: *mut c_void, data: *const FilesContainer),
    nrs_map_container_cb: extern "C" fn(user_data: *mut c_void, data: *const NrsMapContainer),
    err_cb: extern "C" fn(user_data: *mut c_void, result: *const FfiResult),
) -> Result<()> {
    let user_data = OpaqueCtx(user_data);
    match &content {
        Ok(SafeData::PublicImmutableData {
            xorurl,
            data,
            xorname,
            media_type,
            metadata,
            resolved_from,
        }) => {
            let (data, data_len) = vec_into_raw_parts(data.to_vec());
            let public_data = PublicImmutableData {
                xorurl: CString::new(xorurl.clone())?.into_raw(),
                xorname: xorname.0,
                data,
                data_len,
                media_type: match media_type {
                    Some(media_value) => CString::new(media_value.clone())?.into_raw(),
                    None => std::ptr::null(),
                },
                metadata: match metadata {
                    Some(metadata_value) => {
                        CString::new(serde_json::to_string(metadata_value)?)?.into_raw()
                    }
                    None => std::ptr::null(),
                },
                resolved_from: CString::new(resolved_from.clone())?.into_raw(),
            };
            public_immutable_cb(user_data.0, &public_data);
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
                resolved_from: CString::new(resolved_from.clone())?.into_raw(),
            };
            container_cb(user_data.0, &container);
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
                resolved_from: CString::new(resolved_from.clone())?.into_raw(),
            };
            wallet_cb(user_data.0, &wallet);
        }
        Ok(SafeData::SafeKey {
            xorurl,
            xorname,
            resolved_from,
        }) => {
            let keys = SafeKey {
                xorurl: CString::new(xorurl.clone())?.into_raw(),
                xorname: xorname.0,
                resolved_from: CString::new(resolved_from.clone())?.into_raw(),
            };
            keys_cb(user_data.0, &keys);
        }
        Ok(SafeData::NrsMapContainer {
            public_name,
            xorurl,
            xorname,
            type_tag,
            version,
            nrs_map,
            data_type,
            resolved_from,
        }) => {
            let nrs_map_json = serde_json::to_string(&nrs_map)?;
            let nrs_map_container = NrsMapContainer {
                public_name: CString::new(public_name.clone().unwrap_or_else(|| "".to_string()))?
                    .into_raw(),
                xorurl: CString::new(xorurl.clone())?.into_raw(),
                xorname: xorname.0,
                type_tag: *type_tag,
                version: *version,
                nrs_map: CString::new(nrs_map_json)?.into_raw(),
                data_type: (*data_type).clone() as u64,
                resolved_from: CString::new(resolved_from.clone())?.into_raw(),
            };
            nrs_map_container_cb(user_data.0, &nrs_map_container);
        }
        Ok(SafeData::PublicSequence {
            xorurl,
            xorname,
            type_tag,
            version,
            data,
            resolved_from,
        }) => {
            let (data, data_len) = vec_into_raw_parts(data.to_vec());
            let seq_data = SequenceData {
                xorurl: CString::new(xorurl.clone())?.into_raw(),
                xorname: xorname.0,
                type_tag: *type_tag,
                version: *version,
                data,
                data_len,
                resolved_from: CString::new(resolved_from.clone())?.into_raw(),
                is_private: false,
            };
            sequence_cb(user_data.0, &seq_data);
        }
        Ok(SafeData::PrivateSequence {
            xorurl,
            xorname,
            type_tag,
            version,
            data,
            resolved_from,
        }) => {
            let (data, data_len) = vec_into_raw_parts(data.to_vec());
            let seq_data = SequenceData {
                xorurl: CString::new(xorurl.clone())?.into_raw(),
                xorname: xorname.0,
                type_tag: *type_tag,
                version: *version,
                data,
                data_len,
                resolved_from: CString::new(resolved_from.clone())?.into_raw(),
                is_private: true,
            };
            sequence_cb(user_data.0, &seq_data);
        }
        Err(err) => {
            let (error_code, description) = ffi_error!(Error::from(err.clone()));
            let ffi_result = NativeResult {
                error_code,
                description: Some(description),
            };
            err_cb(user_data.0, &ffi_result.into_repr_c()?);
        }
    };
    Ok(())
}
