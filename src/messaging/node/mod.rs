// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod agreement;
mod join;
mod join_as_relocated;
mod network;
mod node_msgs;
mod relocation;
mod section;
mod signed;

pub use agreement::{DkgFailureSig, DkgFailureSigSet, DkgKey, Proposal, SectionAuth};
pub use join::{JoinRejectionReason, JoinRequest, JoinResponse, ResourceProofResponse};
pub use join_as_relocated::{JoinAsRelocatedRequest, JoinAsRelocatedResponse};
pub use network::{Network, OtherSection};
pub use node_msgs::{NodeCmd, NodeQuery, NodeQueryResponse};
pub use relocation::{RelocateDetails, RelocatePayload, RelocatePromise};
pub use section::ElderCandidates;
pub use section::MembershipState;
pub use section::NodeState;
pub use section::Peer;
pub use section::{Section, SectionPeers};
pub use signed::{KeyedSig, SigShare};

use crate::messaging::{
    data::ServiceMsg, EndUser, MessageId, SectionAuthorityProvider, ServiceAuth,
};
use bls::PublicKey as BlsPublicKey;
use bls_dkg::key_gen::message::Message as DkgMessage;
use itertools::Itertools;
use secured_linked_list::SecuredLinkedList;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, fmt};
use xor_name::XorName;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, custom_debug::Debug)]
#[allow(clippy::large_enum_variant)]
/// Message sent over the among nodes
pub enum NodeMsg {
    /// Forward a data message.
    ForwardServiceMsg {
        /// The msg
        msg: ServiceMsg,
        /// The origin
        user: EndUser,
        /// Signature provided by the requester.
        auth: ServiceAuth,
    },
    /// Inform other sections about our section or vice-versa.
    SectionKnowledge {
        /// `SectionAuthorityProvider` and `SecuredLinkedList` of the sender's section, with the proof chain.
        src_info: (SectionAuth<SectionAuthorityProvider>, SecuredLinkedList),
        /// Message
        msg: Option<Box<NodeMsg>>,
    },
    /// Message sent to all members to update them about the state of our section.
    Sync {
        /// Information about our section.
        #[debug(with = "fmt_sync_section")]
        section: Section,
        /// Information about the rest of the network that we know of.
        #[debug(with = "fmt_sync_network")]
        network: Network,
    },
    /// Send from a section to the node to be immediately relocated.
    Relocate(RelocateDetails),
    /// Send:
    /// - from a section to a current elder to be relocated after they are demoted.
    /// - from the node to be relocated back to its section after it was demoted.
    RelocatePromise(RelocatePromise),
    /// Sent from a bootstrapping peer to the section requesting to join as a new member
    JoinRequest(Box<JoinRequest>),
    /// Response to a `JoinRequest`
    JoinResponse(Box<JoinResponse>),
    /// Sent from a peer to the section requesting to join as relocated from another section
    JoinAsRelocatedRequest(Box<JoinAsRelocatedRequest>),
    /// Response to a `JoinAsRelocatedRequest`
    JoinAsRelocatedResponse(Box<JoinAsRelocatedResponse>),
    /// Sent from a node that can't establish the trust of the contained message to its original
    /// source in order for them to provide new proof that the node would trust.
    BouncedUntrustedMessage {
        /// Untrsuted Node message
        msg: Box<NodeMsg>,
        /// Currently known section pk of the source
        dst_section_pk: BlsPublicKey,
    },
    /// Sent to the new elder candidates to start the DKG process.
    DkgStart {
        /// The identifier of the DKG session to start.
        dkg_key: DkgKey,
        /// The DKG particpants.
        elder_candidates: ElderCandidates,
    },
    /// Message exchanged for DKG process.
    DkgMessage {
        /// The identifier of the DKG session this message is for.
        dkg_key: DkgKey,
        /// The DKG message.
        message: DkgMessage,
    },
    /// Broadcast to the other DKG participants when a DKG failure is observed.
    DkgFailureObservation {
        /// The DKG key
        dkg_key: DkgKey,
        /// Signature over the failure
        sig: DkgFailureSig,
        /// Nodes that failed to participate
        failed_participants: BTreeSet<XorName>,
    },
    /// Sent to the current elders by the DKG participants when at least majority of them observe
    /// a DKG failure.
    DkgFailureAgreement(DkgFailureSigSet),
    /// Message containing a single `Proposal` to be aggregated in the proposal aggregator.
    Propose {
        /// The content of the proposal
        content: Proposal,
        /// BLS signature share
        sig_share: SigShare,
    },
    /// Message that notifies a section to test
    /// the connectivity to a node
    StartConnectivityTest(XorName),
    /// Message sent by a node to indicate it received a message from a node which was ahead in knowledge.
    /// A reply is expected with a `SectionKnowledge` message.
    SectionKnowledgeQuery {
        /// Last known key by our node, used to get any newer keys
        last_known_key: Option<BlsPublicKey>,
        /// Routing message
        msg: Box<NodeMsg>,
    },
    /// Cmds only sent internally in the network.
    NodeCmd(NodeCmd),
    /// Queries is a read-only operation.
    NodeQuery(NodeQuery),
    /// The response to a query, containing the query result.
    NodeQueryResponse {
        /// QueryResponse.
        response: NodeQueryResponse,
        /// ID of causing query.
        correlation_id: MessageId,
    },
    /// The returned error, from any msg handling on recipient node.
    NodeMsgError {
        /// The error.
        // TODO: return node::Error instead
        error: crate::messaging::data::Error,
        /// ID of causing cmd.
        correlation_id: MessageId,
    },
}

fn fmt_sync_section(section: &Section, f: &mut fmt::Formatter) -> fmt::Result {
    f.debug_struct("Section")
        .field("section_auth", &section.section_auth.value)
        .field("key", &section.chain.last_key())
        .finish()
}

fn fmt_sync_network(network: &Network, f: &mut fmt::Formatter) -> fmt::Result {
    f.debug_struct("Network")
        .field(
            "sections",
            &format_args!(
                "({:b})",
                network
                    .sections
                    .iter()
                    .map(|info| info.section_auth.value.prefix)
                    .format(", ")
            ),
        )
        .finish()
}
