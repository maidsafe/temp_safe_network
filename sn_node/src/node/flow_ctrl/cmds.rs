// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{messaging::Recipients, XorName};

use qp2p::SendStream;
use sn_consensus::Decision;
use sn_dbc::SpentProofShare;
use sn_fault_detection::IssueType;
use sn_interface::{
    messaging::{
        data::{ClientMsg, DataResponse},
        system::{NodeMsg, SectionSig, SectionSigned},
        AntiEntropyKind, AuthorityProof, ClientAuth, MsgId, NetworkMsg, WireMsg,
    },
    network_knowledge::{
        NodeState, SectionAuthorityProvider, SectionKeyShare, SectionTreeUpdate, SectionsDAG,
    },
    types::{ClientId, DataAddress, NodeId, Participant},
};

use custom_debug::Debug;
use std::{collections::BTreeSet, fmt, time::SystemTime};

/// A struct for the job of controlling the flow
/// of a [`Cmd`] in the system.
///
/// An id is assigned to it, its parent id (if any),
/// a priority by which it is ordered in the queue
/// among other pending cmd jobs, and the time the
/// job was instantiated.
#[derive(Debug)]
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
#[derive(Debug)]
pub enum Cmd {
    TryJoinNetwork,
    /// Validate `wire_msg` from `sender`.
    /// Holding the WireMsg that has been received from the network,
    HandleMsg {
        sender: Participant,
        wire_msg: WireMsg,
        send_stream: Option<SendStream>,
    },
    /// Process a deserialised node msg (after AE checks etc)
    ProcessNodeMsg {
        msg_id: MsgId,
        msg: NodeMsg,
        node_id: NodeId,
        send_stream: Option<SendStream>,
    },
    /// Process a deserialised client msg (after AE checks etc)
    ProcessClientMsg {
        msg_id: MsgId,
        msg: ClientMsg,
        auth: AuthorityProof<ClientAuth>,
        client_id: ClientId,
        send_stream: Option<SendStream>,
    },
    /// Process a deserialised AntiEntropy msg
    ProcessAeMsg {
        msg_id: MsgId,
        kind: AntiEntropyKind,
        section_tree_update: SectionTreeUpdate,
        sender: Participant,
    },
    /// Allows joining of new nodes.
    SetJoinsAllowed(bool),
    /// Allows joining of new nodes until the section splits.
    SetJoinsAllowedUntilSplit(bool),
    /// Add an issue to the tracking of a node's faults
    TrackNodeIssue {
        name: XorName,
        issue: IssueType,
    },
    UpdateNetworkAndHandleValidClientMsg {
        proof_chain: SectionsDAG,
        signed_sap: SectionSigned<SectionAuthorityProvider>,
        msg_id: MsgId,
        msg: ClientMsg,
        client_id: ClientId,
        send_stream: SendStream,
        /// Requester's authority over this message
        auth: AuthorityProof<ClientAuth>,
    },
    /// Handle comms error.
    HandleCommsError {
        participant: Participant,
        error: sn_comms::Error,
    },
    /// Handle agreement on a node off proposal.
    HandleNodeOffAgreement {
        proposal: NodeState,
        sig: SectionSig,
    },
    /// Handle a membership decision.
    HandleMembershipDecision(Decision<NodeState>),
    /// Handle agree on elders. This blocks node message processing until complete.
    HandleNewEldersAgreement {
        new_elders: SectionSigned<SectionAuthorityProvider>,
        sig: SectionSig,
    },
    /// Handle agree on new sections. This blocks node message processing until complete.
    HandleNewSectionsAgreement {
        sap1: SectionSigned<SectionAuthorityProvider>,
        sig1: SectionSig,
        sap2: SectionSigned<SectionAuthorityProvider>,
        sig2: SectionSig,
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
        recipient: NodeId,
        /// Batches of DataAddress to be sent together
        data_batch: Vec<DataAddress>,
    },
    UpdateCaller {
        /// The outdated caller
        caller: NodeId,
        /// The id of the causing msg.
        correlation_id: MsgId,
        /// The kind of anti-entropy response.
        kind: AntiEntropyKind,
        /// The update containing the current `SectionAuthorityProvider`
        /// and the section chain truncated from the triggering msg's dst section_key or genesis_key
        /// if the the dst section_key is not a direct ancestor to our section_key
        section_tree_update: SectionTreeUpdate,
    },
    UpdateCallerOnStream {
        /// The outdated caller
        caller: Participant,
        /// The id of the msg.
        msg_id: MsgId,
        /// The kind of anti-entropy response.
        kind: AntiEntropyKind,
        /// The update containing the current `SectionAuthorityProvider`
        /// and the section chain truncated from the triggering msg's dst section_key or genesis_key
        /// if the the dst section_key is not a direct ancestor to our section_key
        section_tree_update: SectionTreeUpdate,
        /// The id of the causing msg.
        correlation_id: MsgId,
        /// The msg stream to the caller.
        stream: SendStream,
    },
    /// Performs serialisation and signing and sends the msg.
    SendMsg {
        msg: NetworkMsg,
        msg_id: MsgId,
        recipients: Recipients,
    },
    /// Performs serialisation and signing and sends the msg over a bidi connection
    /// and then enqueues any response returned.
    SendMsgEnqueueAnyResponse {
        msg: NodeMsg,
        msg_id: MsgId,
        recipients: BTreeSet<NodeId>,
    },
    /// Performs serialisation and sends the response NodeMsg to the node over the given stream.
    SendNodeMsgResponse {
        msg: NodeMsg,
        msg_id: MsgId,
        correlation_id: MsgId,
        node_id: NodeId,
        send_stream: SendStream,
    },
    /// Performs serialisation and sends the msg to the client over the given stream.
    SendDataResponse {
        msg: DataResponse,
        msg_id: MsgId,
        correlation_id: MsgId,
        send_stream: SendStream,
        client_id: ClientId,
    },
    /// Performs serialisation and sends the msg to the nodes over a new bi-stream,
    /// awaiting for a response which is forwarded to the client.
    SendAndForwardResponseToClient {
        wire_msg: WireMsg,
        targets: BTreeSet<NodeId>,
        client_stream: SendStream,
        client_id: ClientId,
    },
    /// Proposes nodes as offline
    ProposeVoteNodesOffline(BTreeSet<XorName>),
    /// Enqueues a valid spend, with the valid paid fee, in the spend queue.
    EnqueueSpend {
        fee_paid: sn_dbc::Token,
        spent_share: SpentProofShare,
        send_stream: SendStream,
        correlation_id: MsgId,
        client_id: ClientId,
    },
}

impl Cmd {
    // this fn shall be fixed to be more type safe so that unnecessary casting upstream is avoided
    pub(crate) fn send_msg(msg: NodeMsg, recipients: Recipients) -> Self {
        Cmd::send_network_msg(NetworkMsg::Node(msg), recipients)
    }

    pub(crate) fn send_network_msg(msg: NetworkMsg, recipients: Recipients) -> Self {
        let msg_id = MsgId::new();
        debug!("Sending msg {msg_id:?} to {recipients:?}: {msg:?}");
        Cmd::SendMsg {
            msg,
            msg_id,
            recipients,
        }
    }

    pub(crate) fn send_node_response(
        msg: NodeMsg,
        correlation_id: MsgId,
        node_id: NodeId,
        send_stream: SendStream,
    ) -> Self {
        let msg_id = MsgId::new();
        Cmd::SendNodeMsgResponse {
            msg,
            msg_id,
            correlation_id,
            node_id,
            send_stream,
        }
    }

    pub(crate) fn send_data_response(
        msg: DataResponse,
        correlation_id: MsgId,
        client_id: ClientId,
        send_stream: SendStream,
    ) -> Self {
        let msg_id = MsgId::new();
        Cmd::SendDataResponse {
            msg,
            msg_id,
            correlation_id,
            client_id,
            send_stream,
        }
    }

    pub(crate) fn statemap_state(&self) -> sn_interface::statemap::State {
        use sn_interface::statemap::State;
        match self {
            Cmd::UpdateCaller { .. } => State::Ae,
            Cmd::UpdateCallerOnStream { .. } => State::Ae,
            Cmd::SendMsg { .. }
            | Cmd::SendMsgEnqueueAnyResponse { .. }
            | Cmd::SendNodeMsgResponse { .. }
            | Cmd::SendDataResponse { .. }
            | Cmd::SendAndForwardResponseToClient { .. }
            | Cmd::HandleCommsError { .. } => State::Comms,
            Cmd::HandleMsg { .. } => State::HandleMsg,
            Cmd::ProcessNodeMsg { .. } => State::HandleMsg,
            Cmd::ProcessClientMsg { .. } => State::HandleMsg,
            Cmd::ProcessAeMsg { .. } => State::HandleMsg,
            Cmd::UpdateNetworkAndHandleValidClientMsg { .. } => State::ClientMsg,
            Cmd::TrackNodeIssue { .. } => State::FaultDetection,
            Cmd::HandleNodeOffAgreement { .. } => State::Agreement,
            Cmd::HandleMembershipDecision(_) => State::Membership,
            Cmd::ProposeVoteNodesOffline(_) => State::Membership,
            Cmd::HandleNewEldersAgreement { .. } => State::Handover,
            Cmd::HandleNewSectionsAgreement { .. } => State::Handover,
            Cmd::HandleDkgOutcome { .. } => State::Dkg,
            Cmd::EnqueueDataForReplication { .. } => State::Replication,
            Cmd::SetJoinsAllowed { .. } => State::Data,
            Cmd::SetJoinsAllowedUntilSplit { .. } => State::Data,
            Cmd::TryJoinNetwork => State::Join,
            Cmd::EnqueueSpend { .. } => State::Spend,
        }
    }
}

impl fmt::Display for Cmd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Cmd::HandleMsg { wire_msg, .. } => {
                write!(f, "HandleMsg {:?}", wire_msg.msg_id())
            }
            Cmd::ProcessNodeMsg { msg_id, msg, .. } => {
                write!(f, "ProcessNodeMsg {msg_id:?}: {msg}")
            }
            Cmd::ProcessClientMsg { msg_id, msg, .. } => {
                write!(f, "ProcessClientMsg {msg_id:?}: {msg}")
            }
            Cmd::ProcessAeMsg { msg_id, .. } => {
                write!(f, "ProcessAeMsg {msg_id:?}")
            }
            Cmd::UpdateNetworkAndHandleValidClientMsg { msg_id, msg, .. } => {
                write!(f, "UpdateAndHandleValidClientMsg {msg_id:?}: {msg:?}")
            }
            Cmd::HandleCommsError { participant, error } => {
                write!(f, "HandleCommsError({participant:?}, {error:?})")
            }
            Cmd::HandleNodeOffAgreement { .. } => {
                write!(f, "HandleSectionDecisionAgreement")
            }
            Cmd::HandleNewEldersAgreement { .. } => write!(f, "HandleNewEldersAgreement"),
            Cmd::HandleNewSectionsAgreement { .. } => write!(f, "HandleNewSectionsAgreement"),
            Cmd::HandleMembershipDecision(_) => write!(f, "HandleMembershipDecision"),
            Cmd::HandleDkgOutcome { .. } => write!(f, "HandleDkgOutcome"),
            Cmd::SendMsg { .. } => write!(f, "SendMsg"),
            Cmd::SendMsgEnqueueAnyResponse { .. } => write!(f, "SendMsgEnqueueAnyResponse"),
            Cmd::SendNodeMsgResponse { .. } => write!(f, "SendNodeMsgResponse"),
            Cmd::SendDataResponse { .. } => write!(f, "SendDataResponse"),
            Cmd::SendAndForwardResponseToClient { .. } => {
                write!(f, "SendAndForwardResponseToClient")
            }
            Cmd::EnqueueDataForReplication { .. } => write!(f, "EnqueueDataForReplication"),
            Cmd::TrackNodeIssue { name, issue } => {
                write!(f, "TrackNodeIssue {name:?}, {issue:?}")
            }
            Cmd::ProposeVoteNodesOffline(_) => write!(f, "ProposeOffline"),
            Cmd::SetJoinsAllowed { .. } => write!(f, "SetJoinsAllowed"),
            Cmd::SetJoinsAllowedUntilSplit { .. } => write!(f, "SetJoinsAllowedUntilSplit"),
            Cmd::TryJoinNetwork => write!(f, "TryJoinNetwork"),
            Cmd::UpdateCaller { caller, kind, .. } => {
                write!(f, "UpdateCaller {caller:?}: {kind:?}")
            }
            Cmd::UpdateCallerOnStream { caller, kind, .. } => {
                write!(f, "UpdateCallerOnStream {caller:?}: {kind:?}")
            }
            Cmd::EnqueueSpend {
                fee_paid,
                spent_share,
                ..
            } => {
                write!(
                    f,
                    "EnqueueSpend {fee_paid:?}: {:?}",
                    spent_share.public_key()
                )
            }
        }
    }
}
