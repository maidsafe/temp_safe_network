// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

#![allow(dead_code, unused_imports)]
pub(crate) mod cmd_utils;
pub(crate) mod dbc_utils;
pub(crate) mod network_utils;

use crate::{
    comm::{Comm, MsgFromPeer},
    node::{
        api::gen_genesis_dbc,
        cfg::create_test_max_capacity_and_root_storage,
        core::MyNode,
        flow_ctrl::{dispatcher::Dispatcher, event_channel},
        messages::WireMsgUtils,
        messaging::{OutgoingMsg, Peers},
        Cmd, Error, Event, MembershipEvent, Proposal, Result as RoutingResult,
    },
    storage::UsedSpace,
};

use cmd_utils::{
    get_client_msg_parts_for_handling, handle_online_cmd, run_and_collect_cmds,
    run_node_handle_client_msg_and_collect_cmds,
};
use sn_consensus::Decision;
use sn_dbc::{Hash, OwnerOnce, SpentProofShare, TransactionBuilder};
#[cfg(feature = "traceroute")]
use sn_interface::messaging::Traceroute;
use sn_interface::{
    elder_count, init_logger,
    messaging::{
        data::{
            ClientMsg, CmdResponse, DataCmd, Error as MessagingDataError, RegisterCmd, SpentbookCmd,
        },
        system::{
            AntiEntropyKind, JoinAsRelocatedRequest, JoinRequest, JoinResponse, NodeCmd, NodeMsg,
            SectionSig, SectionSigned,
        },
        Dst, MsgId, MsgType, WireMsg,
    },
    network_knowledge::{
        recommended_section_size, supermajority, test_utils::*, Error as NetworkKnowledgeError,
        MembershipState, MyNodeInfo, NetworkKnowledge, NodeState, RelocateDetails,
        SectionAuthorityProvider, SectionKeyShare, SectionKeysProvider, SectionTree,
        SectionTreeUpdate, SectionsDAG, MIN_ADULT_AGE,
    },
    types::{keyed_signed, keys::ed25519, Peer, PublicKey, ReplicatedData, SecretKeySet},
};

use assert_matches::assert_matches;
use bls::Signature;
use ed25519_dalek::Signer;
use eyre::{bail, eyre, Context, Result};
use itertools::Itertools;
use rand::{distributions::Alphanumeric, rngs::OsRng, Rng};
use resource_proof::ResourceProof as ChallengeSolver;
use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    iter,
    net::Ipv4Addr,
    ops::Deref,
    path::Path,
    sync::Arc,
};
use tempfile::tempdir;
use tokio::{
    sync::{mpsc, RwLock},
    time::{timeout, Duration},
};
use xor_name::{Prefix, XorName};

#[tokio::test]
async fn membership_churn_starts_on_join_request_from_relocated_node() -> Result<()> {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async move {
            init_logger();
            let _span = tracing::info_span!("receive_join_request_from_relocated_node").entered();

            let (dispatcher, _, _, sk_set) =
                network_utils::TestNodeBuilder::new(Prefix::default(), elder_count())
                    .build()
                    .await?;
            let section_key = sk_set.public_keys().public_key();
            let node = dispatcher.node();
            let node_info = node.read().await.info();
            let relocated_node_old_name = node_info.name();
            let relocated_node_old_keypair = node_info.keypair.clone();

            let relocated_node = MyNodeInfo::new(
                ed25519::gen_keypair(&Prefix::default().range_inclusive(), MIN_ADULT_AGE + 1),
                gen_addr(),
            );

            let relocate_details = RelocateDetails {
                previous_name: relocated_node_old_name,
                dst: relocated_node_old_name,
                dst_section_key: section_key,
                age: relocated_node.age(),
            };

            let node_state = NodeState::relocated(
                relocated_node.peer(),
                Some(relocated_node_old_name),
                relocate_details,
            );
            let relocate_proof = section_signed(&sk_set.secret_key(), node_state);

            let signature_over_new_name =
                ed25519::sign(&relocated_node.name().0, &relocated_node_old_keypair);

            let wire_msg = WireMsg::single_src(
                &relocated_node,
                Dst {
                    name: XorName::from(PublicKey::Bls(section_key)),
                    section_key,
                },
                NodeMsg::JoinAsRelocatedRequest(Box::new(JoinAsRelocatedRequest {
                    section_key,
                    relocate_proof,
                    signature_over_new_name,
                })),
            )?;

            let _ = run_and_collect_cmds(
                Cmd::ValidateMsg {
                    origin: relocated_node.peer(),
                    wire_msg,
                    send_stream: None,
                },
                &dispatcher,
            )
            .await?;

            assert!(node
                .read()
                .await
                .membership
                .as_ref()
                .ok_or_else(|| eyre!("Membership for the node must be set"))?
                .is_churn_in_progress());

            Result::<()>::Ok(())
        })
        .await
}

#[tokio::test]
async fn handle_agreement_on_online() -> Result<()> {
    let (event_sender, mut event_receiver) =
        event_channel::new(network_utils::TEST_EVENT_CHANNEL_SIZE);
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async move {
            let (dispatcher, section, _, sk_set) =
                network_utils::TestNodeBuilder::new(Prefix::default(), elder_count())
                    .event_sender(event_sender)
                    .build()
                    .await?;

            let new_peer = network_utils::create_peer(MIN_ADULT_AGE);
            let status =
                handle_online_cmd(&new_peer, &sk_set, &dispatcher, &section.section_auth()).await?;
            assert!(status.node_approval_sent);

            assert_matches!(
                event_receiver.next().await,
                Some(Event::Membership(MembershipEvent::MemberJoined { name, age, .. })) => {
                    assert_eq!(name, new_peer.name());
                    assert_eq!(age, MIN_ADULT_AGE);
            });

            Result::<()>::Ok(())
        })
        .await
}

#[tokio::test]
async fn handle_agreement_on_online_of_elder_candidate() -> Result<()> {
    init_logger();
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async move {
            let sk_set = bls::SecretKeySet::random(0, &mut rand::thread_rng());
            let pk = sk_set.secret_key().public_key();
            let section_chain = SectionsDAG::new(pk);

            // Creates nodes where everybody has age 6 except one has 5.
            let mut nodes: Vec<_> = gen_sorted_nodes(&Prefix::default(), elder_count(), true);

            let elders = nodes.iter().map(MyNodeInfo::peer);
            let members = nodes.iter().map(|n| NodeState::joined(n.peer(), None));
            let section_auth = SectionAuthorityProvider::new(
                elders,
                Prefix::default(),
                members,
                sk_set.public_keys(),
                0,
            );
            let section_tree_update = {
                let signed_sap = section_signed(&sk_set.secret_key(), section_auth.clone());
                SectionTreeUpdate::new(signed_sap, section_chain)
            };

            let mut section = NetworkKnowledge::new(SectionTree::new(pk), section_tree_update)?;
            let mut expected_new_elders = BTreeSet::new();

            for peer in section_auth.elders() {
                let node_state = NodeState::joined(*peer, None);
                let sig = prove(&sk_set.secret_key(), &node_state);
                let _updated = section.update_member(SectionSigned {
                    value: node_state,
                    sig,
                });
                if peer.age() == MIN_ADULT_AGE + 1 {
                    let _changed = expected_new_elders.insert(peer);
                }
            }

            let node = nodes.remove(0);
            let node_name = node.name();
            let (dispatcher, _, _, _) =
                network_utils::TestNodeBuilder::new(Prefix::default(), elder_count())
                    .section(section.clone(), sk_set.clone(), node.clone())
                    .build()
                    .await?;

            // Handle agreement on Online of a peer that is older than the youngest
            // current elder - that means this peer is going to be promoted.
            let new_peer = network_utils::create_peer(MIN_ADULT_AGE + 1);
            let node_state = NodeState::joined(new_peer, Some(xor_name::rand::random()));

            let membership_decision = section_decision(&sk_set, node_state.clone());

            // Force this node to join
            dispatcher
                .node()
                .write()
                .await
                .membership
                .as_mut()
                .ok_or_else(|| eyre!("Membership for the node must be set"))?
                .force_bootstrap(node_state);

            let cmds = run_and_collect_cmds(
                Cmd::HandleMembershipDecision(membership_decision),
                &dispatcher,
            )
            .await?;

            // Verify we sent a `DkgStart` message with the expected participants.
            let mut dkg_start_sent = false;
            let _changed = expected_new_elders.insert(&new_peer);

            for cmd in cmds {
                let (msg, recipients) = match cmd {
                    Cmd::SendMsg {
                        recipients,
                        msg: OutgoingMsg::Node(msg),
                        ..
                    } => (msg, recipients),
                    _ => continue,
                };

                let actual_elder_candidates = match msg {
                    NodeMsg::DkgStart(session, _) => session.elders,
                    _ => continue,
                };
                itertools::assert_equal(
                    actual_elder_candidates,
                    expected_new_elders.iter().map(|p| (p.name(), p.addr())),
                );

                let expected_dkg_start_recipients: BTreeSet<_> = expected_new_elders
                    .clone()
                    .into_iter()
                    .filter(|peer| peer.name() != node_name)
                    .cloned()
                    .collect();

                assert_matches!(recipients, Peers::Multiple(peers) => {
                    assert_eq!(peers, expected_dkg_start_recipients);
                });

                dkg_start_sent = true;
            }

            assert!(dkg_start_sent);
            Result::<()>::Ok(())
        })
        .await
}

#[tokio::test]
async fn handle_join_request_of_rejoined_node() -> Result<()> {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async move {
            init_logger();
            let prefix = Prefix::default();
            let (dispatcher, _, _, _) =
                network_utils::TestNodeBuilder::new(Prefix::default(), elder_count())
                    .build()
                    .await?;

            // Make a left peer.
            let peer = network_utils::create_peer_in_prefix(&prefix, MIN_ADULT_AGE);
            dispatcher
                .node()
                .write()
                .await
                .membership
                .as_mut()
                .ok_or_else(|| eyre!("Membership for the node must be set"))?
                .force_bootstrap(NodeState::left(peer, None));

            // Simulate the same peer rejoining
            let node_state = NodeState::joined(peer, None);
            let join_cmd = dispatcher
                .node()
                .write()
                .await
                .propose_membership_change(node_state.clone());

            // A rejoining node will always be rejected
            assert!(join_cmd.is_none()); // no cmd signals this membership proposal was dropped.
            assert!(!dispatcher
                .node()
                .read()
                .await
                .membership
                .as_ref()
                .ok_or_else(|| eyre!("Membership for the node must be set"))?
                .is_churn_in_progress());
            Ok(())
        })
        .await
}

#[tokio::test]
async fn handle_agreement_on_offline_of_non_elder() -> Result<()> {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async move {
            init_logger();
            let _span = tracing::info_span!("handle_agreement_on_offline_of_non_elder").entered();
            let existing_peer = network_utils::create_peer(MIN_ADULT_AGE);

            let (dispatcher, _, _, sk_set) =
                network_utils::TestNodeBuilder::new(Prefix::default(), elder_count())
                    .custom_peer(existing_peer)
                    .build()
                    .await?;

            let node_state = NodeState::left(existing_peer, None);
            let proposal = Proposal::VoteNodeOffline(node_state.clone());
            let sig = keyed_signed(&sk_set.secret_key(), &proposal.as_signable_bytes()?);

            let _cmds =
                run_and_collect_cmds(Cmd::HandleAgreement { proposal, sig }, &dispatcher).await?;

            assert!(!dispatcher
                .node()
                .read()
                .await
                .network_knowledge()
                .section_members()
                .contains(&node_state));
            Result::<()>::Ok(())
        })
        .await
}

#[tokio::test]
async fn handle_agreement_on_offline_of_elder() -> Result<()> {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async move {
            let (section_auth, mut nodes, sk_set) = network_utils::create_section_auth();

            let (mut section, _) =
                network_utils::create_section(&sk_set, &section_auth, None, None)?;

            let existing_peer = network_utils::create_peer(MIN_ADULT_AGE);
            let node_state = NodeState::joined(existing_peer, None);
            let node_state = section_signed(&sk_set.secret_key(), node_state);
            let _updated = section.update_member(node_state);

            // Pick the elder to remove.
            let auth_peers = section_auth.elders();
            let remove_peer = auth_peers.last().expect("section_auth is empty");

            let remove_node_state = section
                .get_section_member(&remove_peer.name())
                .expect("member not found")
                .leave()?;

            let node = nodes.remove(0);
            let (dispatcher, _, _, _) =
                network_utils::TestNodeBuilder::new(Prefix::default(), elder_count())
                    .section(section.clone(), sk_set.clone(), node.clone())
                    .build()
                    .await?;
            // Handle agreement on the Offline proposal
            let proposal = Proposal::VoteNodeOffline(remove_node_state.clone());
            let sig = keyed_signed(&sk_set.secret_key(), &proposal.as_signable_bytes()?);

            let _cmds =
                run_and_collect_cmds(Cmd::HandleAgreement { proposal, sig }, &dispatcher).await?;

            // Verify we initiated a membership churn
            assert!(dispatcher
                .node()
                .read()
                .await
                .membership
                .as_ref()
                .ok_or_else(|| eyre!("Membership for the node must be set"))?
                .is_churn_in_progress());
            Result::<()>::Ok(())
        })
        .await
}

#[derive(PartialEq)]
enum UntrustedMessageSource {
    Peer,
    Accumulation,
}

#[tokio::test]
async fn ae_msg_from_the_future_is_handled() -> Result<()> {
    // The setup here is too complex for the TestNodeBuilder.
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async move {
            init_logger();
            let _span = info_span!("ae_msg_from_the_future_is_handled").entered();

            // Create first `Section` with a chain of length 2
            let sk0 = bls::SecretKey::random();
            let pk0 = sk0.public_key();

            let (old_sap, mut nodes, sk_set1) = network_utils::create_section_auth();
            let members =
                BTreeSet::from_iter(nodes.iter().map(|n| NodeState::joined(n.peer(), None)));
            let pk1 = sk_set1.secret_key().public_key();

            let section_tree_update = gen_section_tree_update(
                &section_signed(&sk_set1.secret_key(), old_sap.clone()),
                &SectionsDAG::new(pk0),
                &sk0,
            );
            let network_knowledge =
                NetworkKnowledge::new(SectionTree::new(pk0), section_tree_update)?;

            // Create our node
            let (event_sender, mut event_receiver) =
                event_channel::new(network_utils::TEST_EVENT_CHANNEL_SIZE);
            let section_key_share = network_utils::create_section_key_share(&sk_set1, 0);
            let node = nodes.remove(0);
            let (max_capacity, root_storage_dir) = create_test_max_capacity_and_root_storage()?;
            let comm = network_utils::create_comm().await?;
            let mut node = MyNode::new(
                comm,
                node.keypair.clone(),
                network_knowledge,
                Some(section_key_share),
                event_sender,
                UsedSpace::new(max_capacity),
                root_storage_dir,
            )
            .await?;

            // Create new `Section` as a successor to the previous one.
            let sk_set2 = SecretKeySet::random(None);
            let sk2 = sk_set2.secret_key();
            let pk2 = sk2.public_key();

            let old_node = nodes.remove(0);

            // Create the new `SectionAuthorityProvider` by replacing the last peer with a new one.
            let new_peer = network_utils::create_peer(MIN_ADULT_AGE);
            let new_elders = old_sap
                .elders()
                .take(old_sap.elder_count() - 1)
                .cloned()
                .chain(vec![new_peer]);

            let new_sap = SectionAuthorityProvider::new(
                new_elders,
                old_sap.prefix(),
                members,
                sk_set2.public_keys(),
                0,
            );
            let new_section_elders: BTreeSet<_> = new_sap.names();
            let section_tree_update = gen_section_tree_update(
                &section_signed(sk2, new_sap.clone()),
                &node.section_chain(),
                &sk_set1.secret_key(),
            );

            // Create the `Sync` message containing the new `Section`.
            let wire_msg = WireMsg::single_src(
                &old_node,
                Dst {
                    name: XorName::from(PublicKey::Bls(pk1)),
                    section_key: pk1,
                },
                NodeMsg::AntiEntropy {
                    section_tree_update,
                    kind: AntiEntropyKind::Update {
                        members: BTreeSet::default(),
                    },
                },
            )?;

            // Simulate DKG round finished succesfully by adding
            // the new section key share to our cache
            node.section_keys_provider
                .insert(network_utils::create_section_key_share(&sk_set2, 0));

            let dispatcher = Dispatcher::new(Arc::new(RwLock::new(node)));

            let _cmds = run_and_collect_cmds(
                Cmd::ValidateMsg {
                    origin: old_node.peer(),
                    wire_msg,
                    send_stream: None,
                },
                &dispatcher,
            )
            .await?;

            // Verify our `Section` got updated.
            assert_matches!(
                event_receiver.next().await,
                Some(Event::Membership(MembershipEvent::EldersChanged { elders, .. })) => {
                    assert_eq!(elders.key, pk2);
                    assert!(elders.added.iter().all(|a| new_section_elders.contains(a)));
                    assert!(elders.remaining.iter().all(|a| new_section_elders.contains(a)));
                    assert!(elders.removed.iter().all(|r| !new_section_elders.contains(r)));
                }
            );
            Result::<()>::Ok(())
        })
        .await
}

/// Checking when we send AE info to a section from untrusted section, we do not handle it and
/// error out.
#[tokio::test]
async fn untrusted_ae_msg_errors() -> Result<()> {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async move {
            init_logger();
            let _span = tracing::info_span!("untrusted_ae_msg_errors").entered();

            let (dispatcher, section, _, sk_set) =
                network_utils::TestNodeBuilder::new(Prefix::default(), elder_count())
                    .build()
                    .await?;
            let pk = sk_set.secret_key().public_key();

            // a valid AE msg but with a non-verifiable SAP...
            let signed_sap = section.signed_sap();
            let bogus_section_pk = bls::SecretKey::random().public_key();
            let bogus_section_tree_update =
                SectionTreeUpdate::new(signed_sap.clone(), SectionsDAG::new(bogus_section_pk));

            let node_msg = NodeMsg::AntiEntropy {
                section_tree_update: bogus_section_tree_update,
                kind: AntiEntropyKind::Update {
                    members: BTreeSet::default(),
                },
            };

            let sender = network_utils::gen_info(MIN_ADULT_AGE, None);
            let wire_msg = WireMsg::single_src(
                &sender,
                Dst {
                    name: XorName::from(PublicKey::Bls(bogus_section_pk)),
                    section_key: bogus_section_pk,
                },
                node_msg.clone(),
                // we use the nonsense here
            )?;

            assert!(matches!(
                run_and_collect_cmds(
                    Cmd::ValidateMsg {
                        origin: sender.peer(),
                        wire_msg,
                        send_stream: None,
                    },
                    &dispatcher,
                )
                .await,
                Err(Error::NetworkKnowledge(
                    NetworkKnowledgeError::UntrustedProofChain(_)
                ))
            ));

            assert_eq!(
                dispatcher
                    .node()
                    .read()
                    .await
                    .network_knowledge()
                    .genesis_key(),
                &pk
            );
            assert_eq!(
                dispatcher
                    .node()
                    .read()
                    .await
                    .network_knowledge()
                    .section_tree()
                    .all()
                    .collect::<Vec<_>>(),
                vec![&signed_sap.value]
            );
            Result::<()>::Ok(())
        })
        .await
}

#[tokio::test]
async fn relocation_of_non_elder() -> Result<()> {
    relocation(RelocatedPeerRole::NonElder).await
}

fn threshold() -> usize {
    supermajority(elder_count()) - 1
}

enum RelocatedPeerRole {
    NonElder,
    Elder,
}

async fn relocation(relocated_peer_role: RelocatedPeerRole) -> Result<()> {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async move {
            let prefix: Prefix = "0".parse().map_err(|_| eyre!("Failed to parse integer to Prefix"))?;
            let section_size = match relocated_peer_role {
                RelocatedPeerRole::Elder => elder_count(),
                RelocatedPeerRole::NonElder => recommended_section_size(),
            };
            let (section_auth, mut nodes, sk_set) = random_sap(prefix, elder_count(), 0, None);
            let (mut section, section_key_share) =
                network_utils::create_section(&sk_set, &section_auth, None, None)?;

            let mut adults = section_size - elder_count();
            while adults > 0 {
                adults -= 1;
                let non_elder_peer = network_utils::create_peer(MIN_ADULT_AGE);
                let node_state = NodeState::joined(non_elder_peer, None);
                let node_state = section_signed(&sk_set.secret_key(), node_state);
                assert!(section.update_member(node_state));
            }

            let non_elder_peer = network_utils::create_peer(MIN_ADULT_AGE - 1);
            let node_state = NodeState::joined(non_elder_peer, None);
            let node_state = section_signed(&sk_set.secret_key(), node_state);
            assert!(section.update_member(node_state));
            let node = nodes.remove(0);
            let (max_capacity, root_storage_dir) = create_test_max_capacity_and_root_storage()?;
            let comm = network_utils::create_comm().await?;
            let node = MyNode::new(
                comm,
                node.keypair.clone(),
                section,
                Some(section_key_share),
                event_channel::new(network_utils::TEST_EVENT_CHANNEL_SIZE).0,
                UsedSpace::new(max_capacity),
                root_storage_dir,
            )
            .await?;
            let dispatcher = Dispatcher::new(Arc::new(RwLock::new(node)));

            let relocated_peer = match relocated_peer_role {
                RelocatedPeerRole::Elder => *section_auth.elders().nth(1).expect("too few elders"),
                RelocatedPeerRole::NonElder => non_elder_peer,
            };

            let membership_decision =
                network_utils::create_relocation_trigger(&sk_set, relocated_peer.age());
            let cmds = run_and_collect_cmds(
                Cmd::HandleMembershipDecision(membership_decision),
                &dispatcher,
            )
            .await?;

            let mut offline_relocate_sent = false;

            for cmd in cmds {
                let msg = match cmd {
                    Cmd::SendMsg {
                        msg: OutgoingMsg::Node(msg),
                        ..
                    } => msg,
                    _ => continue,
                };

                if let NodeMsg::Propose {
                    proposal: sn_interface::messaging::system::Proposal::VoteNodeOffline(node_state),
                    ..
                } = msg
                {
                    assert_eq!(node_state.name(), relocated_peer.name());
                    if let MembershipState::Relocated(relocate_details) = node_state.state() {
                        assert_eq!(relocate_details.age, relocated_peer.age() + 1);
                        offline_relocate_sent = true;
                    }
                }
            }

            assert!(offline_relocate_sent);
            Result::<()>::Ok(())
        })
        .await
}

#[tokio::test]
async fn msg_to_self() -> Result<()> {
    let local = tokio::task::LocalSet::new();
    local.run_until(async move {
        let info = network_utils::gen_info(MIN_ADULT_AGE, None);
        let (event_sender, _) = event_channel::new(network_utils::TEST_EVENT_CHANNEL_SIZE);
        let (comm_tx, mut comm_rx) = mpsc::channel(network_utils::TEST_EVENT_CHANNEL_SIZE);
        let comm = Comm::new(
            (Ipv4Addr::LOCALHOST, 0).into(),
            Default::default(),
            comm_tx,
        )
        .await?;
        let (max_capacity, root_storage_dir) = create_test_max_capacity_and_root_storage()?;

        let genesis_sk_set = bls::SecretKeySet::random(0, &mut rand::thread_rng());
        let (node, _) = MyNode::first_node(
            comm,
            info.keypair.clone(),
            event_sender,
            UsedSpace::new(max_capacity),
            root_storage_dir,
            genesis_sk_set,
        )
        .await?;
        let info = node.info();
        let dispatcher = Dispatcher::new(Arc::new(RwLock::new(node)));

        let node_msg = NodeMsg::NodeCmd(NodeCmd::ReplicateData(vec![]));

        // don't use the cmd collection fn, as it skips Cmd::SendMsg
        let cmds = dispatcher
            .process_cmd(
                Cmd::send_msg(OutgoingMsg::Node(node_msg.clone()), Peers::Single(info.peer()))
            )
            .await?;

        assert!(cmds.is_empty());

        let msg_type = assert_matches!(comm_rx.recv().await, Some(MsgFromPeer { sender, wire_msg, .. }) => {
            assert_eq!(sender.addr(), info.addr);
            assert_matches!(wire_msg.into_msg(), Ok(msg_type) => msg_type)
        });

        assert_matches!(msg_type, MsgType::Node { msg, .. } => {
            assert_eq!(
                msg,
                node_msg
            );
        });
        Result::<()>::Ok(())
   }).await
}

#[tokio::test]
async fn handle_elders_update() -> Result<()> {
    // The setup here is too complex for the TestNodeBuilder.
    let local = tokio::task::LocalSet::new();
    local.run_until(async move {
        init_logger();
        let _span = tracing::info_span!("handle_elders_update").entered();
        // Start with section that has `elder_count()` elders with age 6, 1 non-elder with age 5 and one
        // to-be-elder with age 7:
        let info = network_utils::gen_info(MIN_ADULT_AGE + 1, None);
        let mut other_elder_peers: Vec<_> = iter::repeat_with(|| network_utils::create_peer(MIN_ADULT_AGE + 1))
            .take(elder_count() - 1)
            .collect();
        let adult_peer = network_utils::create_peer(MIN_ADULT_AGE);
        let promoted_peer = network_utils::create_peer(MIN_ADULT_AGE + 2);

        let members = BTreeSet::from_iter(
            [info.peer(), adult_peer, promoted_peer]
                .into_iter()
                .map(|p| NodeState::joined(p, None)),
        );

        let sk_set0 = SecretKeySet::random(None);
        let pk0 = sk_set0.secret_key().public_key();

        let sap0 = SectionAuthorityProvider::new(
            iter::once(info.peer()).chain(other_elder_peers.clone()),
            Prefix::default(),
            members.clone(),
            sk_set0.public_keys(),
            0,
        );

        let (mut section0, section_key_share) = network_utils::create_section_with_elders(&sk_set0, &sap0)?;

        for peer in [&adult_peer, &promoted_peer] {
            let node_state = NodeState::joined(*peer, None);
            let node_state = section_signed(sk_set0.secret_key(), node_state);
            assert!(section0.update_member(node_state));
        }

        let demoted_peer = other_elder_peers.remove(0);

        let sk_set1 = SecretKeySet::random(None);

        let pk1 = sk_set1.secret_key().public_key();
        // Create `HandleAgreement` cmd for an `NewElders` proposal. This will demote one of the
        // current elders and promote the oldest peer.
        let sap1 = SectionAuthorityProvider::new(
            iter::once(info.peer())
                .chain(other_elder_peers.clone())
                .chain(iter::once(promoted_peer)),
            Prefix::default(),
            members,
            sk_set1.public_keys(),
            0,
        );
        let elder_names1: BTreeSet<_> = sap1.names();

        let signed_sap1 = section_signed(sk_set1.secret_key(), sap1);
        let proposal = Proposal::NewElders(signed_sap1.clone());
        let signature = sk_set0.secret_key().sign(&proposal.as_signable_bytes()?);
        let sig = SectionSig {
            signature,
            public_key: pk0,
        };

        let (event_sender, mut event_receiver) = event_channel::new(network_utils::TEST_EVENT_CHANNEL_SIZE);
        let (max_capacity, root_storage_dir) = create_test_max_capacity_and_root_storage()?;
        let comm = network_utils::create_comm().await?;
        let mut node = MyNode::new(
            comm,
            info.keypair.clone(),
            section0.clone(),
            Some(section_key_share),
            event_sender,
            UsedSpace::new(max_capacity),
            root_storage_dir,
        )
        .await?;

        // Simulate DKG round finished successfully by adding
        // the new section key share to our cache
        node.section_keys_provider
            .insert(network_utils::create_section_key_share(&sk_set1, 0));

        let dispatcher = Dispatcher::new(Arc::new(RwLock::new(node)));

        let cmds = run_and_collect_cmds(Cmd::HandleNewEldersAgreement { new_elders: signed_sap1, sig }, &dispatcher).await?;

        let mut update_actual_recipients = HashSet::new();

        for cmd in cmds {
            let (msg, recipients) = match cmd {
                Cmd::SendMsg {
                    msg: OutgoingMsg::Node(msg),
                    recipients: Peers::Multiple(recipients),
                    ..
                } => (msg, recipients),
                _ => continue,
            };

            let section_tree_update = match msg {
                NodeMsg::AntiEntropy { section_tree_update, kind: AntiEntropyKind::Update { .. }, .. } => section_tree_update,
                _ => continue,
            };

            assert_eq!(section_tree_update.proof_chain.last_key()?, pk1);
            // Merging the section contained in the message with the original section succeeds.
            assert!(section0.clone().update_knowledge_if_valid(section_tree_update, None, &info.name()).is_ok());

            update_actual_recipients.extend(recipients);
        }

        let update_expected_recipients: HashSet<_> = other_elder_peers
            .into_iter()
            .chain(iter::once(promoted_peer))
            .chain(iter::once(demoted_peer))
            .chain(iter::once(adult_peer))
            .collect();

        assert_eq!(update_actual_recipients, update_expected_recipients);

        assert_matches!(
            event_receiver.next().await,
            Some(Event::Membership(MembershipEvent::EldersChanged { elders, .. })) => {
                assert_eq!(elders.key, pk1);
                assert_eq!(elder_names1, elders.added.union(&elders.remaining).copied().collect());
                assert!(elders.removed.iter().all(|r| !elder_names1.contains(r)));
            }
        );

       Result::<()>::Ok(())
   }).await
}

/// Test that demoted node still sends `Sync` messages on split.
#[tokio::test]
async fn handle_demote_during_split() -> Result<()> {
    // The setup here is too complex for the TestNodeBuilder.
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async move {
            init_logger();
            let _span = tracing::info_span!("handle_demote_during_split").entered();

            let prefix0 = Prefix::default().pushed(false);
            let prefix1 = Prefix::default().pushed(true);

            //right not info/node could be in either section...
            let info = network_utils::gen_info(MIN_ADULT_AGE, None);
            let node_name = info.name();

            // These peers together with `node` are pre-split elders.
            // These peers together with `peer_c` are prefix-0 post-split elders.
            let peers_a: Vec<_> =
                iter::repeat_with(|| network_utils::create_peer_in_prefix(&prefix0, MIN_ADULT_AGE))
                    .take(elder_count() - 1)
                    .collect();
            // These peers are prefix-1 post-split elders.
            let peers_b: Vec<_> =
                iter::repeat_with(|| network_utils::create_peer_in_prefix(&prefix1, MIN_ADULT_AGE))
                    .take(elder_count())
                    .collect();
            // This peer is a prefix-0 post-split elder.
            let peer_c = network_utils::create_peer_in_prefix(&prefix0, MIN_ADULT_AGE);

            // all members
            let members = BTreeSet::from_iter(
                peers_a
                    .iter()
                    .chain(peers_b.iter())
                    .copied()
                    .chain([info.peer(), peer_c])
                    .map(|peer| NodeState::joined(peer, None)),
            );

            // Create the pre-split section
            let sk_set_v0 = SecretKeySet::random(None);
            let section_auth_v0 = SectionAuthorityProvider::new(
                iter::once(info.peer()).chain(peers_a.iter().cloned()),
                Prefix::default(),
                members.clone(),
                sk_set_v0.public_keys(),
                0,
            );
            let (mut section, section_key_share) =
                network_utils::create_section_with_elders(&sk_set_v0, &section_auth_v0)?;

            // all peers b are added
            for peer in peers_b.iter().chain(iter::once(&peer_c)).cloned() {
                let node_state = NodeState::joined(peer, None);
                let node_state = section_signed(sk_set_v0.secret_key(), node_state);
                assert!(section.update_member(node_state));
            }

            // we make a new full node from info, to see what it does
            let (event_sender, _) = event_channel::new(network_utils::TEST_EVENT_CHANNEL_SIZE);
            let (max_capacity, root_storage_dir) = create_test_max_capacity_and_root_storage()?;
            let comm = network_utils::create_comm().await?;
            let mut node = MyNode::new(
                comm,
                info.keypair.clone(),
                section,
                Some(section_key_share),
                event_sender,
                UsedSpace::new(max_capacity),
                root_storage_dir,
            )
            .await?;

            let sk_set_v1_p0 = SecretKeySet::random(None);
            let sk_set_v1_p1 = SecretKeySet::random(None);

            // Simulate DKG round finished succesfully by adding the new section
            // key share to our cache (according to which split section we'll belong to).
            if prefix0.matches(&node_name) {
                node.section_keys_provider
                    .insert(network_utils::create_section_key_share(&sk_set_v1_p0, 0));
            } else {
                node.section_keys_provider
                    .insert(network_utils::create_section_key_share(&sk_set_v1_p1, 0));
            }

            let dispatcher = Dispatcher::new(Arc::new(RwLock::new(node)));

            // Create agreement on `OurElder` for both sub-sections
            let create_our_elders_cmd =
                |signed_sap: SectionSigned<SectionAuthorityProvider>| -> Result<_> {
                    let proposal = Proposal::NewElders(signed_sap.clone());
                    let signature = sk_set_v0.secret_key().sign(&proposal.as_signable_bytes()?);
                    let sig = SectionSig {
                        signature,
                        public_key: sk_set_v0.public_keys().public_key(),
                    };

                    Ok(Cmd::HandleNewEldersAgreement {
                        new_elders: signed_sap,
                        sig,
                    })
                };

            // Handle agreement on `NewElders` for prefix-0.
            let section_auth = SectionAuthorityProvider::new(
                peers_a.iter().cloned().chain(iter::once(peer_c)),
                prefix0,
                members.clone(),
                sk_set_v1_p0.public_keys(),
                0,
            );

            let signed_sap = section_signed(sk_set_v1_p0.secret_key(), section_auth);
            let cmd = create_our_elders_cmd(signed_sap)?;
            let mut cmds = run_and_collect_cmds(cmd, &dispatcher).await?;

            // Handle agreement on `NewElders` for prefix-1.
            let section_auth = SectionAuthorityProvider::new(
                peers_b.iter().cloned(),
                prefix1,
                members,
                sk_set_v1_p1.public_keys(),
                0,
            );

            let signed_sap = section_signed(sk_set_v1_p1.secret_key(), section_auth);
            let cmd = create_our_elders_cmd(signed_sap)?;

            let new_cmds = run_and_collect_cmds(cmd, &dispatcher).await?;

            cmds.extend(new_cmds);

            let mut update_recipients = BTreeMap::new();

            for cmd in cmds {
                let (msg, recipients) = match cmd {
                    Cmd::SendMsg {
                        msg: OutgoingMsg::Node(msg),
                        recipients: Peers::Multiple(recipients),
                        ..
                    } => (msg, recipients),
                    _ => continue,
                };

                if matches!(
                    msg,
                    NodeMsg::AntiEntropy {
                        kind: AntiEntropyKind::Update { .. },
                        ..
                    }
                ) {
                    for recipient in recipients {
                        let _old = update_recipients.insert(recipient.name(), recipient.addr());
                    }
                }
            }

            // our node's whole section
            assert_eq!(update_recipients.len(), elder_count());
            Result::<()>::Ok(())
        })
        .await
}

#[tokio::test]
async fn spentbook_spend_spent_proof_with_invalid_pk_should_return_spentbook_error() -> Result<()> {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async move {
            init_logger();
            let replication_count = 5;
            let (dispatcher, _, peer, sk_set) =
                network_utils::TestNodeBuilder::new(Prefix::default().pushed(true), elder_count())
                    .adult_count(6)
                    .section_sk_threshold(0)
                    .data_copy_count(replication_count)
                    .build()
                    .await?;

            let (key_image, tx, mut spent_proofs, spent_transactions) =
                dbc_utils::get_genesis_dbc_spend_info(&sk_set)?;
            let pk = bls::SecretKey::random().public_key();
            spent_proofs = spent_proofs
                .into_iter()
                .map(|mut proof| {
                    proof.spentbook_pub_key = pk;
                    proof
                })
                .collect();

            let cmds = run_node_handle_client_msg_and_collect_cmds(
                ClientMsg::Cmd(DataCmd::Spentbook(SpentbookCmd::Spend {
                    key_image,
                    tx,
                    spent_proofs,
                    spent_transactions,
                    network_knowledge: None,
                })),
                peer,
                &dispatcher,
            )
            .await?;

            for cmd in cmds {
                match cmd {
                    Cmd::SendMsg {
                        msg:
                            OutgoingMsg::Client(ClientMsg::CmdResponse {
                                response: CmdResponse::SpendKey(result),
                                ..
                            }),
                        ..
                    } => {
                        if let Some(error) = result.err() {
                            assert_eq!(
                                error.to_string(),
                                MessagingDataError::from(Error::SpentbookError(format!(
                                    "Spent proof signature {:?} is invalid",
                                    pk
                                )))
                                .to_string(),
                                "A different error was expected for this case: {:?}",
                                error
                            );
                            return Ok(());
                        } else {
                            bail!("We expected an error to be returned");
                        }
                    }
                    _ => continue,
                }
            }

            bail!("We expected an error to be returned");
        })
        .await
}

#[tokio::test]
async fn spentbook_spend_spent_proof_with_key_not_in_section_chain_should_return_cmd_error_response(
) -> Result<()> {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async move {
            init_logger();
            let replication_count = 5;
            let (dispatcher, _, peer, _) =
                network_utils::TestNodeBuilder::new(Prefix::default().pushed(true), elder_count())
                    .adult_count(6)
                    .section_sk_threshold(0)
                    .data_copy_count(replication_count)
                    .build()
                    .await?;

            let sk_set = bls::SecretKeySet::random(0, &mut rand::thread_rng());
            let (key_image, tx, spent_proofs, spent_transactions) =
                dbc_utils::get_genesis_dbc_spend_info(&sk_set)?;

            let cmds = run_node_handle_client_msg_and_collect_cmds(
                ClientMsg::Cmd(DataCmd::Spentbook(SpentbookCmd::Spend {
                    key_image,
                    tx,
                    spent_proofs,
                    spent_transactions,
                    network_knowledge: None,
                })),
                peer,
                &dispatcher,
            )
            .await?;

            let section_key = sk_set.public_keys().public_key();

            for cmd in cmds {
                match cmd {
                    Cmd::SendMsg {
                        msg:
                            OutgoingMsg::Client(ClientMsg::CmdResponse {
                                response: CmdResponse::SpendKey(result),
                                ..
                            }),
                        ..
                    } => {
                        if let Some(error) = result.err() {
                            assert_eq!(
                                error.to_string(),
                                Error::SpentProofUnknownSectionKey(section_key).to_string(),
                                "A different error was expected for this case: {:?}",
                                error
                            );
                            return Ok(());
                        } else {
                            bail!("We expected an error to be returned");
                        }
                    }
                    _ => continue,
                }
            }

            bail!("We expected an error to be returned");
        })
        .await
}

#[tokio::test]
async fn spentbook_spend_spent_proofs_do_not_relate_to_input_dbcs_should_return_spentbook_error(
) -> Result<()> {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async move {
            init_logger();
            let replication_count = 5;
            let (dispatcher, section, peer, sk_set) =
                network_utils::TestNodeBuilder::new(Prefix::default().pushed(true), elder_count())
                    .adult_count(6)
                    .section_sk_threshold(0)
                    .data_copy_count(replication_count)
                    .build()
                    .await?;

            // The idea for this test case is to pass the wrong spent proofs and transactions for
            // the key image we're trying to spend. To do so, we reissue `new_dbc` from
            // `genesis_dbc`, then reissue `new_dbc2` from `new_dbc`, then when we try to spend
            // `new_dbc2`, we use the spent proofs/transactions from `genesis_dbc`. This should
            // not be permitted. The correct way would be to pass the spent proofs/transactions
            // from `new_dbc`, which was our input to `new_dbc2`.
            let sap = section.section_auth();
            let keys_provider = dispatcher.node().read().await.section_keys_provider.clone();
            let genesis_dbc = gen_genesis_dbc(&sk_set, &sk_set.secret_key())?;
            let new_dbc = reissue_dbc(
                &genesis_dbc,
                10,
                &bls::SecretKey::random(),
                &sap,
                &keys_provider,
            )?;
            let new_dbc2_sk = bls::SecretKey::random();
            let new_dbc2 = reissue_dbc(&new_dbc, 5, &new_dbc2_sk, &sap, &keys_provider)?;

            let cmds = run_node_handle_client_msg_and_collect_cmds(
                ClientMsg::Cmd(DataCmd::Spentbook(SpentbookCmd::Spend {
                    key_image: new_dbc2_sk.public_key(),
                    tx: new_dbc2.transaction.clone(),
                    spent_proofs: genesis_dbc.spent_proofs.clone(),
                    spent_transactions: genesis_dbc.spent_transactions.clone(),
                    network_knowledge: None,
                })),
                peer,
                &dispatcher,
            )
            .await?;

            for cmd in cmds {
                match cmd {
                    Cmd::SendMsg {
                        msg:
                            OutgoingMsg::Client(ClientMsg::CmdResponse {
                                response: CmdResponse::SpendKey(result),
                                ..
                            }),
                        ..
                    } => {
                        if let Some(error) = result.err() {
                            assert_eq!(
                                error.to_string(),
                                MessagingDataError::from(Error::DbcError(
                                    sn_dbc::Error::CommitmentsInputLenMismatch {
                                        current: 0,
                                        expected: 1
                                    }
                                ))
                                .to_string(),
                                "A different error was expected for this case: {:?}",
                                error
                            );
                            return Ok(());
                        } else {
                            bail!("We expected an error to be returned");
                        }
                    }
                    _ => continue,
                }
            }

            bail!("We expected an error to be returned");
        })
        .await
}

#[tokio::test]
async fn spentbook_spend_transaction_with_no_inputs_should_return_spentbook_error() -> Result<()> {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async move {
            init_logger();
            let replication_count = 5;
            let (dispatcher, section, peer, sk_set) =
                network_utils::TestNodeBuilder::new(Prefix::default().pushed(true), elder_count())
                    .adult_count(6)
                    .section_sk_threshold(0)
                    .data_copy_count(replication_count)
                    .build()
                    .await?;

            // These conditions will produce a failure on `tx.verify` in the message handler.
            let sap = section.section_auth();
            let keys_provider = dispatcher.node().read().await.section_keys_provider.clone();
            let genesis_dbc = gen_genesis_dbc(&sk_set, &sk_set.secret_key())?;
            let new_dbc = reissue_dbc(
                &genesis_dbc,
                10,
                &bls::SecretKey::random(),
                &sap,
                &keys_provider,
            )?;
            let new_dbc2_sk = bls::SecretKey::random();
            let new_dbc2 =
                dbc_utils::reissue_invalid_dbc_with_no_inputs(&new_dbc, 5, &new_dbc2_sk)?;

            let cmds = run_node_handle_client_msg_and_collect_cmds(
                ClientMsg::Cmd(DataCmd::Spentbook(SpentbookCmd::Spend {
                    key_image: new_dbc2_sk.public_key(),
                    tx: new_dbc2.transaction.clone(),
                    spent_proofs: new_dbc.spent_proofs.clone(),
                    spent_transactions: new_dbc.spent_transactions.clone(),
                    network_knowledge: None,
                })),
                peer,
                &dispatcher,
            )
            .await?;

            for cmd in cmds {
                match cmd {
                    Cmd::SendMsg {
                        msg:
                            OutgoingMsg::Client(ClientMsg::CmdResponse {
                                response: CmdResponse::SpendKey(result),
                                ..
                            }),
                        ..
                    } => {
                        if let Some(error) = result.err() {
                            assert_eq!(
                                error.to_string(),
                                MessagingDataError::from(Error::SpentbookError(
                                    "The DBC transaction must have at least one input".to_string()
                                ))
                                .to_string(),
                                "A different error was expected for this case: {:?}",
                                error
                            );
                            return Ok(());
                        } else {
                            bail!("We expected an error to be returned");
                        }
                    }
                    _ => continue,
                }
            }

            bail!("We expected an error to be returned");
        })
        .await
}

/// This could potentially be the start of a case for the updated proof chain and SAP being sent
/// with the spend request, but I don't know exactly what the conditions are for getting the
/// network knowledge to update correctly.
#[tokio::test]
#[ignore = "Needs to be refactored to take into account that ClientMsgs require a stream (or to avoid this)"]
async fn spentbook_spend_with_updated_network_knowledge_should_update_the_node() -> Result<()> {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async move {
            init_logger();
            let replication_count = 5;
            let (dispatcher, section, peer, genesis_sk_set) =
                network_utils::TestNodeBuilder::new(Prefix::default().pushed(false), elder_count())
                    .adult_count(6)
                    .section_sk_threshold(0)
                    .data_copy_count(replication_count)
                    .build()
                    .await?;

            // At this point, only the genesis key should be in the proof chain on this node.
            let tree = dispatcher
                .node()
                .read()
                .await
                .network_knowledge()
                .section_tree()
                .clone();
            let proof_chain = tree.get_sections_dag().clone();
            assert_eq!(proof_chain.keys().into_iter().count(), 1);

            // This will create a section with the following proof chain:
            // genesis_key -> other_section_key
            // The key share also needs to be added to the section keys provider, which is stored
            // on the node.
            let other_section_key = bls::SecretKey::random();
            let (other_section, _, other_section_key_share) =
                network_utils::TestNodeBuilder::new(Prefix::default().pushed(true), elder_count())
                    .genesis_sk_set(genesis_sk_set.clone())
                    .parent_section_tree(section.section_tree().clone())
                    .adult_count(6)
                    .section_sk_threshold(0)
                    .other_section_keys(vec![other_section_key.clone()])
                    .build_section()
                    .await?;
            dispatcher
                .node()
                .write()
                .await
                .section_keys_provider
                .insert(other_section_key_share.clone());

            // Reissue a couple of DBC from genesis. They will be reissued using the section keys
            // provider and SAP from the other section, hence the spent proofs will be signed with
            // the unknown section key.
            // The owners of the DBCs here don't really matter, so we just use random keys.
            let skp = SectionKeysProvider::new(Some(other_section_key_share.clone()));
            let sap = other_section.signed_sap();
            let genesis_dbc = gen_genesis_dbc(&genesis_sk_set, &genesis_sk_set.secret_key())?;
            let new_dbc = reissue_dbc(&genesis_dbc, 10, &bls::SecretKey::random(), &sap, &skp)?;
            let new_dbc2 = reissue_dbc(&new_dbc, 5, &bls::SecretKey::random(), &sap, &skp)?;
            let new_dbc2_spent_proof =
                new_dbc2.spent_proofs.iter().next().ok_or_else(|| {
                    eyre!("This DBC should have been reissued with a spent proof")
                })?;
            assert_eq!(
                new_dbc2_spent_proof.spentbook_pub_key,
                other_section_key.public_key()
            );

            // Finally, spend new_dbc2 as part of the input for another reissue.
            // It needs to be associated with a valid transaction, which is why the util function
            // is used. Again, the owner of the output DBCs don't really matter, so a random key is
            // used.
            let proof_chain = other_section.section_chain();
            let (key_image, tx) =
                get_input_dbc_spend_info(&new_dbc2, 2, &bls::SecretKey::random())?;

            let cmds = run_node_handle_client_msg_and_collect_cmds(
                ClientMsg::Cmd(DataCmd::Spentbook(SpentbookCmd::Spend {
                    key_image,
                    tx,
                    spent_proofs: new_dbc2.spent_proofs,
                    spent_transactions: new_dbc2.spent_transactions,
                    network_knowledge: Some((proof_chain, sap)),
                })),
                peer,
                &dispatcher,
            )
            .await?;

            // The commands returned here should include the new command to update the network
            // knowledge and also the other two commands to replicate the spent proof shares and
            // the ack command, but we've already validated the other two as part of another test.
            assert_eq!(cmds.len(), 3);
            let update_cmd = cmds[0].clone();
            assert_matches!(update_cmd, Cmd::UpdateNetworkAndHandleValidClientMsg { .. });

            // Now the proof chain should have the other section key.
            let tree = dispatcher
                .node()
                .read()
                .await
                .network_knowledge()
                .section_tree()
                .clone();
            let proof_chain = tree.get_sections_dag().clone();
            assert_eq!(proof_chain.keys().into_iter().count(), 2);
            let mut proof_chain_iter = proof_chain.keys();
            let genesis_key = genesis_sk_set.public_keys().public_key();
            assert_eq!(
                genesis_key,
                proof_chain_iter
                    .next()
                    .ok_or_else(|| eyre!("The proof chain should include the genesis key"))?
            );
            assert_eq!(
                other_section_key.public_key(),
                proof_chain_iter
                    .next()
                    .ok_or_else(|| eyre!("The proof chain should include the other section key"))?
            );

            Result::<()>::Ok(())
        })
        .await?;

    Ok(())
}
