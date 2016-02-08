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

use std::sync::mpsc;
use maidsafe_utilities::serialisation::SerialisationError;
use mpid_messaging;

/// Intended for converting Client Errors into numeric codes for propagating some error information
/// across FFI boundaries and specially to C.
pub const CLIENT_ERROR_START_RANGE: i32 = -1;

/// Client Errors
pub enum CoreError {
    /// StructuredData has no space available to fit in any user data inside it.
    StructuredDataHeaderSizeProhibitive,
    /// Could not Serialise or Deserialise
    UnsuccessfulEncodeDecode(SerialisationError),
    /// Asymmetric Key Decryption Failed
    AsymmetricDecipherFailure,
    /// Symmetric Key Decryption Failed
    SymmetricDecipherFailure,
    /// ReceivedUnexpectedData
    ReceivedUnexpectedData,
    /// No such data found in local version cache
    VersionCacheMiss,
    /// No such data found in routing-filled cache
    RoutingMessageCacheMiss,
    /// Cannot overwrite a root directory if it already exists
    RootDirectoryAlreadyExists,
    /// Unable to obtain generator for random data
    RandomDataGenerationFailure,
    /// Forbidden operation requested for this Client
    OperationForbiddenForClient,
    /// Unexpected - Probably a Logic error
    Unexpected(String),
    /// Routing Error
    RoutingError(::routing::RoutingError),
    /// Interface Error
    RoutingInterfaceError(::routing::InterfaceError),
    /// Unable to pack into or operate with size of Salt
    UnsupportedSaltSizeForPwHash,
    /// Unable to complete computation for password hashing - usually because OS refused to
    /// allocate amount of requested memory
    UnsuccessfulPwHash,
    /// Blocking operation was cancelled
    OperationAborted,
    /// MpidMessaging Error
    MpidMessagingError(mpid_messaging::Error),
}

impl<'a> From<&'a str> for CoreError {
    fn from(error: &'a str) -> CoreError {
        CoreError::Unexpected(error.to_string())
    }
}

impl From<SerialisationError> for CoreError {
    fn from(error: SerialisationError) -> CoreError {
        CoreError::UnsuccessfulEncodeDecode(error)
    }
}

impl From<::routing::RoutingError> for CoreError {
    fn from(error: ::routing::RoutingError) -> CoreError {
        CoreError::RoutingError(error)
    }
}

impl From<::routing::InterfaceError> for CoreError {
    fn from(error: ::routing::InterfaceError) -> CoreError {
        CoreError::RoutingInterfaceError(error)
    }
}

impl From<mpsc::RecvError> for CoreError {
    fn from(_: mpsc::RecvError) -> CoreError {
        CoreError::OperationAborted
    }
}

impl From<mpid_messaging::Error> for CoreError {
    fn from(error: mpid_messaging::Error) -> CoreError {
        CoreError::MpidMessagingError(error)
    }
}

impl Into<i32> for CoreError {
    fn into(self) -> i32 {
        match self {
            CoreError::StructuredDataHeaderSizeProhibitive => CLIENT_ERROR_START_RANGE,
            CoreError::UnsuccessfulEncodeDecode(_) => CLIENT_ERROR_START_RANGE - 1,
            CoreError::AsymmetricDecipherFailure => CLIENT_ERROR_START_RANGE - 2,
            CoreError::SymmetricDecipherFailure => CLIENT_ERROR_START_RANGE - 3,
            CoreError::ReceivedUnexpectedData => CLIENT_ERROR_START_RANGE - 4,
            CoreError::VersionCacheMiss => CLIENT_ERROR_START_RANGE - 5,
            CoreError::RoutingMessageCacheMiss => CLIENT_ERROR_START_RANGE - 6,
            CoreError::RootDirectoryAlreadyExists => CLIENT_ERROR_START_RANGE - 8,
            CoreError::RandomDataGenerationFailure => CLIENT_ERROR_START_RANGE - 9,
            CoreError::OperationForbiddenForClient => CLIENT_ERROR_START_RANGE - 10,
            CoreError::Unexpected(_) => CLIENT_ERROR_START_RANGE - 11,
            CoreError::RoutingError(_) => CLIENT_ERROR_START_RANGE - 12,
            CoreError::RoutingInterfaceError(_) => CLIENT_ERROR_START_RANGE - 13,
            CoreError::UnsupportedSaltSizeForPwHash => CLIENT_ERROR_START_RANGE - 14,
            CoreError::UnsuccessfulPwHash => CLIENT_ERROR_START_RANGE - 15,
            CoreError::OperationAborted => CLIENT_ERROR_START_RANGE - 16,
            CoreError::MpidMessagingError(_) => CLIENT_ERROR_START_RANGE - 17,
        }
    }
}

impl ::std::fmt::Debug for CoreError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            CoreError::StructuredDataHeaderSizeProhibitive => {
                write!(f, "CoreError::StructuredDataHeaderSizeProhibitive")
            }
            CoreError::UnsuccessfulEncodeDecode(ref err) => {
                write!(f, "CoreError::UnsuccessfulEncodeDecode -> {:?}", err)
            }
            CoreError::AsymmetricDecipherFailure => {
                write!(f, "CoreError::AsymmetricDecipherFailure")
            }
            CoreError::SymmetricDecipherFailure => write!(f, "CoreError::SymmetricDecipherFailure"),
            CoreError::ReceivedUnexpectedData => write!(f, "CoreError::ReceivedUnexpectedData"),
            CoreError::VersionCacheMiss => write!(f, "CoreError::VersionCacheMiss"),
            CoreError::RoutingMessageCacheMiss => write!(f, "CoreError::RoutingMessageCacheMiss"),
            CoreError::RootDirectoryAlreadyExists => {
                write!(f, "CoreError::RootDirectoryAlreadyExists")
            }
            CoreError::RandomDataGenerationFailure => {
                write!(f, "CoreError::RandomDataGenerationFailure")
            }
            CoreError::OperationForbiddenForClient => {
                write!(f, "CoreError::OperationForbiddenForClient")
            }
            CoreError::Unexpected(ref err) => write!(f, "CoreError::Unexpected::{{{:?}}}", err),
            CoreError::RoutingError(ref err) => write!(f, "CoreError::RoutingError -> {:?}", err),
            CoreError::RoutingInterfaceError(ref err) => {
                write!(f, "CoreError::RoutingInterfaceError -> {:?}", err)
            }
            CoreError::UnsupportedSaltSizeForPwHash => {
                write!(f, "CoreError::UnsupportedSaltSizeForPwHash")
            }
            CoreError::UnsuccessfulPwHash => write!(f, "CoreError::UnsuccessfulPwHash"),
            CoreError::OperationAborted => write!(f, "CoreError::OperationAborted"),
            CoreError::MpidMessagingError(ref err) => write!(f, "CoreError::MpidMessagingError -> {:?}", err),
        }
    }
}
