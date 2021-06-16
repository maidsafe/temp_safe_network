// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{agreement::SectionSigned, section::NodeState};
use crate::messaging::SectionAuthorityProvider;
use bls::PublicKey as BlsPublicKey;
use ed25519_dalek::Signature;
use secured_linked_list::SecuredLinkedList;
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    fmt::{self, Debug, Formatter},
    net::SocketAddr,
};

/// Request to join a section
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct JoinRequest {
    /// The public key of the section to join.
    pub section_key: BlsPublicKey,
    /// Proof of the resouce proofing.
    pub resource_proof_response: Option<ResourceProofResponse>,
}

impl Debug for JoinRequest {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter
            .debug_struct("JoinRequest")
            .field("section_key", &self.section_key)
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
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct ResourceProofResponse {
    #[allow(missing_docs)]
    pub solution: u64,
    #[allow(missing_docs)]
    pub data: VecDeque<u8>,
    #[allow(missing_docs)]
    pub nonce: [u8; 32],
    #[allow(missing_docs)]
    pub nonce_signature: Signature,
}

/// Response to a request to join a section
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub enum JoinResponse {
    /// Challenge sent from existing elder nodes to the joining peer for resource proofing.
    ResourceChallenge {
        #[allow(missing_docs)]
        data_size: usize,
        /// how hard the challenge should be to solve
        difficulty: u8,
        #[allow(missing_docs)]
        nonce: [u8; 32],
        #[allow(missing_docs)]
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
        /// Network genesis key (needed to validate) section_chain
        genesis_key: BlsPublicKey,
        /// SectionAuthorityProvider Signed by (current section)
        section_auth: SectionSigned<SectionAuthorityProvider>,
        /// Current node's state
        node_state: SectionSigned<NodeState>,
        /// Full verifiable section chain
        section_chain: SecuredLinkedList,
    },
    /// Join was rejected
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
                node_state,
                section_chain,
            } => f
                .debug_struct("Approval")
                .field("genesis_key", genesis_key)
                .field("section_auth", section_auth)
                .field("node_state", node_state)
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
