// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{messaging::Recipients, Cmd, MyNode};
use sn_comms::{CommEvent, MsgReceived};
use sn_interface::{
    messaging::{
        data::ClientMsg,
        serialisation::WireMsg,
        system::{JoinResponse, NodeMsg},
        AuthorityProof, ClientAuth, Dst, MsgId, MsgKind, NetworkMsg,
    },
    network_knowledge::{test_utils::*, NodeState},
    types::{Keypair, RewardNodeId},
};

use assert_matches::assert_matches;
use bytes::Bytes;
use eyre::{eyre, Context, Result};
use qp2p::Endpoint;
use std::{
    collections::{btree_map::Entry, BTreeMap, BTreeSet, VecDeque},
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};
use tokio::sync::{
    mpsc::{error::TryRecvError, Receiver},
    RwLock,
};
use xor_name::XorName;

pub(crate) struct JoinApprovalSent(pub(crate) bool);

pub(crate) async fn handle_online_cmd(
    node_id: &RewardNodeId,
    sk_set: &bls::SecretKeySet,
    node: &mut MyNode,
) -> Result<JoinApprovalSent> {
    let node_state = NodeState::joined(*node_id, None);
    let membership_decision = section_decision(sk_set, node_state)?;

    let mut all_cmds =
        ProcessAndInspectCmds::new(Cmd::HandleMembershipDecision(membership_decision));

    let mut approval_sent = JoinApprovalSent(false);

    while let Some(cmd) = all_cmds.next(node).await? {
        let (msg, recipients) = match cmd {
            Cmd::SendMsg {
                recipients,
                msg: NetworkMsg::Node(msg),
                ..
            } => (msg, recipients),
            _ => continue,
        };

        match msg {
            NodeMsg::JoinResponse(JoinResponse::Approved { .. }) => {
                assert_matches!(recipients, Recipients::Multiple(nodes) => {
                    assert_eq!(nodes, &BTreeSet::from([node_id.node_id()]));
                });
                approval_sent.0 = true;
            }
            _ => continue,
        }
    }

    Ok(approval_sent)
}

// Process commands, allowing the user to inspect each and all of the intermediate
// commands that are being returned by the Cmd node.
// All commands that are meant to send msgs over the wire are inspected but not processed further.
pub(crate) struct ProcessAndInspectCmds {
    pending_cmds: VecDeque<Cmd>,
    index_inspected: usize,
}

impl ProcessAndInspectCmds {
    pub(crate) fn new(cmd: Cmd) -> Self {
        Self::from(vec![cmd])
    }

    fn from(cmds: Vec<Cmd>) -> Self {
        // We initialise `index_inspected` with MAX value, it will wraparound to 0 upon the first
        // call to `next()` method, thus making sure the first cmd is inspected in first iteration.
        let index_inspected = usize::MAX;

        Self {
            pending_cmds: VecDeque::from(cmds),
            index_inspected,
        }
    }

    // This constructor invokes `MyNode::handle_valid_client_msg` using the
    // provided ClientMsg, and it uses the outcome (commands) as the
    // starting set of cmds to process by the ProcessAndInspectCmds instance herein created.
    // TODO: the client recv-stream created could be returned for the caller to use if necessary,
    // at this point it's useless since `Cmd::SendDataResponse` is not processed but only inspected.
    pub(crate) async fn new_from_client_msg(
        msg: ClientMsg,
        node: &mut MyNode,
        mut comm_rx: Receiver<CommEvent>,
    ) -> crate::node::error::Result<ProcessAndInspectCmds> {
        let context = node.context();
        let (msg_id, serialised_payload, msg_kind, _auth) =
            get_client_msg_parts_for_handling(&msg)?;

        let client_addr: SocketAddr = (Ipv4Addr::LOCALHOST, 0).into();
        let client_endpoint = Endpoint::builder()
            .addr(client_addr)
            .idle_timeout(70_000)
            .client()
            .expect("failed to create new client endpoint");

        let node_id = context.info.id();
        let node_addr = node_id.addr();
        let (client_conn, _) = client_endpoint
            .connect_to(&node_addr)
            .await
            .unwrap_or_else(|err| panic!("failed to connect to node at {node_addr:?}: {err:?}"));
        let (mut send_stream, _recv_stream) = client_conn
            .open_bi()
            .await
            .expect("failed to open bi-stream from new client endpoint");

        let dst = Dst {
            name: node_id.name(),
            section_key: context.network_knowledge.section_key(),
        };
        let wire_msg = WireMsg::new_msg(msg_id, serialised_payload, msg_kind, dst);
        let user_msg = wire_msg.serialize()?;

        // move send msg off thread so send / receive can both complete
        let _handle = tokio::spawn(async move {
            let _ = send_stream
                .send_user_msg(user_msg)
                .await
                .context("Could not send user msg");
        });

        match comm_rx.recv().await {
            Some(CommEvent::Msg(MsgReceived {
                send_stream: Some(send_stream),
                ..
            })) => {
                let cmds = MyNode::handle_msg(
                    node,
                    sn_interface::types::Participant::from_node(node_id),
                    wire_msg,
                    Some(send_stream),
                )
                .await?;
                Ok(Self::from(cmds))
            }
            _ => Err(crate::node::error::Error::NoClientResponseStream),
        }
    }

    pub(crate) async fn next(
        &mut self,
        node: &mut MyNode,
    ) -> crate::node::error::Result<Option<&Cmd>> {
        let mut next_index = self.index_inspected.wrapping_add(1);
        if next_index < self.pending_cmds.len() {
            let cmd = self.pending_cmds.get(next_index);
            assert!(cmd.is_some());
            self.index_inspected = next_index;
            return Ok(cmd);
        }

        while let Some(cmd) = self.pending_cmds.pop_front() {
            next_index -= 1;
            if !matches!(
                cmd,
                Cmd::SendMsg { .. }
                    | Cmd::SendDataResponse { .. }
                    | Cmd::SendAndForwardResponseToClient { .. }
            ) {
                let new_cmds = MyNode::process_cmd(cmd, node).await?;
                self.pending_cmds.extend(new_cmds);

                if next_index < self.pending_cmds.len() {
                    let cmd = self.pending_cmds.get(next_index);
                    assert!(cmd.is_some());
                    self.index_inspected = next_index;
                    return Ok(cmd);
                }
            }
        }
        Ok(None)
    }

    pub(crate) async fn process_all(
        &mut self,
        node: &mut MyNode,
    ) -> crate::node::error::Result<()> {
        while self.next(node).await?.is_some() { /* we just process all cmds */ }
        Ok(())
    }
}

pub(crate) fn get_client_msg_parts_for_handling(
    msg: &ClientMsg,
) -> crate::node::error::Result<(MsgId, Bytes, MsgKind, AuthorityProof<ClientAuth>)> {
    let payload = WireMsg::serialize_msg_payload(msg)?;
    let src_client_keypair = Keypair::new_ed25519();
    let auth = ClientAuth {
        public_key: src_client_keypair.public_key(),
        signature: src_client_keypair.sign(&payload),
    };
    let auth_proof = AuthorityProof::verify(auth.clone(), &payload)?;
    let kind = MsgKind::Client {
        auth,
        is_spend: false,
        query_index: None,
    };

    Ok((MsgId::new(), payload, kind, auth_proof))
}

/// Bundles the `MyNode` along with the `TestMsgTracker` to easily track the
/// NodeMsgs during tests
pub(crate) struct TestNode {
    pub(crate) node: MyNode,
    pub(crate) msg_tracker: Arc<RwLock<TestMsgTracker>>,
}

impl TestNode {
    pub(crate) fn new(node: MyNode, msg_tracker: Arc<RwLock<TestMsgTracker>>) -> Self {
        Self { node, msg_tracker }
    }

    /// Tracks the cmds before executing them
    pub(crate) async fn process_cmd(&mut self, cmd: Cmd) -> Result<Vec<Cmd>> {
        self.msg_tracker.write().await.track(&cmd);
        let cmd_string = cmd.to_string();
        MyNode::process_cmd(cmd, &mut self.node)
            .await
            .wrap_err(format!("Failed to process {cmd_string}"))
    }

    /// Handle and keep track of msgs from clients and nodes.
    /// Contains optional relocation_old_name to deal with name change during relocation.
    pub(crate) async fn test_handle_msg(
        &mut self,
        msg: MsgReceived,
        relocation_old_name: Option<XorName>,
    ) -> Result<Vec<Cmd>> {
        let msg_id = msg.wire_msg.msg_id();

        // check if we have successfully untracked the msg
        let mut untracked = false;
        if let Some(old_name) = relocation_old_name {
            untracked = untracked || self.msg_tracker.write().await.untrack(msg_id, &old_name);
        }
        let our_name = self.node.name();
        untracked = untracked || self.msg_tracker.write().await.untrack(msg_id, &our_name);
        if !untracked {
            return Err(eyre!(
                "Trying to untrack {msg_id:?} at node {our_name:?}
                \nThe msg was not tracked for this node.
                \nPlease check Cmd::SendMsg* to debug the issue"
            ));
        }

        let handle_node_msg_cmd = Cmd::HandleMsg {
            sender: msg.sender,
            wire_msg: msg.wire_msg,
            send_stream: msg.send_stream,
        };

        let msg_cmds = self
            .process_cmd(handle_node_msg_cmd)
            .await
            .wrap_err("Error while handling node_msg, Cmd::HandleMsg")?;
        let mut cmds = Vec::new();
        for cmd in msg_cmds {
            let cmd_string = cmd.to_string();
            match self.process_cmd(cmd).await {
                Ok(new_cmds) => cmds.extend(new_cmds),
                Err(err) => warn!("Error while handling node_msg, {cmd_string:?}: {err:?}"),
            }
        }
        Ok(cmds)
    }
}

/// Test utility to keep track of the msgs that has been sent.
/// When the msg has been received, it is removed from the tracker.
/// Used to terminate tests.
#[derive(Debug, Default)]
pub(crate) struct TestMsgTracker {
    pub(crate) tracker: BTreeMap<MsgId, BTreeSet<XorName>>,
}

impl TestMsgTracker {
    /// Tracks the msgs during SendMsg* Cmd
    pub(crate) fn track(&mut self, cmd: &Cmd) {
        if let Cmd::SendMsg {
            msg_id, recipients, ..
        } = cmd
        {
            let recp = recipients.clone().into_iter().map(|p| p.name()).collect();
            info!("Tracking {msg_id:?} for {recp:?}, cmd {cmd}");
            let _ = self.tracker.insert(*msg_id, recp);
        } else if let Cmd::SendMsgEnqueueAnyResponse {
            msg_id, recipients, ..
        } = cmd
        {
            let recp = recipients.iter().map(|p| p.name()).collect();
            info!("Tracking {msg_id:?} for {recp:?}, cmd {cmd}");
            let _ = self.tracker.insert(*msg_id, recp);
        } else if let Cmd::SendNodeMsgResponse {
            msg_id, node_id, ..
        } = cmd
        {
            info!("Tracking {msg_id:?} for {node_id:?}, cmd {cmd}");
            let _ = self
                .tracker
                .insert(*msg_id, BTreeSet::from([node_id.name()]));
        } else if let Cmd::UpdateCallerOnStream { caller, msg_id, .. } = cmd {
            info!("Tracking {msg_id:?} for {caller:?}, cmd {cmd}");
            let _ = self
                .tracker
                .insert(*msg_id, BTreeSet::from([caller.name()]));
        }
    }

    // Untrack the msg when we receive a MsgReceived
    pub(crate) fn untrack(&mut self, msg_id: MsgId, our_name: &XorName) -> bool {
        info!("Untracking {msg_id:?} for {our_name:?}");
        let removed;
        if let Entry::Occupied(mut entry) = self.tracker.entry(msg_id) {
            let nodes = entry.get_mut();
            removed = nodes.remove(our_name);
            if nodes.is_empty() {
                let _ = entry.remove();
            }
        } else {
            removed = false;
        }
        removed
    }

    /// When the counter is empty we are sure that all the msgs are processed
    pub(crate) fn is_empty(&self) -> bool {
        self.tracker.is_empty()
    }
}

/// Extend the `Cmd` enum with some utilities for testing.
///
/// Since this is in a module marked as #[test], this functionality will only be present in the
/// testing context.
impl Cmd {
    // Filters the list of recipients in a `SendCmd`
    pub(crate) fn filter_recipients(&mut self, filter_list: BTreeSet<XorName>) -> Result<()> {
        if let Cmd::SendMsg {
            ref mut recipients, ..
        } = self
        {
            let new_recipients = match recipients {
                Recipients::Single(dst) => {
                    if filter_list.contains(&dst.name()) {
                        Recipients::Multiple(BTreeSet::new())
                    } else {
                        Recipients::Single(*dst)
                    }
                }
                Recipients::Multiple(nodes) => {
                    let nodes = nodes
                        .iter()
                        .filter(|node| !filter_list.contains(&node.name()))
                        .cloned()
                        .collect();
                    Recipients::Multiple(nodes)
                }
            };
            *recipients = new_recipients;
        } else {
            return Err(eyre!("Expected a Cmd::SendMsg* to filter the recipients"));
        };
        Ok(())
    }
}

// Receive the next `MsgReceived` if the buffer is not empty. Returns None if the buffer is currently empty
pub(crate) async fn get_next_msg(comm_rx: &mut Receiver<CommEvent>) -> Option<MsgReceived> {
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    match comm_rx.try_recv() {
        Ok(CommEvent::Msg(msg)) => Some(msg),
        Ok(_) => None,
        Err(TryRecvError::Empty) => None,
        Err(TryRecvError::Disconnected) => panic!("the comm_rx channel is closed"),
    }
}
