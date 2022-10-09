// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod agreement;
mod join;
mod join_as_relocated;
mod node_msgs;
mod node_state;
mod section_sig;

use crate::messaging::{AuthorityProof, EndUser, MsgId, SectionTreeUpdate};
use crate::network_knowledge::SapCandidate;
pub use agreement::{DkgSessionId, Proposal, SectionSigned};
pub use join::{JoinRejectionReason, JoinRequest, JoinResponse};
pub use join_as_relocated::{JoinAsRelocatedRequest, JoinAsRelocatedResponse};
pub use node_msgs::{NodeCmd, NodeEvent, NodeQuery, NodeQueryResponse};
pub use node_state::{MembershipState, NodeState, RelocateDetails};
pub use section_sig::{SectionSig, SectionSigShare};

use bls::PublicKey as BlsPublicKey;
use ed25519::Signature;
use qp2p::UsrMsgBytes;
use serde::{Deserialize, Serialize};
use sn_consensus::{Generation, SignedVote};
use sn_sdkg::DkgSignedVote;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};
use xor_name::XorName;

/// List of peers of a section
pub type SectionPeers = BTreeSet<SectionSigned<NodeState>>;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum AntiEntropyKind {
    /// This AE message is sent to a peer when a message with outdated section
    /// information was received, attaching the bounced message so
    /// the peer can resend it with up to date destination information.
    Retry { bounced_msg: UsrMsgBytes },
    /// This AE message is sent to a peer when a message needs to be sent to a
    /// different and/or closest section, attaching the bounced message so the peer
    /// can resend it to the correct section with up to date destination information.
    Redirect { bounced_msg: UsrMsgBytes },
    /// This AE message is sent to update a peer when we notice they are behind
    Update { members: SectionPeers },
}

#[derive(Clone, PartialEq, Serialize, Deserialize, custom_debug::Debug)]
#[allow(clippy::large_enum_variant, clippy::derive_partial_eq_without_eq)]
/// Message sent over the among nodes
pub enum NodeMsg {
    AntiEntropy {
        /// The update to our NetworkKnowledge containing the current `SectionAuthorityProvider`
        /// and the section chain truncated from the triggering msg's dst section_key or genesis_key
        /// if the the dst section_key is not a direct ancestor to our section_key
        section_tree_update: SectionTreeUpdate,
        /// The kind of anti-entropy response.
        kind: AntiEntropyKind,
    },
    /// Probes the network by sending a message to a random or chosen dst triggering an AE flow.
    /// Sends the current section key of target section which we know
    /// This expects a response, even if we're up to date.
    AntiEntropyProbe(BlsPublicKey),
    /// Send from a section to the node to be immediately relocated.
    Relocate(SectionSigned<NodeState>),
    /// Membership Votes, in order they should be processed in.
    MembershipVotes(Vec<SignedVote<NodeState>>),
    /// Membership Anti-Entropy request
    MembershipAE(Generation),
    /// Sent from a bootstrapping peer to the section requesting to join as a new member
    JoinRequest(JoinRequest),
    /// Response to a `JoinRequest`
    JoinResponse(Box<JoinResponse>),
    /// Sent from a peer to the section requesting to join as relocated from another section
    JoinAsRelocatedRequest(Box<JoinAsRelocatedRequest>),
    /// Response to a `JoinAsRelocatedRequest`
    JoinAsRelocatedResponse(Box<JoinAsRelocatedResponse>),
    /// Sent to the new elder candidates to start the DKG process.
    DkgStart(DkgSessionId),
    /// Sent when DKG is triggered to other participant
    DkgEphemeralPubKey {
        /// The identifier of the DKG session this message is for.
        session_id: DkgSessionId,
        /// Section authority for the DKG start message
        section_auth: AuthorityProof<SectionSig>,
        /// The ephemeral bls key chosen by candidate
        pub_key: BlsPublicKey,
        /// The ed25519 signature of the candidate
        sig: Signature,
    },
    /// Votes exchanged for DKG process.
    DkgVotes {
        /// The identifier of the DKG session this message is for.
        session_id: DkgSessionId,
        /// The ephemeral bls public keys used for this Dkg round
        pub_keys: BTreeMap<XorName, (BlsPublicKey, Signature)>,
        /// The DKG message.
        votes: Vec<DkgSignedVote>,
    },
    /// Dkg Anti-Entropy request when receiving votes that are ahead of our knowledge
    DkgAE(DkgSessionId),
    /// Section handover consensus vote message
    HandoverVotes(Vec<SignedVote<SapCandidate>>),
    /// Handover Anti-Entropy request
    HandoverAE(Generation),
    /// Message containing a single `Proposal` to be aggregated in the proposal aggregator.
    Propose {
        /// The content of the proposal
        proposal: Proposal,
        /// BLS signature share of an Elder
        sig_share: SectionSigShare,
    },
    /// Events are facts about something that happened on a node.
    NodeEvent(NodeEvent),
    #[cfg(any(feature = "chunks", feature = "registers"))]
    /// Cmds are orders to perform some operation, only sent internally in the network.
    NodeCmd(NodeCmd),
    #[cfg(any(feature = "chunks", feature = "registers"))]
    /// Queries is a read-only operation.
    NodeQuery(NodeQuery),
    #[cfg(any(feature = "chunks", feature = "registers"))]
    /// The response to a query, containing the query result.
    NodeQueryResponse {
        /// QueryResponse.
        response: NodeQueryResponse,
        /// ID of causing query.
        correlation_id: MsgId,
        /// TEMP: Add user here as part of return flow. Remove this as we have chunk routing etc
        user: EndUser,
    },
}

impl NodeMsg {
    /// The priority of the message, when handled by lower level comms.
    pub fn priority(&self) -> i32 {
        use super::msg_type::{
            ANTIENTROPY_MSG_PRIORITY, DKG_MSG_PRIORITY, JOIN_RELOCATE_MSG_PRIORITY,
            JOIN_RESPONSE_PRIORITY, MEMBERSHIP_PRIORITY, NODE_DATA_MSG_PRIORITY,
        };
        match self {
            // DKG messages
            Self::DkgStart { .. }
            | Self::DkgEphemeralPubKey { .. }
            | Self::DkgVotes { .. }
            | Self::DkgAE { .. } => DKG_MSG_PRIORITY,

            // Inter-node comms for AE updates
            Self::AntiEntropy { .. } | Self::AntiEntropyProbe(_) => ANTIENTROPY_MSG_PRIORITY,

            // Join responses
            Self::JoinResponse(_) | Self::JoinAsRelocatedResponse(_) => JOIN_RESPONSE_PRIORITY,

            Self::Propose { .. }
            | Self::MembershipVotes(_)
            | Self::MembershipAE(_)
            | Self::HandoverAE(_)
            | Self::HandoverVotes(_) => MEMBERSHIP_PRIORITY,

            // Inter-node comms for joining, relocating etc.
            Self::Relocate(_) | Self::JoinRequest(_) | Self::JoinAsRelocatedRequest(_) => {
                JOIN_RELOCATE_MSG_PRIORITY
            }

            #[cfg(any(feature = "chunks", feature = "registers"))]
            // Inter-node comms related to processing client requests
            Self::NodeCmd(_)
            | Self::NodeEvent(NodeEvent::CouldNotStoreData { .. })
            | Self::NodeQuery(_)
            | Self::NodeQueryResponse { .. } => NODE_DATA_MSG_PRIORITY,
        }
    }

    pub fn statemap_states(&self) -> crate::statemap::State {
        use crate::statemap::State;
        match self {
            Self::AntiEntropy { .. } => State::AntiEntropy,
            Self::AntiEntropyProbe { .. } => State::AntiEntropy,
            Self::Relocate(_) => State::Relocate,
            Self::MembershipAE(_) => State::Membership,
            Self::MembershipVotes(_) => State::Membership,
            Self::JoinRequest(_) => State::Join,
            Self::JoinResponse(_) => State::Join,
            Self::JoinAsRelocatedRequest(_) => State::Join,
            Self::JoinAsRelocatedResponse(_) => State::Join,
            Self::DkgStart { .. } => State::Dkg,
            Self::DkgEphemeralPubKey { .. } => State::Dkg,
            Self::DkgVotes { .. } => State::Dkg,
            Self::DkgAE { .. } => State::Dkg,
            Self::HandoverVotes(_) => State::Handover,
            Self::HandoverAE(_) => State::Handover,
            Self::Propose { .. } => State::Propose,
            Self::NodeEvent(_) => State::Node,
            Self::NodeCmd(_) => State::Node,
            Self::NodeQuery(_) => State::Node,
            Self::NodeQueryResponse { .. } => State::Node,
        }
    }
}

impl Display for NodeMsg {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AntiEntropy { .. } => write!(f, "NodeMsg::AntiEntropy"),
            Self::AntiEntropyProbe { .. } => write!(f, "NodeMsg::AntiEntropyProbe"),
            Self::Relocate { .. } => write!(f, "NodeMsg::Relocate"),
            Self::MembershipVotes { .. } => write!(f, "NodeMsg::MembershipVotes"),
            Self::MembershipAE { .. } => write!(f, "NodeMsg::MembershipAE"),
            Self::JoinRequest { .. } => write!(f, "NodeMsg::JoinRequest"),
            Self::JoinResponse { .. } => write!(f, "NodeMsg::JoinResponse"),
            Self::JoinAsRelocatedRequest { .. } => {
                write!(f, "NodeMsg::JoinAsRelocatedRequest")
            }
            Self::JoinAsRelocatedResponse { .. } => {
                write!(f, "NodeMsg::JoinAsRelocatedResponse")
            }
            Self::DkgStart { .. } => write!(f, "NodeMsg::DkgStart"),
            Self::DkgEphemeralPubKey { .. } => write!(f, "NodeMsg::DkgEphemeralPubKey"),
            Self::DkgVotes { .. } => write!(f, "NodeMsg::DkgVotes"),
            Self::DkgAE { .. } => write!(f, "NodeMsg::DkgAE"),
            Self::HandoverVotes { .. } => write!(f, "NodeMsg::HandoverVotes"),
            Self::HandoverAE { .. } => write!(f, "NodeMsg::HandoverAE"),
            Self::Propose { .. } => write!(f, "NodeMsg::Propose"),
            Self::NodeEvent { .. } => write!(f, "NodeMsg::NodeEvent"),
            #[cfg(any(feature = "chunks", feature = "registers"))]
            Self::NodeCmd { .. } => write!(f, "NodeMsg::NodeCmd"),
            #[cfg(any(feature = "chunks", feature = "registers"))]
            Self::NodeQuery { .. } => write!(f, "NodeMsg::NodeQuery"),
            #[cfg(any(feature = "chunks", feature = "registers"))]
            Self::NodeQueryResponse { .. } => write!(f, "NodeMsg::NodeQueryResponse"),
        }
    }
}
