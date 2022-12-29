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
use std::collections::BTreeSet;
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

    let all_cmds = run_and_collect_cmds(
        Cmd::HandleMembershipDecision(membership_decision),
        dispatcher,
    )
    .await?;

    let mut status = HandleOnlineStatus {
        node_approval_sent: false,
        relocate_details: None,
    };

    for cmd in all_cmds {
        let (msg, recipients) = match cmd {
            Cmd::SendMsg {
                recipients, msg, ..
            } => (msg, recipients),
            _ => continue,
        };

        match msg {
            NodeMsg::JoinResponse(JoinResponse::Approved { .. }) => {
                assert_matches!(recipients, Peers::Multiple(peers) => {
                    assert_eq!(peers, BTreeSet::from([*peer]));
                });
                status.node_approval_sent = true;
            }
            NodeMsg::Propose {
                proposal: sn_interface::messaging::system::Proposal::VoteNodeOffline(node_state),
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

pub(crate) async fn run_and_collect_cmds(
    cmd: Cmd,
    dispatcher: &Dispatcher,
) -> crate::node::error::Result<Vec<Cmd>> {
    let mut all_cmds = vec![];

    let mut cmds = dispatcher.process_cmd(cmd).await?;

    while !cmds.is_empty() {
        all_cmds.extend(cmds.clone());
        let mut new_cmds = vec![];
        for cmd in cmds {
            if !matches!(cmd, Cmd::SendMsg { .. }) {
                new_cmds.extend(dispatcher.process_cmd(cmd).await?);
            }
        }
        cmds = new_cmds;
    }

    Ok(all_cmds)
}

pub(crate) async fn run_node_handle_client_msg_and_collect_cmds(
    _msg: ClientMsg,
    _peer: Peer,
    dispatcher: &Dispatcher,
) -> crate::node::error::Result<Vec<Cmd>> {
    let mut all_cmds = vec![];

    let node = dispatcher.node();
    let the_node = node.read().await;

    // let (msg_id, msg, auth) = get_client_msg_parts_for_handling(msg)?;

    // TODO: decide how to test this, w/r/t no client stream.
    let mut cmds = vec![];
    // let mut cmds = the_node
    //     .handle_valid_client_msg(
    //         msg_id,
    //         msg,
    //         auth,
    //         peer,
    //     )
    //     .await?;

    // drop any read locks on the node here
    // we may have commands editing the node, requiring a write lock
    // coming after
    drop(the_node);

    while !cmds.is_empty() {
        all_cmds.extend(cmds.clone());
        let mut new_cmds = vec![];
        for cmd in cmds {
            if !matches!(cmd, Cmd::SendMsg { .. }) {
                new_cmds.extend(dispatcher.process_cmd(cmd).await?);
            }
        }

        cmds = new_cmds;
    }

    Ok(all_cmds)
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
