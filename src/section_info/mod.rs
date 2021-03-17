// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod errors;

use crate::{MessageId, MessageType, WireMsg};
use bytes::Bytes;
pub use errors::Error;
use serde::{Deserialize, Serialize};
use sn_data_types::{PublicKey, ReplicaPublicKeySet, Signature};
use std::{collections::BTreeMap, fmt, net::SocketAddr};
use threshold_crypto::PublicKey as BlsPublicKey;
use xor_name::{Prefix, XorName};

/// Messages for exchanging network info, specifically on a target section for a msg.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Message {
    /// Message to request information about the section that matches the given name.
    GetSectionQuery(PublicKey),
    /// An EndUser that wants to interact with the network,
    /// would send this cmd to the elders received
    /// in the GetSectionResponse.
    RegisterEndUserCmd {
        /// The end user public key.
        end_user: PublicKey,
        /// A sig over the socketaddr from which
        /// this request is made, by the secret key
        /// corresponding to the end_user public key.
        socketaddr_sig: Signature,
    },
    /// If the sig over the sender socketaddr
    /// cannot be verified by the provided public key.
    RegisterEndUserError(Error),
    /// Response to `GetSectionQuery`.
    GetSectionResponse(GetSectionResponse),
    /// Updated info related to section
    SectionInfoUpdate(ErrorResponse),
}

/// All the info a client needs about their section
#[derive(Serialize, Deserialize, Hash, PartialEq, PartialOrd, Ord, Eq, Clone)]
pub struct SectionInfo {
    /// Prefix of the section.
    pub prefix: Prefix,
    /// Public key set of the section.
    pub pk_set: ReplicaPublicKeySet,
    /// Section elders.
    pub elders: BTreeMap<XorName, SocketAddr>,
}

impl fmt::Debug for SectionInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SectionInfo {{ prefix: {:?}, pk_set: PkSet {{ public_key: {:?} }}, elders: {:?} }}",
            self.prefix,
            self.pk_set.public_key(),
            self.elders
        )
    }
}

// Infrastructure error wrapper to add correltion info for triggering message
#[derive(Debug, Serialize, Deserialize, Hash, PartialEq, PartialOrd, Ord, Eq, Clone)]
pub struct ErrorResponse {
    /// Optional correlation id if this messge is in response to some non network info query/cmd
    pub correlation_id: MessageId,
    /// Section data error message
    pub error: Error,
}

/// Information about a section.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum GetSectionResponse {
    /// Successful response to `GetSectionQuery`. Contains information about the requested
    /// section.
    Success(SectionInfo),
    /// Response to `GetSectionQuery` containing addresses of nodes that are closer to the
    /// requested name than the recipient. The request should be repeated to these addresses.
    Redirect(Vec<(XorName, SocketAddr)>),
    /// Request could not be fulfilled due to section constellation updates
    SectionInfoUpdate(Error),
}

impl Message {
    /// Convenience function to deserialize a 'Query' from bytes received over the wire.
    /// It returns an error if the bytes don't correspond to a network info query.
    pub fn from(bytes: Bytes) -> crate::Result<Self> {
        let deserialized = WireMsg::deserialize(bytes)?;
        if let MessageType::SectionInfo { msg, .. } = deserialized {
            Ok(msg)
        } else {
            Err(crate::Error::FailedToParse(
                "bytes as a network info message".to_string(),
            ))
        }
    }

    /// serialize this Query into bytes ready to be sent over the wire.
    pub fn serialize(&self, dest: XorName, dest_section_pk: BlsPublicKey) -> crate::Result<Bytes> {
        WireMsg::serialize_sectioninfo_msg(self, dest, dest_section_pk)
    }
}
