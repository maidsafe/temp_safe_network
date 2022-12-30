use crate::{
    comm::MsgFromPeer,
    node::{
        flow_ctrl::dispatcher::{into_msg_bytes, Dispatcher},
        messaging::Peers,
        Cmd,
    },
};
use sn_interface::{
    messaging::{
        data::ClientMsg,
        serialisation::WireMsg,
        system::{JoinResponse, NodeDataCmd, NodeMsg},
        AuthorityProof, ClientAuth, MsgId,
    },
    network_knowledge::{test_utils::*, MembershipState, NodeState, RelocateDetails},
    types::{Keypair, Peer, ReplicatedData},
};

use assert_matches::assert_matches;
use eyre::{eyre, Result};
use std::collections::{BTreeSet, VecDeque};
use tokio::sync::mpsc::{self, error::TryRecvError};
use xor_name::XorName;

pub(crate) struct HandleOnlineStatus {
    pub(crate) node_approval_sent: bool,
    pub(crate) relocate_details: Option<RelocateDetails>,
}

pub(crate) async fn handle_online_cmd(
    peer: &Peer,
    sk_set: &bls::SecretKeySet,
    dispatcher: &Dispatcher,
) -> Result<HandleOnlineStatus> {
    let node_state = NodeState::joined(*peer, None);
    let membership_decision = section_decision(sk_set, node_state);

    let mut all_cmds = ProcessAndInspectCmds::new(
        Cmd::HandleMembershipDecision(membership_decision),
        dispatcher,
    );

    let mut status = HandleOnlineStatus {
        node_approval_sent: false,
        relocate_details: None,
    };

    while let Some(cmd) = all_cmds.next().await? {
        let (msg, recipients) = match cmd {
            Cmd::SendMsg {
                recipients, msg, ..
            } => (msg, recipients),
            _ => continue,
        };

        match msg {
            NodeMsg::JoinResponse(JoinResponse::Approved { .. }) => {
                assert_matches!(recipients, Peers::Multiple(peers) => {
                    assert_eq!(peers, &BTreeSet::from([*peer]));
                });
                status.node_approval_sent = true;
            }
            NodeMsg::ProposeSectionState {
                proposal:
                    sn_interface::messaging::system::SectionStateVote::NodeIsOffline(node_state),
                ..
            } => {
                if let MembershipState::Relocated(details) = node_state.state() {
                    if details.previous_name != peer.name() {
                        continue;
                    }
                    status.relocate_details = Some(*details.clone());
                }
            }
            _ => continue,
        }
    }

    Ok(status)
}

// Process commands, allowing the user to inspect each and all of the intermediate
// commands that are being returned by the Cmd dispatcher.
pub(crate) struct ProcessAndInspectCmds<'a> {
    pending_cmds: VecDeque<Cmd>,
    cmds_to_inspect: VecDeque<usize>,
    dispatcher: &'a Dispatcher,
}

impl<'a> ProcessAndInspectCmds<'a> {
    pub(crate) fn new(cmd: Cmd, dispatcher: &'a Dispatcher) -> Self {
        Self {
            pending_cmds: VecDeque::from([cmd]),
            cmds_to_inspect: VecDeque::default(),
            dispatcher,
        }
    }

    pub(crate) fn new_with_client_msg(
        _msg: ClientMsg,
        _peer: Peer,
        dispatcher: &'a Dispatcher,
    ) -> crate::node::error::Result<Self> {
        // TODO: decide how to impl this, w/r/t client response stream, to re-enable
        // the currently ignored/disabled Spentbook tests. This used to work by
        // calling `MyNode::handle_valid_client_msg` using the provided ClientMsg,
        // and use the outcome (commands) as the starting set of cmds to process.
        let pending_cmds = VecDeque::default();

        Ok(Self {
            pending_cmds,
            cmds_to_inspect: VecDeque::default(),
            dispatcher,
        })
    }

    pub(crate) async fn next(&mut self) -> crate::node::error::Result<Option<&Cmd>> {
        match self.cmds_to_inspect.pop_front() {
            Some(index) => {
                let cmd = self.pending_cmds.get(index);
                assert!(cmd.is_some());
                Ok(cmd)
            }
            None => {
                while let Some(cmd) = self.pending_cmds.pop_front() {
                    if !matches!(cmd, Cmd::SendMsg { .. }) {
                        let new_cmds = self.dispatcher.process_cmd(cmd).await?;
                        for cmd in new_cmds.into_iter() {
                            let new_cmd_index = self.pending_cmds.len();
                            self.pending_cmds.push_back(cmd);
                            self.cmds_to_inspect.push_back(new_cmd_index);
                        }

                        if let Some(index) = self.cmds_to_inspect.pop_front() {
                            let cmd = self.pending_cmds.get(index);
                            assert!(cmd.is_some());
                            return Ok(cmd);
                        }
                    }
                }

                Ok(None)
            }
        }
    }

    pub(crate) async fn process_all(&mut self) -> crate::node::error::Result<()> {
        while self.next().await?.is_some() { /* we just process all cmds */ }
        Ok(())
    }
}

pub(crate) fn get_client_msg_parts_for_handling(
    msg: ClientMsg,
) -> crate::node::error::Result<(MsgId, ClientMsg, AuthorityProof<ClientAuth>)> {
    let payload = WireMsg::serialize_msg_payload(&msg)?;
    let src_client_keypair = Keypair::new_ed25519();
    let auth = ClientAuth {
        public_key: src_client_keypair.public_key(),
        signature: src_client_keypair.sign(&payload),
    };
    let auth_proof = AuthorityProof::verify(auth, &payload)?;

    Ok((MsgId::new(), msg, auth_proof))
}

/// Extend the `Cmd` enum with some utilities for testing.
///
/// Since this is in a module marked as #[test], this functionality will only be present in the
/// testing context.
impl Cmd {
    /// Get the recipients for a `SendMsg` command.
    pub(crate) fn recipients(&self) -> Result<BTreeSet<Peer>> {
        match self {
            Cmd::SendMsg { recipients, .. } => match recipients {
                Peers::Single(peer) => {
                    let mut set = BTreeSet::new();
                    let _ = set.insert(*peer);
                    Ok(set)
                }
                Peers::Multiple(peers) => Ok(peers.clone()),
            },
            _ => Err(eyre!("A Cmd::SendMsg variant was expected")),
        }
    }

    /// Get the replicated data from a `NodeCmd` message.
    pub(crate) fn get_replicated_data(&self) -> Result<ReplicatedData> {
        match self {
            Cmd::SendMsg { msg, .. } => match msg {
                NodeMsg::NodeDataCmd(node_cmd) => match node_cmd {
                    NodeDataCmd::ReplicateDataBatch(data) => {
                        if data.len() != 1 {
                            return Err(eyre!("Only 1 replicated data instance is expected"));
                        }
                        Ok(data[0].clone())
                    }
                    _ => Err(eyre!("A NodeCmd::ReplicateData variant was expected")),
                },
                _ => Err(eyre!("An NodeMsg::NodeCmd variant was expected")),
            },
            _ => Err(eyre!("A Cmd::SendMsg variant was expected")),
        }
    }

    // /// Get a `ClientDataResponse` from a `Cmd::SendMsg` enum variant.
    // pub(crate) fn get_client_msg_resp(&self) -> Result<ClientDataResponse> {
    //     match self {
    //         Cmd::SendMsg { msg, .. } => match msg {
    //             OutgoingMsg::Client(client_msg) => Ok(client_msg.clone()),
    //             _ => Err(eyre!("A OutgoingMsg::Client variant was expected")),
    //         },
    //         _ => Err(eyre!("A Cmd::SendMsg variant was expected")),
    //     }
    // }

    // /// Get a `sn_interface::messaging::data::Error` from a `Cmd::SendMsg` enum variant.
    // pub(crate) fn get_error(&self) -> Result<MessagingDataError> {
    //     match self {
    //         Cmd::SendMsg { msg, .. } => match msg {
    //             OutgoingMsg::Client(client_msg) => match client_msg {
    //                 ClientDataResponse::CmdResponse { response, .. } => match response.result() {
    //                     Ok(_) => Err(eyre!("A CmdResponse error was expected")),
    //                     Err(error) => Ok(error.clone()),
    //                 },
    //                 _ => Err(eyre!("A ClientDataResponse::CmdResponse variant was expected")),
    //             },
    //             _ => Err(eyre!("A OutgoingMsg::Client variant was expected")),
    //         },
    //         _ => Err(eyre!("A Cmd::SendMsg variant was expected")),
    //     }
    // }
}

impl Dispatcher {
    // Sends out `NodeMsgs` to others synchronously, (process_cmd() spawns tasks to do it).
    // Optionally drop the msgs to the provided set of peers.
    pub(crate) async fn mock_send_msg(&self, cmd: Cmd, filter_recp: Option<BTreeSet<XorName>>) {
        if let Cmd::SendMsg {
            msg,
            msg_id,
            recipients,
            send_stream,
            context,
        } = cmd
        {
            let _ = send_stream;
            let peer_msgs = {
                into_msg_bytes(
                    &context.network_knowledge,
                    context.name,
                    msg.clone(),
                    msg_id,
                    recipients,
                )
                .expect("cannot convert msg into bytes")
            };

            for (peer, msg_bytes) in peer_msgs {
                if let Some(filter) = &filter_recp {
                    if filter.contains(&peer.name()) {
                        continue;
                    }
                }
                context
                    .comm
                    .send_out_bytes_sync(peer, msg_id, msg_bytes)
                    .await;
                info!("Sent {msg} to {}", peer.name());
            }
        } else {
            panic!("mock_send_msg expects Cmd::SendMsg, got {cmd:?}");
        }
    }
}

// Receive the next `MsgFromPeer` if the buffer is not empty. Returns None if the buffer is currently empty
pub(crate) fn get_next_msg(comm_rx: &mut mpsc::Receiver<MsgFromPeer>) -> Option<MsgFromPeer> {
    match comm_rx.try_recv() {
        Ok(msg) => Some(msg),
        Err(TryRecvError::Empty) => None,
        Err(TryRecvError::Disconnected) => panic!("the comm_rx channel is closed"),
    }
}
