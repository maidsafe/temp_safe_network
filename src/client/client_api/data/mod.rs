// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod pac_man;

pub(crate) use pac_man::{encrypt_blob, to_chunk, SecretKey};

use crate::{
    client::{Error, Result},
    url::Scope,
};

use bytes::Bytes;
use self_encryption::MIN_ENCRYPTABLE_BYTES;
use xor_name::XorName;

/// Data of size more than 0 bytes less than [`MIN_ENCRYPTABLE_BYTES`] bytes.
///
/// A `Spot` cannot be self-encrypted, thus is encrypted using the client encryption keys instead.
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct Spot {
    bytes: Bytes,
}

/// Data of size larger than or equal to [`MIN_ENCRYPTABLE_BYTES`] bytes.
///
/// A `Blob` is spread across multiple chunks in the network.
/// This is done using self-encryption, which produces at least 4 chunks (3 for the contents, 1 for the `BlobSecretKey`).
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct Blob {
    bytes: Bytes,
}

impl Spot {
    /// Enforces size > 0 and size < [`MIN_ENCRYPTABLE_BYTES`] bytes.
    pub fn new(bytes: Bytes) -> Result<Self> {
        if bytes.len() >= MIN_ENCRYPTABLE_BYTES {
            Err(Error::Generic(
                "The provided bytes is too large to be a `Spot`".to_string(),
            ))
        } else if bytes.is_empty() {
            Err(Error::Generic("Cannot store empty bytes.".to_string()))
        } else {
            Ok(Self { bytes })
        }
    }

    /// Returns the bytes.
    pub fn bytes(&self) -> Bytes {
        self.bytes.clone()
    }
}

impl Blob {
    /// Enforces size >= [`MIN_ENCRYPTABLE_BYTES`] bytes.
    pub fn new(bytes: Bytes) -> Result<Self> {
        if MIN_ENCRYPTABLE_BYTES > bytes.len() {
            Err(Error::Generic(
                "The provided bytes is too small to be a `Blob`".to_string(),
            ))
        } else {
            Ok(Self { bytes })
        }
    }

    /// Returns the bytes.
    pub fn bytes(&self) -> Bytes {
        self.bytes.clone()
    }
}

/// Address of a Blob.
#[derive(
    Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize, Debug,
)]
pub enum BlobAddress {
    /// Private namespace.
    Private(XorName),
    /// Public namespace.
    Public(XorName),
}

impl BlobAddress {
    /// The xorname.
    pub fn name(&self) -> &XorName {
        match self {
            Self::Public(name) | Self::Private(name) => name,
        }
    }

    /// The namespace scope of the Blob
    pub fn scope(&self) -> Scope {
        if self.is_public() {
            Scope::Public
        } else {
            Scope::Private
        }
    }

    /// Returns true if public.
    pub fn is_public(self) -> bool {
        matches!(self, BlobAddress::Public(_))
    }

    /// Returns true if private.
    pub fn is_private(self) -> bool {
        !self.is_public()
    }
}

/// Address of a Spot.
#[derive(
    Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize, Debug,
)]
pub enum SpotAddress {
    /// Private namespace.
    Private(XorName),
    /// Public namespace.
    Public(XorName),
}

impl SpotAddress {
    /// The xorname.
    pub fn name(&self) -> &XorName {
        match self {
            Self::Public(name) | Self::Private(name) => name,
        }
    }

    /// The namespace scope of the Spot
    pub fn scope(&self) -> Scope {
        if self.is_public() {
            Scope::Public
        } else {
            Scope::Private
        }
    }

    /// Returns true if public.
    pub fn is_public(self) -> bool {
        matches!(self, SpotAddress::Public(_))
    }

    /// Returns true if private.
    pub fn is_private(self) -> bool {
        !self.is_public()
    }
}
