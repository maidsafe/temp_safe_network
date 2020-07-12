// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use safe_nd::{MsgEnvelope, XorName};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Node internal cmds, about what requests to make.
/// Any network node
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub(crate) enum OutboundMsgChain {
    Single(OutboundMsg),
    Multiple(Vec<OutboundMsg>),
}

/// Any network node
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub(crate) enum OutboundMsg {
    /// Send to a client.
    SendToClient(MsgEnvelope),
    /// Send to a single node.
    SendToNode(MsgEnvelope),
    /// Send to a section.
    SendToSection(MsgEnvelope),
    /// Send the same request to each individual Adult.
    SendToAdults {
        targets: BTreeSet<XorName>,
        msg: MsgEnvelope,
    },
    /// Vote for a cmd so we can process the deferred action on consensus.
    /// (Currently immediately.)
    VoteFor(GroupDecision),
}

// Need to Serialize/Deserialize to go through the consensus process.
/// A GroupDecision is something only
/// taking place at the network Gateways.
#[derive(Debug, Clone, Serialize, Deserialize)] // Debug,
pub(crate) enum GroupDecision {
    /// When Gateway nodes consider a request
    /// valid, they will vote for it to be forwarded.
    /// As they reach consensus, this is then carried out.
    Forward(MsgEnvelope),
}
