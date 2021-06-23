// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{CmdError, Error, QueryResponse};
use crate::types::{
    PublicKey, Sequence, SequenceAddress as Address, SequenceEntry as Entry,
    SequenceIndex as Index, SequenceOp, SequenceUser as User,
};
use serde::{Deserialize, Serialize};
use std::fmt;
use xor_name::XorName;

/// TODO: docs
#[derive(Hash, Eq, PartialEq, PartialOrd, Clone, Serialize, Deserialize, Debug)]
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
}

///
#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct SequenceCmd {
    ///
    pub write: SequenceWrite,
    ///
    pub msg_id: crate::messaging::MessageId,
    ///
    pub client_sig: crate::messaging::ClientSigned,
    ///
    pub origin: crate::messaging::EndUser,
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
            | GetUserPermissions { ref address, .. } => *address.name(),
        }
    }
}

impl SequenceWrite {
    /// Creates a Response containing an error, with the Response variant corresponding to the
    /// Request variant.
    pub fn error(&self, error: Error) -> CmdError {
        CmdError::Data(error)
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

    /// Returns the address of the sequence.
    pub fn address(&self) -> &Address {
        match self {
            Self::New(map) => map.address(),
            Self::Delete(address) => address,
            Self::Edit(ref op) => &op.address,
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
            match self {
                New(seq) => format!("New({:?})", seq.address()),
                Delete(address) => format!("Delete({:?})", address),
                Edit(op) => format!("Edit({:?})", op),
            }
        )
    }
}
