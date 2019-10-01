use ffi_utils::from_c_str;
use safe_api::Error;
use serde::de::{Deserialize, DeserializeOwned, Deserializer};
use serde::ser::{Serialize, Serializer};
use serde_json;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::slice;

#[inline]
pub unsafe fn from_c_str_to_string_option(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        None
    } else {
        CStr::from_ptr(ptr).to_owned().into_string().ok()
    }
}

#[inline]
pub unsafe fn from_c_str_to_str_option(ptr: *const c_char) -> Option<&'static str> {
    if ptr.is_null() {
        None
    } else {
        CStr::from_ptr(ptr).to_str().ok()
    }
}

#[inline]
pub fn string_vec_to_c_str_str(argv: Vec<String>) -> Result<*const *const c_char, Error> {
    let cstr_argv: Vec<_> = argv
        .iter()
        .map(|arg| CString::new(arg.as_str()).unwrap())
        .collect();

    let p_argv: Vec<_> = cstr_argv.iter().map(|arg| arg.as_ptr()).collect();

    Ok(p_argv.as_ptr())
}

#[inline]
pub unsafe fn c_str_str_to_string_vec(
    argv: *const *const c_char,
    len: usize,
) -> Option<Vec<String>> {
    if len > 0 {
        let data_vec = slice::from_raw_parts(argv, len).to_vec();
        let string_vec: Vec<String> = data_vec
            .iter()
            .map(|s| from_c_str(*s).expect("Cannot convert"))
            .collect();
        Some(string_vec)
    } else {
        None
    }
}

// Serialize to a JSON string, then serialize the string to the output
// format.
pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: Serialize,
    S: Serializer,
{
    use serde::ser::Error;
    let j = serde_json::to_string(value).map_err(Error::custom)?;
    j.serialize(serializer)
}

// Deserialize a string from the input format, then deserialize the content
// of that string as JSON.
pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: DeserializeOwned,
    D: Deserializer<'de>,
{
    use serde::de::Error;
    let j = String::deserialize(deserializer)?;
    serde_json::from_str(&j).map_err(Error::custom)
}
