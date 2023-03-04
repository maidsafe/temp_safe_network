// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::NodeState;
use crate::network_knowledge::RelocationProof;

use sn_consensus::Decision;

use serde::{Deserialize, Serialize};
use std::{fmt, net::SocketAddr};

/// Details of a joining node, included when joining a section.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum JoinDetails {
    /// New node with its reward key.
    New(bls::PublicKey),
    /// Relocating node with the proof that it came from another section.
    Relocation(RelocationProof),
}

/// Response to a request to join a section
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum JoinResponse {
    /// Message sent to joining node containing the current node's
    /// state as a member of the section.
    Approved(Decision<NodeState>),
    /// Join was rejected
    Rejected(JoinRejectReason),
    /// Join is being considered
    UnderConsideration,
}

/// Reason of a join request being rejected
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum JoinRejectReason {
    /// No new nodes are currently accepted for joining
    /// NB: Relocated nodes that try to join, are accepted even if joins are disallowed.
    JoinsDisallowed,
    /// The requesting node is not externally reachable
    NodeNotReachable(SocketAddr),
}

impl fmt::Display for JoinRejectReason {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
