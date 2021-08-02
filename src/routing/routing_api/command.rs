// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::{
    data::ServiceMsg,
    node::{DkgFailureSigSet, KeyedSig, Proposal, Section},
    AuthorityProof, EndUser, MessageId, SectionAuthorityProvider, ServiceAuth, WireMsg,
};
use crate::routing::{node::Node, routing_api::Peer, section::SectionKeyShare, XorName};

use std::{
    net::SocketAddr,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

/// Command for node.
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub(crate) enum Command {
    /// Handle `message` from `sender`.
    /// holding the WireMsg that has been received from the network,
    HandleMessage {
        sender: SocketAddr,
        wire_msg: WireMsg,
    },
    /// Handle ServiceMsg, either directly or notify via event listener
    HandleServiceMessage {
        msg_id: MessageId,
        msg: ServiceMsg,
        user: EndUser,
        auth: AuthorityProof<ServiceAuth>,
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
    /// Send a message to the given `recipients`.
    SendMessage {
        recipients: Vec<(XorName, SocketAddr)>,
        wire_msg: WireMsg,
    },
    /// Parses WireMsg to send to the correct location
    ParseAndSendWireMsg(WireMsg),
    /// Send a message to `delivery_group_size` peers out of the given `recipients`.
    SendMessageDeliveryGroup {
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

/// Generate unique timer token.
pub(crate) fn next_timer_token() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    NEXT.fetch_add(1, Ordering::Relaxed)
}
