// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{
    agreement::{DkgFailureSigned, DkgFailureSignedSet, DkgKey, Proposal, Proven},
    join::{JoinRequest, JoinResponse},
    network::Network,
    relocation::{RelocateDetails, RelocatePromise},
    section::{ElderCandidates, Section},
    signed::SignedShare,
    RoutingMsg,
};
use crate::{DestInfo, SectionAuthorityProvider};
use bls_dkg::key_gen::message::Message as DkgMessage;
use hex_fmt::HexFmt;
use itertools::Itertools;
use secured_linked_list::SecuredLinkedList;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fmt::{self, Debug, Formatter},
};
use threshold_crypto::PublicKey as BlsPublicKey;
use xor_name::XorName;

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
/// Message variant
pub enum Variant {
    /// Inform other sections about our section or vice-versa.
    SectionKnowledge {
        /// `SectionAuthorityProvider` and `SecuredLinkedList` of the sender's section, with the proof chain.
        src_info: (Proven<SectionAuthorityProvider>, SecuredLinkedList),
        /// Message
        msg: Option<Box<RoutingMsg>>,
    },
    /// User-facing message
    #[serde(with = "serde_bytes")]
    UserMessage(Vec<u8>),
    /// Message sent to all members to update them about the state of our section.
    Sync {
        // Information about our section.
        section: Section,
        // Information about the rest of the network that we know of.
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
    /// Sent from a node that can't establish the trust of the contained message to its original
    /// source in order for them to provide new proof that the node would trust.
    BouncedUntrustedMessage {
        msg: Box<RoutingMsg>,
        dest_info: DestInfo,
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
    /// Broadcasted to the other DKG participants when a DKG failure is observed.
    DkgFailureObservation {
        dkg_key: DkgKey,
        signed: DkgFailureSigned,
        non_participants: BTreeSet<XorName>,
    },
    /// Sent to the current elders by the DKG participants when at least majority of them observe
    /// a DKG failure.
    DkgFailureAgreement(DkgFailureSignedSet),
    /// Message containing a single `Proposal` to be aggregated in the proposal aggregator.
    Propose {
        content: Proposal,
        signed_share: SignedShare,
    },
    /// Message sent by a node to indicate it received a message from a node which was ahead in knowledge.
    /// A reply is expected with a `SectionKnowledge` message.
    SectionKnowledgeQuery {
        last_known_key: Option<BlsPublicKey>,
        msg: Box<RoutingMsg>,
    },
}

impl Debug for Variant {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::SectionKnowledge { .. } => f.debug_struct("SectionKnowledge").finish(),
            Self::UserMessage(payload) => write!(f, "UserMessage({:10})", HexFmt(payload)),
            Self::Sync { section, network } => f
                .debug_struct("Sync")
                .field("section_auth", &section.section_auth.value)
                .field("section_key", section.chain.last_key())
                .field(
                    "other_prefixes",
                    &format_args!(
                        "({:b})",
                        network
                            .sections
                            .iter()
                            .map(|info| &info.section_auth.value.prefix)
                            .format(", ")
                    ),
                )
                .finish(),
            Self::Relocate(payload) => write!(f, "Relocate({:?})", payload),
            Self::RelocatePromise(payload) => write!(f, "RelocatePromise({:?})", payload),
            Self::JoinRequest(payload) => write!(f, "JoinRequest({:?})", payload),
            Self::JoinResponse(response) => write!(f, "JoinResponse({:?})", response),
            Self::BouncedUntrustedMessage { msg, dest_info } => f
                .debug_struct("BouncedUntrustedMessage")
                .field("message", msg)
                .field("dest_info", dest_info)
                .finish(),
            Self::DkgStart {
                dkg_key,
                elder_candidates,
            } => f
                .debug_struct("DkgStart")
                .field("dkg_key", dkg_key)
                .field("elder_candidates", elder_candidates)
                .finish(),
            Self::DkgMessage { dkg_key, message } => f
                .debug_struct("DkgMessage")
                .field("dkg_key", &dkg_key)
                .field("message", message)
                .finish(),
            Self::DkgFailureObservation {
                dkg_key,
                signed,
                non_participants,
            } => f
                .debug_struct("DkgFailureObservation")
                .field("dkg_key", dkg_key)
                .field("signed", signed)
                .field("non_participants", non_participants)
                .finish(),
            Self::DkgFailureAgreement(proofs) => {
                f.debug_tuple("DkgFailureAgreement").field(proofs).finish()
            }
            Self::Propose {
                content,
                signed_share,
            } => f
                .debug_struct("Propose")
                .field("content", content)
                .field("signed_share", signed_share)
                .finish(),
            Self::SectionKnowledgeQuery { .. } => write!(f, "SectionKnowledgeQuery"),
        }
    }
}
