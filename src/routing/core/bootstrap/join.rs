// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::{
    node::{
        DstInfo, JoinRejectionReason, JoinRequest, JoinResponse, NodeMsg, ResourceProofResponse,
        Section,
    },
    DstLocation, MessageType, MsgKind, NodeMsgAuthority, NodeSigned, WireMsg,
};
use crate::routing::{
    dkg::SectionSignedUtils,
    ed25519,
    error::{Error, Result},
    messages::WireMsgUtils,
    node::Node,
    peer::PeerUtils,
    routing_api::{
        comm::{Comm, ConnectionEvent, SendStatus},
        command::Command,
    },
    section::{SectionAuthorityProviderUtils, SectionUtils},
    FIRST_SECTION_MAX_AGE, FIRST_SECTION_MIN_AGE, MIN_ADULT_AGE,
};
use futures::future;
use rand::seq::IteratorRandom;
use resource_proof::ResourceProof;
use std::{
    collections::{HashSet, VecDeque},
    net::SocketAddr,
};
use tokio::sync::mpsc;
use tracing::Instrument;
use xor_name::{Prefix, XorName};

const BACKLOG_CAPACITY: usize = 100;

/// Join the network as new node.
///
/// NOTE: It's not guaranteed this function ever returns. This can happen due to messages being
/// lost in transit or other reasons. It's the responsibility of the caller to handle this case,
/// for example by using a timeout.
pub(crate) async fn join_network(
    node: Node,
    comm: &Comm,
    incoming_conns: &mut mpsc::Receiver<ConnectionEvent>,
    bootstrap_addr: SocketAddr,
) -> Result<(Node, Section, Vec<Command>)> {
    let (send_tx, send_rx) = mpsc::channel(1);

    let span = trace_span!("bootstrap", name = %node.name());

    let state = Join::new(node, send_tx, incoming_conns);

    future::join(state.run(bootstrap_addr), send_messages(send_rx, comm))
        .instrument(span)
        .await
        .0
}

struct Join<'a> {
    // Sender for outgoing messages.
    send_tx: mpsc::Sender<(WireMsg, Vec<(XorName, SocketAddr)>)>,
    // Receiver for incoming messages.
    recv_rx: &'a mut mpsc::Receiver<ConnectionEvent>,
    node: Node,
    // Backlog for unknown messages
    backlog: VecDeque<Command>,
}

impl<'a> Join<'a> {
    fn new(
        node: Node,
        send_tx: mpsc::Sender<(WireMsg, Vec<(XorName, SocketAddr)>)>,
        recv_rx: &'a mut mpsc::Receiver<ConnectionEvent>,
    ) -> Self {
        Self {
            send_tx,
            recv_rx,
            node,
            backlog: VecDeque::with_capacity(BACKLOG_CAPACITY),
        }
    }

    // Send `JoinRequest` and wait for the response. If the response is:
    // - `Retry`: repeat with the new info.
    // - `Redirect`: repeat with the new set of addresses.
    // - `ResourceChallenge`: carry out resource proof calculation.
    // - `Approval`: returns the initial `Section` value to use by this node,
    //    completing the bootstrap.
    async fn run(self, bootstrap_addr: SocketAddr) -> Result<(Node, Section, Vec<Command>)> {
        // Use our XorName as we do not know their name or section key yet.
        let section_key = bls::SecretKey::random().public_key();
        let dst_xorname = self.node.name();

        let recipients = vec![(dst_xorname, bootstrap_addr)];

        self.join(section_key, recipients).await
    }

    async fn join(
        mut self,
        mut section_key: bls::PublicKey,
        mut recipients: Vec<(XorName, SocketAddr)>,
    ) -> Result<(Node, Section, Vec<Command>)> {
        // We send a first join request to obtain the resource challenge, which
        // we will then use to generate the challenge proof and send the
        // `JoinRequest` again with it.
        let join_request = JoinRequest {
            section_key,
            resource_proof_response: None,
        };

        self.send_join_requests(join_request, &recipients, section_key)
            .await?;

        // Avoid sending more than one request to the same peer.
        let mut used_recipient = HashSet::<SocketAddr>::new();

        loop {
            used_recipient.extend(recipients.iter().map(|(_, addr)| addr));

            let (response, sender, src_name) = self.receive_join_response().await?;

            match response {
                JoinResponse::Rejected(JoinRejectionReason::NodeNotReachable(addr)) => {
                    error!(
                        "Node cannot join the network since it is not externally reachable: {}",
                        addr
                    );
                    return Err(Error::NodeNotReachable(addr));
                }
                JoinResponse::Rejected(JoinRejectionReason::JoinsDisallowed) => {
                    error!("Network is set to not taking any new joining node, try join later.");
                    return Err(Error::TryJoinLater);
                }
                JoinResponse::Approval {
                    section_auth,
                    genesis_key,
                    section_chain,
                    ..
                } => {
                    return Ok((
                        self.node,
                        Section::new(genesis_key, section_chain, section_auth)?,
                        self.backlog.into_iter().collect(),
                    ));
                }
                JoinResponse::Retry(section_auth) => {
                    if section_auth.section_key() == section_key {
                        debug!("Ignoring JoinResponse::Retry with invalid section authority provider key");
                        continue;
                    }

                    let new_recipients: Vec<(XorName, SocketAddr)> = section_auth
                        .elders
                        .iter()
                        .map(|(name, addr)| (*name, *addr))
                        .collect();

                    let prefix = section_auth.prefix;

                    // For the first section, using age random among 6 to 100 to avoid
                    // relocating too many nodes at the same time.
                    if prefix.is_empty() && self.node.age() < FIRST_SECTION_MIN_AGE {
                        let age: u8 = (FIRST_SECTION_MIN_AGE..FIRST_SECTION_MAX_AGE)
                            .choose(&mut rand::thread_rng())
                            .unwrap_or(FIRST_SECTION_MAX_AGE);

                        let new_keypair =
                            ed25519::gen_keypair(&Prefix::default().range_inclusive(), age);
                        let new_name = ed25519::name(&new_keypair.public);

                        info!("Setting Node name to {}", new_name);
                        self.node = Node::new(new_keypair, self.node.addr);
                    }

                    if prefix.matches(&self.node.name()) {
                        // After section split, new node must join with the age of MIN_ADULT_AGE.
                        if !prefix.is_empty() && self.node.age() != MIN_ADULT_AGE {
                            let new_keypair =
                                ed25519::gen_keypair(&prefix.range_inclusive(), MIN_ADULT_AGE);
                            let new_name = ed25519::name(&new_keypair.public);

                            info!("Setting Node name to {}", new_name);
                            self.node = Node::new(new_keypair, self.node.addr);
                        }

                        info!(
                            "Newer Join response for our prefix {:?} from {:?}",
                            section_auth, sender
                        );
                        section_key = section_auth.section_key();
                        let join_request = JoinRequest {
                            section_key,
                            resource_proof_response: None,
                        };

                        recipients = new_recipients;
                        self.send_join_requests(join_request, &recipients, section_key)
                            .await?;
                    } else {
                        warn!(
                            "Newer Join response not for our prefix {:?} from {:?}",
                            section_auth, sender,
                        );
                    }
                }
                JoinResponse::Redirect(section_auth) => {
                    if section_auth.section_key() == section_key {
                        continue;
                    }

                    // Ignore already used recipients
                    let new_recipients: Vec<(XorName, SocketAddr)> = section_auth
                        .elders
                        .iter()
                        .filter(|(_, addr)| !used_recipient.contains(addr))
                        .map(|(name, addr)| (*name, *addr))
                        .collect();

                    if new_recipients.is_empty() {
                        debug!("Joining redirected to the same set of peers we already contacted - ignoring response");
                        continue;
                    } else {
                        info!(
                            "Joining redirected to another set of peers: {:?}",
                            new_recipients,
                        );
                    }

                    if section_auth.prefix.matches(&self.node.name()) {
                        info!(
                            "Newer Join response for our prefix {:?} from {:?}",
                            section_auth, sender
                        );
                        section_key = section_auth.section_key();
                        let join_request = JoinRequest {
                            section_key,
                            resource_proof_response: None,
                        };

                        recipients = new_recipients;
                        self.send_join_requests(join_request, &recipients, section_key)
                            .await?;
                    } else {
                        warn!(
                            "Newer Join response not for our prefix {:?} from {:?}",
                            section_auth, sender,
                        );
                    }
                }
                JoinResponse::ResourceChallenge {
                    data_size,
                    difficulty,
                    nonce,
                    nonce_signature,
                } => {
                    let rp = ResourceProof::new(data_size, difficulty);
                    let data = rp.create_proof_data(&nonce);
                    let mut prover = rp.create_prover(data.clone());
                    let solution = prover.solve();

                    let join_request = JoinRequest {
                        section_key,
                        resource_proof_response: Some(ResourceProofResponse {
                            solution,
                            data,
                            nonce,
                            nonce_signature,
                        }),
                    };
                    let recipients = &[(src_name, sender)];
                    self.send_join_requests(join_request, recipients, section_key)
                        .await?;
                }
            }
        }
    }

    async fn send_join_requests(
        &mut self,
        join_request: JoinRequest,
        recipients: &[(XorName, SocketAddr)],
        section_key: bls::PublicKey,
    ) -> Result<()> {
        info!("Sending {:?} to {:?}", join_request, recipients);

        let node_msg = NodeMsg::JoinRequest(Box::new(join_request));
        let wire_msg = WireMsg::single_src(
            &self.node,
            DstLocation::DirectAndUnrouted(section_key),
            node_msg,
            section_key,
        )?;

        let _ = self.send_tx.send((wire_msg, recipients.to_vec())).await;

        Ok(())
    }

    // TODO: receive JoinResponse from the JoinResponse handler directly,
    // analogous to the JoinAsRelocated flow.
    async fn receive_join_response(&mut self) -> Result<(JoinResponse, SocketAddr, XorName)> {
        while let Some(event) = self.recv_rx.recv().await {
            // we are interested only in `JoinResponse` type of messages
            let (join_response, sender, src_name) = match event {
                ConnectionEvent::Received((sender, bytes)) => match WireMsg::from(bytes) {
                    Ok(wire_msg) => match wire_msg.msg_kind() {
                        MsgKind::ClientMsg(_) | MsgKind::SectionInfoMsg => continue,
                        MsgKind::NodeBlsShareSignedMsg(_) | MsgKind::SectionSignedMsg(_) => {
                            self.backlog_message(Command::HandleMessage { sender, wire_msg });
                            continue;
                        }
                        MsgKind::NodeSignedMsg(NodeSigned { public_key, .. }) => {
                            // TOOD: find a way we don't need to reconstruct the WireMsg
                            let msg_id = wire_msg.msg_id();
                            let payload = wire_msg.payload.clone();
                            let msg_kind = wire_msg.msg_kind().clone();
                            let dst_location = wire_msg.dst_location().clone();
                            let pk = public_key.clone();
                            match wire_msg.to_message() {
                                Ok(MessageType::Node {
                                    msg: NodeMsg::JoinResponse(resp),
                                    ..
                                }) => (*resp, sender, ed25519::name(&pk)),
                                Ok(
                                    MessageType::Client { .. }
                                    | MessageType::SectionInfo { .. }
                                    | MessageType::Node { .. },
                                ) => {
                                    // We just put the WireMsg in the backlog then
                                    if let Ok(wire_msg) =
                                        WireMsg::new_msg(msg_id, payload, msg_kind, dst_location)
                                    {
                                        self.backlog_message(Command::HandleMessage {
                                            sender,
                                            wire_msg,
                                        });
                                    }
                                    continue;
                                }
                                Err(err) => {
                                    debug!("Failed to deserialize message payload: {}", err);
                                    continue;
                                }
                            }
                        }
                    },
                    Err(err) => {
                        debug!("Failed to deserialize message: {}", err);
                        continue;
                    }
                },
                ConnectionEvent::Disconnected(_) => continue,
            };

            match join_response {
                JoinResponse::ResourceChallenge { .. }
                | JoinResponse::Rejected(JoinRejectionReason::NodeNotReachable(_))
                | JoinResponse::Rejected(JoinRejectionReason::JoinsDisallowed) => {
                    return Ok((join_response, sender, src_name));
                }
                JoinResponse::Retry(ref section_auth)
                | JoinResponse::Redirect(ref section_auth) => {
                    if section_auth.elders.is_empty() {
                        error!(
                            "Invalid JoinResponse::Retry/Redirect, empty list of Elders: {:?}",
                            join_response
                        );
                        continue;
                    }
                    trace!("Received a redirect/retry JoinResponse. Sending request to the latest contacts");

                    return Ok((join_response, sender, src_name));
                }
                JoinResponse::Approval {
                    ref section_auth,
                    ref node_state,
                    ref section_chain,
                    ..
                } => {
                    if node_state.value.peer.name() != &self.node.name() {
                        trace!("Ignore NodeApproval not for us");
                        continue;
                    }

                    if !section_auth.verify(section_chain) {
                        error!(
                            "Verification of section authority failed: {:?}",
                            join_response
                        );
                        continue;
                    }

                    if !node_state.verify(section_chain) {
                        error!("Verification of node state failed: {:?}", join_response);
                        continue;
                    }

                    trace!(
                        "This node has been approved to join the network at {:?}!",
                        section_auth.value.prefix,
                    );

                    return Ok((join_response, sender, src_name));
                }
            }
        }

        error!("NodeMsg sender unexpectedly closed");
        // TODO: consider more specific error here (e.g. `BootstrapInterrupted`)
        Err(Error::InvalidState)
    }

    fn backlog_message(&mut self, cmd: Command) {
        while self.backlog.len() >= BACKLOG_CAPACITY {
            let _ = self.backlog.pop_front();
        }

        self.backlog.push_back(cmd)
    }
}

// Keep reading messages from `rx` and send them using `comm`.
async fn send_messages(
    mut rx: mpsc::Receiver<(WireMsg, Vec<(XorName, SocketAddr)>)>,
    comm: &Comm,
) -> Result<()> {
    while let Some((wire_msg, recipients)) = rx.recv().await {
        match comm
            .send(&recipients, recipients.len(), wire_msg.clone())
            .await
        {
            Ok(SendStatus::AllRecipients) | Ok(SendStatus::MinDeliveryGroupSizeReached(_)) => {}
            Ok(SendStatus::MinDeliveryGroupSizeFailed(recipients)) => {
                error!("Failed to send message {:?} to {:?}", wire_msg, recipients)
            }
            Err(err) => {
                error!(
                    "Failed to send message {:?} to {:?}: {:?}",
                    wire_msg, recipients, err
                )
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messaging::{node::NodeState, SectionAuthorityProvider};
    use crate::routing::{
        dkg::test_utils::*,
        error::Error as RoutingError,
        messages::WireMsgUtils,
        section::test_utils::*,
        section::{NodeStateUtils, SectionAuthorityProviderUtils},
        ELDER_SIZE, MIN_ADULT_AGE, MIN_AGE,
    };
    use anyhow::{anyhow, Error, Result};
    use assert_matches::assert_matches;
    use futures::{
        future::{self, Either},
        pin_mut,
    };
    use secured_linked_list::SecuredLinkedList;
    use std::collections::BTreeMap;
    use tokio::task;

    #[tokio::test]
    async fn join_as_adult() -> Result<()> {
        let (send_tx, mut send_rx) = mpsc::channel(1);
        let (recv_tx, mut recv_rx) = mpsc::channel(1);

        let (section_auth, mut nodes, sk_set) =
            gen_section_authority_provider(Prefix::default(), ELDER_SIZE);
        let bootstrap_node = nodes.remove(0);
        let bootstrap_addr = bootstrap_node.addr;

        let sk = sk_set.secret_key();
        let pk = sk.public_key();

        // Node in first section has to have an age higher than MIN_ADULT_AGE
        // Otherwise during the bootstrap process, node will change its id and age.
        let node_age = MIN_AGE + 2;
        let node = Node::new(
            ed25519::gen_keypair(&Prefix::default().range_inclusive(), node_age),
            gen_addr(),
        );
        let peer = node.peer();
        let state = Join::new(node, send_tx, &mut recv_rx);

        // Create the bootstrap task, but don't run it yet.
        let bootstrap = async move { state.run(bootstrap_addr).await.map_err(Error::from) };

        // Create the task that executes the body of the test, but don't run it either.
        let others = async {
            // Receive JoinRequest
            let (message, recipients) = send_rx
                .recv()
                .await
                .ok_or_else(|| anyhow!("JoinRequest was not received"))?;

            let bootstrap_addrs: Vec<SocketAddr> =
                recipients.iter().map(|(_name, addr)| *addr).collect();
            assert_eq!(bootstrap_addrs, [bootstrap_addr]);

            let (message, dst_info) = assert_matches!(message, MessageType::Routing { msg, dst_info } =>
                (msg, dst_info));

            assert_eq!(dst_info.dst, *peer.name());
            assert_matches!(message.variant, NodeMsg::JoinRequest(request) => {
                assert!(request.resource_proof_response.is_none());
            });

            // Send JoinResponse::Retry with section auth provider info
            send_response(
                &recv_tx,
                NodeMsg::JoinResponse(Box::new(JoinResponse::Retry(section_auth.clone()))),
                &bootstrap_node,
                section_auth.section_key(),
                *peer.name(),
            )?;

            // Receive the second JoinRequest with correct section info
            let (message, recipients) = send_rx
                .recv()
                .await
                .ok_or_else(|| anyhow!("JoinRequest was not received"))?;
            let (message, dst_info) = assert_matches!(message, MessageType::Routing { msg, dst_info } =>
                (msg, dst_info));

            assert_eq!(dst_info.dst_section_pk, pk);
            itertools::assert_equal(
                recipients,
                section_auth
                    .elders()
                    .iter()
                    .map(|(name, addr)| (*name, *addr))
                    .collect::<Vec<_>>(),
            );
            assert_matches!(message.variant, NodeMsg::JoinRequest(request) => {
                assert_eq!(request.section_key, pk);
            });

            // Send JoinResponse::Approval
            let section_auth = section_signed(sk, section_auth.clone())?;
            let node_state = section_signed(sk, NodeState::joined(peer))?;
            let proof_chain = SecuredLinkedList::new(pk);
            send_response(
                &recv_tx,
                NodeMsg::JoinResponse(Box::new(JoinResponse::Approval {
                    genesis_key: pk,
                    section_auth: section_auth.clone(),
                    node_state,
                    section_chain: proof_chain,
                })),
                &bootstrap_node,
                section_auth.value.section_key(),
                *peer.name(),
            )?;

            Ok(())
        };

        // Drive both tasks to completion concurrently (but on the same thread).
        let ((node, section, _backlog), _) = future::try_join(bootstrap, others).await?;

        assert_eq!(*section.authority_provider(), section_auth);
        assert_eq!(*section.chain().last_key(), pk);
        assert_eq!(node.age(), node_age);

        Ok(())
    }

    #[tokio::test]
    async fn join_receive_redirect_response() -> Result<()> {
        let (send_tx, mut send_rx) = mpsc::channel(1);
        let (recv_tx, mut recv_rx) = mpsc::channel(1);

        let (section_auth, mut nodes, sk_set) =
            gen_section_authority_provider(Prefix::default(), ELDER_SIZE);
        let bootstrap_node = nodes.remove(0);
        let pk_set = sk_set.public_keys();

        let node = Node::new(
            ed25519::gen_keypair(&Prefix::default().range_inclusive(), MIN_ADULT_AGE),
            gen_addr(),
        );
        let name = node.name();
        let state = Join::new(node, send_tx, &mut recv_rx);

        let bootstrap_task = state.run(bootstrap_node.addr);
        let test_task = async move {
            // Receive JoinRequest
            let (message, recipients) = send_rx
                .recv()
                .await
                .ok_or_else(|| anyhow!("JoinRequest was not received"))?;

            assert_eq!(
                recipients
                    .into_iter()
                    .map(|peer| peer.1)
                    .collect::<Vec<_>>(),
                vec![bootstrap_node.addr]
            );

            assert_matches!(message, MessageType::Routing { msg, .. } =>
                assert_matches!(msg.variant, NodeMsg::JoinRequest{..}));

            // Send JoinResponse::Redirect
            let new_bootstrap_addrs: BTreeMap<_, _> = (0..ELDER_SIZE)
                .map(|_| (XorName::random(), gen_addr()))
                .collect();

            send_response(
                &recv_tx,
                NodeMsg::JoinResponse(Box::new(JoinResponse::Redirect(SectionAuthorityProvider {
                    prefix: Prefix::default(),
                    public_key_set: pk_set.clone(),
                    elders: new_bootstrap_addrs.clone(),
                }))),
                &bootstrap_node,
                section_auth.section_key(),
                name,
            )?;
            task::yield_now().await;

            // Receive new JoinRequest with redirected bootstrap contacts
            let (message, recipients) = send_rx
                .recv()
                .await
                .ok_or_else(|| anyhow!("JoinRequest was not received"))?;

            assert_eq!(
                recipients
                    .into_iter()
                    .map(|peer| peer.1)
                    .collect::<Vec<_>>(),
                new_bootstrap_addrs
                    .iter()
                    .map(|(_, addr)| *addr)
                    .collect::<Vec<_>>()
            );

            let (message, dst_info) = assert_matches!(message, MessageType::Routing { msg, dst_info } =>
                (msg, dst_info));

            assert_eq!(dst_info.dst_section_pk, pk_set.public_key());
            assert_matches!(message.variant, NodeMsg::JoinRequest(req) => {
                assert_eq!(req.section_key, pk_set.public_key());
            });

            Ok(())
        };

        pin_mut!(bootstrap_task);
        pin_mut!(test_task);

        match future::select(bootstrap_task, test_task).await {
            Either::Left(_) => unreachable!(),
            Either::Right((output, _)) => output,
        }
    }

    #[tokio::test]
    async fn join_invalid_redirect_response() -> Result<()> {
        let (send_tx, mut send_rx) = mpsc::channel(1);
        let (recv_tx, mut recv_rx) = mpsc::channel(1);

        let (section_auth, mut nodes, sk_set) =
            gen_section_authority_provider(Prefix::default(), ELDER_SIZE);
        let bootstrap_node = nodes.remove(0);
        let pk_set = sk_set.public_keys();

        let node = Node::new(
            ed25519::gen_keypair(&Prefix::default().range_inclusive(), MIN_ADULT_AGE),
            gen_addr(),
        );
        let node_name = node.name();
        let state = Join::new(node, send_tx, &mut recv_rx);

        let bootstrap_task = state.run(bootstrap_node.addr);
        let test_task = async {
            let (message, _) = send_rx
                .recv()
                .await
                .ok_or_else(|| anyhow!("JoinRequest was not received"))?;

            assert_matches!(message, MessageType::Routing { msg, .. } =>
                    assert_matches!(msg.variant, NodeMsg::JoinRequest{..}));

            send_response(
                &recv_tx,
                NodeMsg::JoinResponse(Box::new(JoinResponse::Redirect(SectionAuthorityProvider {
                    prefix: Prefix::default(),
                    public_key_set: pk_set.clone(),
                    elders: BTreeMap::new(),
                }))),
                &bootstrap_node,
                section_auth.section_key(),
                node_name,
            )?;
            task::yield_now().await;

            let addrs = (0..ELDER_SIZE)
                .map(|_| (XorName::random(), gen_addr()))
                .collect();

            send_response(
                &recv_tx,
                NodeMsg::JoinResponse(Box::new(JoinResponse::Redirect(SectionAuthorityProvider {
                    prefix: Prefix::default(),
                    public_key_set: pk_set.clone(),
                    elders: addrs,
                }))),
                &bootstrap_node,
                section_auth.section_key(),
                node_name,
            )?;
            task::yield_now().await;

            let (message, _) = send_rx
                .recv()
                .await
                .ok_or_else(|| anyhow!("JoinRequest was not received"))?;

            assert_matches!(message, MessageType::Routing { msg, .. } =>
                        assert_matches!(msg.variant, NodeMsg::JoinRequest{..}));

            Ok(())
        };

        pin_mut!(bootstrap_task);
        pin_mut!(test_task);

        match future::select(bootstrap_task, test_task).await {
            Either::Left(_) => unreachable!(),
            Either::Right((output, _)) => output,
        }
    }

    #[tokio::test]
    async fn join_disallowed_response() -> Result<()> {
        let (send_tx, mut send_rx) = mpsc::channel(1);
        let (recv_tx, mut recv_rx) = mpsc::channel(1);

        let (section_auth, mut nodes, _) =
            gen_section_authority_provider(Prefix::default(), ELDER_SIZE);
        let bootstrap_node = nodes.remove(0);

        let node = Node::new(
            ed25519::gen_keypair(&Prefix::default().range_inclusive(), MIN_ADULT_AGE),
            gen_addr(),
        );

        let node_name = node.name();
        let state = Join::new(node, send_tx, &mut recv_rx);

        let bootstrap_task = state.run(bootstrap_node.addr);
        let test_task = async {
            let (message, _) = send_rx
                .recv()
                .await
                .ok_or_else(|| anyhow!("JoinRequest was not received"))?;

            assert_matches!(message, MessageType::Routing { msg, .. } =>
                            assert_matches!(msg.variant, NodeMsg::JoinRequest{..}));

            send_response(
                &recv_tx,
                NodeMsg::JoinResponse(Box::new(JoinResponse::Rejected(
                    JoinRejectionReason::JoinsDisallowed,
                ))),
                &bootstrap_node,
                section_auth.section_key(),
                node_name,
            )?;

            Ok(())
        };

        let (join_result, test_result) = future::join(bootstrap_task, test_task).await;

        if let Err(RoutingError::TryJoinLater) = join_result {
        } else {
            return Err(anyhow!("Not getting an execpted network rejection."));
        }

        test_result
    }

    #[tokio::test]
    async fn join_invalid_retry_prefix_response() -> Result<()> {
        let (send_tx, mut send_rx) = mpsc::channel(1);
        let (recv_tx, mut recv_rx) = mpsc::channel(1);

        let bootstrap_node = Node::new(
            ed25519::gen_keypair(&Prefix::default().range_inclusive(), MIN_ADULT_AGE),
            gen_addr(),
        );

        let node = Node::new(
            ed25519::gen_keypair(&Prefix::default().range_inclusive(), MIN_ADULT_AGE),
            gen_addr(),
        );
        let node_name = node.name();

        let (good_prefix, bad_prefix) = {
            let p0 = Prefix::default().pushed(false);
            let p1 = Prefix::default().pushed(true);

            if node.name().bit(0) {
                (p1, p0)
            } else {
                (p0, p1)
            }
        };

        let state = Join::new(node, send_tx, &mut recv_rx);

        let section_key = bls::SecretKey::random().public_key();
        let elders = (0..ELDER_SIZE)
            .map(|_| (good_prefix.substituted_in(rand::random()), gen_addr()))
            .collect();
        let join_task = state.join(section_key, elders);

        let test_task = async {
            let (message, _) = send_rx
                .recv()
                .await
                .ok_or_else(|| anyhow!("NodeMsg was not received"))?;

            let message = assert_matches!(message, MessageType::Routing{ msg, .. } => msg);
            assert_matches!(message.variant, NodeMsg::JoinRequest(_));

            // Send `Retry` with bad prefix
            send_response(
                &recv_tx,
                NodeMsg::JoinResponse(Box::new(JoinResponse::Retry(
                    gen_section_authority_provider(bad_prefix, ELDER_SIZE).0,
                ))),
                &bootstrap_node,
                section_key,
                node_name,
            )?;
            task::yield_now().await;

            // Send `Retry` with good prefix
            send_response(
                &recv_tx,
                NodeMsg::JoinResponse(Box::new(JoinResponse::Retry(
                    gen_section_authority_provider(good_prefix, ELDER_SIZE).0,
                ))),
                &bootstrap_node,
                section_key,
                node_name,
            )?;

            let (message, _) = send_rx
                .recv()
                .await
                .ok_or_else(|| anyhow!("NodeMsg was not received"))?;

            let message = assert_matches!(message, MessageType::Routing{ msg, .. } => msg);
            assert_matches!(message.variant, NodeMsg::JoinRequest(_));

            Ok(())
        };

        pin_mut!(join_task);
        pin_mut!(test_task);

        match future::select(join_task, test_task).await {
            Either::Left(_) => unreachable!(),
            Either::Right((output, _)) => output,
        }
    }

    // test helper
    fn send_response(
        recv_tx: &mpsc::Sender<ConnectionEvent>,
        variant: NodeMsg,
        bootstrap_node: &Node,
        section_key: bls::PublicKey,
        node_name: XorName,
    ) -> Result<()> {
        let message = NodeMsg::single_src(
            bootstrap_node,
            DstLocation::DirectAndUnrouted,
            variant,
            section_key,
        )?;

        recv_tx.try_send(ConnectionEvent::Received((
            bootstrap_node.addr,
            MessageType::Routing {
                msg: message,
                dst_info: DstInfo {
                    dst: node_name,
                    dst_section_pk: section_key,
                },
            }
            .serialize()?,
        )))?;

        Ok(())
    }
}
