// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{errors::Result, helpers::string_vec_to_c_str_str};
use ffi_utils::{vec_from_raw_parts, vec_into_raw_parts};
use safe_api::{
    files::{
        FileItem as NativeFileItem, FilesMap as NativeFilesMap,
        ProcessedFiles as NativeProcessedFiles,
    },
    nrs_map::{NrsMap as NativeNrsMap, SubNamesMap as NativeSubNamesMap},
    wallet::{
        WalletSpendableBalance as NativeWalletSpendableBalance,
        WalletSpendableBalances as NativeWalletSpendableBalances,
    },
    xorurl::SafeUrl as NativeSafeUrl,
    BlsKeyPair as NativeBlsKeyPair, ProcessedEntries as NativeProcessedEntries,
};
use safe_nd::XOR_NAME_LEN;
use std::ffi::CString;
use std::os::raw::c_char;

/// Array containing `XorName` bytes.
/// Adding this here because bindgen not picking this correctly from the safe-nd.
pub type XorNameArray = [u8; XOR_NAME_LEN];

#[repr(C)]
pub struct BlsKeyPair {
    pub pk: *const c_char,
    pub sk: *const c_char,
}

impl Drop for BlsKeyPair {
    fn drop(&mut self) {
        unsafe {
            if !self.pk.is_null() {
                let _ = CString::from_raw(self.pk as *mut _);
            }

            if !self.sk.is_null() {
                let _ = CString::from_raw(self.sk as *mut _);
            }
        }
    }
}

pub fn bls_key_pair_into_repr_c(key_pair: &NativeBlsKeyPair) -> Result<BlsKeyPair> {
    Ok(BlsKeyPair {
        pk: CString::new(key_pair.pk.clone())?.into_raw(),
        sk: CString::new(key_pair.sk.clone())?.into_raw(),
    })
}

#[repr(C)]
pub struct SafeKey {
    pub xorurl: *const c_char,
    pub xorname: XorNameArray,
    pub resolved_from: *const c_char,
}

impl Drop for SafeKey {
    fn drop(&mut self) {
        unsafe {
            if !self.xorurl.is_null() {
                let _ = CString::from_raw(self.xorurl as *mut _);
            }

            if !self.resolved_from.is_null() {
                let _ = CString::from_raw(self.resolved_from as *mut _);
            }
        }
    }
}

#[repr(C)]
pub struct Wallet {
    pub xorurl: *const c_char,
    pub xorname: XorNameArray,
    pub type_tag: u64,
    pub balances: WalletSpendableBalances,
    pub data_type: u64,
    pub resolved_from: *const c_char,
}

impl Drop for Wallet {
    fn drop(&mut self) {
        unsafe {
            if !self.xorurl.is_null() {
                let _ = CString::from_raw(self.xorurl as *mut _);
            }

            if !self.resolved_from.is_null() {
                let _ = CString::from_raw(self.resolved_from as *mut _);
            }
        }
    }
}

#[repr(C)]
pub struct FilesContainer {
    pub xorurl: *const c_char,
    pub xorname: XorNameArray,
    pub type_tag: u64,
    pub version: u64,
    pub files_map: FilesMap,
    pub data_type: u64,
    pub resolved_from: *const c_char,
}

impl Drop for FilesContainer {
    fn drop(&mut self) {
        unsafe {
            if !self.xorurl.is_null() {
                let _ = CString::from_raw(self.xorurl as *mut _);
            }

            if !self.resolved_from.is_null() {
                let _ = CString::from_raw(self.resolved_from as *mut _);
            }
        }
    }
}

#[repr(C)]
pub struct PublicImmutableData {
    pub xorurl: *const c_char,
    pub xorname: XorNameArray,
    pub data: *const u8,
    pub data_len: usize,
    pub media_type: *const c_char,
    pub metadata: *const c_char,
    pub resolved_from: *const c_char,
}

impl Drop for PublicImmutableData {
    fn drop(&mut self) {
        unsafe {
            if !self.xorurl.is_null() {
                let _ = CString::from_raw(self.xorurl as *mut _);
            }

            if !self.media_type.is_null() {
                let _ = CString::from_raw(self.media_type as *mut _);
            }

            if !self.metadata.is_null() {
                let _ = CString::from_raw(self.metadata as *mut _);
            }

            if !self.resolved_from.is_null() {
                let _ = CString::from_raw(self.resolved_from as *mut _);
            }

            let _ = vec_from_raw_parts(self.data as *mut u8, self.data_len);
        }
    }
}

#[repr(C)]
pub struct NrsMapContainer {
    pub public_name: *const c_char,
    pub xorurl: *const c_char,
    pub xorname: XorNameArray,
    pub type_tag: u64,
    pub version: u64,
    pub nrs_map: *const c_char,
    pub data_type: u64,
    pub resolved_from: *const c_char,
}

impl Drop for NrsMapContainer {
    fn drop(&mut self) {
        unsafe {
            if !self.public_name.is_null() {
                let _ = CString::from_raw(self.public_name as *mut _);
            }

            if !self.xorurl.is_null() {
                let _ = CString::from_raw(self.xorurl as *mut _);
            }

            if !self.nrs_map.is_null() {
                let _ = CString::from_raw(self.nrs_map as *mut _);
            }

            if !self.resolved_from.is_null() {
                let _ = CString::from_raw(self.resolved_from as *mut _);
            }
        }
    }
}

#[repr(C)]
pub struct SequenceData {
    pub xorurl: *const c_char,
    pub xorname: XorNameArray,
    pub type_tag: u64,
    pub version: u64,
    pub data: *const u8,
    pub data_len: usize,
    pub resolved_from: *const c_char,
    pub is_private: bool,
}

impl Drop for SequenceData {
    fn drop(&mut self) {
        unsafe {
            if !self.xorurl.is_null() {
                let _ = CString::from_raw(self.xorurl as *mut _);
            }

            if !self.resolved_from.is_null() {
                let _ = CString::from_raw(self.resolved_from as *mut _);
            }

            let _ = vec_from_raw_parts(self.data as *mut u8, self.data_len);
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct SafeUrl {
    pub encoding_version: u64,
    pub xorname: XorNameArray,
    pub public_name: *const c_char,
    pub top_name: *const c_char,
    pub sub_names: *const c_char,
    pub sub_names_list: *const *const c_char,
    pub sub_names_list_len: usize,
    pub type_tag: u64,
    pub data_type: u64,
    pub content_type: u16,
    pub path: *const c_char,
    pub query_string: *const c_char,
    pub fragment: *const c_char,
    pub content_version: u64,
    pub safeurl_type: u16,
}

impl Drop for SafeUrl {
    fn drop(&mut self) {
        unsafe {
            if !self.public_name.is_null() {
                let _ = CString::from_raw(self.public_name as *mut _);
            }

            if !self.top_name.is_null() {
                let _ = CString::from_raw(self.top_name as *mut _);
            }

            if !self.sub_names.is_null() {
                let _ = CString::from_raw(self.sub_names as *mut _);
            }

            if !self.path.is_null() {
                let _ = CString::from_raw(self.path as *mut _);
            }

            if !self.query_string.is_null() {
                let _ = CString::from_raw(self.query_string as *mut _);
            }

            if !self.fragment.is_null() {
                let _ = CString::from_raw(self.fragment as *mut _);
            }

            let _ = vec_from_raw_parts(
                self.sub_names_list as *mut *const c_char,
                self.sub_names_list_len,
            );
        }
    }
}

pub unsafe fn safe_url_into_repr_c(safe_url: NativeSafeUrl) -> Result<SafeUrl> {
    let sub_names_list = if safe_url.sub_names_vec().is_empty() {
        std::ptr::null()
    } else {
        string_vec_to_c_str_str(safe_url.sub_names_vec().to_vec())?
    };
    Ok(SafeUrl {
        encoding_version: safe_url.encoding_version(),
        xorname: safe_url.xorname().0,
        public_name: CString::new(safe_url.public_name())?.into_raw(),
        top_name: CString::new(safe_url.top_name())?.into_raw(),
        sub_names: CString::new(safe_url.sub_names())?.into_raw(),
        sub_names_list,
        sub_names_list_len: safe_url.sub_names_vec().len(),
        type_tag: safe_url.type_tag(),
        data_type: safe_url.data_type() as u64,
        content_type: safe_url.content_type().value()?,
        path: CString::new(safe_url.path())?.into_raw(),
        query_string: CString::new(safe_url.query_string())?.into_raw(),
        fragment: CString::new(safe_url.fragment())?.into_raw(),
        content_version: safe_url.content_version().unwrap_or_else(|| 0),
        safeurl_type: safe_url.safeurl_type().value()?,
    })
}

#[repr(C)]
pub struct WalletSpendableBalance {
    pub xorurl: *const c_char,
    pub sk: *const c_char,
}

impl Drop for WalletSpendableBalance {
    fn drop(&mut self) {
        unsafe {
            if !self.xorurl.is_null() {
                let _ = CString::from_raw(self.xorurl as *mut _);
            }

            if !self.sk.is_null() {
                let _ = CString::from_raw(self.sk as *mut _);
            }
        }
    }
}

pub fn wallet_spendable_balance_into_repr_c(
    wallet_balance: &NativeWalletSpendableBalance,
) -> Result<WalletSpendableBalance> {
    Ok(WalletSpendableBalance {
        xorurl: CString::new(wallet_balance.xorurl.clone())?.into_raw(),
        sk: CString::new(wallet_balance.sk.clone())?.into_raw(),
    })
}

#[repr(C)]
pub struct WalletSpendableBalanceInfo {
    pub wallet_name: *const c_char,
    pub is_default: bool,
    pub spendable_balance: WalletSpendableBalance,
}

impl Drop for WalletSpendableBalanceInfo {
    fn drop(&mut self) {
        unsafe {
            if !self.wallet_name.is_null() {
                let _ = CString::from_raw(self.wallet_name as *mut _);
            }
        }
    }
}

#[repr(C)]
pub struct WalletSpendableBalances {
    pub wallet_balances: *const WalletSpendableBalanceInfo,
    pub wallet_balances_len: usize,
}

impl Drop for WalletSpendableBalances {
    fn drop(&mut self) {
        unsafe {
            let _ = vec_from_raw_parts(
                self.wallet_balances as *mut WalletSpendableBalanceInfo,
                self.wallet_balances_len,
            );
        }
    }
}

pub fn wallet_spendable_balances_into_repr_c(
    wallet_balances: &NativeWalletSpendableBalances,
) -> Result<WalletSpendableBalances> {
    let mut vec = Vec::with_capacity(wallet_balances.len());

    for (name, (is_default, spendable_balance)) in wallet_balances {
        vec.push(WalletSpendableBalanceInfo {
            wallet_name: CString::new(name.to_string())?.into_raw(),
            is_default: *is_default,
            spendable_balance: wallet_spendable_balance_into_repr_c(&spendable_balance)?,
        })
    }

    let (balance, balance_len) = vec_into_raw_parts(vec);
    Ok(WalletSpendableBalances {
        wallet_balances: balance,
        wallet_balances_len: balance_len,
    })
}

#[repr(C)]
pub struct ProcessedFile {
    pub file_name: *const c_char,
    pub file_meta_data: *const c_char,
    pub file_xorurl: *const c_char,
}

impl Drop for ProcessedFile {
    fn drop(&mut self) {
        unsafe {
            if !self.file_name.is_null() {
                let _ = CString::from_raw(self.file_name as *mut _);
            }

            if !self.file_meta_data.is_null() {
                let _ = CString::from_raw(self.file_meta_data as *mut _);
            }

            if !self.file_xorurl.is_null() {
                let _ = CString::from_raw(self.file_xorurl as *mut _);
            }
        }
    }
}

#[repr(C)]
pub struct ProcessedFiles {
    pub files: *const ProcessedFile,
    pub files_len: usize,
}

impl Drop for ProcessedFiles {
    fn drop(&mut self) {
        unsafe {
            let _ = vec_from_raw_parts(self.files as *mut ProcessedFile, self.files_len);
        }
    }
}

pub unsafe fn processed_files_into_repr_c(map: &NativeProcessedFiles) -> Result<ProcessedFiles> {
    let mut vec = Vec::with_capacity(map.len());

    for (file_name, (file_meta_data, file_xorurl)) in map {
        vec.push(ProcessedFile {
            file_name: CString::new(file_name.to_string())?.into_raw(),
            file_meta_data: CString::new(file_meta_data.to_string())?.into_raw(),
            file_xorurl: CString::new(file_xorurl.to_string())?.into_raw(),
        })
    }

    let (files, files_len) = vec_into_raw_parts(vec);
    Ok(ProcessedFiles { files, files_len })
}

#[repr(C)]
pub struct FileMetaDataItem {
    pub meta_data_key: *const c_char,
    pub meta_data_value: *const c_char,
}

impl Drop for FileMetaDataItem {
    fn drop(&mut self) {
        unsafe {
            if !self.meta_data_key.is_null() {
                let _ = CString::from_raw(self.meta_data_key as *mut _);
            }

            if !self.meta_data_value.is_null() {
                let _ = CString::from_raw(self.meta_data_value as *mut _);
            }
        }
    }
}

#[repr(C)]
pub struct FileInfo {
    pub file_name: *const c_char,
    pub file_meta_data: *const FileMetaDataItem,
    pub file_meta_data_len: usize,
}

impl Drop for FileInfo {
    fn drop(&mut self) {
        unsafe {
            if !self.file_name.is_null() {
                let _ = CString::from_raw(self.file_name as *mut _);
            }

            let _ = vec_from_raw_parts(
                self.file_meta_data as *mut FileMetaDataItem,
                self.file_meta_data_len,
            );
        }
    }
}

pub unsafe fn file_info_into_repr_c(
    file_name: &str,
    file_item_map: &NativeFileItem,
) -> Result<FileInfo> {
    let mut vec = Vec::with_capacity(file_item_map.len());

    for (file_meta_data, xorurl) in file_item_map {
        vec.push(FileMetaDataItem {
            meta_data_key: CString::new(file_meta_data.to_string())?.into_raw(),
            meta_data_value: CString::new(xorurl.to_string())?.into_raw(),
        })
    }

    let (file_meta_data, file_meta_data_len) = vec_into_raw_parts(vec);
    Ok(FileInfo {
        file_name: CString::new(file_name.to_string())?.into_raw(),
        file_meta_data,
        file_meta_data_len,
    })
}

#[repr(C)]
pub struct FilesMap {
    pub files: *const FileInfo,
    pub files_len: usize,
}

impl Drop for FilesMap {
    fn drop(&mut self) {
        unsafe {
            let _ = vec_from_raw_parts(self.files as *mut FileInfo, self.files_len);
        }
    }
}

pub unsafe fn files_map_into_repr_c(files_map: &NativeFilesMap) -> Result<FilesMap> {
    let mut vec = Vec::with_capacity(files_map.len());

    for (file_name, file_items) in files_map {
        vec.push(file_info_into_repr_c(file_name, file_items)?);
    }

    let (files, files_len) = vec_into_raw_parts(vec);
    Ok(FilesMap { files, files_len })
}

#[repr(C)]
pub struct ProcessedEntry {
    pub name: *const c_char,
    pub action: *const c_char,
    pub link: *const c_char,
}

impl Drop for ProcessedEntry {
    fn drop(&mut self) {
        unsafe {
            if !self.name.is_null() {
                let _ = CString::from_raw(self.name as *mut _);
            }

            if !self.action.is_null() {
                let _ = CString::from_raw(self.action as *mut _);
            }

            if !self.link.is_null() {
                let _ = CString::from_raw(self.link as *mut _);
            }
        }
    }
}

#[repr(C)]
pub struct ProcessedEntries {
    pub processed_entries: *const ProcessedEntry,
    pub processed_entries_len: usize,
}

impl Drop for ProcessedEntries {
    fn drop(&mut self) {
        unsafe {
            let _ = vec_from_raw_parts(
                self.processed_entries as *mut ProcessedEntry,
                self.processed_entries_len,
            );
        }
    }
}

pub unsafe fn processed_entries_into_repr_c(
    entries: &NativeProcessedEntries,
) -> Result<ProcessedEntries> {
    let mut vec = Vec::with_capacity(entries.len());

    for (name, (action, link)) in entries {
        vec.push(ProcessedEntry {
            name: CString::new(name.to_string())?.into_raw(),
            action: CString::new(action.to_string())?.into_raw(),
            link: CString::new(link.to_string())?.into_raw(),
        })
    }

    let (processed_entries, processed_entries_len) = vec_into_raw_parts(vec);
    Ok(ProcessedEntries {
        processed_entries,
        processed_entries_len,
    })
}

#[repr(C)]
pub struct NrsMap {
    pub sub_names_map: SubNamesMap,
    pub default: *const c_char,
}

pub fn nrs_map_into_repr_c(nrs_map: &NativeNrsMap) -> Result<NrsMap> {
    Ok(NrsMap {
        sub_names_map: sub_names_map_into_repr_c(nrs_map.sub_names_map.clone())?,
        default: std::ptr::null(), // todo: update to return correct format
    })
}

#[repr(C)]
pub struct SubNamesMapEntry {
    pub sub_name: *const c_char,
    pub sub_name_rdf: *const c_char, // Needs to be updated to correct format
}

#[repr(C)]
pub struct SubNamesMap {
    pub sub_names: *const SubNamesMapEntry,
    pub sub_name_len: usize,
}

pub fn sub_names_map_into_repr_c(map: NativeSubNamesMap) -> Result<SubNamesMap> {
    let mut vec = Vec::with_capacity(map.len());

    for (sub_name, _sub_name_rdf) in map {
        vec.push(SubNamesMapEntry {
            sub_name: CString::new(sub_name)?.into_raw(),
            sub_name_rdf: std::ptr::null(), // todo: update to return correct format
        })
    }

    let (sub_names, sub_name_len) = vec_into_raw_parts(vec);
    Ok(SubNamesMap {
        sub_names,
        sub_name_len,
    })
}

impl Drop for SubNamesMap {
    fn drop(&mut self) {
        unsafe {
            let _ = vec_from_raw_parts(self.sub_names as *mut SubNamesMapEntry, self.sub_name_len);
        }
    }
}
