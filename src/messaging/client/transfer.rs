// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{CmdError, Error, QueryResponse, TransferError};
#[cfg(feature = "simulated-payouts")]
use crate::types::Transfer;
use crate::types::{PublicKey, SignedTransfer, TransferAgreementProof};
use serde::{Deserialize, Serialize};
use std::fmt;
use xor_name::XorName;

/// Token cmd that is sent to network.
#[allow(clippy::large_enum_variant)]
#[derive(Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum TransferCmd {
    #[cfg(feature = "simulated-payouts")]
    /// Cmd to simulate a farming payotu
    SimulatePayout(Transfer),
    /// The cmd to validate a transfer.
    ValidateTransfer(SignedTransfer),
    /// The cmd to register the consensused transfer.
    RegisterTransfer(TransferAgreementProof),
}

/// Token query that is sent to network.
#[allow(clippy::large_enum_variant)]
#[derive(Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum TransferQuery {
    /// Get key balance.
    GetBalance(PublicKey),
    /// Get key transfers since specified version.
    GetHistory {
        /// The balance key.
        at: PublicKey,
        /// The last version of transfers we know of.
        since_version: usize,
    },
    /// Get the latest cost for writing given number of bytes to network.
    GetStoreCost {
        /// The requester's key.
        requester: PublicKey,
        ///
        bytes: u64,
    },
}

impl TransferCmd {
    /// Creates a Response containing an error, with the Response variant corresponding to the
    /// Request variant.
    pub fn error(&self, error: Error) -> CmdError {
        use CmdError::*;
        use TransferCmd::*;
        use TransferError::*;
        match *self {
            ValidateTransfer(_) => Transfer(TransferValidation(error)),
            RegisterTransfer(_) => Transfer(TransferRegistration(error)),
            #[cfg(feature = "simulated-payouts")]
            SimulatePayout(_) => Transfer(TransferRegistration(error)),
        }
    }

    /// Returns the address of the destination for `request`.
    pub fn dst_address(&self) -> XorName {
        use TransferCmd::*;
        match self {
            RegisterTransfer(ref proof) => XorName::from(proof.sender()), // this is handled where the debit is made
            ValidateTransfer(ref signed_transfer) => XorName::from(signed_transfer.sender()), // this is handled where the debit is made
            #[cfg(feature = "simulated-payouts")]
            SimulatePayout(ref transfer) => XorName::from(transfer.debit().sender()), // this is handled where the debit is made
        }
    }
}

impl fmt::Debug for TransferCmd {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use TransferCmd::*;
        write!(
            formatter,
            "TransferCmd::{}",
            match *self {
                RegisterTransfer { .. } => "RegisterTransfer",
                ValidateTransfer { .. } => "ValidateTransfer",
                #[cfg(feature = "simulated-payouts")]
                SimulatePayout { .. } => "SimulatePayout",
            }
        )
    }
}

impl TransferQuery {
    /// Creates a QueryResponse containing an error, with the QueryResponse variant corresponding to the
    /// Request variant.
    pub fn error(&self, error: Error) -> QueryResponse {
        use TransferQuery::*;
        match *self {
            GetBalance(_) => QueryResponse::GetBalance(Err(error)),
            GetHistory { .. } => QueryResponse::GetHistory(Err(error)),
            GetStoreCost { .. } => QueryResponse::GetStoreCost(Err(error)),
        }
    }

    /// Returns the address of the destination for the query.
    pub fn dst_address(&self) -> XorName {
        use TransferQuery::*;
        match self {
            GetBalance(at) | GetHistory { at, .. } | GetStoreCost { requester: at, .. } => {
                XorName::from(*at)
            }
        }
    }
}

impl fmt::Debug for TransferQuery {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use TransferQuery::*;
        match *self {
            GetBalance(_) => write!(formatter, "TransferQuery::GetBalance"),
            GetHistory { .. } => write!(formatter, "TransferQuery::GetHistory"),
            GetStoreCost { bytes, .. } => {
                write!(formatter, "TransferQuery::GetStoreCost of {:?}", bytes)
            }
        }
    }
}
