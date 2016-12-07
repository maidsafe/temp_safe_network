// Copyright 2016 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net
// Commercial License, version 1.0 or later, or (2) The General Public License
// (GPL), version 3, depending on which licence you accepted on initial access
// to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project
// generally, you agree to be bound by the terms of the MaidSafe Contributor
// Agreement, version 1.0.
// This, along with the Licenses can be found in the root directory of this
// project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network
// Software distributed under the GPL Licence is distributed on an "AS IS"
// BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied.
//
// Please review the Licences for the specific language governing permissions
// and limitations relating to use of the SAFE Network Software.

use core::CORE_ERROR_START_RANGE;
use maidsafe_utilities::serialisation::SerialisationError;
use rustc_serialize::base64::FromBase64Error;
use std::str::Utf8Error;

const IPC_ERROR_START_RANGE: i32 = CORE_ERROR_START_RANGE - 1000;

/// Ipc error
#[derive(RustcEncodable, RustcDecodable, Debug, Eq, PartialEq)]
pub enum IpcError {
    /// Authentication denied
    AuthDenied,
    /// Containers denied
    ContainersDenied,
    /// Invalid IPC message
    InvalidMsg,
    /// Generic encoding / decoding failure.
    EncodeDecodeError,

    /// Unexpected error
    Unexpected(String),
}

impl From<Utf8Error> for IpcError {
    fn from(_err: Utf8Error) -> Self {
        IpcError::EncodeDecodeError
    }
}

impl From<FromBase64Error> for IpcError {
    fn from(_err: FromBase64Error) -> Self {
        IpcError::EncodeDecodeError
    }
}

impl From<SerialisationError> for IpcError {
    fn from(_err: SerialisationError) -> Self {
        IpcError::EncodeDecodeError
    }
}

impl<'a> From<&'a str> for IpcError {
    fn from(s: &'a str) -> Self {
        IpcError::Unexpected(s.to_string())
    }
}

impl From<String> for IpcError {
    fn from(s: String) -> Self {
        IpcError::Unexpected(s)
    }
}

impl Into<i32> for IpcError {
    fn into(self) -> i32 {
        match self {
            IpcError::AuthDenied => IPC_ERROR_START_RANGE - 1,
            IpcError::ContainersDenied => IPC_ERROR_START_RANGE - 2,
            IpcError::InvalidMsg => IPC_ERROR_START_RANGE - 3,
            IpcError::EncodeDecodeError => IPC_ERROR_START_RANGE - 4,
            IpcError::Unexpected(_) => IPC_ERROR_START_RANGE - 100,
        }
    }
}
