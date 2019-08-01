// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use std::fmt;

pub type ResultReturn<T> = Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    AuthError(String),
    ConnectionError(String),
    NetDataError(String),
    ContentNotFound(String),
    ContentError(String),
    EmptyContent(String),
    VersionNotFound(String),
    EntryNotFound(String),
    InvalidInput(String),
    InvalidAmount(String),
    InvalidXorUrl(String),
    NotEnoughBalance(String),
    FilesSystemError(String),
    Unexpected(String),
    Unknown(String),
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
        Error::EntryNotFound(info) => ("EntryNotFound".to_string(), info.to_string()),
        Error::InvalidInput(info) => ("InvalidInput".to_string(), info.to_string()),
        Error::InvalidAmount(info) => ("InvalidAmount".to_string(), info.to_string()),
        Error::InvalidXorUrl(info) => ("InvalidXorUrl".to_string(), info.to_string()),
        Error::NotEnoughBalance(info) => ("NotEnoughBalance".to_string(), info.to_string()),
        Error::FilesSystemError(info) => ("FilesSystemError".to_string(), info.to_string()),
        Error::Unexpected(info) => ("Unexpected".to_string(), info.to_string()),
        Error::Unknown(info) => ("Unknown".to_string(), info.to_string()),
    };
    format!("[Error] {} - {}", error_type, error_msg)
}
