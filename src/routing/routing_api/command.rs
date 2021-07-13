// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::{
    node::{DkgFailureSigSet, KeyedSig, Proposal, Section},
    SectionAuthorityProvider, WireMsg,
};
use crate::routing::{node::Node, routing_api::Peer, section::SectionKeyShare, XorName};
use std::{
    fmt::{self, Debug, Formatter},
    net::SocketAddr,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

/// Command for node.
#[allow(clippy::large_enum_variant)]
pub(crate) enum Command {
    /// Handle `message` from `sender`.
    /// holding the WireMsg that has been received from the network,
    HandleMessage {
        sender: SocketAddr,
        wire_msg: WireMsg,
    },
    /// Handle a timeout previously scheduled with `ScheduleTimeout`.
    HandleTimeout(u64),
    /// Handle lost connection to a peer.
    HandleConnectionLost(SocketAddr),
    /// Handle peer that's been detected as lost.
    HandlePeerLost(SocketAddr),
    /// Handle agreement on a proposal.
    HandleAgreement { proposal: Proposal, sig: KeyedSig },
    /// Handle the outcome of a DKG session where we are one of the participants (that is, one of
    /// the proposed new elders).
    HandleDkgOutcome {
        section_auth: SectionAuthorityProvider,
        outcome: SectionKeyShare,
    },
    /// Handle a DKG failure that was observed by a majority of the DKG participants.
    HandleDkgFailure(DkgFailureSigSet),
    /// Send a message to `delivery_group_size` peers out of the given `recipients`.
    SendMessage {
        recipients: Vec<(XorName, SocketAddr)>,
        delivery_group_size: usize,
        wire_msg: WireMsg,
    },
    /// Relay a message according to its destination.
    RelayMessage(WireMsg),
    /// Schedule a timeout after the given duration. When the timeout expires, a `HandleTimeout`
    /// command is raised. The token is used to identify the timeout.
    ScheduleTimeout { duration: Duration, token: u64 },
    /// Relocation process is complete, switch to new section
    HandleRelocationComplete {
        /// New Node state and information
        node: Node,
        /// New section where we relocated
        section: Section,
    },
    /// Attempt to set JoinsAllowed flag.
    SetJoinsAllowed(bool),
    /// Test peer's connectivity
    ProposeOnline {
        peer: Peer,
        // Previous name if relocated.
        previous_name: Option<XorName>,
        // The key of the destination section that the joining node knows, if any.
        dst_key: Option<bls::PublicKey>,
    },
    /// Proposes a peer as offline
    ProposeOffline(XorName),
    /// Send a signal to all Elders to
    /// test the connectivity to a specific node
    StartConnectivityTest(XorName),
    /// Test Connectivity
    TestConnectivity(XorName),
}

impl Debug for Command {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::HandleMessage { sender, wire_msg } => f
                .debug_struct("HandleMessage")
                .field("sender", sender)
                .field("wire_msg", wire_msg)
                .finish(),
            Self::HandleTimeout(token) => f.debug_tuple("HandleTimeout").field(token).finish(),
            Self::HandleConnectionLost(addr) => {
                f.debug_tuple("HandleConnectionLost").field(addr).finish()
            }
            Self::HandlePeerLost(addr) => f.debug_tuple("HandlePeerLost").field(addr).finish(),
            Self::HandleAgreement { proposal, sig } => f
                .debug_struct("HandleAgreement")
                .field("proposal", proposal)
                .field("sig.public_key", &sig.public_key)
                .finish(),
            Self::HandleDkgOutcome {
                section_auth,
                outcome,
            } => f
                .debug_struct("HandleDkgOutcome")
                .field("section_auth", section_auth)
                .field("outcome", &outcome.public_key_set.public_key())
                .finish(),
            Self::HandleDkgFailure(signeds) => {
                f.debug_tuple("HandleDkgFailure").field(signeds).finish()
            }
            Self::SendMessage {
                recipients,
                delivery_group_size,
                wire_msg,
            } => f
                .debug_struct("SendMessage")
                .field("recipients", recipients)
                .field("delivery_group_size", delivery_group_size)
                .field("wire_msg", wire_msg)
                .finish(),
            Self::RelayMessage(wire_msg) => f.debug_tuple("RelayMessage").field(wire_msg).finish(),
            Self::ScheduleTimeout { duration, token } => f
                .debug_struct("ScheduleTimeout")
                .field("duration", duration)
                .field("token", token)
                .finish(),
            Self::HandleRelocationComplete { node, section } => f
                .debug_struct("HandleRelocationComplete")
                .field("node", node)
                .field("section", section)
                .finish(),
            Self::SetJoinsAllowed(joins_allowed) => f
                .debug_tuple("SetJoinsAllowed")
                .field(joins_allowed)
                .finish(),
            Self::ProposeOnline {
                peer,
                previous_name,
                ..
            } => f
                .debug_struct("ProposeOnline")
                .field("peer", peer)
                .field("previous_name", previous_name)
                .finish(),
            Self::ProposeOffline(name) => f.debug_tuple("ProposeOffline").field(name).finish(),
            Self::TestConnectivity(name) => f.debug_tuple("TestConnectivity").field(name).finish(),
            Self::StartConnectivityTest(name) => {
                f.debug_tuple("StartConnectivityTest").field(name).finish()
            }
        }
    }
}

/// Generate unique timer token.
pub(crate) fn next_timer_token() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    NEXT.fetch_add(1, Ordering::Relaxed)
}
