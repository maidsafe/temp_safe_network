use safe_api::Error;
use serde::de::{Deserialize, DeserializeOwned, Deserializer};
use serde::ser::{Serialize, Serializer};
use serde_json;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

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
pub unsafe fn to_c_str(native_string: String) -> Result<CString, Error> {
    CString::new(native_string)
        .map_err(|_| Error::StringError("Couldn't convert to string".to_string()))
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
