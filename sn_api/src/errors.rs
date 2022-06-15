// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::ipc::IpcError;
use super::nrs::NrsMap;

use sn_client::Error as ClientError;
use sn_dbc::Error as DbcError;
use sn_interface::types::Error as InterfaceError;
use sn_url::{Error as UrlError, SafeUrl, XorUrl};
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
    /// ConflictingNrsEntries
    #[error("ConflictingNrsEntries: {0}")]
    ConflictingNrsEntries(String, Vec<(String, SafeUrl)>, NrsMap),
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
    /// A file with same name already exists on target FilesContainer with same link"
    #[error("File named \"{0}\" already exists on target with same link.")]
    FileAlreadyExists(String),
    /// A file with same name already exists on target FilesContainer with same link"
    #[error("File named \"{0}\" already exists on target. Use the 'force' flag to replace it.")]
    FileNameConflict(String),
    /// InvalidAmount
    #[error("InvalidAmount: {0}")]
    InvalidAmount(String),
    /// InvalidUtf8Sequence
    #[error("InvalidUtf8Sequence: {0}")]
    InvalidUtf8Sequence(#[from] std::str::Utf8Error),
    /// InvalidXorUrl
    #[error("InvalidXorUrl: {0}")]
    InvalidXorUrl(String),
    /// InvalidInput
    #[error("InvalidInput: {0}")]
    InvalidInput(String),
    /// InvalidMediaType
    #[error("InvalidMediaType: {0}")]
    InvalidMediaType(String),
    /// Not enough balance to perform a transaction
    #[error("Not enough balance: {0}")]
    NotEnoughBalance(String),
    /// NrsNameAlreadyExists
    #[error("NrsNameAlreadyExists: {0}")]
    NrsNameAlreadyExists(String),
    /// Serialisation
    #[error("Serialisation: {0}")]
    Serialisation(String),
    /// FileSystemError
    #[error("FileSystemError: {0}")]
    FileSystemError(String),
    /// UrlError
    #[error("UrlError: {0}")]
    UrlError(#[from] UrlError),
    /// DbcError
    #[error("DbcError: {0}")]
    DbcError(#[from] DbcError),
    /// InterfaceError
    #[error("InterfaceError: {0}")]
    InterfaceError(#[from] InterfaceError),
    #[error("IoError: {0}")]
    IoError(#[from] std::io::Error),
    /// UnversionedContentError
    #[error("UnversionedContentError: {0}")]
    UnversionedContentError(String),
    /// Content may have been correctly stored on the network, but verification failed
    #[error("Content may have been correctly stored on the network, but verification failed: {0}")]
    ContentUploadVerificationFailed(XorUrl),
    /// DbcReissueError
    #[error("DbcReissueError: {0}")]
    DbcReissueError(String),
    /// NotImplementedError
    #[error("NotImplementedError: {0}")]
    NotImplementedError(String),
}
