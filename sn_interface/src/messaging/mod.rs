// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! The Safe Network messaging interface.
//!
//! This modules defines the messages that can be handled by the Safe Network. In particular:
//!
//! - This module contains types that are common across the messaging API.
//! - The [`serialisation`] module defines the wire format and message (de)serialization API.
//! - The [`data`] module defines the data messages that clients and nodes send, and their possible responses.
//! - The [`signature_aggregator`] module defines the BLS signature aggregator.
//! - The [`system`] module defines Operational Messages that can be exchanged on the network.

/// Data messages that clients and nodes can send.
pub mod data;
/// The wire format and message (de)serialization API.
pub mod serialisation;
/// BLS Signature aggregator
pub mod signature_aggregator;
/// Operational Messages that can be exchanged on the network.
pub mod system;

// Network membership state sync.
mod anti_entropy;
// Message authority - keys and signatures.
mod authority;
// Error types definitions
mod errors;
// Message ID definition
mod msg_id;
// Message types
mod network_msg;
// Types of messages and corresponding source authorities
mod msg_kind;
// Msg dst
mod dst;

pub use self::{
    anti_entropy::{AntiEntropyKind, AntiEntropyMsg},
    authority::{
        AuthorityProof, ClientAuth, NodeSig, SectionSig, SectionSigShare, VerifyAuthority,
    },
    dst::Dst,
    errors::{Error, Result},
    msg_id::{MsgId, MESSAGE_ID_LEN},
    msg_kind::MsgKind,
    network_msg::NetworkMsg,
    serialisation::WireMsg,
};

use serde::{Deserialize, Serialize};
use xor_name::XorName;

/// An `EndUser` is represented by a name which is mapped to
// a SocketAddr at the Elders where the `EndUser` is proxied through.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Debug)]
pub struct EndUser(pub XorName);
