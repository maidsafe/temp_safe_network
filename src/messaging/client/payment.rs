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
pub enum PaymentCmd {
    /// The cmd to register the consensused transfer.
    RegisterPayment(RegisterPayment),
}

/// Token query that is sent to network.
#[allow(clippy::large_enum_variant)]
#[derive(Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum PaymentQuery {
    /// Get a quote for storing a set of chunks to the network.
    GetQuote(BTreeSet<XorName>),
}

// 1. GetQuote(data)
// 2. Aggregate responses
// 3. RegisterPayment(quote, payment)
// 4. PaymentRegistered(receipt)

/// The quote must be signed by a known section key (this is at DbcSection).
/// The DBCs must be valid.
/// The provided payment must match the payees and amounts specified in the quote.
/// The set of chunk names (specified in the quote) are then guaranteed to be signed as paid for.
#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct RegisterPayment {
    ///
    pub quote: GuaranteedQuote,
    ///
    pub payment: BTreeMap<PublicKey, sn_dbc::Dbc>,
}

/// A given piece of data, which must match the name and bytes specified,
/// is guaranteed to be accepted, if payment matching this quote
/// is provided together with the quote.
#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct PaymentQuote {
    /// Uploader is responsible for sending 
    pub data: BTreeMap<XorName, u64>,
    ///
    pub payable: BTreeMap<PublicKey, Token>,
}

///
#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct GuaranteedQuoteShare {
    ///
    pub quote: PaymentQuote,
    ///
    pub sig: SignatureShare,
}

///
#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct GuaranteedQuote {
    ///
    pub quote: PaymentQuote,
    ///
    pub sig: Signature,
}

///
#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct PaymentReceiptShare {
    /// The size of each chunk must be specified
    pub data: BTreeMap<XorName, u64>,
    ///
    pub sig: SignatureShare,
    ///
    pub signer: bls::PublicKey,
}

///
#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct PaymentReceipt {
    ///
    pub data: BTreeSet<XorName>,
    ///
    pub sig: bls::Signature,
    ///
    pub signers: bls::PublicKeySet,
}

/// The provided data must match the name and bytes specified
/// in the quote.
/// Also the quote must be signed by a known section key (this is at DbcSection).
/// It is then guaranteed to be accepted (at DataSection), if payment provided
/// matches the quote, and the dbcs are valid.

///
#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Debug)]
pub enum DebitableCmd {
    ///
    Data {
        cmd: DataCmd,
        payment: PaymentReceipt,
    }
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
