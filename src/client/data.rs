// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{
    blob::{BlobRead, BlobWrite},
    map::{MapRead, MapWrite},
    sequence::{SequenceRead, SequenceWrite},
    AuthorisationKind, CmdError, Error, QueryResponse,
};
use sn_data_types::PublicKey;
use xor_name::XorName;

use serde::{Deserialize, Serialize};
use std::fmt;

/// Data command operations. Creating, updating or removing data
#[allow(clippy::large_enum_variant)]
#[derive(Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum DataCmd {
    /// Blob write operation
    Blob(BlobWrite),
    /// Map write operation
    Map(MapWrite),
    /// Sequence write operation
    Sequence(SequenceWrite),
}

impl DataCmd {
    /// Creates a Response containing an error, with the Response variant corresponding to the
    /// cuest variant.
    pub fn error(&self, error: Error) -> CmdError {
        use DataCmd::*;
        match self {
            Blob(c) => c.error(error),
            Map(c) => c.error(error),
            Sequence(c) => c.error(error),
        }
    }
    /// Returns the type of authorisation needed for the cuest.
    pub fn authorisation_kind(&self) -> AuthorisationKind {
        use DataCmd::*;
        match self {
            Blob(c) => c.authorisation_kind(),
            Map(c) => c.authorisation_kind(),
            Sequence(c) => c.authorisation_kind(),
        }
    }

    /// Returns the address of the destination for `cuest`.
    pub fn dst_address(&self) -> XorName {
        use DataCmd::*;
        match self {
            Blob(c) => c.dst_address(),
            Map(c) => c.dst_address(),
            Sequence(c) => c.dst_address(),
        }
    }

    /// Returns the owner of the data.
    pub fn owner(&self) -> Option<PublicKey> {
        match self {
            Self::Blob(write) => write.owner(),
            Self::Map(write) => write.owner(),
            Self::Sequence(write) => write.owner(),
        }
    }
}

impl fmt::Debug for DataCmd {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use DataCmd::*;
        match self {
            Blob(c) => write!(formatter, "{:?}", c),
            Map(c) => write!(formatter, "{:?}", c),
            Sequence(c) => write!(formatter, "{:?}", c),
        }
    }
}

/// TODO: docs
#[allow(clippy::large_enum_variant)]
#[derive(Hash, Eq, PartialEq, PartialOrd, Clone, Serialize, Deserialize)]
pub enum DataQuery {
    /// TODO: docs
    Blob(BlobRead),
    /// TODO: docs
    Map(MapRead),
    /// TODO: docs
    Sequence(SequenceRead),
}

impl DataQuery {
    /// Creates a Response containing an error, with the Response variant corresponding to the
    /// Request variant.
    pub fn error(&self, error: Error) -> QueryResponse {
        use DataQuery::*;
        match self {
            Blob(q) => q.error(error),
            Map(q) => q.error(error),
            Sequence(q) => q.error(error),
        }
    }

    /// Returns the type of authorisation needed for the request.
    pub fn authorisation_kind(&self) -> AuthorisationKind {
        use DataQuery::*;
        match self {
            Blob(q) => q.authorisation_kind(),
            Map(q) => q.authorisation_kind(),
            Sequence(q) => q.authorisation_kind(),
        }
    }

    /// Returns the address of the destination for `request`.
    pub fn dst_address(&self) -> XorName {
        use DataQuery::*;
        match self {
            Blob(q) => q.dst_address(),
            Map(q) => q.dst_address(),
            Sequence(q) => q.dst_address(),
        }
    }
}

impl fmt::Debug for DataQuery {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use DataQuery::*;
        match self {
            Blob(q) => write!(formatter, "{:?}", q),
            Map(q) => write!(formatter, "{:?}", q),
            Sequence(q) => write!(formatter, "{:?}", q),
        }
    }
}
