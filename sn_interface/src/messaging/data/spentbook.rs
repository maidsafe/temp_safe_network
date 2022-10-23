// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{CmdResponse, Error, QueryResponse};

use crate::messaging::system::SectionSigned;
use crate::network_knowledge::{SectionAuthorityProvider, SectionsDAG};
use crate::types::SpentbookAddress;

use serde::{Deserialize, Serialize};
use sn_dbc::{KeyImage, RingCtTransaction, SpentProof};
use std::collections::BTreeSet;
use xor_name::XorName;

/// Spentbook read operations.
#[allow(clippy::large_enum_variant)]
#[derive(Hash, Eq, PartialEq, PartialOrd, Clone, Serialize, Deserialize, Debug)]
pub enum SpentbookQuery {
    /// Query the set of spent proofs if the provided key image has already been spent with a Tx
    SpentProofShares(SpentbookAddress),
}

/// A Spentbook cmd.
#[allow(clippy::large_enum_variant)]
#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Debug)]
pub enum SpentbookCmd {
    /// Request to Elders to log a key image and its associated transaction as spent.
    ///
    /// If successful, a spent proof signature share will be returned to the sender.
    ///
    /// There is an optional `network_knowledge` field to submit an updated proof chain and the
    /// latest SAP for that proof chain. This can be used in the case where one of the spent proofs
    /// was signed with a section key that the spend request section was not aware of.
    Spend {
        key_image: KeyImage,
        tx: RingCtTransaction,
        spent_proofs: BTreeSet<SpentProof>,
        spent_transactions: BTreeSet<RingCtTransaction>,
        network_knowledge: Option<(SectionsDAG, SectionSigned<SectionAuthorityProvider>)>,
    },
}

impl SpentbookCmd {
    /// Creates a Response containing an error, with the Response variant corresponding to the
    /// Request variant.
    pub fn to_error_response(&self, error: Error) -> CmdResponse {
        match self {
            Self::Spend { .. } => CmdResponse::SpendKey(Err(error)),
        }
    }
}

impl SpentbookQuery {
    /// Creates a Response containing an error, with the Response variant corresponding to the
    /// Request variant.
    pub fn to_error_response(&self, error: Error) -> QueryResponse {
        match self {
            Self::SpentProofShares(_) => QueryResponse::SpentProofShares(Err(error)),
        }
    }

    /// Returns the dst address for the request.
    pub fn dst_address(&self) -> SpentbookAddress {
        match self {
            Self::SpentProofShares(address) => *address,
        }
    }

    /// Returns the xorname of the data for request.
    pub fn dst_name(&self) -> XorName {
        *self.dst_address().name()
    }
}

impl SpentbookCmd {
    /// Returns the name of the register.
    /// This is not a unique identifier.
    pub fn name(&self) -> XorName {
        *self.dst_address().name()
    }

    /// Returns the dst address of the spentbook.
    pub fn dst_address(&self) -> SpentbookAddress {
        match self {
            Self::Spend { key_image, .. } => {
                SpentbookAddress::new(XorName::from_content(&key_image.to_bytes()))
            }
        }
    }
}
