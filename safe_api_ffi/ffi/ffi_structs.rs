use super::helpers::{from_c_str_to_str_option, to_c_str};
use ffi_utils::{from_c_str, vec_into_raw_parts};
use safe_api::files::{FilesMap as NativeFilesMap, ProcessedFiles as NativeProcessedFiles};
use safe_api::nrs_map::{NrsMap as NativeNrsMap, SubNamesMap as NativeSubNamesMap};
use safe_api::wallet::{
    WalletSpendableBalance as NativeWalletSpendableBalance,
    WalletSpendableBalances as NativeWalletSpendableBalances,
};
use safe_api::xorurl::{SafeContentType, SafeDataType, XorUrlEncoder as NativeXorUrlEncoder};
use safe_api::{
    BlsKeyPair as NativeBlsKeyPair, NrsMapContainerInfo as NativeNrsMapContainerInfo, ResultReturn,
};
use safe_core::ffi::arrays::XorNameArray;
use safe_nd::XorName;
use std::ffi::CString;
use std::os::raw::c_char;

#[repr(C)]
pub struct BlsKeyPair {
    pub pk: *const c_char,
    pub sk: *const c_char,
}

pub fn bls_key_pair_into_repr_c(key_pair: &NativeBlsKeyPair) -> ResultReturn<BlsKeyPair> {
    Ok(BlsKeyPair {
        pk: CString::new(key_pair.pk.clone())?.into_raw(),
        sk: CString::new(key_pair.sk.clone())?.into_raw(),
    })
}

#[repr(C)]
pub struct SafeKey {
    pub xorname: XorNameArray,
    pub resolved_from: *const c_char,
}

#[repr(C)]
pub struct Wallet {
    pub xorname: XorNameArray,
    pub type_tag: u64,
    pub balances: *const c_char,
    pub data_type: u64,
    pub resolved_from: *const c_char,
}

#[repr(C)]
pub struct FilesContainer {
    pub xorname: XorNameArray,
    pub type_tag: u64,
    pub version: u64,
    pub files_map: *const c_char,
    pub data_type: u64,
    pub resolved_from: *const c_char,
}

#[repr(C)]
pub struct PublishedImmutableData {
    pub xorname: XorNameArray,
    pub data: *const u8,
    pub data_len: usize,
    pub resolved_from: *const c_char,
    pub media_type: *const c_char,
}

#[repr(C)]
pub struct FfiXorUrlEncoder {
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
        path: to_c_str(xorurl_encoder.path().to_string())?.as_ptr(),
        sub_names: to_c_str(sub_names)?.as_ptr(),
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

pub fn wallet_spendable_balance_into_repr_c(
    wallet_balance: &NativeWalletSpendableBalance,
) -> ResultReturn<WalletSpendableBalance> {
    Ok(WalletSpendableBalance {
        xorurl: CString::new(wallet_balance.xorurl.clone())?.into_raw(),
        sk: CString::new(wallet_balance.sk.clone())?.into_raw(),
    })
}

#[repr(C)]
pub struct SependableWalletBalance {
    pub wallet_name: *const c_char,
    pub is_default: bool,
    pub spendable_wallet_balance: WalletSpendableBalance,
}

#[repr(C)]
pub struct WalletSpendableBalances {
    pub wallet_balances: *const SependableWalletBalance,
    pub wallet_balances_len: usize,
    pub wallet_balances_cap: usize,
}

impl Drop for WalletSpendableBalances {
    fn drop(&mut self) {
        unsafe {
            let _ = Vec::from_raw_parts(
                self.wallet_balances as *mut SependableWalletBalance,
                self.wallet_balances_len,
                self.wallet_balances_cap,
            );
        }
    }
}

pub fn wallet_spendable_balances_into_repr_c(
    wallet_balances: NativeWalletSpendableBalances,
) -> ResultReturn<WalletSpendableBalances> {
    let mut vec = Vec::with_capacity(wallet_balances.len());

    for (name, (is_default, spendable_balance)) in wallet_balances {
        vec.push(SependableWalletBalance {
            wallet_name: CString::new(name)?.into_raw(),
            is_default: is_default,
            spendable_wallet_balance: wallet_spendable_balance_into_repr_c(&spendable_balance)?,
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
pub struct ProcessedFiles {
    // todo
}

pub fn processed_files_into_repr_c(
    _nrs_map: &NativeProcessedFiles,
) -> ResultReturn<ProcessedFiles> {
    Ok(ProcessedFiles {}) // todo
}

#[repr(C)]
pub struct FilesMap {
    // todo
}

pub fn files_map_into_repr_c(_nrs_map: &NativeFilesMap) -> ResultReturn<FilesMap> {
    Ok(FilesMap {}) //todo
}

#[repr(C)]
pub struct NrsMapContainerInfo {
    pub public_name: *const c_char,
    pub xorurl: *const c_char,
    pub xorname: XorNameArray,
    pub type_tag: u64,
    pub version: u64,
    pub nrs_map: NrsMap,
    pub data_type: u64,
}

pub fn nrs_map_container_info_into_repr_c(
    nrs_container_info: &NativeNrsMapContainerInfo,
) -> ResultReturn<NrsMapContainerInfo> {
    Ok(NrsMapContainerInfo {
        public_name: CString::new(nrs_container_info.public_name.clone())?.into_raw(),
        xorurl: CString::new(nrs_container_info.xorurl.clone())?.into_raw(),
        xorname: nrs_container_info.xorname.0,
        type_tag: nrs_container_info.type_tag,
        version: nrs_container_info.version,
        nrs_map: nrs_map_into_repr_c(&nrs_container_info.nrs_map)?,
        data_type: nrs_container_info.data_type.clone() as u64,
    })
}

#[repr(C)]
pub struct NrsMap {
    // TODO
}

pub fn nrs_map_into_repr_c(_nrs_map: &NativeNrsMap) -> ResultReturn<NrsMap> {
    Ok(NrsMap {})
}
