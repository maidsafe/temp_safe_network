use safe_core::ffi::arrays::XorNameArray;
use std::os::raw::c_char;

#[repr(C)]
pub struct FfiXorUrlEncoder {
    pub encoding_version: u64,
    pub xorname: XorNameArray,
    pub type_tag: u64,
    pub data_type: u64,
    pub content_type: u16,
    pub path: *const c_char,
    pub sub_names: *const c_char,
    pub content_version: u64
}

#[repr(C)]
pub struct FfiBlsKeyPair {
    pub pk: *const c_char,
    pub sk: *const c_char,
}

pub fn bls_key_pair_into_repr_c(key_pair: &BlsKeyPair) -> ResultReturn<FfiBlsKeyPair> {
    Ok(FfiBlsKeyPair {
        pk: CString::new(key_pair.pk)?.into_raw(),
        sk: CString::new(key_pair.sk)?.into_raw()
    })
}

pub fn xorurl_encoder_into_repr_c(xorurl_encoder: XorUrlEncoder) -> ResultReturn<FfiXorUrlEncoder> {
    let XorUrlEncoder {
        encoding_version,
        xorname,
        type_tag,
        data_type,
        content_type,
        path: _,
        sub_names: _,
        content_version: _,
    } = xorurl_encoder;

    Ok(FfiXorUrlEncoder {
        encoding_version,
        xorname: xorname.0,
        type_tag,
        data_type: data_type as u64,
        content_type: content_type.value(),
        path: std::ptr::null(),
        sub_names: std::ptr::null(),
        content_version: 0,
    })
}
