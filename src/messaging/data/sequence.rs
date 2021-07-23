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
use xor_name::XorName;

/// [`Sequence`] read operations.
#[derive(Hash, Eq, PartialEq, PartialOrd, Clone, Serialize, Deserialize, Debug)]
pub enum SequenceRead {
    /// Retrieve the [`Sequence`] at the given address.
    ///
    /// This should eventually lead to a [`GetSequence`] response.
    ///
    /// Note that alternative sequence queries may be more efficient and convenient if you do not
    /// need the full value of the sequence.
    ///
    /// [`GetSequence`]: QueryResponse::GetSequence
    Get(Address),
    /// Retrieve a range of entries from the [`Sequence`] at the given address.
    ///
    /// This should eventially lead to a [`GetSequenceRange`] response.
    ///
    /// [`GetSequenceRange`]: QueryResponse::GetSequenceRange
    GetRange {
        /// Sequence address.
        address: Address,
        /// Range of entries to fetch.
        ///
        /// For example, get 10 last entries:
        ///
        /// ```no_run
        /// # #[allow(warnings)] {
        /// # use safe_network::messaging::client::SequenceRead::*;
        /// # use safe_network::types::{SequenceAddress, SequenceIndex};
        /// # let address: SequenceAddress = todo!();
        /// GetRange {
        ///     address,
        ///     range: (SequenceIndex::FromEnd(10), SequenceIndex::FromEnd(0)),
        /// }
        /// # };
        /// ```
        ///
        /// Get all entries:
        ///
        /// ```no_run
        /// # #[allow(warnings)] {
        /// # use safe_network::messaging::client::SequenceRead::*;
        /// # use safe_network::types::{SequenceAddress, SequenceIndex};
        /// # let address: SequenceAddress = todo!();
        /// GetRange {
        ///     address,
        ///     range: (SequenceIndex::FromStart(0), SequenceIndex::FromEnd(0)),
        /// }
        /// # };
        /// ```
        ///
        /// Get first 5 entries:
        ///
        /// ```no_run
        /// # #[allow(warnings)] {
        /// # use safe_network::messaging::client::SequenceRead::*;
        /// # use safe_network::types::{SequenceAddress, SequenceIndex};
        /// # let address: SequenceAddress = todo!();
        /// GetRange {
        ///     address,
        ///     range: (SequenceIndex::FromStart(0), SequenceIndex::FromStart(5)),
        /// }
        /// # };
        /// ```
        range: (Index, Index),
    },
    /// Retrieve the last entry from the [`Sequence`] at the given address.
    ///
    /// This should eventually lead to a [`GetSequenceLastEntry`] response.
    ///
    /// [`GetSequenceLastEntry`]: QueryResponse::GetSequenceLastEntry
    GetLastEntry(Address),
    /// Retrieve the permissions for the public [`Sequence`] at the given address.
    ///
    /// This should eventually lead to a [`GetSequencePublicPolicy`] response.
    ///
    /// [`GetSequencePublicPolicy`]: QueryResponse::GetSequencePublicPolicy
    GetPublicPolicy(Address),
    /// Retrieve the permissions for the private [`Sequence`] at the given address.
    ///
    /// This should eventually lead to a [`GetSequencePrivatePolicy`] response.
    ///
    /// [`GetSequencePrivatePolicy`]: QueryResponse::GetSequencePrivatePolicy
    GetPrivatePolicy(Address),
    /// Retrieve the permissions of the given user for the [`Sequence`] at the given address.
    ///
    /// This should eventually lead to a [`GetSequenceUserPermissions`] response.
    ///
    /// [`GetSequenceUserPermissions`]: QueryResponse::GetSequenceUserPermissions
    GetUserPermissions {
        /// Sequence address.
        address: Address,
        /// User to get permissions for.
        user: User,
    },
}

/// A [`Sequence`] write operation.
#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct SequenceCmd {
    /// The operation to perform.
    pub write: SequenceWrite,
    /// A signature carrying authority to perform the operation.
    ///
    /// This will be verified against the sequence's owner and permissions.
    pub client_sig: crate::messaging::DataSigned,
}

/// [`Sequence`] write operations.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum SequenceWrite {
    /// Create a new [`Sequence`] on the network.
    New(Sequence),
    /// Edit a [`Sequence]`.
    Edit(SequenceOp<Entry>),
    /// Delete a private [`Sequence`].
    ///
    /// This operation will result in an error if applied to a public sequence. Only private
    /// sequences can be deleted, and only by their current owner(s).
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
