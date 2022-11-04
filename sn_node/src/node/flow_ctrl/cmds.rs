// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{
    messaging::{OutgoingMsg, Peers},
    Proposal, XorName,
};

use qp2p::SendStream;
use sn_consensus::Decision;
use sn_dysfunction::IssueType;
#[cfg(feature = "traceroute")]
use sn_interface::messaging::Traceroute;
use sn_interface::{
    messaging::{
        data::ClientMsg,
        system::{NodeMsg, SectionSig, SectionSigned},
        AuthorityProof, ClientAuth, MsgId, WireMsg,
    },
    network_knowledge::{NodeState, SectionAuthorityProvider, SectionKeyShare, SectionsDAG},
    types::{DataAddress, Peer},
};

use custom_debug::Debug;
use std::sync::Arc;
use std::{collections::BTreeSet, fmt, time::SystemTime};
use tokio::sync::Mutex;

/// A struct for the job of controlling the flow
/// of a [`Cmd`] in the system.
///
/// An id is assigned to it, its parent id (if any),
/// a priority by which it is ordered in the queue
/// among other pending cmd jobs, and the time the
/// job was instantiated.
#[derive(Debug, Clone)]
pub(crate) struct CmdJob {
    id: usize,
    parent_id: Option<usize>,
    cmd: Cmd,
    created_at: SystemTime,
}

/// Commands for a node.
///
/// Cmds are used to connect different modules, allowing
/// for a better modularization of the code base.
/// Modelling a call like this also allows for throttling
/// and prioritization, which is not something e.g. tokio tasks allow.
/// In other words, it enables enhanced flow control.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub(crate) enum Cmd {
    /// Validate `wire_msg` from `sender`.
    /// Holding the WireMsg that has been received from the network,
    ValidateMsg {
        origin: Peer,
        wire_msg: WireMsg,
        send_stream: Option<Arc<Mutex<SendStream>>>,
    },
    /// Log a Node's Punishment, this pulls dysfunction and write locks out of some functions
    TrackNodeIssueInDysfunction { name: XorName, issue: IssueType },
    HandleValidNodeMsg {
        msg_id: MsgId,
        msg: NodeMsg,
        origin: Peer,
        send_stream: Option<Arc<Mutex<SendStream>>>,
        #[cfg(feature = "traceroute")]
        traceroute: Traceroute,
    },
    UpdateNetworkAndHandleValidClientMsg {
        proof_chain: SectionsDAG,
        signed_sap: SectionSigned<SectionAuthorityProvider>,
        msg_id: MsgId,
        msg: ClientMsg,
        origin: Peer,
        send_stream: Arc<Mutex<SendStream>>,
        /// Requester's authority over this message
        auth: AuthorityProof<ClientAuth>,
        #[cfg(feature = "traceroute")]
        traceroute: Traceroute,
    },
    /// Handle peer that's been detected as lost.
    HandleFailedSendToNode { peer: Peer, msg_id: MsgId },
    /// Handle agreement on a proposal.
    HandleAgreement { proposal: Proposal, sig: SectionSig },
    /// Handle a membership decision.
    HandleMembershipDecision(Decision<NodeState>),
    /// Handle agree on elders. This blocks node message processing until complete.
    HandleNewEldersAgreement {
        new_elders: SectionSigned<SectionAuthorityProvider>,
        sig: SectionSig,
    },
    /// Handle the outcome of a DKG session where we are one of the participants (that is, one of
    /// the proposed new elders).
    HandleDkgOutcome {
        section_auth: SectionAuthorityProvider,
        outcome: SectionKeyShare,
    },
    /// Send the batch of data messages in a throttled/controlled fashion to the given `recipients`.
    /// chunks addresses are provided, so that we only retrieve the data right before we send it,
    /// hopefully reducing memory impact or data replication
    EnqueueDataForReplication {
        // throttle_duration: Duration,
        recipient: Peer,
        /// Batches of DataAddress to be sent together
        data_batch: Vec<DataAddress>,
    },
    /// Performs serialisation and signing and sends the msg.
    SendMsg {
        msg: OutgoingMsg,
        msg_id: MsgId,
        recipients: Peers,
        send_stream: Option<Arc<Mutex<SendStream>>>,
        #[cfg(feature = "traceroute")]
        traceroute: Traceroute,
    },
    /// Proposes peers as offline
    ProposeVoteNodesOffline(BTreeSet<XorName>),
}

impl Cmd {
    pub(crate) fn send_msg(msg: OutgoingMsg, recipients: Peers) -> Self {
        Self::send_traced_msg(
            msg,
            recipients,
            #[cfg(feature = "traceroute")]
            Traceroute(vec![]),
        )
    }

    pub(crate) fn send_msg_via_response_stream(
        msg: OutgoingMsg,
        recipients: Peers,
        send_stream: Option<Arc<Mutex<SendStream>>>,
    ) -> Self {
        Cmd::SendMsg {
            msg,
            msg_id: MsgId::new(),
            recipients,
            send_stream,
            #[cfg(feature = "traceroute")]
            traceroute: Traceroute(vec![]),
        }
    }

    pub(crate) fn send_traced_msg(
        msg: OutgoingMsg,
        recipients: Peers,
        #[cfg(feature = "traceroute")] traceroute: Traceroute,
    ) -> Self {
        let msg_id = MsgId::new();
        debug!("Sending msg {msg_id:?} {msg:?}");
        Cmd::SendMsg {
            msg,
            msg_id,
            recipients,
            send_stream: None,
            #[cfg(feature = "traceroute")]
            traceroute,
        }
    }

    pub(crate) fn statemap_state(&self) -> sn_interface::statemap::State {
        use sn_interface::statemap::State;
        match self {
            Cmd::SendMsg { .. } => State::Comms,
            Cmd::HandleFailedSendToNode { .. } => State::Comms,
            Cmd::ValidateMsg { .. } => State::Validation,
            Cmd::HandleValidNodeMsg { msg, .. } => msg.statemap_states(),
            Cmd::UpdateNetworkAndHandleValidClientMsg { .. } => State::ClientMsg,
            Cmd::TrackNodeIssueInDysfunction { .. } => State::Dysfunction,
            Cmd::HandleAgreement { .. } => State::Agreement,
            Cmd::HandleMembershipDecision(_) => State::Membership,
            Cmd::ProposeVoteNodesOffline(_) => State::Membership,
            Cmd::HandleNewEldersAgreement { .. } => State::Handover,
            Cmd::HandleDkgOutcome { .. } => State::Dkg,
            Cmd::EnqueueDataForReplication { .. } => State::Replication,
        }
    }
}

impl fmt::Display for Cmd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(not(feature = "test-utils"))]
            Cmd::ValidateMsg { wire_msg, .. } => {
                write!(f, "ValidateMsg {:?}", wire_msg.msg_id())
            }
            #[cfg(feature = "test-utils")]
            Cmd::ValidateMsg { wire_msg, .. } => {
                write!(
                    f,
                    "ValidateMsg {:?} {:?}",
                    wire_msg.msg_id(),
                    wire_msg.payload_debug
                )
            }
            Cmd::HandleValidNodeMsg { msg_id, msg, .. } => {
                write!(f, "HandleValidNodeMsg {:?}: {:?}", msg_id, msg)
            }
            Cmd::UpdateNetworkAndHandleValidClientMsg { msg_id, msg, .. } => {
                write!(f, "UpdateAndHandleValidClientMsg {:?}: {:?}", msg_id, msg)
            }
            Cmd::HandleFailedSendToNode { peer, msg_id } => {
                write!(f, "HandlePeerFailedSend({:?}, {:?})", peer.name(), msg_id)
            }
            Cmd::HandleAgreement { .. } => write!(f, "HandleAgreement"),
            Cmd::HandleNewEldersAgreement { .. } => write!(f, "HandleNewEldersAgreement"),
            Cmd::HandleMembershipDecision(_) => write!(f, "HandleMembershipDecision"),
            Cmd::HandleDkgOutcome { .. } => write!(f, "HandleDkgOutcome"),
            Cmd::SendMsg { .. } => write!(f, "SendMsg"),
            Cmd::EnqueueDataForReplication { .. } => write!(f, "ThrottledSendBatchMsgs"),
            Cmd::TrackNodeIssueInDysfunction { name, issue } => {
                write!(f, "TrackNodeIssueInDysfunction {:?}, {:?}", name, issue)
            }
            Cmd::ProposeVoteNodesOffline(_) => write!(f, "ProposeOffline"),
        }
    }
}
