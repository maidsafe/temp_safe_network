// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{AuthorisationKind, CmdError, DataAuthKind, Error, QueryResponse};
use serde::{Deserialize, Serialize};
use sn_data_types::{Blob, BlobAddress, PublicKey};
use std::fmt;
use xor_name::XorName;

/// TODO: docs
#[derive(Hash, Eq, PartialEq, PartialOrd, Clone, Serialize, Deserialize)]
pub enum BlobRead {
    /// TODO: docs
    Get(BlobAddress),
}

/// TODO: docs
#[allow(clippy::large_enum_variant)]
#[derive(Hash, Eq, PartialEq, PartialOrd, Clone, Serialize, Deserialize)]
pub enum BlobWrite {
    /// TODO: docs
    New(Blob),
    /// TODO: docs
    DeletePrivate(BlobAddress),
}

impl BlobRead {
    // /// Get the `Type` of this `Request`.
    // pub fn get_type(&self) -> Type {
    //     use BlobRead::*;
    //     match self {
    //         Get(BlobAddress::Public(_)) => Type::PublicRead,
    //         Get(BlobAddress::Private(_)) => Type::PrivateRead,
    //     }
    // }

    /// Creates a Response containing an error, with the Response variant corresponding to the
    /// Request variant.
    pub fn error(&self, error: Error) -> QueryResponse {
        QueryResponse::GetBlob(Err(error))
    }

    /// Returns the type of authorisation needed for the request.
    pub fn authorisation_kind(&self) -> AuthorisationKind {
        use BlobRead::*;
        match self {
            Get(BlobAddress::Public(_)) => AuthorisationKind::Data(DataAuthKind::PublicRead),
            Get(BlobAddress::Private(_)) => AuthorisationKind::Data(DataAuthKind::PrivateRead),
        }
    }

    /// Returns the address of the destination for `request`.
    pub fn dst_address(&self) -> XorName {
        use BlobRead::*;
        match self {
            Get(ref address) => *address.name(),
        }
    }
}

impl BlobWrite {
    /// Creates a Response containing an error, with the Response variant corresponding to the
    /// Request variant.
    pub fn error(&self, error: Error) -> CmdError {
        CmdError::Data(error)
    }

    /// Returns the type of authorisation needed for the request.
    pub fn authorisation_kind(&self) -> AuthorisationKind {
        AuthorisationKind::Data(DataAuthKind::Write)
    }

    /// Returns the address of the destination for `request`.
    pub fn dst_address(&self) -> XorName {
        use BlobWrite::*;
        match self {
            New(ref data) => *data.name(),
            DeletePrivate(ref address) => *address.name(),
        }
    }

    /// Returns the owner of the data on a New Blob write.
    pub fn owner(&self) -> Option<PublicKey> {
        match self {
            Self::New(data) => data.owner().cloned(),
            Self::DeletePrivate(_) => None,
        }
    }
}

impl fmt::Debug for BlobRead {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use BlobRead::*;
        match self {
            Get(req) => write!(formatter, "{:?}", req),
        }
    }
}

impl fmt::Debug for BlobWrite {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use BlobWrite::*;
        match self {
            New(blob) => write!(formatter, "BlobWrite::New({:?})", blob),
            DeletePrivate(address) => write!(formatter, "BlobWrite::DeletePrivate({:?})", address),
        }
    }
}
