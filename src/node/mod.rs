// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

mod agreement;
mod network;
mod node_msg;
mod plain_message;
mod prefix_map;
mod relocation;
mod section;
mod src_authority;
mod variant;

pub use agreement::{DkgFailureProof, DkgFailureProofSet, DkgKey, Proposal, Proven};
pub use network::{Network, OtherSection};
pub use node_msg::{
    NodeCmd, NodeCmdError, NodeDataError, NodeDataQueryResponse, NodeEvent, NodeMsg, NodeQuery,
    NodeQueryResponse, NodeRewardQuery, NodeSystemCmd, NodeSystemQuery, NodeSystemQueryResponse,
    NodeTransferCmd, NodeTransferError, NodeTransferQuery, NodeTransferQueryResponse,
};
pub use plain_message::PlainMessage;
pub use prefix_map::PrefixMap;
pub use relocation::{RelocateDetails, RelocatePayload, RelocatePromise, SignedRelocateDetails};
pub use section::{
    ElderCandidates, MemberInfo, Peer, PeerState, Section, SectionAuthorityProvider, SectionPeers,
};
pub use src_authority::SrcAuthority;
pub use variant::{JoinRequest, ResourceProofResponse, Variant};

use crate::{Aggregation, DstLocation, MessageId, MessageType, WireMsg};
use bytes::Bytes;
use secured_linked_list::SecuredLinkedList;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug, Formatter};
use threshold_crypto::PublicKey as BlsPublicKey;
use xor_name::XorName;

/// Routing message sent over the network.
#[derive(Clone, Eq, Serialize, Deserialize)]
pub struct RoutingMsg {
    /// Message ID.
    pub id: MessageId,
    /// Source authority.
    /// Messages do not need to sign this field as it is all verifiable (i.e. if the sig validates
    /// agains the public key and we know the pub key then we are good. If the proof is not recognised we
    /// ask for a longer chain that can be recognised). Therefore we don't need to sign this field.
    pub src: SrcAuthority,
    /// Destination location.
    pub dst: DstLocation,
    /// The aggregation scheme to be used.
    pub aggregation: Aggregation,
    /// The body of the message.
    pub variant: Variant,
    /// Proof chain to verify the message trust. Does not need to be signed.
    pub proof_chain: Option<SecuredLinkedList>,
}

impl RoutingMsg {
    /// Gets the message ID.
    pub fn id(&self) -> MessageId {
        self.id
    }

    /// Convenience function to deserialize a 'RoutingMsg' from bytes received over the wire.
    /// It returns an error if the bytes don't correspond to a node message.
    pub fn from(bytes: Bytes) -> crate::Result<Self> {
        let deserialized = WireMsg::deserialize(bytes)?;
        if let MessageType::Routing { msg, .. } = deserialized {
            Ok(msg)
        } else {
            Err(crate::Error::FailedToParse(
                "bytes as a node message".to_string(),
            ))
        }
    }

    /// serialize this RoutingMsg into bytes ready to be sent over the wire.
    pub fn serialize(&self, dest: XorName, dest_section_pk: BlsPublicKey) -> crate::Result<Bytes> {
        WireMsg::serialize_routing_msg(self, dest, dest_section_pk)
    }
}

impl PartialEq for RoutingMsg {
    fn eq(&self, other: &RoutingMsg) -> bool {
        self.src == other.src
            && self.dst == other.dst
            && self.id == other.id
            && self.variant == other.variant
            && self.proof_chain == other.proof_chain
    }
}

impl Debug for RoutingMsg {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter
            .debug_struct("RoutingMsg")
            .field("id", &self.id)
            .field("src", &self.src)
            .field("dst", &self.dst)
            .field("variant", &self.variant)
            .finish()
    }
}
