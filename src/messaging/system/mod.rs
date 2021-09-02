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
mod node_msgs;
mod relocation;
mod section;
mod signed;

use crate::messaging::{EndUser, MessageId, SectionAuthorityProvider};
pub use agreement::{DkgFailureSig, DkgFailureSigSet, DkgKey, Proposal, SectionAuth};
use bls::PublicKey as BlsPublicKey;
use bls_dkg::key_gen::message::Message as DkgMessage;
use bytes::Bytes;
pub use join::{JoinRejectionReason, JoinRequest, JoinResponse, ResourceProofResponse};
pub use join_as_relocated::{JoinAsRelocatedRequest, JoinAsRelocatedResponse};
pub use node_msgs::{NodeCmd, NodeQuery, NodeQueryResponse};
pub use relocation::{RelocateDetails, RelocatePayload, RelocatePromise};
pub use section::ElderCandidates;
pub use section::MembershipState;
pub use section::NodeState;
pub use section::Peer;
pub use section::{Section, SectionPeers};
use secured_linked_list::SecuredLinkedList;
use serde::{Deserialize, Serialize};
pub use signed::{KeyedSig, SigShare};
use std::collections::BTreeSet;
use xor_name::XorName;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, custom_debug::Debug)]
#[allow(clippy::large_enum_variant)]
/// Message sent over the among nodes
pub enum SystemMsg {
    /// Message sent to a peer when a message with outdated section
    /// information was received, attaching the bounced message so
    /// the peer can resend it with up to date destination information.
    AntiEntropyRetry {
        /// Current `SectionAuthorityProvider` of the sender's section.
        section_auth: SectionAuthorityProvider,
        /// Sender's section signature over the `SectionAuthorityProvider`.
        section_signed: KeyedSig,
        /// Sender's section chain truncated from the dest section key found in the `bounced_msg`.
        proof_chain: SecuredLinkedList,
        /// Message bounced due to outdated destination section information.
        #[debug(skip)]
        bounced_msg: Bytes,
    },
    /// Message sent to a peer when a message needs to be sent to a different
    /// and/or closest section, attaching the bounced message so the peer can
    /// resend it to the correct section with up to date destination information.
    AntiEntropyRedirect {
        /// Current `SectionAuthorityProvider` of a closest section.
        section_auth: SectionAuthorityProvider,
        /// Section signature over the `SectionAuthorityProvider` of the closest
        /// section the bounced message shall be resent to.
        section_signed: KeyedSig,
        /// Message bounced that shall be resent by the peer.
        #[debug(skip)]
        bounced_msg: Bytes,
    },
    /// Message to update a section when they bounced a message as untrusted back at us.
    /// That section must be behind our current knowledge.
    AntiEntropyUpdate {
        /// Current `SectionAuthorityProvider` of a our section.
        section_auth: SectionAuthorityProvider,
        /// Section signature over the `SectionAuthorityProvider` of our
        /// section the bounced message shall be resent to.
        section_signed: KeyedSig,
        /// Original sender's section chain truncated from the sender's section key found in the triggering msg (or genesis key for full proof)
        proof_chain: SecuredLinkedList,
        /// Optional section members if we're updating our own section
        members: Option<SectionPeers>,
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
        msg: Box<SystemMsg>,
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
        /// TEMP: Add user here as part of return flow. Remove this as we have chunk routing etc
        user: EndUser,
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
