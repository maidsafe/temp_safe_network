// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::ipc::IpcError;
use safe_network::client::Error as ClientError;
use safe_network::url::Error as NativeUrlError;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

/// Error type returned by the API
#[derive(Debug, Error)]
#[allow(clippy::large_enum_variant)]
#[non_exhaustive]
pub enum Error {
    /// AuthError
    #[error("AuthError: {0}")]
    AuthError(String),
    /// AuthIpcError
    #[error("AuthIpcError: {0}")]
    AuthIpcError(#[from] IpcError),
    /// AuthdClientError
    #[error("AuthdClientError: {0}")]
    AuthdClientError(String),
    /// AuthdError
    #[error("AuthdError: {0}")]
    AuthdError(String),
    /// AuthdAlreadyStarted
    #[error("AuthdAlreadyStarted: {0}")]
    AuthdAlreadyStarted(String),
    /// AuthenticatorError
    #[error("AuthenticatorError: {0}")]
    AuthenticatorError(String),
    /// ConnectionError
    #[error("ConnectionError: {0}")]
    ConnectionError(String),
    /// NetDataError
    #[error("NetDataError: {0}")]
    NetDataError(String),
    /// ContentNotFound
    #[error("ContentNotFound: {0}")]
    ContentNotFound(String),
    /// ContentError
    #[error("ContentError: {0}")]
    ContentError(String),
    /// ClientError
    #[error("ClientError: {0}")]
    ClientError(#[from] ClientError),
    /// EmptyContent
    #[error("EmptyContent: {0}")]
    EmptyContent(String),
    /// AccessDenied
    #[error("AccessDenied: {0}")]
    AccessDenied(String),
    /// VersionNotFound
    #[error("VersionNotFound: {0}")]
    VersionNotFound(String),
    #[cfg(feature = "app")]
    /// HashNotFound
    #[error("No entry with hash {0:?}")]
    HashNotFound(crate::register::EntryHash),
    /// EntryNotFound
    #[error("EntryNotFound: {0}")]
    EntryNotFound(String),
    /// EntryExists
    #[error("EntryExists: {0}")]
    EntryExists(String),
    /// InvalidAmount
    #[error("InvalidAmount: {0}")]
    InvalidAmount(String),
    /// InvalidXorUrl
    #[error("InvalidXorUrl: {0}")]
    InvalidXorUrl(String),
    /// InvalidInput
    #[error("InvalidInput: {0}")]
    InvalidInput(String),
    /// InvalidMediaType
    #[error("InvalidMediaType: {0}")]
    InvalidMediaType(String),
    /// NotEnoughBalance
    #[error("NotEnoughBalance: {0}")]
    NotEnoughBalance(String),
    /// Serialisation
    #[error("Serialisation: {0}")]
    Serialisation(String),
    /// FileSystemError
    #[error("FileSystemError: {0}")]
    FileSystemError(String),
    /// NativeUrlError
    #[error("NativeUrlError: {0}")]
    NativeUrlError(#[from] NativeUrlError),
    /// NotImplementedError
    #[error("NotImplementedError: {0}")]
    NotImplementedError(String),
    /// MultimapFork
    #[error("MultimapFork: {0}")]
    MultimapFork(String),
}
