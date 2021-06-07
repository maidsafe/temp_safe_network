// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{
    agreement::Proven,
    relocation::RelocatePayload,
    section::{MemberInfo, SectionAuthorityProvider},
};
use ed25519_dalek::Signature;
use secured_linked_list::SecuredLinkedList;
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    fmt::{self, Debug, Formatter},
    net::SocketAddr,
};
use threshold_crypto::PublicKey as BlsPublicKey;

/// Request to join a section
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct JoinRequest {
    /// The public key of the section to join.
    pub section_key: BlsPublicKey,
    /// If the peer is being relocated, contains `RelocatePayload`. Otherwise contains `None`.
    pub relocate_payload: Option<RelocatePayload>,
    /// Proof of the resouce proofing.
    pub resource_proof_response: Option<ResourceProofResponse>,
}

impl Debug for JoinRequest {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter
            .debug_struct("JoinRequest")
            .field("section_key", &self.section_key)
            .field(
                "relocate_payload",
                &self
                    .relocate_payload
                    .as_ref()
                    .map(|payload| &payload.details),
            )
            .field(
                "resource_proof_response",
                &self
                    .resource_proof_response
                    .as_ref()
                    .map(|proof| proof.solution),
            )
            .finish()
    }
}

/// Joining peer's proof of resolvement of given resource proofing challenge.
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ResourceProofResponse {
    pub solution: u64,
    pub data: VecDeque<u8>,
    pub nonce: [u8; 32],
    pub nonce_signature: Signature,
}

/// Response to a request to join a section
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum JoinResponse {
    /// Challenge sent from existing elder nodes to the joining peer for resource proofing.
    ResourceChallenge {
        data_size: usize,
        difficulty: u8,
        nonce: [u8; 32],
        nonce_signature: Signature,
    },
    /// Up to date section information for a joining peer to retry its join request with
    Retry(SectionAuthorityProvider),
    /// Response redirecting a joining peer to join a different section,
    /// containing addresses of nodes that are closer (than the recipient) to the
    /// requested name. The `JoinRequest` should be re-sent to these addresses.
    Redirect(SectionAuthorityProvider),
    /// Message sent to joining peer containing the necessary
    /// info to become a member of the section.
    Approval {
        genesis_key: BlsPublicKey,
        section_auth: Proven<SectionAuthorityProvider>,
        member_info: Proven<MemberInfo>,
        section_chain: SecuredLinkedList,
    },
    Rejected(JoinRejectionReason),
}

impl Debug for JoinResponse {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::ResourceChallenge {
                data_size,
                difficulty,
                ..
            } => f
                .debug_struct("ResourceChallenge")
                .field("data_size", data_size)
                .field("difficulty", difficulty)
                .finish(),
            Self::Retry(section_auth) => write!(f, "Retry({:?})", section_auth),
            Self::Redirect(section_auth) => write!(f, "Redirect({:?})", section_auth),
            Self::Approval {
                genesis_key,
                section_auth,
                member_info,
                section_chain,
            } => f
                .debug_struct("Approval")
                .field("genesis_key", genesis_key)
                .field("section_auth", section_auth)
                .field("member_info", member_info)
                .field("section_chain", section_chain)
                .finish(),
            Self::Rejected(reason) => write!(f, "Rejected({:?})", reason),
        }
    }
}

/// Reason of a join request being rejected
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum JoinRejectionReason {
    /// No new peers are currently accepted for joining
    JoinsDisallowed,
    /// The requesting node is not externally reachable
    NodeNotReachable(SocketAddr),
}
