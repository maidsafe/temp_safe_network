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
