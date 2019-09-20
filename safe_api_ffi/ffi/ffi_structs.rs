use ffi_utils::vec_into_raw_parts;
use safe_api::wallet::{
    WalletSpendableBalance as NativeWalletSpendableBalance,
    WalletSpendableBalances as NativeWalletSpendableBalances,
};
use safe_api::NrsMap as NativeNrsMap;
use safe_api::{
    BlsKeyPair as NativeBlsKeyPair, ResultReturn, XorUrlEncoder as NativeXorUrlEncoder,
};
use safe_core::ffi::arrays::XorNameArray;
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
pub struct XorUrlEncoder {
    pub encoding_version: u64,
    pub xorname: XorNameArray,
    pub type_tag: u64,
    pub data_type: u64,
    pub content_type: u16,
    pub path: *const c_char,
    pub sub_names: *const c_char,
    pub content_version: u64,
}

pub fn xorurl_encoder_into_repr_c(
    xorurl_encoder: NativeXorUrlEncoder,
) -> ResultReturn<XorUrlEncoder> {
    Ok(XorUrlEncoder {
        encoding_version: xorurl_encoder.encoding_version(),
        xorname: xorurl_encoder.xorname().0,
        type_tag: xorurl_encoder.type_tag(),
        data_type: xorurl_encoder.data_type() as u64,
        content_type: xorurl_encoder.content_type().value()?,
        path: std::ptr::null(),
        sub_names: std::ptr::null(),
        content_version: xorurl_encoder.content_version().unwrap_or_else(|| 0),
    })
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
pub struct WalletSpendableBalances {
    pub wallet_spendable_balance: *const WalletSpendableBalance,
    pub balance_len: usize,
    pub balance_cap: usize,
}

pub fn wallet_spendable_balances_into_repr_c(
    wallet_balances: NativeWalletSpendableBalances,
) -> ResultReturn<WalletSpendableBalances> {
    let mut vec = Vec::with_capacity(wallet_balances.len());

    for (_name, (_bool_value, spendable_balance)) in wallet_balances {
        vec.push(WalletSpendableBalance {
            xorurl: CString::new(spendable_balance.xorurl)?.into_raw(),
            sk: CString::new(spendable_balance.sk)?.into_raw(),
        })
    }

    let (balance, balance_len, balance_cap) = vec_into_raw_parts(vec);
    Ok(WalletSpendableBalances {
        wallet_spendable_balance: balance,
        balance_len,
        balance_cap,
    })
}

#[repr(C)]
pub struct NrsMap {
    // TODO
}

pub fn nrs_map_into_repr_c(_nrs_map: &NativeNrsMap) -> ResultReturn<NrsMap> {
    Ok(NrsMap {})
}
