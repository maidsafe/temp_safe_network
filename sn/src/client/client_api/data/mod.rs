// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod pac_man;

pub(crate) use pac_man::{encrypt_blob, to_chunk, DataMapLevel};

use crate::client::{Error, Result};

use bytes::Bytes;
use self_encryption::MIN_ENCRYPTABLE_BYTES;

/// Data of size more than 0 bytes less than [`MIN_ENCRYPTABLE_BYTES`] bytes.
///
/// A `Spot` cannot be self-encrypted, thus is encrypted using the client encryption keys instead.
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub(crate) struct Spot {
    bytes: Bytes,
}

/// Data of size larger than or equal to [`MIN_ENCRYPTABLE_BYTES`] bytes.
///
/// A `Blob` is spread across multiple chunks in the network.
/// This is done using self-encryption, which produces at least 4 chunks (3 for the contents, 1 for the `DataMap`).
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub(crate) struct Blob {
    bytes: Bytes,
}

impl Spot {
    /// Enforces size > 0 and size < [`MIN_ENCRYPTABLE_BYTES`] bytes.
    pub(crate) fn new(bytes: Bytes) -> Result<Self> {
        if bytes.len() >= MIN_ENCRYPTABLE_BYTES {
            Err(Error::TooLargeToBeSpot)
        } else if bytes.is_empty() {
            Err(Error::EmptyBytesProvided)
        } else {
            Ok(Self { bytes })
        }
    }

    /// Returns the bytes.
    pub(crate) fn bytes(&self) -> Bytes {
        self.bytes.clone()
    }
}

impl Blob {
    /// Enforces size >= [`MIN_ENCRYPTABLE_BYTES`] bytes.
    pub(crate) fn new(bytes: Bytes) -> Result<Self> {
        if MIN_ENCRYPTABLE_BYTES > bytes.len() {
            Err(Error::TooSmallToBeBlob)
        } else {
            Ok(Self { bytes })
        }
    }

    /// Returns the bytes.
    pub(crate) fn bytes(&self) -> Bytes {
        self.bytes.clone()
    }
}
