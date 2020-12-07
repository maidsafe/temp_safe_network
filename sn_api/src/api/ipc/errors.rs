// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use serde::{Deserialize, Serialize};
//use threshold_crypto::error::FromBytesError;

/// Ipc error.
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub enum IpcError {
    /// Authentication denied.
    AuthDenied,
    /// Invalid IPC message.
    InvalidMsg,
    /// Generic encoding / decoding failure.
    EncodeDecodeError,
    /// App is already authorised.
    AlreadyAuthorised,
    /// App is not registered.
    UnknownApp,
    /// Unexpected error.
    Unexpected(String),
}
