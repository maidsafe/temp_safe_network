// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{
    flow_ctrl::cmds::Cmd,
    relocated::JoiningAsRelocated,
    relocation::{find_nodes_to_relocate, ChurnId},
    MyNode, Result,
};

use sn_interface::{
    elder_count,
    messaging::system::SectionSigned,
    network_knowledge::{MembershipState, NodeState},
    types::log_markers::LogMarker,
};

use std::collections::BTreeSet;
use xor_name::XorName;

// Relocation
impl MyNode {
    pub(crate) fn relocate_peers(
        &mut self,
        churn_id: ChurnId,
        excluded: BTreeSet<XorName>,
    ) -> Result<Vec<Cmd>> {
        // Do not carry out relocations in the first section
        // TODO: consider avoiding relocations in first 16 sections instead.
        if self.network_knowledge.prefix().is_empty() {
            return Ok(vec![]);
        }
        debug!("Try to find relocate peers, excluded {excluded:?}");
        // Do not carry out relocation when there is not enough elder nodes.
        if self.network_knowledge.section_auth().elder_count() < elder_count() {
            debug!(
                "Not enough elders current {:?} vs. expected {:?}",
                self.network_knowledge.section_auth().elder_count(),
                elder_count()
            );
            return Ok(vec![]);
        }

        let mut cmds = vec![];
        for (node_state, relocate_details) in
            find_nodes_to_relocate(&self.network_knowledge, &churn_id, excluded)
        {
            debug!(
                "Relocating {:?} to {} (on churn of {churn_id})",
                node_state.peer(),
                relocate_details.dst,
            );

            let relocated_node_state = node_state.relocate(relocate_details);
            cmds.extend(self.propose_membership_change(relocated_node_state));
        }

        Ok(cmds)
    }

    pub(crate) fn handle_relocate(
        &mut self,
        relocate_proof: SectionSigned<NodeState>,
    ) -> Result<Option<Cmd>> {
        trace!("Handle relocate {:?}", relocate_proof);
        let (dst_xorname, dst_section_key, new_age) =
            if let MembershipState::Relocated(ref relocate_details) = relocate_proof.state() {
                (
                    relocate_details.dst,
                    relocate_details.dst_section_key,
                    relocate_details.age,
                )
            } else {
                debug!(
                    "Ignoring Relocate msg containing invalid NodeState: {:?}",
                    relocate_proof.state()
                );
                return Ok(None);
            };

        let node = self.info();
        // `relocate_details.dst` is not the name of the relocated node,
        // but the target section for the peer to be relocated to.
        // TODO: having an additional field within reolocate_details for that info?
        // if dst_xorname != node.name() {
        //     // This `Relocate` message is not for us - it's most likely a duplicate of a previous
        //     // message that we already handled.
        //     return Ok(None);
        // }

        debug!("Received Relocate message to join the section at {dst_xorname}");

        if self.relocate_state.is_some() {
            trace!("Ignore Relocate - relocation already in progress");
            return Ok(None);
        }

        trace!("Relocation has started. previous_name: {:?}", node.name());
        trace!("{}", LogMarker::RelocateStart);

        // Create a new instance of JoiningAsRelocated to start the relocation
        // flow. This same instance will handle responses till relocation is complete.
        let bootstrap_addrs =
            if let Ok(sap) = self.network_knowledge.section_auth_by_name(&dst_xorname) {
                sap.addresses()
            } else {
                self.network_knowledge.section_auth().addresses()
            };
        let (joining_as_relocated, cmd) = JoiningAsRelocated::start(
            node,
            relocate_proof,
            bootstrap_addrs,
            dst_xorname,
            dst_section_key,
            new_age,
        )?;

        self.relocate_state = Some(Box::new(joining_as_relocated));

        Ok(Some(cmd))
    }
}

#[cfg(test)]
mod tests {
    use crate::node::{
        core::MyNode,
        flow_ctrl::{
            cmds::Cmd,
            dispatcher::Dispatcher,
            tests::{
                cmd_utils::get_next_msg, create_relocation_trigger,
                network_builder::TestNetworkBuilder,
            },
        },
    };
    use sn_comms::MsgFromPeer;
    use sn_interface::{
        init_logger,
        messaging::system::{AntiEntropyKind, JoinResponse, NodeMsg, SectionStateVote},
        network_knowledge::{supermajority, MembershipState, RelocateDetails},
        test_utils::prefix,
    };

    use assert_matches::assert_matches;
    use eyre::{eyre, Result};
    use std::{
        collections::{BTreeMap, BTreeSet},
        sync::Arc,
    };
    use tokio::sync::{mpsc::Receiver, RwLock};
    use xor_name::{Prefix, XorName};

    #[tokio::test]
    async fn relocate_adults_to_the_same_section() -> Result<()> {
        init_logger();
        let elder_count = 7;
        let adult_count = 1;

        // Test environment setup
        let mut env = TestNetworkBuilder::new(rand::thread_rng())
            .sap(Prefix::default(), elder_count, adult_count, None, None)
            .build();
        let node_instances = env
            .get_nodes(Prefix::default(), elder_count, adult_count, None)
            .into_iter()
            .map(|node| {
                let prefix = node.network_knowledge().prefix();
                let name = node.name();
                let (dispatcher, _) = Dispatcher::new(Arc::new(RwLock::new(node)));
                ((prefix, name), dispatcher)
            })
            .collect::<BTreeMap<(Prefix, XorName), Dispatcher>>();
        let mut comm_receivers = BTreeMap::new();
        for (name, node) in node_instances.iter() {
            let pk = node.node().read().await.info().public_key();
            let comm = env.take_comm_rx(pk);
            let _ = comm_receivers.insert(*name, comm);
        }

        // Initialize relocation manually without using ChurnIds
        let relocation_node_old_name = env
            .get_peers(Prefix::default(), 0, 1, None)
            .remove(0)
            .name();
        for dispatcher in node_instances.iter().filter_map(|((_, name), dispatcher)| {
            if name != &relocation_node_old_name {
                Some(dispatcher)
            } else {
                None
            }
        }) {
            initialize_relocation(dispatcher, relocation_node_old_name, Prefix::default()).await?;
        }

        relocation_loop(&node_instances, &mut comm_receivers, elder_count).await?;

        // Validate the membership changes
        let relocation_node_new_name = node_instances
            .get(&(Prefix::default(), relocation_node_old_name))
            .expect("Node should be present")
            .node()
            .read()
            .await
            .name();
        for dispatcher in node_instances.values() {
            let network_knowledge = dispatcher.node().read().await.network_knowledge().clone();
            // Make sure the relocation_node (new_name) is part of the elder's network_knowledge
            if !network_knowledge.is_adult(&relocation_node_new_name) {
                panic!("The relocation node should've joined with a new name");
            }

            // Make sure the relocation_node's old_name is removed
            // The membership changes are actively monitored by the elders, so skip this check
            // for the adult nodes
            if dispatcher.node().read().await.is_elder()
                && network_knowledge.is_adult(&relocation_node_old_name)
            {
                panic!(
                    "The old name of the relocation node should be removed from network_knowledge"
                );
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn relocate_adults_to_different_section() -> Result<()> {
        init_logger();
        let elder_count_0 = 7;
        let adult_count_0 = 1;
        let elder_count_1 = 7;
        let adult_count_1 = 0;
        let prefix0 = prefix("0");
        let prefix1 = prefix("1");

        // Test environment setup
        let mut env = TestNetworkBuilder::new(rand::thread_rng())
            .sap(prefix0, elder_count_0, adult_count_0, None, None)
            .sap(prefix1, elder_count_1, adult_count_1, None, None)
            .build();
        let network_knowledge_0 = env.get_network_knowledge(prefix0, None);
        let st_update_0 = network_knowledge_0
            .section_tree()
            .generate_section_tree_update(&prefix0)?;
        let network_knowledge_1 = env.get_network_knowledge(prefix1, None);
        let st_update_1 = network_knowledge_1
            .section_tree()
            .generate_section_tree_update(&prefix1)?;

        let mut node_instances = env.get_nodes(prefix0, elder_count_0, adult_count_0, None);
        node_instances.extend(env.get_nodes(prefix1, elder_count_1, adult_count_1, None));
        let node_instances = node_instances
            .into_iter()
            .map(|mut node| {
                let name = node.name();
                let prefix = node.network_knowledge().prefix();
                // the nodes obtained from TestNetworkBuilder do not have knowledge about the other
                // sections, hence update their SectionTree
                if prefix == prefix0 {
                    let _ = node
                        .network_knowledge
                        .section_tree_mut()
                        .update_the_section_tree(st_update_1.clone())
                        .expect("Section tree update failed");
                } else if prefix == prefix1 {
                    let _ = node
                        .network_knowledge
                        .section_tree_mut()
                        .update_the_section_tree(st_update_0.clone())
                        .expect("Section tree update failed");
                }
                let (dispatcher, _) = Dispatcher::new(Arc::new(RwLock::new(node)));
                ((prefix, name), dispatcher)
            })
            .collect::<BTreeMap<(Prefix, XorName), Dispatcher>>();
        let mut comm_receivers = BTreeMap::new();
        for (name, node) in node_instances.iter() {
            let pk = node.node().read().await.info().public_key();
            let comm = env.take_comm_rx(pk);
            let _ = comm_receivers.insert(*name, comm);
        }
        // Initialize relocation manually without using ChurnIds. Here, the adult from prefix0 is
        // to be relocated to prefix1
        let relocation_node_old_name = env.get_peers(prefix0, 0, 1, None).remove(0).name();

        // Only the elders in prefix0 should propose the SectionStateVote
        for dispatcher in node_instances
            .iter()
            .filter_map(|((pre, name), dispatcher)| {
                if name != &relocation_node_old_name && pre == &prefix0 {
                    Some(dispatcher)
                } else {
                    None
                }
            })
        {
            initialize_relocation(dispatcher, relocation_node_old_name, prefix1).await?;
        }

        relocation_loop(&node_instances, &mut comm_receivers, elder_count_0).await?;

        // Validate the membership changes
        let relocation_node_new_name = node_instances
            .get(&(prefix0, relocation_node_old_name))
            .expect("Node should be present")
            .node()
            .read()
            .await
            .name();
        for ((pref, node_name), dispatcher) in node_instances.iter() {
            let network_knowledge = dispatcher.node().read().await.network_knowledge().clone();
            // the dispatcher for the relocation_node is still under the old name
            if node_name == &relocation_node_old_name {
                // the relocation node should be part of prefix1
                assert_eq!(network_knowledge.prefix(), prefix1);
                continue;
            }

            // Make sure the relocation_node with the new_name is part of prefix1
            if pref == &prefix1 {
                if !network_knowledge.is_adult(&relocation_node_new_name) {
                    panic!("The relocation node should've joined with a new name");
                }

                if network_knowledge.is_adult(&relocation_node_old_name) {
                    panic!("The relocation node not have joined with its old name")
                }
            }

            // Make sure the relocation_node's old_name is removed from prefix0. Also make sure the
            // new_name is not a part of prefix0
            if pref == &prefix0 {
                if network_knowledge.is_adult(&relocation_node_old_name) {
                    panic!("The old name of the relocation node should be removed from the network_knowledge");
                }

                if network_knowledge.is_adult(&relocation_node_new_name) {
                    panic!("The new name should not have joined prefix0")
                }
            }
        }

        Ok(())
    }

    // Try to relocate an adult from prefix000. But (supermajority-1) elders from prefix000 have knowledge about
    // prefix010 which is closer to our chosen dst XorName. Hence relocation is not initialized as
    // we will not have enough signature share for a particular proposal.
    #[tokio::test]
    async fn relocation_is_not_initialized_if_elders_have_different_network_view() -> Result<()> {
        init_logger();
        let (elder_count_000, adult_count_000) = (7, 1);
        let (elder_count_001, adult_count_001) = (7, 0);
        let prefix000 = prefix("000");
        let prefix001 = prefix("001");
        let prefix010 = prefix("010");

        // Test environment setup
        let mut env = TestNetworkBuilder::new(rand::thread_rng())
            .sap(prefix000, elder_count_000, adult_count_000, None, None)
            .sap(prefix001, elder_count_001, adult_count_001, None, None)
            .sap(prefix010, 7, 0, None, None)
            .build();
        let network_knowledge_001 = env.get_network_knowledge(prefix001, None);
        let network_knowledge_010 = env.get_network_knowledge(prefix010, None);

        let st_update_001 = network_knowledge_001
            .section_tree()
            .generate_section_tree_update(&prefix001)?;
        let st_update_010 = network_knowledge_010
            .section_tree()
            .generate_section_tree_update(&prefix010)?;

        let node_instances = env.get_nodes(prefix000, elder_count_000, adult_count_000, None);
        // ignoring nodes from the other prefixes, since we will fail way before reaching them

        let mut elders_with_extra_knowledge = 0;
        let node_instances = node_instances
            .into_iter()
            .map(|mut node| {
                let name = node.name();
                let prefix = node.network_knowledge().prefix();
                // update all the nodes about prefix001
                let _ = node
                    .network_knowledge
                    .section_tree_mut()
                    .update_the_section_tree(st_update_001.clone())
                    .expect("Section tree update failed");
                // but certain nodes have knowledge about prefix010
                if elders_with_extra_knowledge < (supermajority(elder_count_000) - 1) {
                    let _ = node
                        .network_knowledge
                        .section_tree_mut()
                        .update_the_section_tree(st_update_010.clone())
                        .expect("Section tree update failed");
                    elders_with_extra_knowledge += 1;
                }
                let (dispatcher, _) = Dispatcher::new(Arc::new(RwLock::new(node)));
                ((prefix, name), dispatcher)
            })
            .collect::<BTreeMap<(Prefix, XorName), Dispatcher>>();
        let mut comm_receivers = BTreeMap::new();
        for (name, node) in node_instances.iter() {
            let pk = node.node().read().await.info().public_key();
            let comm = env.take_comm_rx(pk);
            let _ = comm_receivers.insert(*name, comm);
        }

        // Initialize Relocation; The adult from prefix000 is to be relocated to prefix001
        let relocation_node_old_name = env.get_peers(prefix000, 0, 1, None).remove(0).name();
        // Only the elders in prefix000 should propose the SectionStateVote
        for node in node_instances
            .iter()
            .filter_map(|((pre, name), dispatcher)| {
                if name != &relocation_node_old_name && pre == &prefix000 {
                    Some(dispatcher)
                } else {
                    None
                }
            })
        {
            // dst XorName is closer to prefix010, hence some nodes choose 010 while the others
            // choose 001
            initialize_relocation(node, relocation_node_old_name, prefix010).await?;
        }

        let mut done = false;
        while !done {
            let mut empty_nodes = BTreeSet::new();
            for (key, dispatcher) in node_instances.iter() {
                let name = key.1;
                info!("\n\n NODE: {}", name);
                let comm_rx = comm_receivers
                    .get_mut(key)
                    .ok_or_else(|| eyre!("comm_rx should be present"))?;

                // used to check if the buffer is empty during the first iteration of the loop.
                // If all the nodes (msg buffers) are empty during the first try, done = true
                let mut first_try_done = false;
                'cmd_loop: loop {
                    if let Some(msg) = get_next_msg(comm_rx) {
                        if !first_try_done {
                            first_try_done = true;
                        }
                        // Make sure AE is triggered if there is any
                        // AE is not triggered even though some nodes have no knowledge about the
                        // prefix010
                        let MsgFromPeer {
                            sender,
                            wire_msg,
                            send_stream,
                        } = msg;
                        let cmds = MyNode::handle_msg(
                            dispatcher.node().clone(),
                            sender,
                            wire_msg,
                            send_stream,
                        )
                        .await?;
                        for cmd in cmds {
                            info!("Got cmd {}", cmd);
                            if let Cmd::SendMsg { .. } = &cmd {
                                dispatcher.mock_send_msg(cmd, None).await;
                            } else {
                                panic!("got a different cmd {:?}", cmd);
                            }
                        }
                    } else {
                        // the msg buffer is empty,
                        if !first_try_done {
                            let _ = empty_nodes.insert(name);
                        }
                        break 'cmd_loop;
                    }
                }
            }
            // done, if all the buffers are empty
            done = empty_nodes.len() == node_instances.len();
        }

        Ok(())
    }

    #[tokio::test]
    async fn elders_should_not_be_relocated_to_the_same_section() -> Result<()> {
        init_logger();
        let elder_count = 7;
        let adult_count = 10;
        let prefix0 = prefix("0");
        let prefix1 = prefix("1");

        // Test environment setup
        let env = TestNetworkBuilder::new(rand::thread_rng())
            .sap(prefix0, elder_count, adult_count, Some(&[6, 10]), None)
            .build();
        let sk_set_0 = env.get_secret_key_set(prefix0, None);
        let network_knowledge_1 = env.get_network_knowledge(prefix1, None);
        let expected_dst_section_key = network_knowledge_1.section_key();
        let st_update_1 = network_knowledge_1
            .section_tree()
            .generate_section_tree_update(&prefix1)?;

        let node_instances = env
            .get_nodes(prefix0, elder_count, 0, None)
            .into_iter()
            .map(|mut node| {
                let prefix = node.network_knowledge().prefix();
                // give the nodes info about the sibling section
                let _ = node
                    .network_knowledge
                    .section_tree_mut()
                    .update_the_section_tree(st_update_1.clone())
                    .expect("Section tree update failed");
                let name = node.name();
                let (dispatcher, _) = Dispatcher::new(Arc::new(RwLock::new(node)));
                ((prefix, name), dispatcher)
            })
            .collect::<BTreeMap<(Prefix, XorName), Dispatcher>>();

        // Create Membership change that will result in the relocation of our elder node with age 6
        let relocation_trigger = create_relocation_trigger(&sk_set_0, 6);
        for dispatcher in node_instances.values() {
            let cmds = dispatcher
                .node()
                .write()
                .await
                .handle_membership_decision(relocation_trigger.clone())
                .await?;
            for cmd in cmds {
                if let Cmd::SendMsg {
                    msg:
                        NodeMsg::ProposeSectionState {
                            proposal: SectionStateVote::NodeIsOffline(relocation_node),
                            ..
                        },
                    ..
                } = &cmd
                {
                    assert_matches!(
                        relocation_node.state(),
                        MembershipState::Relocated(relocate_details) => {
                         assert_eq!(expected_dst_section_key, relocate_details.dst_section_key);
                        }
                    );
                }
            }
        }

        Ok(())
    }

    // Propose NodeIsOffline(relocation) for the provided node
    async fn initialize_relocation(
        dispatcher: &Dispatcher,
        relocation_node_name: XorName,
        dst_prefix: Prefix,
    ) -> Result<()> {
        let relocation_node_state = dispatcher
            .node()
            .read()
            .await
            .network_knowledge()
            .get_section_member(&relocation_node_name)
            .expect("relocation node should be present");

        let relocate_details = RelocateDetails::with_age(
            dispatcher.node().read().await.network_knowledge(),
            relocation_node_state.peer(),
            dst_prefix.name(),
            relocation_node_state.age() + 1,
        );
        let relocation_node_state = relocation_node_state.relocate(relocate_details);
        let mut relocation_send_msg = dispatcher.node().write().await.propose_section_state(
            SectionStateVote::NodeIsOffline(relocation_node_state.clone()),
        )?;
        assert_eq!(relocation_send_msg.len(), 1);
        let relocation_send_msg = relocation_send_msg.remove(0);

        dispatcher.mock_send_msg(relocation_send_msg, None).await;

        Ok(())
    }

    /// Main loop that sends and processes Cmds
    async fn relocation_loop(
        node_instances: &BTreeMap<(Prefix, XorName), Dispatcher>,
        comm_receivers: &mut BTreeMap<(Prefix, XorName), Receiver<MsgFromPeer>>,
        from_section_n_elders: usize,
    ) -> Result<()> {
        let mut relocation_membership_decision_done = BTreeSet::new();
        let mut done = false;
        while !done {
            let mut empty_nodes = BTreeSet::new();
            for (key, dispatcher) in node_instances.iter() {
                let name = key.1;
                info!("\n\n NODE: {}", name);
                let comm_rx = comm_receivers
                    .get_mut(key)
                    .ok_or_else(|| eyre!("comm_rx should be present"))?;

                // used to check if the buffer is empty during the first iteration of the loop.
                // If all the nodes (msg buffers) are empty during the first try, done = true
                let mut first_try_done = false;
                'cmd_loop: loop {
                    if let Some(msg) = get_next_msg(comm_rx) {
                        if !first_try_done {
                            first_try_done = true;
                        }

                        let cmds = dispatcher.mock_handle_node_msg_skip_ae(msg).await;
                        for cmd in cmds {
                            info!("Got cmd {}", cmd);
                            if let Cmd::SendMsg { .. } = &cmd {
                                dispatcher.mock_send_msg(cmd, None).await;
                            } else if let Cmd::SendLockingJoinMsg {
                                msg,
                                msg_id,
                                recipients,
                            } = cmd
                            {
                                let context = dispatcher.node().read().await.context();
                                let send_msg = Cmd::SendMsg {
                                    msg,
                                    msg_id,
                                    recipients,
                                    context,
                                };
                                dispatcher.mock_send_msg(send_msg, None).await;
                            }
                            // The elders aggregate the relocation SectionStateVote which results
                            // in a membership change to relocate the node
                            else if let Cmd::HandleSectionDecisionAgreement { proposal, sig } =
                                cmd
                            {
                                let mut send_cmd = dispatcher
                                    .node()
                                    .write()
                                    .await
                                    .handle_section_decision_agreement(proposal, sig)?;
                                assert_eq!(send_cmd.len(), 1);
                                let send_cmd = send_cmd.remove(0);
                                assert_matches!(&send_cmd, Cmd::SendMsg { msg, .. } => {
                                    assert_matches!(msg, NodeMsg::MembershipVotes(_));
                                });
                                dispatcher.mock_send_msg(send_cmd, None).await;
                            }
                            // There are 2 Membership changes here, one to relocate the
                            // node and the other one to allow the node to join as adult after
                            // being relocated
                            else if let Cmd::HandleMembershipDecision(decision) = cmd {
                                let mut send_cmds = dispatcher
                                    .node()
                                    .write()
                                    .await
                                    .handle_membership_decision(decision)
                                    .await?;

                                if relocation_membership_decision_done.len()
                                    != from_section_n_elders
                                {
                                    // the first membership change contains an extra step to send the
                                    // relocate msg to the relocation_node
                                    assert_eq!(send_cmds.len(), 3);
                                    let send_cmd = send_cmds.remove(0);
                                    assert_matches!(&send_cmd, Cmd::SendMsg { msg, .. } => {
                                        assert_matches!(msg, NodeMsg::Relocate(_));
                                    });
                                    dispatcher.mock_send_msg(send_cmd, None).await;

                                    let _ = relocation_membership_decision_done.insert(name);
                                } else {
                                    assert_eq!(send_cmds.len(), 2);
                                }

                                // Common for both the cases
                                let send_cmd = send_cmds.remove(0);
                                assert_matches!(&send_cmd, Cmd::SendMsg { msg, .. } => {
                                    assert_matches!(msg,  NodeMsg::JoinResponse(JoinResponse::Approved { .. }));
                                });
                                dispatcher.mock_send_msg(send_cmd, None).await;

                                let send_cmd = send_cmds.remove(0);
                                assert_matches!(&send_cmd, Cmd::SendMsg { msg, .. } => {
                                    assert_matches!(msg, NodeMsg::AntiEntropy { section_tree_update: _, kind  } => {
                                        assert_matches!(kind, AntiEntropyKind::Update { .. });
                                    });
                                });
                                dispatcher.mock_send_msg(send_cmd, None).await;
                            } else {
                                panic!("got a different cmd {:?}", cmd);
                            }
                        }
                    } else {
                        // the msg buffer is empty,
                        if !first_try_done {
                            let _ = empty_nodes.insert(name);
                        }
                        break 'cmd_loop;
                    }
                }
            }
            // done, if all the buffers are empty
            done = empty_nodes.len() == node_instances.len();
        }
        Ok(())
    }
}
