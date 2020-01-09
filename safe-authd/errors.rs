// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    GeneralError(String),
    AuthdAlreadyStarted(String),
    Unexpected(String),
}

impl From<Error> for String {
    fn from(error: Error) -> String {
        error.to_string()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::GeneralError(error.to_string())
    }
}

impl Error {
    pub fn error_code(&self) -> i32 {
        use Error::*;
        // Don't use any of the reserved exit codes:
        // http://tldp.org/LDP/abs/html/exitcodes.html#AEN23549
        match self {
            GeneralError(ref _error) => 1,
            AuthdAlreadyStarted(ref _error) => 10,
            Unexpected(ref _error) => 20,
        }
    }

    pub fn description(&self) -> String {
        use Error::*;
        let (error_type, error_msg) = match self {
            GeneralError(info) => ("GeneralError", info),
            AuthdAlreadyStarted(info) => ("AuthdAlreadyStarted", info),
            Unexpected(info) => ("Unexpected", info),
        };

        format!("[Error] {} - {}", error_type, error_msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display() {
        let err = Error::GeneralError("test error".to_string());
        let s: String = err.into();
        assert_eq!(s, "[Error] GeneralError - test error");
    }
}
