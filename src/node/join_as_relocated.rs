// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{agreement::SectionSigned, relocation::RelocatePayload, section::NodeState};
use crate::SectionAuthorityProvider;
use secured_linked_list::SecuredLinkedList;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Debug, Formatter},
    net::SocketAddr,
};
use threshold_crypto::PublicKey as BlsPublicKey;

/// Request to join a section as relocated from another section
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct JoinAsRelocatedRequest {
    /// The public key of the section to join.
    pub section_key: BlsPublicKey,
    /// The relocation details signed by the previous section.
    pub relocate_payload: Option<RelocatePayload>,
}

impl Debug for JoinAsRelocatedRequest {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter
            .debug_struct("JoinAsRelocatedRequest")
            .field("section_key", &self.section_key)
            .field(
                "relocate_payload",
                &self
                    .relocate_payload
                    .as_ref()
                    .map(|payload| &payload.details),
            )
            .finish()
    }
}

/// Response to a request to join a section as relocated
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
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
        section_auth: SectionSigned<SectionAuthorityProvider>,
        member_info: SectionSigned<NodeState>,
        section_chain: SecuredLinkedList,
    },
    /// The requesting node is not externally reachable
    NodeNotReachable(SocketAddr),
}

impl Debug for JoinAsRelocatedResponse {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Retry(section_auth) => write!(f, "Retry({:?})", section_auth),
            Self::Redirect(section_auth) => write!(f, "Redirect({:?})", section_auth),
            Self::Approval {
                section_auth,
                member_info,
                section_chain,
            } => f
                .debug_struct("Approval")
                .field("section_auth", section_auth)
                .field("member_info", member_info)
                .field("section_chain", section_chain)
                .finish(),
            Self::NodeNotReachable(addr) => write!(f, "NodeNotReachable({})", addr),
        }
    }
}
