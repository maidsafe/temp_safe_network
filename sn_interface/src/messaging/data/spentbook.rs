// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{CmdResponse, Error, QueryResponse};

use crate::dbcs::DbcReason;
use crate::messaging::system::SectionSigned;
use crate::network_knowledge::{SectionAuthorityProvider, SectionsDAG};
use crate::types::{
    fees::{FeeCiphers, SpendPriority},
    SpentbookAddress,
};

use sn_dbc::{DbcTransaction, PublicKey, SpentProof};

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use xor_name::XorName;

/// Spend related read operations.
#[derive(Hash, Eq, PartialEq, PartialOrd, Clone, Serialize, Deserialize, Debug)]
pub enum SpendQuery {
    /// Query for the individual reward keys and their respective
    /// fee amount for processing a `Spend` of a Dbc with the given id.
    GetFees {
        dbc_id: PublicKey,
        priority: SpendPriority,
    },
    /// Query for the set of spent proofs if the provided public key has already been spent with a Tx.
    GetSpentProofShares(SpentbookAddress),
}

/// A Spentbook cmd.
#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, custom_debug::Debug)]
pub enum SpentbookCmd {
    /// Request to Elders to log a public key and its associated transaction as spent.
    ///
    /// If successful, a spent proof signature share will be returned to the sender.
    ///
    /// There is an optional `network_knowledge` field to submit an updated proof chain and the
    /// latest SAP for that proof chain. This can be used in the case where one of the spent proofs
    /// was signed with a section key that the spend request section was not aware of.
    Spend {
        public_key: PublicKey,
        #[debug(skip)]
        tx: DbcTransaction,
        /// Reason for spending a DBC, used for data payments
        reason: DbcReason,
        #[debug(skip)]
        spent_proofs: BTreeSet<SpentProof>,
        #[debug(skip)]
        spent_transactions: BTreeSet<DbcTransaction>,
        #[debug(skip)]
        network_knowledge: Option<(SectionsDAG, SectionSigned<SectionAuthorityProvider>)>,
        /// As to avoid impl separate cmd flow, we send
        /// all fee ciphers to all Elders for now.
        #[debug(skip)]
        #[cfg(not(feature = "data-network"))]
        fee_ciphers: BTreeMap<XorName, FeeCiphers>,
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

impl SpendQuery {
    /// Creates a Response containing an error, with the Response variant corresponding to the
    /// Request variant.
    pub fn to_error_response(&self, error: Error) -> QueryResponse {
        match self {
            Self::GetFees { .. } => QueryResponse::GetFees(Err(error)),
            Self::GetSpentProofShares(_) => QueryResponse::GetSpentProofShares(Err(error)),
        }
    }

    /// Returns the dst address for the request.
    pub fn dst_address(&self) -> SpentbookAddress {
        match self {
            Self::GetFees { dbc_id, .. } => {
                SpentbookAddress::new(XorName::from_content(&dbc_id.to_bytes()))
            }
            Self::GetSpentProofShares(address) => *address,
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
            Self::Spend { public_key, .. } => {
                SpentbookAddress::new(XorName::from_content(&public_key.to_bytes()))
            }
        }
    }
}
