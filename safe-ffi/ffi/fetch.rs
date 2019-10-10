use super::ffi_structs::{
    nrs_map_container_info_into_repr_c, wallet_spendable_balances_into_repr_c, FilesContainer,
    NrsMapContainerInfo, PublishedImmutableData, SafeKey, Wallet,
};
use super::{ResultReturn, Safe};
use ffi_utils::{catch_unwind_cb, from_c_str, vec_into_raw_parts, FfiResult, OpaqueCtx};
use safe_api::fetch::SafeData;
use std::ffi::CString;
use std::os::raw::{c_char, c_void};

#[no_mangle]
pub unsafe extern "C" fn fetch(
    app: *mut Safe,
    url: *const c_char,
    user_data: *mut c_void,
    o_published: extern "C" fn(user_data: *mut c_void, *const PublishedImmutableData),
    o_wallet: extern "C" fn(user_data: *mut c_void, *const Wallet),
    o_keys: extern "C" fn(user_data: *mut c_void, *const SafeKey),
    o_container: extern "C" fn(user_data: *mut c_void, *const FilesContainer),
    o_err: extern "C" fn(user_data: *mut c_void, result: *const FfiResult),
) {
    catch_unwind_cb(user_data, o_err, || -> ResultReturn<()> {
        let user_data = OpaqueCtx(user_data);
        let url = from_c_str(url)?;
        let content = (*app).fetch(&url)?;
        match &content {
            SafeData::PublishedImmutableData {
                xorurl,
                data,
                xorname,
                resolved_from,
                media_type,
            } => {
                let (data, data_len, data_cap) = vec_into_raw_parts(data.to_vec());
                let xorurl_c_str = CString::new(xorurl.clone())?;
                let published_data = PublishedImmutableData {
                    xorurl: xorurl_c_str.into_raw(),
                    xorname: xorname.0,
                    data,
                    data_len,
                    data_cap,
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
            SafeData::FilesContainer {
                xorurl,
                version,
                files_map,
                type_tag,
                xorname,
                data_type,
                resolved_from,
            } => {
                let files_map_json = serde_json::to_string(&files_map)?;
                let xorurl_c_str = CString::new(xorurl.clone())?;
                let container = FilesContainer {
                    xorurl: xorurl_c_str.into_raw(),
                    version: *version,
                    files_map: CString::new(files_map_json)?.into_raw(),
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
            SafeData::Wallet {
                xorurl,
                xorname,
                type_tag,
                balances,
                data_type,
                resolved_from,
            } => {
                let xorurl_c_str = CString::new(xorurl.clone())?;
                let wallet = Wallet {
                    xorurl: xorurl_c_str.into_raw(),
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
            SafeData::SafeKey {
                xorurl,
                xorname,
                resolved_from,
            } => {
                let xorurl_c_str = CString::new(xorurl.clone())?;
                let keys = SafeKey {
                    xorurl: xorurl_c_str.into_raw(),
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
        };
        Ok(())
    })
}
