// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::NodeState;
use crate::messaging::{SectionAuthorityProvider, SectionTreeUpdate};
use serde::{Deserialize, Serialize};
use sn_consensus::Decision;
use std::net::SocketAddr;

/// Response to a request to join a section
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub struct JoinRequest {
    /// The public key of the section to join.
    pub section_key: bls::PublicKey,
}

impl JoinRequest {
    pub fn section_key(&self) -> &bls::PublicKey {
        &self.section_key
    }

    pub fn set_section_key(&mut self, section_key: bls::PublicKey) {
        self.section_key = section_key;
    }
}

/// Response to a request to join a section
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub enum JoinResponse {
    /// Up to date section information for a joining peer to retry its join request with
    Retry {
        /// The update to our NetworkKnowledge containing the current `SectionAuthorityProvider`
        /// and the section chain truncated from the section key found in the join request.
        section_tree_update: SectionTreeUpdate,
        /// The age of the node as expected by the section.
        expected_age: u8,
    },
    /// Response redirecting a joining peer to join a different section,
    /// containing addresses of nodes that are closer (than the recipient) to the
    /// requested name. The `JoinRequest` should be re-sent to these addresses.
    Redirect(SectionAuthorityProvider),
    /// Message sent to joining peer containing the necessary
    /// info to become a member of the section.
    Approved {
        /// The update to our NetworkKnowledge containing the current `SectionAuthorityProvider`
        /// and a fully verifiable section chain
        section_tree_update: SectionTreeUpdate,
        /// Current node's state
        decision: Decision<NodeState>,
    },
    /// Join was rejected
    Rejected(JoinRejectionReason),
}

/// Reason of a join request being rejected
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum JoinRejectionReason {
    /// No new peers are currently accepted for joining
    JoinsDisallowed,
    /// The requesting node is not externally reachable
    NodeNotReachable(SocketAddr),
}
