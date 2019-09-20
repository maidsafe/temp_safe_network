use safe_api::{BlsKeyPair, NrsMap, ResultReturn, XorUrlEncoder};
use safe_core::ffi::arrays::XorNameArray;
use std::ffi::CString;
use std::os::raw::c_char;

#[repr(C)]
pub struct FfiBlsKeyPair {
    pub pk: *const c_char,
    pub sk: *const c_char,
}

pub fn bls_key_pair_into_repr_c(key_pair: &BlsKeyPair) -> ResultReturn<FfiBlsKeyPair> {
    Ok(FfiBlsKeyPair {
        pk: CString::new(key_pair.pk.clone())?.into_raw(),
        sk: CString::new(key_pair.sk.clone())?.into_raw(),
    })
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
    pub content_version: u64,
}

pub fn xorurl_encoder_into_repr_c(xorurl_encoder: XorUrlEncoder) -> ResultReturn<FfiXorUrlEncoder> {
    Ok(FfiXorUrlEncoder {
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
pub struct FfiNrsMap {
    // TODO
}

pub fn nrs_map_into_repr_c(_nrs_map: &NrsMap) -> ResultReturn<FfiNrsMap> {
    Ok(FfiNrsMap {})
}
