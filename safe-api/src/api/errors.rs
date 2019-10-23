// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, PartialEq)]
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
    StringError(String),
}

impl From<Error> for String {
    fn from(error: Error) -> String {
        error.into()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (error_type, error_msg) = match self {
            Error::AuthError(info) => ("AuthError", info),
            Error::ConnectionError(info) => ("ConnectionError", info),
            Error::NetDataError(info) => ("NetDataError", info),
            Error::ContentNotFound(info) => ("ContentNotFound", info),
            Error::VersionNotFound(info) => ("VersionNotFound", info),
            Error::ContentError(info) => ("ContentError", info),
            Error::EmptyContent(info) => ("EmptyContent", info),
            Error::AccessDenied(info) => ("AccessDenied", info),
            Error::EntryNotFound(info) => ("EntryNotFound", info),
            Error::EntryExists(info) => ("EntryExists", info),
            Error::InvalidInput(info) => ("InvalidInput", info),
            Error::InvalidAmount(info) => ("InvalidAmount", info),
            Error::InvalidXorUrl(info) => ("InvalidXorUrl", info),
            Error::InvalidMediaType(info) => ("InvalidMediaType", info),
            Error::NotEnoughBalance(info) => ("NotEnoughBalance", info),
            Error::FilesSystemError(info) => ("FilesSystemError", info),
            Error::Unexpected(info) => ("Unexpected", info),
            Error::Unknown(info) => ("Unknown", info),
            Error::StringError(info) => ("StringError", info),
        };
        let description = format!("[Error] {} - {}", error_type, error_msg);

        write!(f, "{}", description)
    }
}
