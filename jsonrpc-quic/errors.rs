// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use std::fmt;

#[derive(Debug)]
pub enum Error {
    ClientError(String),
    RemoteEndpointError(String),
    GeneralError(String),
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::GeneralError(error.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<Error> for String {
    fn from(error: Error) -> String {
        error.to_string()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;

        let (error_type, error_msg) = match self {
            ClientError(info) => ("ClientError", info),
            RemoteEndpointError(info) => ("RemoteEndpointError", info),
            GeneralError(info) => ("GeneralError", info),
        };
        let description = format!("[Error] {} - {}", error_type, error_msg);

        write!(f, "{}", description)
    }
}
