// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use std::fmt;
use ffi_utils::{ErrorCode, StringError};
use std::ffi::NulError;
pub use self::codes::*;

pub type ResultReturn<T> = Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    AuthError(String),
    ConnectionError(String),
    NetDataError(String),
    ContentNotFound(String),
    ContentError(String),
    EmptyContent(String),
    AccessDenied(String),
    VersionNotFound(String),
    EntryNotFound(String),
    EntryExists(String),
    InvalidInput(String),
    InvalidAmount(String),
    InvalidXorUrl(String),
    InvalidMediaType(String),
    NotEnoughBalance(String),
    FilesSystemError(String),
    Unexpected(String),
    Unknown(String),
    StringError(String)
}

impl From<Error> for String {
    fn from(error: Error) -> String {
        get_error_info(&error)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", get_error_info(self))
    }
}

fn get_error_info(error: &Error) -> String {
    let (error_type, error_msg) = match error {
        Error::AuthError(info) => ("AuthError".to_string(), info.to_string()),
        Error::ConnectionError(info) => ("ConnectionError".to_string(), info.to_string()),
        Error::NetDataError(info) => ("NetDataError".to_string(), info.to_string()),
        Error::ContentNotFound(info) => ("ContentNotFound".to_string(), info.to_string()),
        Error::VersionNotFound(info) => ("VersionNotFound".to_string(), info.to_string()),
        Error::ContentError(info) => ("ContentError".to_string(), info.to_string()),
        Error::EmptyContent(info) => ("EmptyContent".to_string(), info.to_string()),
        Error::AccessDenied(info) => ("AccessDenied".to_string(), info.to_string()),
        Error::EntryNotFound(info) => ("EntryNotFound".to_string(), info.to_string()),
        Error::EntryExists(info) => ("EntryExists".to_string(), info.to_string()),
        Error::InvalidInput(info) => ("InvalidInput".to_string(), info.to_string()),
        Error::InvalidAmount(info) => ("InvalidAmount".to_string(), info.to_string()),
        Error::InvalidXorUrl(info) => ("InvalidXorUrl".to_string(), info.to_string()),
        Error::InvalidMediaType(info) => ("InvalidMediaType".to_string(), info.to_string()),
        Error::NotEnoughBalance(info) => ("NotEnoughBalance".to_string(), info.to_string()),
        Error::FilesSystemError(info) => ("FilesSystemError".to_string(), info.to_string()),
        Error::Unexpected(info) => ("Unexpected".to_string(), info.to_string()),
        Error::Unknown(info) => ("Unknown".to_string(), info.to_string()),
        Error::StringError(info) => ("StringError".to_string(), info.to_string()),
    };
    format!("[Error] {} - {}", error_type, error_msg)
}

#[allow(missing_docs)]
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

    // Balance Errors
    pub const ERR_INVALID_AMOUNT_ERROR: i32 = -300;
    pub const ERR_NOT_ENOUGH_BALANCE_ERROR: i32 = -301;
    pub const ERR_INVALID_XOR_URL_ERROR: i32 = -400;

    // Misc Errors
    pub const ERR_UNEXPECTED_ERROR: i32 = -500;
    pub const ERR_UNKNOWN_ERROR: i32 = -501;
    pub const ERR_STRING_ERROR: i32 = -502;
}

impl ErrorCode for Error {
    fn error_code(&self) -> i32 {
        match *self {
            Error::AuthError(ref _error) => { ERR_AUTH_ERROR },
            Error::ConnectionError(ref _error) => { ERR_CONNECTION_ERROR },
            Error::NetDataError(ref _error) => { ERR_NET_DATA_ERROR },
            Error::ContentNotFound(ref _error) => { ERR_CONTENT_NOT_FOUND_ERROR },
            Error::VersionNotFound(ref _error) => { ERR_VERSION_NOT_FOUND_ERROR },
            Error::ContentError(ref _error) => { ERR_CONTENT_ERROR },
            Error::EmptyContent(ref _error) => { ERR_EMPTY_CONTENT_ERROR },
            Error::AccessDenied(ref _error) => { ERR_ACCESS_DENIED_ERROR },
            Error::EntryNotFound(ref _error) => { ERR_ENTRY_NOT_FOUND_ERROR },
            Error::EntryExists(ref _error) => { ERR_ENTRY_EXISTS_ERROR },
            Error::InvalidInput(ref _error) => { ERR_INVALID_INPUT_ERROR },
            Error::InvalidAmount(ref _error) => { ERR_INVALID_AMOUNT_ERROR },
            Error::InvalidXorUrl(ref _error) => { ERR_INVALID_XOR_URL_ERROR },
            Error::NotEnoughBalance(ref _error) => { ERR_NOT_ENOUGH_BALANCE_ERROR },
            Error::FilesSystemError(ref _error) => { ERR_FILE_SYSTEM_ERROR },
            Error::Unexpected(ref _error) => { ERR_UNEXPECTED_ERROR },
            Error::Unknown(ref _error) => { ERR_UNKNOWN_ERROR },
            Error::StringError(ref _error) => { ERR_STRING_ERROR },
        }
    }
}

impl From<StringError> for Error {
    fn from(_error: StringError) -> Self {
        Error::StringError("string conversion error".to_string())
    }
}

impl<'a> From<&'a str> for Error {
    fn from(s: &'a str) -> Self {
        Error::Unexpected(s.to_string())
    }
}

impl From<String> for Error {
    fn from(error: String) -> Self {
        Error::Unexpected(error)
    }
}

impl From<NulError> for Error {
    fn from(_error: NulError) -> Self {
        Error::Unexpected("Null error".to_string())
    }
}
