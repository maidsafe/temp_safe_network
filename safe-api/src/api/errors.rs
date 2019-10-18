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
    AuthdClientError(String),
    AuthenticatorError(String),
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
}

impl From<Error> for String {
    fn from(error: Error) -> String {
        error.to_string()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;

        let (error_type, error_msg) = match self {
            AuthError(info) => ("AuthError", info),
            AuthdClientError(info) => ("AuthdClientError", info),
            AuthenticatorError(info) => ("AuthenticatorError", info),
            ConnectionError(info) => ("ConnectionError", info),
            NetDataError(info) => ("NetDataError", info),
            ContentNotFound(info) => ("ContentNotFound", info),
            VersionNotFound(info) => ("VersionNotFound", info),
            ContentError(info) => ("ContentError", info),
            EmptyContent(info) => ("EmptyContent", info),
            AccessDenied(info) => ("AccessDenied", info),
            EntryNotFound(info) => ("EntryNotFound", info),
            EntryExists(info) => ("EntryExists", info),
            InvalidInput(info) => ("InvalidInput", info),
            InvalidAmount(info) => ("InvalidAmount", info),
            InvalidXorUrl(info) => ("InvalidXorUrl", info),
            InvalidMediaType(info) => ("InvalidMediaType", info),
            NotEnoughBalance(info) => ("NotEnoughBalance", info),
            FilesSystemError(info) => ("FilesSystemError", info),
            Unexpected(info) => ("Unexpected", info),
            Unknown(info) => ("Unknown", info),
        };
        let description = format!("[Error] {} - {}", error_type, error_msg);

        write!(f, "{}", description)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display() {
        let err = Error::Unknown("test error".to_string());
        let s: String = err.into();
        assert_eq!(s, "[Error] Unknown - test error");
    }
}
