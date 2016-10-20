// Copyright 2015 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement, version 1.0.  This, along with the
// Licenses can be found in the root directory of this project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.
//
// Please review the Licences for the specific language governing permissions and limitations
// relating to use of the SAFE Network Software.

//! Errors thrown by the FFI operations

use core::errors::CoreError;
use dns::errors::{DNS_ERROR_START_RANGE, DnsError};
use maidsafe_utilities::serialisation::SerialisationError;
use nfs::errors::NfsError;
use std::ffi::NulError;
use std::fmt;

/// Intended for converting Launcher Errors into numeric codes for propagating some error
/// information across FFI boundaries and specially to C.
pub const FFI_ERROR_START_RANGE: i32 = DNS_ERROR_START_RANGE - 500;

/// Launcher Errors
pub enum FfiError {
    /// Error from safe_core. Boxed to hold a pointer instead of value so that this enum variant is
    /// not insanely bigger than others.
    CoreError(Box<CoreError>),
    /// Errors from safe_nfs
    NfsError(Box<NfsError>),
    /// Errors from safe_nfs
    DnsError(Box<DnsError>),
    /// Unable to find/traverse directory or file path
    PathNotFound,
    /// Supplied path was invalid
    InvalidPath,
    /// Permission denied - e.g. permission to access SAFEDrive etc.
    PermissionDenied,
    /// Unable to Read from or Write to a Local Config file.
    LocalConfigAccessFailed(String),
    /// Unexpected - Probably a Logic error
    Unexpected(String),
    /// Could not serialise or deserialise data
    UnsuccessfulEncodeDecode(SerialisationError),
    /// Could not convert String to nul-terminated string because it contains
    /// internal nuls.
    NulError(NulError),
    /// Invalid StructuredData handle
    InvalidStructDataHandle,
    /// Invalid DataIdentifier handle
    InvalidDataIdHandle,
    /// Invalid Pub/PrivAppendableData handle
    InvalidAppendableDataHandle,
    /// Invalid Self Encryptor handle
    InvalidSelfEncryptorHandle,
    /// Invalid CipherOpt handle
    InvalidCipherOptHandle,
    /// Invalid encrypt (box_) key handle
    InvalidEncryptKeyHandle,
    /// Invalid sign key handle
    InvalidSignKeyHandle,
    /// The requested operation is forbidded for the given app.
    OperationForbiddenForApp,
    /// Invalid type tag for StructuredData
    InvalidStructuredDataTypeTag,
    /// Invalid version number requested for a versioned StructuredData
    InvalidVersionNumber,
    /// Invalid offsets (from-position and lenght combination) provided for reading form Self
    /// Encryptor. Would have probably caused an overflow.
    InvalidSelfEncryptorReadOffsets,
    /// Invalid indexing
    InvalidIndex,
    /// Unsupported Operation (e.g. mixing Pub/PrivAppendableData operations
    UnsupportedOperation,
    /// Input/output Error
    IoError(::std::io::Error),
}

impl From<SerialisationError> for FfiError {
    fn from(error: SerialisationError) -> FfiError {
        FfiError::UnsuccessfulEncodeDecode(error)
    }
}
impl<'a> From<&'a str> for FfiError {
    fn from(error: &'a str) -> FfiError {
        FfiError::Unexpected(error.to_string())
    }
}

impl From<::std::io::Error> for FfiError {
    fn from(error: ::std::io::Error) -> FfiError {
        FfiError::IoError(error)
    }
}

impl From<CoreError> for FfiError {
    fn from(error: CoreError) -> FfiError {
        match error {
            CoreError::InvalidStructuredDataTypeTag => FfiError::InvalidStructuredDataTypeTag,
            _ => FfiError::CoreError(Box::new(error)),
        }
    }
}

impl From<NfsError> for FfiError {
    fn from(error: NfsError) -> FfiError {
        FfiError::NfsError(Box::new(error))
    }
}

impl From<DnsError> for FfiError {
    fn from(error: DnsError) -> FfiError {
        FfiError::DnsError(Box::new(error))
    }
}

impl From<NulError> for FfiError {
    fn from(error: NulError) -> Self {
        FfiError::NulError(error)
    }
}

impl Into<i32> for FfiError {
    fn into(self) -> i32 {
        match self {
            FfiError::CoreError(error) => (*error).into(),
            FfiError::NfsError(error) => (*error).into(),
            FfiError::DnsError(error) => (*error).into(),
            FfiError::PathNotFound => FFI_ERROR_START_RANGE - 1,
            FfiError::InvalidPath => FFI_ERROR_START_RANGE - 2,
            FfiError::PermissionDenied => FFI_ERROR_START_RANGE - 3,
            FfiError::LocalConfigAccessFailed(_) => FFI_ERROR_START_RANGE - 8,
            FfiError::Unexpected(_) => FFI_ERROR_START_RANGE - 9,
            FfiError::UnsuccessfulEncodeDecode(_) => FFI_ERROR_START_RANGE - 10,
            FfiError::NulError(_) => FFI_ERROR_START_RANGE - 11,
            FfiError::InvalidStructDataHandle => FFI_ERROR_START_RANGE - 12,
            FfiError::InvalidDataIdHandle => FFI_ERROR_START_RANGE - 13,
            FfiError::InvalidAppendableDataHandle => FFI_ERROR_START_RANGE - 14,
            FfiError::InvalidSelfEncryptorHandle => FFI_ERROR_START_RANGE - 15,
            FfiError::InvalidCipherOptHandle => FFI_ERROR_START_RANGE - 16,
            FfiError::InvalidEncryptKeyHandle => FFI_ERROR_START_RANGE - 17,
            FfiError::InvalidSignKeyHandle => FFI_ERROR_START_RANGE - 18,
            FfiError::OperationForbiddenForApp => FFI_ERROR_START_RANGE - 19,
            FfiError::InvalidStructuredDataTypeTag => FFI_ERROR_START_RANGE - 20,
            FfiError::InvalidVersionNumber => FFI_ERROR_START_RANGE - 21,
            FfiError::InvalidSelfEncryptorReadOffsets => FFI_ERROR_START_RANGE - 22,
            FfiError::InvalidIndex => FFI_ERROR_START_RANGE - 23,
            FfiError::UnsupportedOperation => FFI_ERROR_START_RANGE - 24,
            FfiError::IoError(_) => FFI_ERROR_START_RANGE - 25,
        }
    }
}

impl fmt::Debug for FfiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FfiError::CoreError(ref error) => write!(f, "FfiError::CoreError -> {:?}", error),
            FfiError::NfsError(ref error) => write!(f, "FfiError::NfsError -> {:?}", error),
            FfiError::DnsError(ref error) => write!(f, "FfiError::DnsError -> {:?}", error),
            FfiError::PathNotFound => write!(f, "FfiError::PathNotFound"),
            FfiError::InvalidPath => write!(f, "FfiError::InvalidPath"),
            FfiError::PermissionDenied => write!(f, "FfiError::PermissionDenied"),
            FfiError::LocalConfigAccessFailed(ref error) => {
                write!(f, "FfiError::LocalConfigAccessFailed -> {:?}", error)
            }
            FfiError::Unexpected(ref error) => write!(f, "FfiError::Unexpected{{{:?}}}", error),
            FfiError::UnsuccessfulEncodeDecode(ref error) => {
                write!(f, "FfiError::UnsuccessfulEncodeDecode -> {:?}", error)
            }
            FfiError::NulError(ref error) => write!(f, "FfiError::NulError -> {:?}", error),
            FfiError::InvalidStructDataHandle => write!(f, "FfiError::InvalidStructDataHandle"),
            FfiError::InvalidDataIdHandle => write!(f, "FfiError::InvalidDataIdHandle"),
            FfiError::InvalidAppendableDataHandle => {
                write!(f, "FfiError::InvalidAppendableDataHandle")
            }
            FfiError::InvalidSelfEncryptorHandle => {
                write!(f, "FfiError::InvalidSelfEncryptorHandle")
            }
            FfiError::InvalidCipherOptHandle => write!(f, "FfiError::InvalidCipherOptHandle"),
            FfiError::InvalidEncryptKeyHandle => write!(f, "FfiError::InvalidEncryptKeyHandle"),
            FfiError::InvalidSignKeyHandle => write!(f, "FfiError::InvalidSignKeyHandle"),
            FfiError::OperationForbiddenForApp => write!(f, "FfiError::OperationForbiddenForApp"),
            FfiError::InvalidStructuredDataTypeTag => {
                write!(f, "FfiError::InvalidStructuredDataTypeTag")
            }
            FfiError::InvalidVersionNumber => write!(f, "FfiError::InvalidVersionNumber"),
            FfiError::InvalidSelfEncryptorReadOffsets => {
                write!(f, "FfiError::InvalidSelfEncryptorReadOffsets")
            }
            FfiError::InvalidIndex => write!(f, "FfiError::InvalidIndex"),
            FfiError::UnsupportedOperation => write!(f, "FfiError::UnsupportedOperation"),
            FfiError::IoError(ref error) => write!(f, "FfiError::IoError -> {:?}", error),
        }
    }
}
