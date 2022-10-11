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
        system::{JoinRejectionReason, JoinRequest, JoinResponse, NodeMsg},
        Dst, MsgType, WireMsg,
    },
    network_knowledge::{MembershipState, MyNodeInfo, NetworkKnowledge, SectionTree},
    types::{keys::ed25519, log_markers::LogMarker, Peer},
};

use backoff::{backoff::Backoff, ExponentialBackoff};
use bls::PublicKey as BlsPublicKey;
use futures::future;

use std::collections::{BTreeMap, BTreeSet};
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

    future::join(
        joiner.try_join(join_timeout),
        send_messages(outgoing_msgs_receiver, comm),
    )
    .instrument(span)
    .await
    .0
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
    aggregated: bool,
    retry_responses_cache: BTreeMap<Peer, u8>,
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
            aggregated: false,
            retry_responses_cache: Default::default(),
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
        let (target_section_key, recipients) = self.join_target()?;

        debug!("Initiating join with {recipients:?}");

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
                        trace!("Ignore join approval decision not for us: {decision:?}");
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
                    expected_age,
                } => {
                    let signed_sap = section_tree_update.signed_sap.clone();

                    trace!(
                        "Joining node {:?} - {:?}/{:?} received a Retry from {}, expected_age: {expected_age}, SAP: {}, proof_chain: {:?}",
                        self.prefix,
                        self.node.name(),
                        self.node.age(),
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
                    let _is_new_sap = match self.section_tree.update(section_tree_update) {
                        Ok(updated) => updated,
                        Err(err) => {
                            debug!("Ignoring JoinResponse::Retry with an invalid SAP: {err:?}");
                            continue;
                        }
                    };

                    if let Some(expected_age) =
                        self.insert_retry_response(sender, expected_age, signed_sap.elders_set())
                    {
                        // adjust our joining age to the expected by the network
                        trace!(
                            "Re-generating name due to mis-matched age, current {} vs. expected {}",
                            self.node.age(),
                            expected_age
                        );
                        // The expected_age is a sequence of 98, 96, 94, 92, ...
                        // The prefix is deduced from the age's bits.
                        let mut cur_age = expected_age / 2;
                        let mut new_prefix = Prefix::default();
                        while cur_age > 0 {
                            let push_prefix_0 = cur_age % 2 == 1;
                            new_prefix = new_prefix.pushed(push_prefix_0);
                            cur_age /= 2;
                        }
                        trace!("Name shall have the prefix of {:?}", new_prefix);

                        let new_keypair =
                            ed25519::gen_keypair(&new_prefix.range_inclusive(), expected_age);
                        let new_name = ed25519::name(&new_keypair.public);

                        info!("Setting Node name to {} (age {})", new_name, expected_age);
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

    // The retry responses may containing different expected ages, or only having
    // more than 1/3 of elders voted (which blocks the aggregation).
    // So a restart with new name shall happen when: received 1/3 of elders voted
    // And with an age of: the most expected age
    fn insert_retry_response(
        &mut self,
        sender: Peer,
        expected_age: u8,
        elders: BTreeSet<Peer>,
    ) -> Option<u8> {
        if !elders.contains(&sender) {
            error!("Sender {sender:?} of the retry-response is not part of the elders {elders:?}");
            return None;
        }
        let _ = self.retry_responses_cache.insert(sender, expected_age);
        let mut expected_ages: BTreeMap<u8, u8> = Default::default();
        for (elder, expected_age) in self.retry_responses_cache.iter() {
            if elders.contains(elder) {
                *expected_ages.entry(*expected_age).or_insert(0) += 1;
            }
        }

        // To avoid restarting too quick, the voted age must got supermajority votes.
        // In case there is split votes, leave to the overall join_timeout to restart.
        if !self.aggregated {
            let threshold = 2 * elders.len() / 3;
            let voted_expected_age = expected_ages
                .iter()
                .find(|(_age, count)| **count >= threshold as u8)
                .map(|(age, _count)| *age)?;
            if self.node.age() != voted_expected_age {
                return Some(voted_expected_age);
            }
            trace!(
                "No restart as current age({}) same to the expected {voted_expected_age}",
                self.node.age()
            );
        }
        None
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

        info!("Sending {:?} to {:?}", msg, recipients);

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
        while let Some(MsgFromPeer {
            sender,
            wire_msg,
            send_stream: _,
        }) = self.incoming_msgs.recv().await
        {
            // We are interested only in `JoinResponse` type of messages

            match wire_msg.into_msg() {
                Ok(MsgType::Node {
                    msg: NodeMsg::JoinResponse(resp),
                    ..
                }) => return Ok((*resp, sender)),
                Ok(MsgType::Client { msg_id, .. } | MsgType::Node { msg_id, .. }) => {
                    trace!("Bootstrap message discarded: sender: {sender:?} msg_id: {msg_id:?}");
                }
                Err(err) => {
                    error!("Failed to deserialize message payload: {:?}", err);
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
            match comm.send_out_bytes(peer, msg_id, bytes, false).await {
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

    use sn_interface::{
        elder_count, init_logger,
        network_knowledge::{
            test_utils::*, NodeState, SectionAuthorityProvider, SectionTreeUpdate, SectionsDAG,
        },
        types::PublicKey,
    };

    use assert_matches::assert_matches;
    use eyre::{eyre, Error, Result};
    use futures::{
        future::{self, Either},
        pin_mut,
    };
    use itertools::Itertools;
    use tokio::task;
    use xor_name::XorName;

    const JOIN_TIMEOUT_SEC: u64 = 15;

    #[tokio::test]
    async fn join_as_adult() -> Result<()> {
        init_logger();

        let join_timeout = Duration::from_secs(JOIN_TIMEOUT_SEC);
        let (send_tx, mut send_rx) = mpsc::channel(1);
        let (recv_tx, mut recv_rx) = mpsc::channel(10);

        let (genesis_sap, _genesis_nodes, genesis_sk_set) =
            random_sap(Prefix::default(), elder_count(), 0, None);
        let genesis_sk = genesis_sk_set.secret_key();
        let genesis_pk = genesis_sk.public_key();

        let node = MyNodeInfo::new(
            ed25519::gen_keypair(&Prefix::default().range_inclusive(), MIN_ADULT_AGE + 1),
            gen_addr(),
        );

        let signed_genesis_sap = section_signed(&genesis_sk, genesis_sap.clone())?;
        let mut tree = SectionTree::new(genesis_pk);
        assert!(tree.insert_without_chain(signed_genesis_sap));

        let state = Joiner::new(node.clone(), send_tx, &mut recv_rx, tree);

        // Create the bootstrap task, but don't run it yet.
        let bootstrap = async move { state.try_join(join_timeout).await.map_err(Error::from) };

        let (next_sap, next_elders, next_sk_set) =
            random_sap(Prefix::default(), elder_count(), 0, None);

        let next_section_key = next_sk_set.public_keys().public_key();
        let section_tree_update = gen_section_tree_update(
            &section_signed(&next_sk_set.secret_key(), next_sap.clone())?,
            &SectionsDAG::new(genesis_pk),
            &genesis_sk,
        )?;

        // Create the task that executes the body of the test, but don't run it either.
        let others = async {
            // Receive JoinRequest
            let (wire_msg, recipients) = send_rx
                .recv()
                .await
                .ok_or_else(|| eyre!("JoinRequest was not received"))?;

            itertools::assert_equal(recipients, genesis_sap.elders());

            let node_msg = assert_matches!(wire_msg.into_msg(), Ok(MsgType::Node { msg, .. }) =>
                msg);

            assert_matches!(node_msg, NodeMsg::JoinRequest(JoinRequest { .. }));

            // Send JoinResponse::Retry with new SAP
            let other_elders: Vec<&MyNodeInfo> =
                next_elders.iter().take(2 * elder_count() / 3).collect_vec();
            for elder in other_elders.iter() {
                send_response(
                    &recv_tx,
                    JoinResponse::Retry {
                        section_tree_update: section_tree_update.clone(),
                        expected_age: MIN_ADULT_AGE,
                    },
                    elder,
                    next_sap.section_key(),
                )?;
            }

            // Receive the second JoinRequest with correct section info
            let (wire_msg, recipients) = send_rx
                .recv()
                .await
                .ok_or_else(|| eyre!("JoinRequest was not received"))?;
            let (node_msg, dst) = assert_matches!(wire_msg.into_msg(), Ok(MsgType::Node { msg, dst,.. }) =>
                (msg, dst));

            assert_eq!(dst.section_key, next_section_key);
            itertools::assert_equal(recipients, next_sap.elders());
            assert_matches!(node_msg, NodeMsg::JoinRequest(JoinRequest{ section_key }) => {
                assert_eq!(section_key, next_section_key);
            });

            // Name changed
            let new_peer = Peer::new(dst.name, node.peer().addr());
            // Send JoinResponse::Approved
            let decision = section_decision(&next_sk_set, NodeState::joined(new_peer, None))?;
            send_response(
                &recv_tx,
                JoinResponse::Approved {
                    section_tree_update,
                    decision,
                },
                &next_elders[0],
                next_sap.section_key(),
            )?;

            Ok(())
        };

        // Drive both tasks to completion concurrently (but on the same thread).
        let ((node, section), _) = future::try_join(bootstrap, others).await?;

        assert_eq!(section.section_auth(), next_sap);
        assert_eq!(section.section_key(), next_section_key);
        assert_eq!(node.age(), MIN_ADULT_AGE);

        Ok(())
    }

    #[tokio::test]
    async fn join_receive_redirect_response() -> Result<()> {
        init_logger();

        let join_timeout = Duration::from_secs(JOIN_TIMEOUT_SEC);
        let (send_tx, mut send_rx) = mpsc::channel(1);
        let (recv_tx, mut recv_rx) = mpsc::channel(1);

        let (genesis_sap, genesis_nodes, genesis_sk_set) =
            random_sap(Prefix::default(), elder_count(), 0, None);
        let genesis_sk = genesis_sk_set.secret_key();
        let genesis_pk = genesis_sk.public_key();

        let node = MyNodeInfo::new(
            ed25519::gen_keypair(&Prefix::default().range_inclusive(), MIN_ADULT_AGE),
            gen_addr(),
        );

        let signed_genesis_sap = section_signed(&genesis_sk, genesis_sap.clone())?;
        let mut tree = SectionTree::new(genesis_pk);
        assert!(tree.insert_without_chain(signed_genesis_sap));

        let state = Joiner::new(node, send_tx, &mut recv_rx, tree);

        let bootstrap_task = state.try_join(join_timeout);
        let test_task = async move {
            // Receive JoinRequest
            let (wire_msg, recipients) = send_rx
                .recv()
                .await
                .ok_or_else(|| eyre!("JoinRequest was not received"))?;

            itertools::assert_equal(recipients, genesis_sap.elders());

            assert_matches!(wire_msg.into_msg(), Ok(MsgType::Node { msg, .. }) =>
                    assert_matches!(msg, NodeMsg::JoinRequest{..}));

            // Send JoinResponse::Redirect
            let (new_sap, _, _) = random_sap(Prefix::default(), elder_count(), 0, None);

            send_response(
                &recv_tx,
                JoinResponse::Redirect(new_sap.clone()),
                &genesis_nodes[0],
                new_sap.section_key(),
            )?;

            task::yield_now().await;

            // Receive new JoinRequest with redirected bootstrap contacts
            let (wire_msg, recipients) = send_rx
                .recv()
                .await
                .ok_or_else(|| eyre!("JoinRequest was not received"))?;

            itertools::assert_equal(recipients, new_sap.elders());

            let (node_msg, dst) = assert_matches!(wire_msg.into_msg(), Ok(MsgType::Node { msg, dst,.. }) =>
                    (msg, dst));

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

        let (genesis_sap, genesis_nodes, genesis_sk_set) =
            random_sap(Prefix::default(), elder_count(), 0, None);
        let genesis_sk = genesis_sk_set.secret_key();
        let genesis_pk = genesis_sk.public_key();

        let node = MyNodeInfo::new(
            ed25519::gen_keypair(&Prefix::default().range_inclusive(), MIN_ADULT_AGE),
            gen_addr(),
        );

        let signed_genesis_sap = section_signed(&genesis_sk, genesis_sap.clone())?;
        let mut tree = SectionTree::new(genesis_pk);
        assert!(tree.insert_without_chain(signed_genesis_sap));

        let state = Joiner::new(node, send_tx, &mut recv_rx, tree);

        let bootstrap_task = state.try_join(join_timeout);
        let test_task = async {
            let (wire_msg, _) = send_rx
                .recv()
                .await
                .ok_or_else(|| eyre!("JoinRequest was not received"))?;

            assert_matches!(wire_msg.into_msg(), Ok(MsgType::Node { msg, .. }) =>
            assert_matches!(msg, NodeMsg::JoinRequest{..}));

            let (new_sap, _, new_sk_set) = random_sap(Prefix::default(), elder_count(), 0, None);
            let new_pk_set = new_sk_set.public_keys();

            send_response(
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
            )?;
            task::yield_now().await;

            send_response(
                &recv_tx,
                JoinResponse::Redirect(new_sap.clone()),
                &genesis_nodes[0],
                new_sap.section_key(),
            )?;
            task::yield_now().await;

            let (wire_msg, _) = send_rx
                .recv()
                .await
                .ok_or_else(|| eyre!("JoinRequest was not received"))?;

            assert_matches!(wire_msg.into_msg(), Ok(MsgType::Node { msg, .. }) =>
            assert_matches!(msg, NodeMsg::JoinRequest{..}));

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

        let (genesis_sap, genesis_nodes, genesis_sk_set) =
            random_sap(Prefix::default(), elder_count(), 0, None);
        let genesis_sk = genesis_sk_set.secret_key();
        let genesis_pk = genesis_sk.public_key();

        let node = MyNodeInfo::new(
            ed25519::gen_keypair(&Prefix::default().range_inclusive(), MIN_ADULT_AGE),
            gen_addr(),
        );

        let signed_genesis_sap = section_signed(&genesis_sk, genesis_sap.clone())?;
        let mut tree = SectionTree::new(genesis_pk);
        assert!(tree.insert_without_chain(signed_genesis_sap));

        let state = Joiner::new(node, send_tx, &mut recv_rx, tree);

        let bootstrap_task = state.try_join(join_timeout);
        let test_task = async {
            let (wire_msg, _) = send_rx
                .recv()
                .await
                .ok_or_else(|| eyre!("JoinRequest was not received"))?;

            assert_matches!(wire_msg.into_msg(), Ok(MsgType::Node { msg, .. }) =>
                                assert_matches!(msg, NodeMsg::JoinRequest{..}));

            send_response(
                &recv_tx,
                JoinResponse::Rejected(JoinRejectionReason::JoinsDisallowed),
                &genesis_nodes[0],
                genesis_sap.section_key(),
            )?;

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

        let (genesis_sap, genesis_nodes, genesis_sk_set) =
            random_sap(Prefix::default(), 1, 0, None);
        let genesis_sk = genesis_sk_set.secret_key();
        let genesis_pk = genesis_sk.public_key();

        let signed_genesis_sap = section_signed(&genesis_sk, genesis_sap.clone())?;
        let mut tree = SectionTree::new(genesis_pk);
        assert!(tree.insert_without_chain(signed_genesis_sap.clone()));

        let state = Joiner::new(node, send_tx, &mut recv_rx, tree);

        let join_task = state.join(join_timeout);

        let test_task = async {
            let (wire_msg, _) = send_rx.recv().await.expect("NodeMsg was not received");

            let node_msg =
                assert_matches!(wire_msg.into_msg(), Ok(MsgType::Node{ msg, .. }) => msg);
            assert_matches!(node_msg, NodeMsg::JoinRequest(JoinRequest { .. }));

            let proof_chain = SectionsDAG::new(genesis_pk);

            // Send `Retry` with bad prefix
            let bad_section_tree_update = {
                let (bad_sap, _, _) = random_sap(bad_prefix, 1, 0, None);
                let mut bad_signed_sap = signed_genesis_sap.clone();
                bad_signed_sap.value = bad_sap;
                SectionTreeUpdate::new(bad_signed_sap, proof_chain.clone())
            };
            send_response(
                &recv_tx,
                JoinResponse::Retry {
                    section_tree_update: bad_section_tree_update,
                    expected_age: MIN_ADULT_AGE,
                },
                &genesis_nodes[0],
                genesis_pk,
            )?;
            task::yield_now().await;

            // Send `Retry` with valid update
            let (next_sap, next_elders, next_sk_set) = random_sap(Prefix::default(), 1, 0, None);
            let next_section_key = next_sk_set.public_keys().public_key();
            let section_tree_update = gen_section_tree_update(
                &section_signed(&next_sk_set.secret_key(), next_sap)?,
                &SectionsDAG::new(genesis_pk),
                &genesis_sk,
            )?;
            let good_elders: Vec<&MyNodeInfo> =
                next_elders.iter().take(2 * elder_count() / 3).collect_vec();
            for elder in good_elders.iter() {
                send_response(
                    &recv_tx,
                    JoinResponse::Retry {
                        section_tree_update: section_tree_update.clone(),
                        expected_age: MIN_ADULT_AGE + 1,
                    },
                    elder,
                    next_section_key,
                )?;
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

    // test helper
    #[instrument]
    fn send_response(
        recv_tx: &mpsc::Sender<MsgFromPeer>,
        response: JoinResponse,
        bootstrap_node: &MyNodeInfo,
        section_pk: BlsPublicKey,
    ) -> Result<()> {
        let wire_msg = WireMsg::single_src(
            bootstrap_node,
            Dst {
                name: XorName::from(PublicKey::Bls(section_pk)),
                section_key: section_pk,
            },
            NodeMsg::JoinResponse(Box::new(response)),
        )?;

        debug!("wire msg built");

        recv_tx.try_send(MsgFromPeer {
            sender: bootstrap_node.peer(),
            wire_msg,
            send_stream: None,
        })?;

        Ok(())
    }
}
