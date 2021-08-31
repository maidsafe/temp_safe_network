// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{agreement::SectionAuth, relocation::RelocatePayload, section::NodeState};
use crate::messaging::SectionAuthorityProvider;
use bls::PublicKey as BlsPublicKey;
use secured_linked_list::SecuredLinkedList;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// Request to join a section as relocated from another section
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct JoinAsRelocatedRequest {
    /// The public key of the section to join.
    pub section_key: BlsPublicKey,
    /// The relocation details signed by the previous section.
    pub relocate_payload: Option<RelocatePayload>,
}

/// Response to a request to join a section as relocated
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub enum JoinAsRelocatedResponse {
    /// Up to date section information for a joining peer to retry its join request with
    Retry(SectionAuthorityProvider),
    /// Response redirecting a joining peer to join a different section,
    /// containing the section authority provider of the section that is closer to the
    /// requested name. The `JoinAsRelocatedRequest` should be re-sent to these addresses.
    Redirect(SectionAuthorityProvider),
    /// Message sent to joining peer containing the necessary
    /// info to become a member of the section.
    Approval {
        /// Section Authority over this message for validation
        section_auth: SectionAuth<SectionAuthorityProvider>,
        /// info on current members of the section
        node_state: SectionAuth<NodeState>,
        /// The secured (signed) and verifiable section chain
        section_chain: SecuredLinkedList,
    },
    /// The requesting node is not externally reachable
    NodeNotReachable(SocketAddr),
}
