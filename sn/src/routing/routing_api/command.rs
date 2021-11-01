// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::{
    system::{DkgFailureSigSet, KeyedSig, Proposal, SystemMsg},
    DstLocation, MessageId, NodeMsgAuthority, SectionAuthorityProvider, WireMsg,
};
use crate::routing::{
    node::Node,
    routing_api::Peer,
    section::{NetworkKnowledge, SectionKeyShare},
    XorName,
};
use bls::PublicKey as BlsPublicKey;
use bytes::Bytes;
use custom_debug::Debug;
use std::{
    fmt,
    net::SocketAddr,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

/// Command for node.
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub(crate) enum Command {
    /// Handle `message` from `sender`.
    /// Holding the WireMsg that has been received from the network,
    HandleMessage {
        sender: SocketAddr,
        wire_msg: WireMsg,
        #[debug(skip)]
        // original bytes to avoid reserializing for entropy checks
        original_bytes: Option<Bytes>,
    },
    // TODO: rename this as/when this is all node for clarity
    /// Handle Node, either directly or notify via event listener
    HandleSystemMessage {
        sender: SocketAddr,
        msg_id: MessageId,
        msg: SystemMsg,
        msg_authority: NodeMsgAuthority,
        dst_location: DstLocation,
        #[debug(skip)]
        payload: Bytes,
        #[debug(skip)]
        known_keys: Vec<BlsPublicKey>,
    },
    /// Handle verified node message after aggregation either directly or notify via event listener
    HandleBlockingMessage {
        sender: SocketAddr,
        msg_id: MessageId,
        msg: SystemMsg,
        msg_authority: NodeMsgAuthority,
    },
    /// Handle Node data messages directly
    HandleNonBlockingMessage {
        msg_id: MessageId,
        msg: SystemMsg,
        msg_authority: NodeMsgAuthority,
        dst_location: DstLocation,
        sender: SocketAddr,
        #[debug(skip)]
        known_keys: Vec<BlsPublicKey>,
    },
    /// Handle a timeout previously scheduled with `ScheduleTimeout`.
    HandleTimeout(u64),
    /// Handle peer that's been detected as lost.
    HandlePeerLost(SocketAddr),
    /// Handle agreement on a proposal.
    HandleAgreement { proposal: Proposal, sig: KeyedSig },
    /// Handle agree on elders. This blocks node message processing until complete.
    HandleElderAgreement { proposal: Proposal, sig: KeyedSig },
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
    /// Performs serialisation and signing for sending of NodeMst
    PrepareNodeMsgToSend { msg: SystemMsg, dst: DstLocation },
    /// Send a message to `delivery_group_size` peers out of the given `recipients`.
    SendMessageDeliveryGroup {
        recipients: Vec<(XorName, SocketAddr)>,
        delivery_group_size: usize,
        wire_msg: WireMsg,
    },
    /// Schedule a timeout after the given duration. When the timeout expires, a `HandleTimeout`
    /// command is raised. The token is used to identify the timeout.
    ScheduleTimeout { duration: Duration, token: u64 },
    /// Relocation process is complete, switch to new section
    HandleRelocationComplete {
        /// New Node state and information
        node: Node,
        /// New section where we relocated
        section: NetworkKnowledge,
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

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::HandleTimeout(_) => write!(f, "HandlePeerLost"),
            Command::ScheduleTimeout { .. } => write!(f, "ScheduleTimeout"),
            Command::HandleSystemMessage { msg_id, .. } => {
                write!(f, "HandleSystemMessage {:?}", msg_id)
            }
            Command::HandleMessage { wire_msg, .. } => {
                write!(f, "HandleMessage {:?}", wire_msg.msg_id())
            }
            Command::HandleBlockingMessage { msg_id, .. } => {
                write!(f, "HandleBlockingMessage {:?}", msg_id)
            }
            Command::HandleNonBlockingMessage { msg_id, .. } => {
                write!(f, "HandleNonBlockingMessage {:?}", msg_id)
            }
            Command::HandlePeerLost(_) => write!(f, "HandlePeerLost"),
            Command::HandleAgreement { .. } => write!(f, "HandleAgreement"),
            Command::HandleElderAgreement { .. } => write!(f, "HandleElderAgreement"),
            Command::HandleDkgOutcome { .. } => write!(f, "HandleDkgOutcome"),
            Command::HandleDkgFailure(_) => write!(f, "HandleDkgFailure"),
            Command::SendMessage { wire_msg, .. } => {
                write!(f, "SendMessage {:?}", wire_msg.msg_id())
            }
            Command::ParseAndSendWireMsg(wire_msg) => {
                write!(f, "ParseAndSendWireMsg {:?}", wire_msg.msg_id())
            }
            Command::PrepareNodeMsgToSend { .. } => write!(f, "PrepareNodeMsgToSend"),
            Command::SendMessageDeliveryGroup { wire_msg, .. } => {
                write!(f, "SendMessageDeliveryGroup {:?}", wire_msg.msg_id())
            }
            Command::HandleRelocationComplete { .. } => {
                write!(f, "HandleRelocationComplete")
            }
            Command::SetJoinsAllowed(_) => write!(f, "SetJoinsAllowed"),
            Command::ProposeOnline { .. } => write!(f, "ProposeOnline"),
            Command::ProposeOffline(_) => write!(f, "ProposeOffline"),
            Command::StartConnectivityTest(_) => write!(f, "StartConnectivityTest"),
            Command::TestConnectivity(_) => write!(f, "TestConnectivity"),
        }
    }
}

/// Generate unique timer token.
pub(crate) fn next_timer_token() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    NEXT.fetch_add(1, Ordering::Relaxed)
}
