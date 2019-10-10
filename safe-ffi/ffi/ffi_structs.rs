use super::helpers::from_c_str_to_str_option;
use ffi_utils::{from_c_str, vec_into_raw_parts};
use safe_api::files::{
    FileItem as NativeFileItem, FilesMap as NativeFilesMap, ProcessedFiles as NativeProcessedFiles,
};
use safe_api::nrs_map::{NrsMap as NativeNrsMap, SubNamesMap as NativeSubNamesMap};
use safe_api::wallet::{
    WalletSpendableBalance as NativeWalletSpendableBalance,
    WalletSpendableBalances as NativeWalletSpendableBalances,
};
use safe_api::xorurl::{SafeContentType, SafeDataType, XorUrlEncoder as NativeXorUrlEncoder};
use safe_api::{
    BlsKeyPair as NativeBlsKeyPair, NrsMapContainerInfo as NativeNrsMapContainerInfo,
    ProcessedEntries as NativeProcessedEntries, ResultReturn,
};
use safe_nd::{XorName, XOR_NAME_LEN};
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

pub fn bls_key_pair_into_repr_c(key_pair: &NativeBlsKeyPair) -> ResultReturn<BlsKeyPair> {
    Ok(BlsKeyPair {
        pk: CString::new(key_pair.pk.clone())?.into_raw(),
        sk: CString::new(key_pair.sk.clone())?.into_raw(),
    })
}

#[repr(C)]
pub struct SafeKey {
    pub xorurl: *const c_char,
    pub xorname: XorNameArray,
    pub resolved_from: NrsMapContainerInfo,
}

impl Drop for SafeKey {
    fn drop(&mut self) {
        unsafe {
            if !self.xorurl.is_null() {
                let _ = CString::from_raw(self.xorurl as *mut _);
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
    pub resolved_from: NrsMapContainerInfo,
}

impl Drop for Wallet {
    fn drop(&mut self) {
        unsafe {
            if !self.xorurl.is_null() {
                let _ = CString::from_raw(self.xorurl as *mut _);
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
    pub files_map: *const c_char,
    pub data_type: u64,
    pub resolved_from: NrsMapContainerInfo,
}

impl Drop for FilesContainer {
    fn drop(&mut self) {
        unsafe {
            if !self.xorurl.is_null() {
                let _ = CString::from_raw(self.xorurl as *mut _);
            }

            if !self.files_map.is_null() {
                let _ = CString::from_raw(self.files_map as *mut _);
            }
        }
    }
}

#[repr(C)]
pub struct PublishedImmutableData {
    pub xorurl: *const c_char,
    pub xorname: XorNameArray,
    pub data: *const u8,
    pub data_len: usize,
    pub data_cap: usize,
    pub resolved_from: NrsMapContainerInfo,
    pub media_type: *const c_char,
}

impl Drop for PublishedImmutableData {
    fn drop(&mut self) {
        unsafe {
            if !self.xorurl.is_null() {
                let _ = CString::from_raw(self.xorurl as *mut _);
            }

            if !self.media_type.is_null() {
                let _ = CString::from_raw(self.media_type as *mut _);
            }

            let _ = Vec::from_raw_parts(self.data as *mut u8, self.data_len, self.data_cap);
        }
    }
}

#[repr(C)]
pub struct XorUrlEncoder {
    pub encoding_version: u64,
    pub xorname: XorNameArray,
    pub type_tag: u64,
    pub data_type: u64,
    pub content_type: u16,
    pub path: *const c_char,
    pub sub_names: *const c_char,
    // pub sub_names: *const *const c_char, // Todo: update to String Vec
    // pub sub_names_len: usize,
    pub content_version: u64,
}

impl Drop for XorUrlEncoder {
    fn drop(&mut self) {
        unsafe {
            if !self.path.is_null() {
                let _ = CString::from_raw(self.path as *mut _);
            }

            if !self.sub_names.is_null() {
                let _ = CString::from_raw(self.sub_names as *mut _);
            }
        }
    }
}

pub unsafe fn xorurl_encoder_into_repr_c(
    xorurl_encoder: NativeXorUrlEncoder,
) -> ResultReturn<XorUrlEncoder> {
    // let sub_names = string_vec_to_c_str_str(xorurl_encoder.sub_names())?; // Todo: update to String Vec
    let sub_names = serde_json::to_string(&xorurl_encoder.sub_names())?;
    Ok(XorUrlEncoder {
        encoding_version: xorurl_encoder.encoding_version(),
        xorname: xorurl_encoder.xorname().0,
        type_tag: xorurl_encoder.type_tag(),
        data_type: xorurl_encoder.data_type() as u64,
        content_type: xorurl_encoder.content_type().value()?,
        path: CString::new(xorurl_encoder.path().to_string())?.into_raw(),
        sub_names: CString::new(sub_names)?.into_raw(),
        // sub_names: sub_names, // Todo: update to String Vec
        // sub_names_len: xorurl_encoder.sub_names().len(),
        content_version: xorurl_encoder.content_version().unwrap_or_else(|| 0),
    })
}

pub unsafe fn native_xorurl_encoder_from_repr_c(
    encoder: &XorUrlEncoder,
) -> ResultReturn<NativeXorUrlEncoder> {
    let sub_names: Vec<String> = serde_json::from_str(&from_c_str(encoder.sub_names)?)?;
    Ok(NativeXorUrlEncoder::new(
        XorName(encoder.xorname),
        encoder.type_tag,
        SafeDataType::from_u64(encoder.data_type)?,
        SafeContentType::from_u16(encoder.content_type)?,
        from_c_str_to_str_option(encoder.path),
        Some(sub_names),
        // c_str_str_to_string_vec(encoder.sub_names, encoder.sub_names_len), // Todo: update to String Vec
        Some(encoder.content_version),
    )?)
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
) -> ResultReturn<WalletSpendableBalance> {
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
    pub wallet_balances_cap: usize,
}

impl Drop for WalletSpendableBalances {
    fn drop(&mut self) {
        unsafe {
            let _ = Vec::from_raw_parts(
                self.wallet_balances as *mut WalletSpendableBalanceInfo,
                self.wallet_balances_len,
                self.wallet_balances_cap,
            );
        }
    }
}

pub fn wallet_spendable_balances_into_repr_c(
    wallet_balances: &NativeWalletSpendableBalances,
) -> ResultReturn<WalletSpendableBalances> {
    let mut vec = Vec::with_capacity(wallet_balances.len());

    for (name, (is_default, spendable_balance)) in wallet_balances {
        vec.push(WalletSpendableBalanceInfo {
            wallet_name: CString::new(name.to_string())?.into_raw(),
            is_default: *is_default,
            spendable_balance: wallet_spendable_balance_into_repr_c(&spendable_balance)?,
        })
    }

    let (balance, balance_len, balance_cap) = vec_into_raw_parts(vec);
    Ok(WalletSpendableBalances {
        wallet_balances: balance,
        wallet_balances_len: balance_len,
        wallet_balances_cap: balance_cap,
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
    pub files_cap: usize,
}

impl Drop for ProcessedFiles {
    fn drop(&mut self) {
        unsafe {
            let _ = Vec::from_raw_parts(
                self.files as *mut ProcessedFile,
                self.files_len,
                self.files_cap,
            );
        }
    }
}

pub unsafe fn processed_files_into_repr_c(
    map: &NativeProcessedFiles,
) -> ResultReturn<ProcessedFiles> {
    let mut vec = Vec::with_capacity(map.len());

    for (file_name, (file_meta_data, file_xorurl)) in map {
        vec.push(ProcessedFile {
            file_name: CString::new(file_name.to_string())?.into_raw(),
            file_meta_data: CString::new(file_meta_data.to_string())?.into_raw(),
            file_xorurl: CString::new(file_xorurl.to_string())?.into_raw(),
        })
    }

    let (files, files_len, files_cap) = vec_into_raw_parts(vec);
    Ok(ProcessedFiles {
        files,
        files_len,
        files_cap,
    })
}

#[repr(C)]
pub struct FileItem {
    pub file_meta_data: *const c_char,
    pub xorurl: *const c_char,
}

impl Drop for FileItem {
    fn drop(&mut self) {
        unsafe {
            if !self.file_meta_data.is_null() {
                let _ = CString::from_raw(self.file_meta_data as *mut _);
            }

            if !self.xorurl.is_null() {
                let _ = CString::from_raw(self.xorurl as *mut _);
            }
        }
    }
}

#[repr(C)]
pub struct FileInfo {
    pub file_name: *const c_char,
    pub file_items: *const FileItem,
    pub file_items_len: usize,
    pub file_items_cap: usize,
}

impl Drop for FileInfo {
    fn drop(&mut self) {
        unsafe {
            if !self.file_name.is_null() {
                let _ = CString::from_raw(self.file_name as *mut _);
            }

            let _ = Vec::from_raw_parts(
                self.file_items as *mut FileItem,
                self.file_items_len,
                self.file_items_cap,
            );
        }
    }
}

pub unsafe fn file_info_into_repr_c(
    file_name: &str,
    file_item_map: &NativeFileItem,
) -> ResultReturn<FileInfo> {
    let mut vec = Vec::with_capacity(file_item_map.len());

    for (file_meta_data, xorurl) in file_item_map {
        vec.push(FileItem {
            file_meta_data: CString::new(file_meta_data.to_string())?.into_raw(),
            xorurl: CString::new(xorurl.to_string())?.into_raw(),
        })
    }

    let (file_items, file_items_len, file_items_cap) = vec_into_raw_parts(vec);
    Ok(FileInfo {
        file_name: CString::new(file_name.to_string())?.into_raw(),
        file_items,
        file_items_len,
        file_items_cap,
    })
}

#[repr(C)]
pub struct FilesMap {
    pub files: *const FileInfo,
    pub files_len: usize,
    pub files_cap: usize,
}

impl Drop for FilesMap {
    fn drop(&mut self) {
        unsafe {
            let _ =
                Vec::from_raw_parts(self.files as *mut FileInfo, self.files_len, self.files_cap);
        }
    }
}

pub unsafe fn files_map_into_repr_c(files_map: &NativeFilesMap) -> ResultReturn<FilesMap> {
    let mut vec = Vec::with_capacity(files_map.len());

    for (file_name, file_items) in files_map {
        vec.push(file_info_into_repr_c(file_name, file_items)?);
    }

    let (files, files_len, files_cap) = vec_into_raw_parts(vec);
    Ok(FilesMap {
        files,
        files_len,
        files_cap,
    })
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
    pub processed_entries_cap: usize,
}

impl Drop for ProcessedEntries {
    fn drop(&mut self) {
        unsafe {
            let _ = Vec::from_raw_parts(
                self.processed_entries as *mut ProcessedEntry,
                self.processed_entries_len,
                self.processed_entries_cap,
            );
        }
    }
}

pub unsafe fn processed_entries_into_repr_c(
    entries: &NativeProcessedEntries,
) -> ResultReturn<ProcessedEntries> {
    let mut vec = Vec::with_capacity(entries.len());

    for (name, (action, link)) in entries {
        vec.push(ProcessedEntry {
            name: CString::new(name.to_string())?.into_raw(),
            action: CString::new(action.to_string())?.into_raw(),
            link: CString::new(link.to_string())?.into_raw(),
        })
    }

    let (processed_entries, processed_entries_len, processed_entries_cap) = vec_into_raw_parts(vec);
    Ok(ProcessedEntries {
        processed_entries,
        processed_entries_len,
        processed_entries_cap,
    })
}

#[repr(C)]
pub struct NrsMapContainerInfo {
    pub public_name: *const c_char,
    pub xorurl: *const c_char,
    pub xorname: XorNameArray,
    pub type_tag: u64,
    pub version: u64,
    pub nrs_map: *const c_char,
    pub data_type: u64,
}

impl Drop for NrsMapContainerInfo {
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
        }
    }
}

impl NrsMapContainerInfo {
    pub fn new() -> ResultReturn<Self> {
        Ok(Self {
            public_name: std::ptr::null(),
            xorurl: std::ptr::null(),
            xorname: [0; 32],
            type_tag: 0,
            version: 0,
            nrs_map: std::ptr::null(),
            data_type: 0,
        })
    }
}

pub unsafe fn nrs_map_container_info_into_repr_c(
    nrs_container_info: &NativeNrsMapContainerInfo,
) -> ResultReturn<NrsMapContainerInfo> {
    let nrs_map_json = serde_json::to_string(&nrs_container_info.nrs_map)?;
    Ok(NrsMapContainerInfo {
        public_name: CString::new(nrs_container_info.public_name.clone())?.into_raw(),
        xorurl: CString::new(nrs_container_info.xorurl.clone())?.into_raw(),
        xorname: nrs_container_info.xorname.0,
        type_tag: nrs_container_info.type_tag,
        version: nrs_container_info.version,
        nrs_map: CString::new(nrs_map_json)?.into_raw(),
        data_type: nrs_container_info.data_type.clone() as u64,
    })
}

#[repr(C)]
pub struct NrsMap {
    pub sub_names_map: SubNamesMap,
    pub default: *const c_char,
}

pub fn nrs_map_into_repr_c(nrs_map: &NativeNrsMap) -> ResultReturn<NrsMap> {
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
    pub sub_name_cap: usize,
}

pub fn sub_names_map_into_repr_c(map: NativeSubNamesMap) -> ResultReturn<SubNamesMap> {
    let mut vec = Vec::with_capacity(map.len());

    for (sub_name, _sub_name_rdf) in map {
        vec.push(SubNamesMapEntry {
            sub_name: CString::new(sub_name)?.into_raw(),
            sub_name_rdf: std::ptr::null(), // todo: update to return correct format
        })
    }

    let (sub_names, sub_name_len, sub_name_cap) = vec_into_raw_parts(vec);
    Ok(SubNamesMap {
        sub_names,
        sub_name_len,
        sub_name_cap,
    })
}

impl Drop for SubNamesMap {
    fn drop(&mut self) {
        unsafe {
            let _ = Vec::from_raw_parts(
                self.sub_names as *mut SubNamesMapEntry,
                self.sub_name_len,
                self.sub_name_cap,
            );
        }
    }
}
