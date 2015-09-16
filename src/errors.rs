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

/// Intended for converting Client Errors into numeric codes for propagating some error information
/// across FFI boundaries and specially to C.
pub const CLIENT_ERROR_START_RANGE: i32 = -1;

/// Client Errors
#[allow(variant_size_differences)] // TODO
pub enum ClientError {
    /// StructuredData has no space available to fit in any user data inside it.
    StructuredDataHeaderSizeProhibitive,
    /// Could not Serialise or Deserialise
    UnsuccessfulEncodeDecode,
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
    /// Network operation failed
    ResponseError(::routing::error::ResponseError),
    /// Cannot overwrite a root directory if it already exists
    RootDirectoryAlreadyExists,
    /// Unable to obtain generator for random data
    RandomDataGenerationFailure,
    /// Forbidden operation requested for this Client
    OperationForbiddenForClient,
    /// Unexpected - Probably a Logic error
    Unexpected(String),
    /// Routing Error
    RoutingError(::routing::error::RoutingError),
    /// Unable to pack into or operate with size of Salt
    UnsupportedSaltSizeForPwHash,
    /// Unable to complete computation for password hashing - usually because OS refused to
    /// allocate amount of requested memory
    UnsuccessfulPwHash,
    /// Blocking operation was cancelled
    OperationAborted,
}

impl<'a> From<&'a str> for ClientError {
    fn from(error: &'a str) -> ClientError {
        ClientError::Unexpected(error.to_string())
    }
}

impl From<::cbor::CborError> for ClientError {
    fn from(error: ::cbor::CborError) -> ClientError {
        debug!("Error: {:?}", error);
        ClientError::UnsuccessfulEncodeDecode
    }
}

impl From<::routing::error::ResponseError> for ClientError {
    fn from(error: ::routing::error::ResponseError) -> ClientError {
        ClientError::ResponseError(error)
    }
}

impl From<::routing::error::RoutingError> for ClientError {
    fn from(error: ::routing::error::RoutingError) -> ClientError {
        ClientError::RoutingError(error)
    }
}

impl From<::std::sync::mpsc::RecvError> for ClientError {
    fn from(_: ::std::sync::mpsc::RecvError) -> ClientError {
        ClientError::OperationAborted
    }
}

impl Into<i32> for ClientError {
    fn into(self) -> i32 {
        match self {
            ClientError::StructuredDataHeaderSizeProhibitive => CLIENT_ERROR_START_RANGE,
            ClientError::UnsuccessfulEncodeDecode            => CLIENT_ERROR_START_RANGE - 1,
            ClientError::AsymmetricDecipherFailure           => CLIENT_ERROR_START_RANGE - 2,
            ClientError::SymmetricDecipherFailure            => CLIENT_ERROR_START_RANGE - 3,
            ClientError::ReceivedUnexpectedData              => CLIENT_ERROR_START_RANGE - 4,
            ClientError::VersionCacheMiss                    => CLIENT_ERROR_START_RANGE - 5,
            ClientError::RoutingMessageCacheMiss             => CLIENT_ERROR_START_RANGE - 6,
            ClientError::ResponseError(_)                    => CLIENT_ERROR_START_RANGE - 7,
            ClientError::RootDirectoryAlreadyExists          => CLIENT_ERROR_START_RANGE - 8,
            ClientError::RandomDataGenerationFailure         => CLIENT_ERROR_START_RANGE - 9,
            ClientError::OperationForbiddenForClient         => CLIENT_ERROR_START_RANGE - 10,
            ClientError::Unexpected(_)                       => CLIENT_ERROR_START_RANGE - 11,
            ClientError::RoutingError(_)                     => CLIENT_ERROR_START_RANGE - 12,
            ClientError::UnsupportedSaltSizeForPwHash        => CLIENT_ERROR_START_RANGE - 13,
            ClientError::UnsuccessfulPwHash                  => CLIENT_ERROR_START_RANGE - 14,
            ClientError::OperationAborted                    => CLIENT_ERROR_START_RANGE - 15,
        }
    }
}

impl ::std::fmt::Debug for ClientError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            ClientError::StructuredDataHeaderSizeProhibitive => write!(f, "ClientError::StructuredDataHeaderSizeProhibitive"),
            ClientError::UnsuccessfulEncodeDecode            => write!(f, "ClientError::UnsuccessfulEncodeDecode"),
            ClientError::AsymmetricDecipherFailure           => write!(f, "ClientError::AsymmetricDecipherFailure"),
            ClientError::SymmetricDecipherFailure            => write!(f, "ClientError::SymmetricDecipherFailure"),
            ClientError::ReceivedUnexpectedData              => write!(f, "ClientError::ReceivedUnexpectedData"),
            ClientError::VersionCacheMiss                    => write!(f, "ClientError::VersionCacheMiss"),
            ClientError::RoutingMessageCacheMiss             => write!(f, "ClientError::RoutingMessageCacheMiss"),
            ClientError::ResponseError(ref error)            => write!(f, "ClientError::ResponseError -> {:?}", error),
            ClientError::RootDirectoryAlreadyExists          => write!(f, "ClientError::RootDirectoryAlreadyExists"),
            ClientError::RandomDataGenerationFailure         => write!(f, "ClientError::RandomDataGenerationFailure"),
            ClientError::OperationForbiddenForClient         => write!(f, "ClientError::OperationForbiddenForClient"),
            ClientError::Unexpected(ref error)               => write!(f, "ClientError::Unexpected::{{{:?}}}", error),
            ClientError::RoutingError(ref error)             => write!(f, "ClientError::RoutingError -> {:?}", error),
            ClientError::UnsupportedSaltSizeForPwHash        => write!(f, "ClientError::UnsupportedSaltSizeForPwHash"),
            ClientError::UnsuccessfulPwHash                  => write!(f, "ClientError::UnsuccessfulPwHash"),
            ClientError::OperationAborted                    => write!(f, "ClientError::OperationAborted"),
        }
    }
}
