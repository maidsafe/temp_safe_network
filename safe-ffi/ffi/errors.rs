// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use ffi_utils::{ErrorCode, StringError};
use safe_api::Error as NativeError;
use std::ffi::NulError;
use std::fmt;

mod codes {
    // Auth Errors
    pub const ERR_AUTH_ERROR: i32 = -100;
    pub const ERR_CONNECTION_ERROR: i32 = -101;
    pub const ERR_ACCESS_DENIED_ERROR: i32 = -102;

    // Data Errors
    pub const ERR_NET_DATA_ERROR: i32 = -200;
    pub const ERR_CONTENT_NOT_FOUND_ERROR: i32 = -201;
    pub const ERR_VERSION_NOT_FOUND_ERROR: i32 = -202;
    pub const ERR_CONTENT_ERROR: i32 = -203;
    pub const ERR_EMPTY_CONTENT_ERROR: i32 = -204;
    pub const ERR_ENTRY_NOT_FOUND_ERROR: i32 = -205;
    pub const ERR_ENTRY_EXISTS_ERROR: i32 = -206;
    pub const ERR_INVALID_INPUT_ERROR: i32 = -207;
    pub const ERR_FILE_SYSTEM_ERROR: i32 = -208;
    pub const ERR_INVALID_MEDIA_TYPE_ERROR: i32 = -209;

    // Balance Errors
    pub const ERR_INVALID_AMOUNT_ERROR: i32 = -300;
    pub const ERR_NOT_ENOUGH_BALANCE_ERROR: i32 = -301;
    pub const ERR_INVALID_XOR_URL_ERROR: i32 = -400;

    // Misc Errors
    pub const ERR_UNEXPECTED_ERROR: i32 = -500;
    pub const ERR_UNKNOWN_ERROR: i32 = -501;
    pub const ERR_STRING_ERROR: i32 = -502;

    // Authd/Authd-Client Errors
    pub const ERR_AUTHD_CLIENT_ERROR: i32 = -600;
    pub const ERR_AUTHD_ERROR: i32 = -601;
    pub const ERR_AUTHD_ALREADY_STARTED_ERROR: i32 = -602;

    // Authenticator Errors
    pub const ERR_AUTHENTICATOR_ERROR: i32 = -700;
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, PartialEq)]
pub struct Error(NativeError);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ErrorCode for Error {
    fn error_code(&self) -> i32 {
        use codes::*;
        use NativeError::*;

        match (*self).0 {
            AuthError(ref _error) => ERR_AUTH_ERROR,
            AuthdClientError(ref _error) => ERR_AUTHD_CLIENT_ERROR,
            AuthdError(ref _error) => ERR_AUTHD_ERROR,
            AuthdAlreadyStarted(ref _error) => ERR_AUTHD_ALREADY_STARTED_ERROR,
            AuthenticatorError(ref _error) => ERR_AUTHENTICATOR_ERROR,
            ConnectionError(ref _error) => ERR_CONNECTION_ERROR,
            NetDataError(ref _error) => ERR_NET_DATA_ERROR,
            ContentNotFound(ref _error) => ERR_CONTENT_NOT_FOUND_ERROR,
            VersionNotFound(ref _error) => ERR_VERSION_NOT_FOUND_ERROR,
            ContentError(ref _error) => ERR_CONTENT_ERROR,
            EmptyContent(ref _error) => ERR_EMPTY_CONTENT_ERROR,
            AccessDenied(ref _error) => ERR_ACCESS_DENIED_ERROR,
            EntryNotFound(ref _error) => ERR_ENTRY_NOT_FOUND_ERROR,
            EntryExists(ref _error) => ERR_ENTRY_EXISTS_ERROR,
            InvalidInput(ref _error) => ERR_INVALID_INPUT_ERROR,
            InvalidAmount(ref _error) => ERR_INVALID_AMOUNT_ERROR,
            InvalidXorUrl(ref _error) => ERR_INVALID_XOR_URL_ERROR,
            NotEnoughBalance(ref _error) => ERR_NOT_ENOUGH_BALANCE_ERROR,
            FilesSystemError(ref _error) => ERR_FILE_SYSTEM_ERROR,
            InvalidMediaType(ref _error) => ERR_INVALID_MEDIA_TYPE_ERROR,
            Unexpected(ref _error) => ERR_UNEXPECTED_ERROR,
            Unknown(ref _error) => ERR_UNKNOWN_ERROR,
        }
    }
}

impl From<NativeError> for Error {
    fn from(error: NativeError) -> Self {
        Self(error)
    }
}

impl From<StringError> for Error {
    fn from(_error: StringError) -> Self {
        StringError::IntoString("string conversion error".into()).into()
    }
}

impl<'a> From<&'a str> for Error {
    fn from(s: &'a str) -> Self {
        NativeError::Unexpected(s.into()).into()
    }
}

impl From<NulError> for Error {
    fn from(_error: NulError) -> Self {
        NativeError::Unexpected("Null error".into()).into()
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(_error: serde_json::error::Error) -> Self {
        StringError::IntoString("Failed to serialize or deserialize to json".into()).into()
    }
}
