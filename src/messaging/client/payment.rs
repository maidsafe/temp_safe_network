// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{CmdError, Error, PointerEdit, PointerEditKind, QueryResponse, TransferError};
use crate::types::{Chunk, PublicKey, SignatureShare, Token};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};
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
    ///
    pub inquiry: CostInquiry,
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
    ///
    pub key: bls::PublicKey,
}

///
#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct GuaranteedQuote {
    ///
    pub quote: PaymentQuote,
    ///
    pub sig: bls::Signature,
    ///
    pub key_set: bls::PublicKeySet,
}

///
#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct PaymentReceiptShare {
    ///
    pub data: BTreeSet<XorName>,
    ///
    pub sig: SignatureShare,
    ///
    pub key: bls::PublicKey,
}

///
#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct PaymentReceipt {
    ///
    pub data: BTreeSet<XorName>,
    ///
    pub sig: bls::Signature,
    ///
    pub key_set: bls::PublicKeySet,
}

/// The provided data must match the name and bytes specified
/// in the quote.
/// Also the quote must be signed by a known section key (this is at DbcSection).
/// It is then guaranteed to be accepted (at DataSection), if payment provided
/// matches the quote, and the dbcs are valid.

///
#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Debug)]
pub enum DebitableOp {
    ///
    Upload {
        ///
        data: BTreeSet<Chunk>,
        ///
        payment: PaymentReceipt,
    },
    ///
    PointerEdit {
        ///
        ops: BTreeSet<PointerEdit>,
        ///
        payment: PaymentReceipt,
    },
}

///
#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Debug)]
pub enum CostInquiry {
    ///
    Upload(BTreeSet<XorName>),
    ///
    PointerEdit(BTreeSet<PointerEditKind>),
}

impl PaymentCmd {
    /// Creates a Response containing an error, with the Response variant corresponding to the
    /// Request variant.
    pub fn error(&self, error: Error) -> CmdError {
        use CmdError::*;
        use PaymentCmd::*;
        use TransferError::*;
        match *self {
            RegisterPayment(_) => Transfer(TransferRegistration(error)),
        }
    }

    /// Returns the address of the destination for `request`.
    pub fn dst_address(&self) -> XorName {
        use PaymentCmd::*;
        match self {
            RegisterPayment(ref _reg) => XorName::random(), //XorName::from(reg.quote.signers.public_key()), // this is handled where the debit is made
        }
    }
}

impl fmt::Debug for PaymentCmd {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use PaymentCmd::*;
        write!(
            formatter,
            "PaymentCmd::{}",
            match *self {
                RegisterPayment { .. } => "RegisterPayment",
            }
        )
    }
}

impl PaymentQuery {
    /// Creates a QueryResponse containing an error, with the QueryResponse variant corresponding to the
    /// Request variant.
    #[allow(unused)]
    pub fn error(&self, error: Error) -> QueryResponse {
        use PaymentQuery::*;
        match *self {
            GetQuote { .. } => QueryResponse::GetStoreCost(Err(error)),
        }
    }

    /// Returns the address of the destination for the query.
    #[allow(unused)]
    pub fn dst_address(&self) -> XorName {
        use PaymentQuery::*;
        match self {
            GetQuote { .. } => XorName::random(), // XorName::from(*at),
        }
    }
}

impl fmt::Debug for PaymentQuery {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use PaymentQuery::*;
        match *self {
            GetQuote { .. } => write!(formatter, "PaymentQuery::GetQuote"),
        }
    }
}
