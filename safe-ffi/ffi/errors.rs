// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use ffi_utils::{ErrorCode, StringError};
use safe_api::Error;
use std::ffi::NulError;
use std::fmt;

pub type Result<T> = std::result::Result<T, FfiError>;

#[derive(Debug)]
pub struct FfiError(Error);

impl FfiError {
    pub fn error_code(&self) -> i32 {
        self.0.error_code()
    }

    pub fn description(&self) -> String {
        self.0.description()
    }
}

impl fmt::Display for FfiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.description())
    }
}

impl ErrorCode for FfiError {
    fn error_code(&self) -> i32 {
        self.error_code()
    }
}

impl From<Error> for FfiError {
    fn from(error: Error) -> Self {
        FfiError(error)
    }
}

impl From<StringError> for FfiError {
    fn from(_error: StringError) -> Self {
        FfiError(Error::StringError("string conversion error".to_string()))
    }
}

impl<'a> From<&'a str> for FfiError {
    fn from(s: &'a str) -> Self {
        FfiError(Error::Unexpected(s.to_string()))
    }
}

impl From<NulError> for FfiError {
    fn from(_error: NulError) -> Self {
        FfiError(Error::Unexpected("Null error".to_string()))
    }
}

impl From<serde_json::error::Error> for FfiError {
    fn from(_error: serde_json::error::Error) -> Self {
        FfiError(Error::StringError(
            "Failed to serialize or deserialize to json".to_string(),
        ))
    }
}
