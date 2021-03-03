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
use sn_data_types::{
    PublicKey, Sequence, SequenceAddress as Address, SequenceEntry as Entry,
    SequenceIndex as Index, SequenceOp, SequenceUser as User,
};
use std::fmt;
use xor_name::XorName;

/// TODO: docs
#[derive(Hash, Eq, PartialEq, PartialOrd, Clone, Serialize, Deserialize)]
pub enum SequenceRead {
    /// Get Sequence from the network.
    Get(Address),
    /// Get a range of entries from an Sequence object on the network.
    GetRange {
        /// Sequence address.
        address: Address,
        /// Range of entries to fetch.
        ///
        /// For example, get 10 last entries:
        /// range: (Index::FromEnd(10), Index::FromEnd(0))
        ///
        /// Get all entries:
        /// range: (Index::FromStart(0), Index::FromEnd(0))
        ///
        /// Get first 5 entries:
        /// range: (Index::FromStart(0), Index::FromStart(5))
        range: (Index, Index),
    },
    /// Get last entry from the Sequence.
    GetLastEntry(Address),
    /// List current policy
    GetPublicPolicy(Address),
    /// List current policy
    GetPrivatePolicy(Address),
    /// Get current permissions for a specified user(s).
    GetUserPermissions {
        /// Sequence address.
        address: Address,
        /// User to get permissions for.
        user: User,
    },
    /// Get current owner.
    GetOwner(Address),
}

/// TODO: docs
#[allow(clippy::large_enum_variant)]
#[derive(Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum SequenceWrite {
    /// Create a new Sequence on the network.
    New(Sequence),
    /// Edit the Sequence (insert/remove entry).
    Edit(SequenceOp<Entry>),
    /// Delete a private Sequence.
    ///
    /// This operation MUST return an error if applied to public Sequence. Only the current
    /// owner(s) can perform this action.
    Delete(Address),
}

impl SequenceRead {
    /// Creates a Response containing an error, with the Response variant corresponding to the
    /// Request variant.
    pub fn error(&self, error: Error) -> QueryResponse {
        use SequenceRead::*;
        match *self {
            Get(_) => QueryResponse::GetSequence(Err(error)),
            GetRange { .. } => QueryResponse::GetSequenceRange(Err(error)),
            GetLastEntry(_) => QueryResponse::GetSequenceLastEntry(Err(error)),
            GetPublicPolicy(_) => QueryResponse::GetSequencePublicPolicy(Err(error)),
            GetPrivatePolicy(_) => QueryResponse::GetSequencePrivatePolicy(Err(error)),
            GetUserPermissions { .. } => QueryResponse::GetSequenceUserPermissions(Err(error)),
            GetOwner(_) => QueryResponse::GetSequenceOwner(Err(error)),
        }
    }

    /// Returns the access categorisation of the request.
    pub fn authorisation_kind(&self) -> AuthorisationKind {
        use SequenceRead::*;
        match *self {
            Get(address)
            | GetRange { address, .. }
            | GetLastEntry(address)
            | GetPublicPolicy(address)
            | GetPrivatePolicy(address)
            | GetUserPermissions { address, .. }
            | GetOwner(address) => {
                if address.is_public() {
                    AuthorisationKind::Data(DataAuthKind::PublicRead)
                } else {
                    AuthorisationKind::Data(DataAuthKind::PrivateRead)
                }
            }
        }
    }

    /// Returns the address of the destination for request.
    pub fn dst_address(&self) -> XorName {
        use SequenceRead::*;
        match self {
            Get(ref address)
            | GetRange { ref address, .. }
            | GetLastEntry(ref address)
            | GetPublicPolicy(ref address)
            | GetPrivatePolicy(ref address)
            | GetUserPermissions { ref address, .. }
            | GetOwner(ref address) => *address.name(),
        }
    }
}

impl fmt::Debug for SequenceRead {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use SequenceRead::*;
        write!(
            formatter,
            "SequenceRead::{}",
            match *self {
                Get(_) => "GetSequence",
                GetRange { .. } => "GetSequenceRange",
                GetLastEntry(_) => "GetSequenceLastEntry",
                GetPublicPolicy { .. } => "GetSequencePublicPolicy",
                GetPrivatePolicy { .. } => "GetSequencePrivatePolicy",
                GetUserPermissions { .. } => "GetUserPermissions",
                GetOwner { .. } => "GetOwner",
            }
        )
    }
}

impl SequenceWrite {
    /// Creates a Response containing an error, with the Response variant corresponding to the
    /// Request variant.
    pub fn error(&self, error: Error) -> CmdError {
        CmdError::Data(error)
    }

    /// Returns the access categorisation of the request.
    pub fn authorisation_kind(&self) -> AuthorisationKind {
        AuthorisationKind::Data(DataAuthKind::Write)
    }

    /// Returns the address of the destination for request.
    pub fn dst_address(&self) -> XorName {
        use SequenceWrite::*;
        match self {
            New(ref data) => *data.name(),
            Delete(ref address) => *address.name(),
            Edit(ref op) => *op.address.name(),
        }
    }

    /// Owner of the SequenceWrite
    pub fn owner(&self) -> Option<PublicKey> {
        match self {
            Self::New(data) => Some(data.owner()),
            _ => None,
        }
    }
}

impl fmt::Debug for SequenceWrite {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use SequenceWrite::*;
        write!(
            formatter,
            "SequenceWrite::{}",
            match *self {
                New(_) => "NewSequence",
                Delete(_) => "DeleteSequence",
                Edit(_) => "EditSequence",
            }
        )
    }
}
