// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::UsedRecipientSaps;

use crate::comm::{Comm, MsgFromPeer};
use crate::log_sleep;
use crate::node::{messages::WireMsgUtils, Error, Result};

use sn_interface::{
    messaging::{
        system::{JoinRejectionReason, JoinRequest, JoinResponse, NodeMsg, SectionSigned},
        Dst, MsgType, WireMsg,
    },
    network_knowledge::{
        MembershipState, MyNodeInfo, NetworkKnowledge, SectionTree, SectionTreeUpdate,
    },
    types::{keys::ed25519, log_markers::LogMarker, Peer},
    SectionAuthorityProvider,
};

use backoff::{backoff::Backoff, ExponentialBackoff};
use bls::PublicKey as BlsPublicKey;
use futures::future;

use std::collections::BTreeSet;
use tokio::{sync::mpsc, time::Duration};
use tracing::Instrument;
use xor_name::Prefix;

/// Join the network as new node.
///
/// NOTE: It's not guaranteed this function ever returns. This can happen due to messages being
/// lost in transit or other reasons. It's the responsibility of the caller to handle this case,
/// for example by using a timeout.
pub(crate) async fn join_network(
    node: MyNodeInfo,
    comm: &Comm,
    incoming_msgs: &mut mpsc::Receiver<MsgFromPeer>,
    section_tree: SectionTree,
    join_timeout: Duration,
) -> Result<(MyNodeInfo, NetworkKnowledge)> {
    let (outgoing_msgs_sender, outgoing_msgs_receiver) = mpsc::channel(100);

    let span = trace_span!("bootstrap");
    let joiner = Joiner::new(node, outgoing_msgs_sender, incoming_msgs, section_tree);

    let (res, _) = future::join(
        joiner.try_join(join_timeout),
        send_messages(outgoing_msgs_receiver, comm),
    )
    .instrument(span)
    .await;

    match res {
        Ok(node) => Ok(node),
        Err(error) => {
            // We need to manually closing endpoint or listeners will persist
            comm.our_endpoint.close();
            Err(error)
        }
    }
}

struct Joiner<'a> {
    // Sender for outgoing messages.
    outgoing_msgs: mpsc::Sender<(WireMsg, Vec<Peer>)>,
    // Receiver for incoming messages.
    incoming_msgs: &'a mut mpsc::Receiver<MsgFromPeer>,
    node: MyNodeInfo,
    prefix: Prefix,
    section_tree: SectionTree,
    backoff: ExponentialBackoff,
    /// cache of retry response sending peers. When we exceed 1/3rd responses we retry
    /// (the rety_response cache makes sure we retry only once per name/sap)
    retry_responses_cache: BTreeSet<Peer>,
    /// Cache SAPs we have retried for to prevent repeated retries to same cache (if more responses come in later eg)
    retry_sap_cache: Vec<SectionSigned<SectionAuthorityProvider>>,
}

impl<'a> Joiner<'a> {
    fn new(
        node: MyNodeInfo,
        outgoing_msgs: mpsc::Sender<(WireMsg, Vec<Peer>)>,
        incoming_msgs: &'a mut mpsc::Receiver<MsgFromPeer>,
        section_tree: SectionTree,
    ) -> Self {
        let mut backoff = ExponentialBackoff {
            initial_interval: Duration::from_millis(50),
            max_interval: Duration::from_millis(750),
            max_elapsed_time: Some(Duration::from_secs(60)),
            ..Default::default()
        };

        // this seems needed for custom settings to take effect
        backoff.reset();

        Self {
            outgoing_msgs,
            incoming_msgs,
            node,
            prefix: Prefix::default(),
            section_tree,
            backoff,
            retry_responses_cache: Default::default(),
            retry_sap_cache: Default::default(),
        }
    }

    // Send `JoinRequest` and wait for the response. If the response is:
    // - `Retry`: repeat with the new info.
    // - `Redirect`: repeat with the new set of addresses.
    // - `ResourceChallenge`: carry out resource proof calculation.
    // - `Approval`: returns the initial `Section` value to use by this node,
    //    completing the bootstrap.
    async fn try_join(self, join_timeout: Duration) -> Result<(MyNodeInfo, NetworkKnowledge)> {
        trace!(
            "Bootstrap run, section tree as we have it: {:?}",
            self.section_tree
        );

        tokio::time::timeout(join_timeout, self.join(join_timeout / 10))
            .await
            .map_err(|e| {
                error!("Failed join: {:?}", e);
                Error::JoinTimeout
            })?
    }

    fn join_target(&self) -> Result<(BlsPublicKey, Vec<Peer>)> {
        let our_name = self.node.name();
        let sap = self.section_tree.section_by_name(&our_name)?;
        Ok((sap.section_key(), sap.elders_vec()))
    }

    #[tracing::instrument(skip(self))]
    async fn join(mut self, response_timeout: Duration) -> Result<(MyNodeInfo, NetworkKnowledge)> {
        self.bootstrap_section_tree(response_timeout).await?;

        let (target_section_key, recipients) = self.join_target()?;

        debug!(
            "Initiating join as node_name {:?} with {recipients:?}",
            self.node.name()
        );

        // We first use genesis key as the target section key, we'll be getting
        // a response with the latest section key for us to retry with.
        // Once we are approved to join, we will make sure the SAP we receive can
        // be validated with the received proof chain and the 'network_genesis_key'.
        let mut section_key = target_section_key;

        // We send a first join request to obtain the resource challenge, which
        // we will then use to generate the challenge proof and send the
        // `JoinRequest` again with it.
        let msg = JoinRequest { section_key };

        self.send(NodeMsg::JoinRequest(msg), &recipients, section_key, false)
            .await?;

        // Avoid sending more than one duplicated request (with same SectionKey) to the same peer.
        let mut used_recipient_saps = UsedRecipientSaps::new();

        loop {
            let (response, sender) =
                tokio::time::timeout(response_timeout, self.receive_join_response())
                    .await
                    .map_err(|e| {
                        error!("Failed to receive join response: {:?}", e);
                        Error::JoinTimeout
                    })??;

            match response {
                JoinResponse::Approved {
                    section_tree_update,
                    decision,
                } => {
                    info!("{}", LogMarker::ReceivedJoinApproval);
                    if let Err(e) =
                        decision.validate(&section_tree_update.signed_sap.public_key_set())
                    {
                        error!("Dropping invalid join decision: {e:?}");
                        continue;
                    }

                    // Ensure this decision includes us as a joining node
                    if decision
                        .proposals
                        .keys()
                        .filter(|n| n.state() == MembershipState::Joined)
                        .all(|n| n.name() != self.node.name())
                    {
                        trace!("MyNode named: {:?} Ignore join approval decision not for us: {decision:?}", self.node.name());
                        continue;
                    }

                    trace!(
                        "=========>> This node has been approved to join the network at {:?}!",
                        section_tree_update.signed_sap.prefix(),
                    );

                    // Building our network knowledge instance will validate the section_tree_update

                    let network_knowledge =
                        NetworkKnowledge::new(self.section_tree, section_tree_update)?;

                    return Ok((self.node, network_knowledge));
                }
                JoinResponse::Retry {
                    section_tree_update,
                } => {
                    let signed_sap = section_tree_update.signed_sap.clone();

                    trace!(
                        "My joining node with {:?} - name: {:?} ; received a Retry from {}, SAP: {}, proof_chain: {:?}",
                        self.prefix,
                        self.node.name(),
                        sender.name(),
                        signed_sap.value,
                        section_tree_update.proof_chain
                    );

                    let prefix = signed_sap.prefix();
                    if !prefix.matches(&self.node.name()) {
                        warn!(
                            "Ignoring newer JoinResponse::Retry response not for us {:?}, SAP {signed_sap:?} from {sender:?}",
                            self.node.name()
                        );
                        continue;
                    }

                    // make sure we received a valid and trusted new SAP
                    let is_new_sap = match self.section_tree.update(section_tree_update) {
                        Ok(updated) => updated,
                        Err(err) => {
                            debug!("Ignoring section tree updated in JoinResponse::Retry with an invalid or known SAP: {err:?}");
                            continue;
                        }
                    };

                    if is_new_sap
                        || self.should_retry_after_response(sender, signed_sap.elders_set())
                    {
                        let already_retried_for_this_sap =
                            self.retry_sap_cache.contains(&signed_sap);

                        if already_retried_for_this_sap {
                            info!("We have already triggered a retry flow for this sap: {signed_sap:?}");
                            continue;
                        } else {
                            self.retry_sap_cache.push(signed_sap.clone())
                        }

                        trace!("Re-generating name for retry");
                        let new_keypair =
                            ed25519::gen_keypair(&prefix.range_inclusive(), self.node.age());
                        let new_name = ed25519::name(&new_keypair.public);

                        info!("Setting Node name to {new_name}");
                        self.node = MyNodeInfo::new(new_keypair, self.node.addr);
                        self.retry_responses_cache = Default::default();

                        section_key = signed_sap.section_key();

                        let msg = JoinRequest { section_key };
                        let new_recipients = signed_sap.elders_vec();

                        self.send(
                            NodeMsg::JoinRequest(msg),
                            &new_recipients,
                            section_key,
                            true,
                        )
                        .await?;
                    }
                }
                JoinResponse::Redirect(section_auth) => {
                    trace!("Received a redirect JoinResponse from {}. Sending request to the latest contacts", sender);
                    if section_auth.elders().next().is_none() {
                        error!(
                            "Invalid JoinResponse::Redirect, empty list of Elders: {:?}",
                            section_auth
                        );
                        continue;
                    }

                    if !section_auth.prefix().matches(&self.node.name()) {
                        warn!(
                            "Ignoring newer JoinResponse::Redirect response not for us {:?}, SAP {:?} from {:?}",
                            self.node.name(),
                            section_auth,
                            sender,
                        );
                        continue;
                    }

                    let new_section_key = section_auth.section_key();
                    let new_recipients: Vec<_> = section_auth
                        .elders()
                        .filter(|peer| used_recipient_saps.insert((peer.addr(), new_section_key)))
                        .cloned()
                        .collect();

                    if new_recipients.is_empty() {
                        debug!(
                            "Ignoring JoinResponse::Redirect with old SAP that has been sent to: {:?}",
                            section_auth
                        );
                        continue;
                    }

                    info!(
                        "Newer JoinResponse::Redirect for us {:?}, SAP {:?} from {:?}",
                        self.node.name(),
                        section_auth,
                        sender
                    );

                    section_key = new_section_key;
                    self.prefix = section_auth.prefix();

                    let msg = JoinRequest { section_key };

                    self.send(
                        NodeMsg::JoinRequest(msg),
                        &new_recipients,
                        section_key,
                        true,
                    )
                    .await?;
                }
                JoinResponse::Rejected(JoinRejectionReason::JoinsDisallowed) => {
                    error!("Network is set to not taking any new joining node, try join later.");
                    return Err(Error::TryJoinLater);
                }
                JoinResponse::Rejected(JoinRejectionReason::NodeNotReachable(addr)) => {
                    error!("Join rejected since node is not externally reachable: {addr}");
                    return Err(Error::NodeNotReachable(addr));
                }
            }
        }
    }

    async fn bootstrap_section_tree(&mut self, response_timeout: Duration) -> Result<()> {
        loop {
            let (section_key, elders) = self.join_target()?;

            self.send(
                NodeMsg::AntiEntropyProbe(section_key),
                &elders,
                section_key,
                true,
            )
            .await?;

            let section_tree_update =
                tokio::time::timeout(response_timeout, self.receive_section_tree_update())
                    .await
                    .map_err(|_| Error::JoinTimeout)??;

            if !self.section_tree.update(section_tree_update)? {
                return Ok(());
            }
        }
    }

    // We'll restart the join process once we receive Retry responses from >1/3 of elders
    fn should_retry_after_response(&mut self, sender: Peer, elders: BTreeSet<Peer>) -> bool {
        if !elders.contains(&sender) {
            error!("Sender {sender:?} of the retry-response is not part of the elders {elders:?}");
            return false;
        }
        let _ = self.retry_responses_cache.insert(sender);

        self.retry_responses_cache.len() > elders.len() / 3
    }

    #[tracing::instrument(skip(self))]
    async fn send(
        &mut self,
        msg: NodeMsg,
        recipients: &[Peer],
        section_key: BlsPublicKey,
        should_backoff: bool,
    ) -> Result<()> {
        if should_backoff {
            // use exponential backoff here to delay our responses and avoid any intensive join reqs
            let next_wait = self.backoff.next_backoff();

            if let Some(wait) = next_wait {
                log_sleep!(Duration::from_millis(wait.as_millis() as u64));
            } else {
                error!("Waiting before attempting to join again");
                log_sleep!(Duration::from_millis(
                    self.backoff.max_interval.as_millis() as u64
                ));
                self.backoff.reset();
            }
        }

        info!(
            "Sending {msg:?} to {:?}",
            Vec::from_iter(recipients.iter().map(Peer::name))
        );

        let wire_msg = WireMsg::single_src(
            &self.node,
            Dst {
                name: self.node.name(), // we want to target a section where our name fits
                section_key,
            },
            msg,
        )?;

        let _res = self
            .outgoing_msgs
            .send((wire_msg, recipients.to_vec()))
            .await;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn receive_join_response(&mut self) -> Result<(JoinResponse, Peer)> {
        loop {
            let (msg, sender) = self.receive_node_msg().await?;
            match msg {
                NodeMsg::JoinResponse(resp) => return Ok((*resp, sender)),
                _ => {
                    trace!("Bootstrap message discarded: sender: {sender:?} msg: {msg:?}")
                }
            }
        }
    }

    #[tracing::instrument(skip(self))]
    async fn receive_section_tree_update(&mut self) -> Result<SectionTreeUpdate> {
        loop {
            let (msg, sender) = self.receive_node_msg().await?;
            match msg {
                NodeMsg::AntiEntropy {
                    section_tree_update,
                    ..
                } => return Ok(section_tree_update),
                _ => {
                    trace!("Bootstrap message discarded: sender: {sender:?} msg: {msg:?}")
                }
            }
        }
    }

    #[tracing::instrument(skip(self))]
    async fn receive_node_msg(&mut self) -> Result<(NodeMsg, Peer)> {
        while let Some(MsgFromPeer {
            sender, wire_msg, ..
        }) = self.incoming_msgs.recv().await
        {
            // We are interested only in `Node` type of messages
            match wire_msg.into_msg()? {
                MsgType::Node { msg, .. } => return Ok((msg, sender)),
                MsgType::Client { msg_id, .. }
                | MsgType::ClientDataResponse { msg_id, .. }
                | MsgType::NodeDataResponse { msg_id, .. } => {
                    trace!("Non-NodeMsg bootstrap message discarded: sender: {sender:?} msg_id: {msg_id:?}")
                }
            };
        }

        error!("NodeMsg sender unexpectedly closed");
        Err(Error::BootstrapConnectionClosed)
    }
}

// Keep reading messages from `rx` and send them using `comm`.
async fn send_messages(
    mut outgoing_msgs: mpsc::Receiver<(WireMsg, Vec<Peer>)>,
    comm: &Comm,
) -> Result<()> {
    while let Some((msg, peers)) = outgoing_msgs.recv().await {
        for peer in peers {
            let dst = *msg.dst();
            let msg_id = msg.msg_id();

            let bytes = msg.serialize()?;
            match comm.send_out_bytes(peer, msg_id, bytes, None).await {
                Ok(()) => trace!("Msg {msg_id:?} sent on {dst:?}"),
                Err(error) => {
                    warn!("Error in comms when sending msg {msg_id:?} to peer {peer:?}: {error}")
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::{messages::WireMsgUtils, Error as RoutingError, MIN_ADULT_AGE};
    use assert_matches::assert_matches;
    use eyre::{eyre, Result};
    use futures::{
        future::{self, Either},
        pin_mut,
    };
    use itertools::Itertools;
    use sn_interface::{
        elder_count, init_logger,
        messaging::system::AntiEntropyKind,
        network_knowledge::{NodeState, SectionAuthorityProvider, SectionTreeUpdate, SectionsDAG},
        test_utils::*,
        types::PublicKey,
    };
    use tokio::task;
    use xor_name::XorName;

    const JOIN_TIMEOUT_SEC: u64 = 15;

    #[tokio::test]
    async fn join_as_adult() {
        init_logger();

        let join_timeout = Duration::from_secs(JOIN_TIMEOUT_SEC);
        let (send_tx, mut send_rx) = mpsc::channel(1);
        let (recv_tx, mut recv_rx) = mpsc::channel(10);

        let (genesis_sap, genesis_sk_set, ..) = TestSapBuilder::new(Prefix::default()).build();
        let genesis_sk = genesis_sk_set.secret_key();
        let genesis_pk = genesis_sk.public_key();

        let node = MyNodeInfo::new(
            ed25519::gen_keypair(&Prefix::default().range_inclusive(), MIN_ADULT_AGE),
            gen_addr(),
        );

        let signed_genesis_sap = TestKeys::get_section_signed(&genesis_sk, genesis_sap.clone());
        let mut tree = SectionTree::new(genesis_pk);
        assert!(tree.insert_without_chain(signed_genesis_sap));

        let state = Joiner::new(node.clone(), send_tx, &mut recv_rx, tree);

        // Create the bootstrap task, but don't run it yet.
        let bootstrap = async { state.try_join(join_timeout).await.expect("Failed to join") };

        let (next_sap, next_sk_set, next_elders, _) =
            TestSapBuilder::new(Prefix::default()).build();

        let next_section_key = next_sk_set.public_keys().public_key();
        let section_tree_update = TestSectionTree::get_section_tree_update(
            &TestKeys::get_section_signed(&next_sk_set.secret_key(), next_sap.clone()),
            &SectionsDAG::new(genesis_pk),
            &genesis_sk,
        );

        // Create the task that executes the body of the test, but don't run it either.
        let others = async {
            // First the joining node bootstraps it's network knowledge.
            // We expect two probes, one to the genesis elders, then another to the next sap elders.

            for expected_recipients in [genesis_sap.elders(), next_sap.elders()] {
                let (node_msg, _, recipients) = recv_node_msg(&mut send_rx).await;

                itertools::assert_equal(recipients, expected_recipients);
                assert_matches!(node_msg, NodeMsg::AntiEntropyProbe(_));

                info!("Received anti-entropy probe");

                let ae_update_msg = NodeMsg::AntiEntropy {
                    section_tree_update: section_tree_update.clone(),
                    kind: AntiEntropyKind::Update {
                        members: Default::default(),
                    },
                };

                send_node_msg(
                    &recv_tx,
                    ae_update_msg,
                    next_elders.first().expect("Should have at least one elder"),
                    next_sap.section_key(),
                );
            }

            // Receive the second JoinRequest with correct section info
            let (node_msg, dst, recipients) = recv_node_msg(&mut send_rx).await;

            itertools::assert_equal(recipients, next_sap.elders());

            assert_eq!(dst.section_key, next_section_key);
            assert_matches!(node_msg, NodeMsg::JoinRequest(JoinRequest{ section_key }) => {
                assert_eq!(section_key, next_section_key);
            });

            // Name changed
            let new_peer = Peer::new(dst.name, node.peer().addr());
            // Send JoinResponse::Approved
            let decision = section_decision(&next_sk_set, NodeState::joined(new_peer, None));
            send_join_response(
                &recv_tx,
                JoinResponse::Approved {
                    section_tree_update,
                    decision,
                },
                &next_elders[0],
                next_sap.section_key(),
            );
        };

        // Drive both tasks to completion concurrently (but on the same thread).
        let ((node, section), _) = future::join(bootstrap, others).await;

        assert_eq!(section.section_auth(), next_sap);
        assert_eq!(section.section_key(), next_section_key);
        assert_eq!(node.age(), MIN_ADULT_AGE);
    }

    #[tokio::test]
    async fn join_receive_redirect_response() -> Result<()> {
        init_logger();

        let join_timeout = Duration::from_secs(JOIN_TIMEOUT_SEC);
        let (send_tx, mut send_rx) = mpsc::channel(1);
        let (recv_tx, mut recv_rx) = mpsc::channel(1);

        let (genesis_sap, genesis_sk_set, genesis_nodes, _) =
            TestSapBuilder::new(Prefix::default()).build();
        let genesis_sk = genesis_sk_set.secret_key();
        let genesis_pk = genesis_sk.public_key();

        let node = MyNodeInfo::new(
            ed25519::gen_keypair(&Prefix::default().range_inclusive(), MIN_ADULT_AGE),
            gen_addr(),
        );

        let signed_genesis_sap = TestKeys::get_section_signed(&genesis_sk, genesis_sap.clone());
        let mut tree = SectionTree::new(genesis_pk);
        assert!(tree.insert_without_chain(signed_genesis_sap));

        let state = Joiner::new(node, send_tx, &mut recv_rx, tree);

        let bootstrap_task = state.try_join(join_timeout);
        let test_task = async move {
            // Receive JoinRequest
            let (node_msg, _, recipients) = recv_node_msg(&mut send_rx).await;
            itertools::assert_equal(recipients, genesis_sap.elders());

            assert_matches!(node_msg, NodeMsg::JoinRequest { .. });

            // Send JoinResponse::Redirect
            let (new_sap, ..) = TestSapBuilder::new(Prefix::default()).build();

            send_join_response(
                &recv_tx,
                JoinResponse::Redirect(new_sap.clone()),
                &genesis_nodes[0],
                new_sap.section_key(),
            );

            task::yield_now().await;

            // Receive new JoinRequest with redirected bootstrap contacts
            let (node_msg, dst, recipients) = recv_node_msg(&mut send_rx).await;

            itertools::assert_equal(recipients, new_sap.elders());

            assert_eq!(dst.section_key, new_sap.section_key());
            assert_matches!(node_msg, NodeMsg::JoinRequest(JoinRequest{ section_key }) => {
                assert_eq!(section_key, new_sap.section_key());
            });

            Ok(())
        };

        pin_mut!(bootstrap_task);
        pin_mut!(test_task);

        match future::select(bootstrap_task, test_task).await {
            Either::Left((res, _)) => panic!("Bootstrap should not have finished: {res:?}"),
            Either::Right((output, _)) => output,
        }
    }

    #[tokio::test]
    async fn join_invalid_redirect_response() -> Result<()> {
        init_logger();
        let _span = tracing::info_span!("join_invalid_redirect_response").entered();

        let join_timeout = Duration::from_secs(JOIN_TIMEOUT_SEC);
        let (send_tx, mut send_rx) = mpsc::channel(1);
        let (recv_tx, mut recv_rx) = mpsc::channel(1);

        let (genesis_sap, genesis_sk_set, genesis_nodes, _) =
            TestSapBuilder::new(Prefix::default()).build();
        let genesis_sk = genesis_sk_set.secret_key();
        let genesis_pk = genesis_sk.public_key();

        let node = MyNodeInfo::new(
            ed25519::gen_keypair(&Prefix::default().range_inclusive(), MIN_ADULT_AGE),
            gen_addr(),
        );

        let signed_genesis_sap = TestKeys::get_section_signed(&genesis_sk, genesis_sap.clone());
        let mut tree = SectionTree::new(genesis_pk);
        assert!(tree.insert_without_chain(signed_genesis_sap));

        let state = Joiner::new(node, send_tx, &mut recv_rx, tree);

        let bootstrap_task = state.try_join(join_timeout);
        let test_task = async {
            let (node_msg, _, _) = recv_node_msg(&mut send_rx).await;

            assert_matches!(node_msg, NodeMsg::JoinRequest { .. });

            let (new_sap, new_sk_set, ..) = TestSapBuilder::new(Prefix::default()).build();
            let new_pk_set = new_sk_set.public_keys();

            send_join_response(
                &recv_tx,
                JoinResponse::Redirect(SectionAuthorityProvider::new(
                    BTreeSet::new(),
                    Prefix::default(),
                    BTreeSet::new(),
                    new_pk_set.clone(),
                    0,
                )),
                &genesis_nodes[0],
                new_sap.section_key(),
            );
            task::yield_now().await;

            send_join_response(
                &recv_tx,
                JoinResponse::Redirect(new_sap.clone()),
                &genesis_nodes[0],
                new_sap.section_key(),
            );
            task::yield_now().await;

            let (node_msg, _, _) = recv_node_msg(&mut send_rx).await;
            assert_matches!(node_msg, NodeMsg::JoinRequest { .. });

            Ok(())
        };

        pin_mut!(bootstrap_task);
        pin_mut!(test_task);

        match future::select(bootstrap_task, test_task).await {
            Either::Left((res, _)) => panic!("Bootstrap should not have finished {res:?}"),
            Either::Right((output, _)) => output,
        }
    }

    #[tokio::test]
    async fn join_disallowed_response() -> Result<()> {
        let join_timeout = Duration::from_secs(JOIN_TIMEOUT_SEC);
        let (send_tx, mut send_rx) = mpsc::channel(1);
        let (recv_tx, mut recv_rx) = mpsc::channel(1);

        let (genesis_sap, genesis_sk_set, genesis_nodes, _) =
            TestSapBuilder::new(Prefix::default()).build();
        let genesis_sk = genesis_sk_set.secret_key();
        let genesis_pk = genesis_sk.public_key();

        let node = MyNodeInfo::new(
            ed25519::gen_keypair(&Prefix::default().range_inclusive(), MIN_ADULT_AGE),
            gen_addr(),
        );

        let signed_genesis_sap = TestKeys::get_section_signed(&genesis_sk, genesis_sap.clone());
        let mut tree = SectionTree::new(genesis_pk);
        assert!(tree.insert_without_chain(signed_genesis_sap));

        let state = Joiner::new(node, send_tx, &mut recv_rx, tree.clone());

        let bootstrap_task = state.try_join(join_timeout);
        let test_task = async {
            let (node_msg, _, _) = recv_node_msg(&mut send_rx).await;
            assert_matches!(node_msg, NodeMsg::AntiEntropyProbe { .. });

            let section_tree_update = tree
                .generate_section_tree_update(&prefix(""))
                .expect("Failed to create update");

            send_node_msg(
                &recv_tx,
                NodeMsg::AntiEntropy {
                    section_tree_update,
                    kind: AntiEntropyKind::Update {
                        members: Default::default(),
                    },
                },
                &genesis_nodes[0],
                genesis_sap.section_key(),
            );

            let (node_msg, _, _) = recv_node_msg(&mut send_rx).await;
            assert_matches!(node_msg, NodeMsg::JoinRequest { .. });

            send_join_response(
                &recv_tx,
                JoinResponse::Rejected(JoinRejectionReason::JoinsDisallowed),
                &genesis_nodes[0],
                genesis_sap.section_key(),
            );

            Ok(())
        };

        let (join_result, test_result) = future::join(bootstrap_task, test_task).await;

        if let Err(RoutingError::TryJoinLater) = join_result {
        } else {
            return Err(eyre!("Not getting an execpted network rejection."));
        }

        test_result
    }

    #[tokio::test]
    async fn join_invalid_retry_prefix_response() -> Result<()> {
        init_logger();
        let _span = tracing::info_span!("join_invalid_retry_prefix_response").entered();

        let join_timeout = Duration::from_secs(JOIN_TIMEOUT_SEC);
        let (send_tx, mut send_rx) = mpsc::channel(1);
        let (recv_tx, mut recv_rx) = mpsc::channel(10);

        let node = MyNodeInfo::new(
            ed25519::gen_keypair(&Prefix::default().range_inclusive(), MIN_ADULT_AGE),
            gen_addr(),
        );

        let first_bit = node.name().bit(0);
        let bad_prefix = Prefix::default().pushed(!first_bit);

        let (genesis_sap, genesis_sk_set, genesis_nodes, _) =
            TestSapBuilder::new(Prefix::default())
                .elder_count(1)
                .build();
        let genesis_sk = genesis_sk_set.secret_key();
        let genesis_pk = genesis_sk.public_key();

        let signed_genesis_sap = TestKeys::get_section_signed(&genesis_sk, genesis_sap.clone());
        let mut tree = SectionTree::new(genesis_pk);
        assert!(tree.insert_without_chain(signed_genesis_sap.clone()));

        let state = Joiner::new(node, send_tx, &mut recv_rx, tree);

        let join_task = state.join(join_timeout);

        let test_task = async {
            let (node_msg, _, _) = recv_node_msg(&mut send_rx).await;

            assert_matches!(node_msg, NodeMsg::JoinRequest(JoinRequest { .. }));

            let proof_chain = SectionsDAG::new(genesis_pk);

            // Send `Retry` with bad prefix
            let bad_section_tree_update = {
                let (bad_sap, ..) = TestSapBuilder::new(bad_prefix).elder_count(1).build();
                let mut bad_signed_sap = signed_genesis_sap.clone();
                bad_signed_sap.value = bad_sap;
                SectionTreeUpdate::new(bad_signed_sap, proof_chain.clone())
            };
            send_join_response(
                &recv_tx,
                JoinResponse::Retry {
                    section_tree_update: bad_section_tree_update,
                },
                &genesis_nodes[0],
                genesis_pk,
            );
            task::yield_now().await;

            // Send `Retry` with valid update
            let (next_sap, next_sk_set, next_elders, _) = TestSapBuilder::new(Prefix::default())
                .elder_count(1)
                .build();
            let next_section_key = next_sk_set.public_keys().public_key();
            let section_tree_update = TestSectionTree::get_section_tree_update(
                &TestKeys::get_section_signed(&next_sk_set.secret_key(), next_sap),
                &SectionsDAG::new(genesis_pk),
                &genesis_sk,
            );
            let good_elders: Vec<&MyNodeInfo> =
                next_elders.iter().take(2 * elder_count() / 3).collect_vec();
            for elder in good_elders.iter() {
                send_join_response(
                    &recv_tx,
                    JoinResponse::Retry {
                        section_tree_update: section_tree_update.clone(),
                    },
                    elder,
                    next_section_key,
                );
            }

            Ok(())
        };

        pin_mut!(join_task);
        pin_mut!(test_task);

        match future::select(join_task, test_task).await {
            Either::Left((res, _)) => panic!("Join task should not have completed {res:?}"),
            Either::Right((output, _)) => output,
        }
    }

    async fn recv_node_msg(
        channel: &mut mpsc::Receiver<(WireMsg, Vec<Peer>)>,
    ) -> (NodeMsg, Dst, Vec<Peer>) {
        let (wire_msg, recipients) = channel.recv().await.expect("Should have received node msg");
        let msg = wire_msg.into_msg().expect("Failed to decode msg");
        let (node_msg, dst) = assert_matches!(msg, MsgType::Node { msg, dst,.. } => (msg, dst));
        (node_msg, dst, recipients)
    }

    fn send_join_response(
        recv_tx: &mpsc::Sender<MsgFromPeer>,
        resp: JoinResponse,
        sender: &MyNodeInfo,
        section_pk: BlsPublicKey,
    ) {
        let node_msg = NodeMsg::JoinResponse(Box::new(resp));
        send_node_msg(recv_tx, node_msg, sender, section_pk);
    }

    // test helper
    #[instrument]
    fn send_node_msg(
        recv_tx: &mpsc::Sender<MsgFromPeer>,
        msg: NodeMsg,
        sender: &MyNodeInfo,
        section_pk: BlsPublicKey,
    ) {
        let wire_msg = WireMsg::single_src(
            sender,
            Dst {
                name: XorName::from(PublicKey::Bls(section_pk)),
                section_key: section_pk,
            },
            msg,
        )
        .expect("Failed to build wire msg");

        recv_tx
            .try_send(MsgFromPeer {
                sender: sender.peer(),
                wire_msg,
                send_stream: None,
            })
            .expect("Failed to send message");
    }
}
