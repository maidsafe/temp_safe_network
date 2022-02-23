// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use bls::Signature;
use sn_membership::{consensus::VoteResponse, Ballot, Reconfig, SignedVote};
use std::collections::{BTreeMap, BTreeSet};
use tiny_keccak::{Hasher, Sha3};
use xor_name::XorName;

use crate::elder_count;
use crate::messaging::system::{
    KeyedSig, MembershipState, NodeState, RelocateDetails, SectionAuth,
};
use crate::node::{
    api::cmds::Cmd,
    core::{
        relocation::{find_nodes_to_relocate, ChurnId, RelocateDetailsUtils},
        Node,
    },
    ed25519::Digest256,
    Event, Peer, MIN_ADULT_AGE,
};
use crate::types::log_markers::LogMarker;

impl Node {
    #[instrument(skip(self), level = "trace")]
    pub(crate) async fn handle_membership_msg(
        &self,
        signed_vote: SignedVote<Reconfig<NodeState>>,
    ) -> Vec<Cmd> {
        debug!(">>> {}: {:?}", LogMarker::MembershipMsg, signed_vote);

        // Before we handle the signed vote msg, let's verify we are ok with voting as well
        if !self.do_we_agree_with_vote_msg(&signed_vote).await {
            return vec![];
        }

        let state_write_guard = &mut *self.network_knowledge.membership_voting.write().await;
        let state = if let Some(state) = state_write_guard {
            state
        } else {
            warn!(
                ">>> Dropping membership Vote msg since we don't hold a membership voting state: {:?}",
                signed_vote
            );
            return vec![];
        };

        match state.handle_signed_vote(signed_vote.clone()) {
            Err(err) => {
                error!(">>> Failed to handle membership Vote msg: {:?}", err);
                vec![]
            }
            Ok(VoteResponse::Broadcast(signed_vote)) => {
                trace!(
                    ">>> Membership Vote msg successfully handled, broadcasting our vote: {:?}",
                    signed_vote
                );
                self.broadcast_membership_vote_msg(signed_vote).await
            }
            Ok(VoteResponse::WaitingForMoreVotes) => {
                trace!(
                    ">>> Membership Vote msg successfully handled, awaiting for more votes: {:?}",
                    signed_vote
                );
                vec![]
            }
            Ok(VoteResponse::Decided(decision)) => {
                trace!(
                    ">>> Membership Vote msg successfully handled and consensus reached: {:?}",
                    signed_vote
                );
                // We now update our knowledge of peers in our section as per consensus
                let mut section_peers = self.network_knowledge.section_signed_members().await;
                let public_key = self.network_knowledge.section_key().await;
                for (reconfig, signature) in decision.proposals.clone().into_iter() {
                    let sig = KeyedSig {
                        public_key,
                        signature,
                    };

                    match reconfig {
                        Reconfig::Join(value) => {
                            let _ = section_peers
                                .insert(SectionAuth { value, sig }.into_authed_state());
                        }
                        Reconfig::Leave(value) => {
                            let _ = section_peers
                                .remove(&SectionAuth { value, sig }.into_authed_state());
                        }
                    }
                }

                self.network_knowledge.set_members(section_peers).await;

                self.handle_new_membership_consensus(decision.proposals)
                    .await
            }
        }
    }

    // Private helper to check if we agree with reconfigs in an incoming signed_vote msg
    async fn do_we_agree_with_vote_msg(
        &self,
        signed_vote: &SignedVote<Reconfig<NodeState>>,
    ) -> bool {
        let mut reconfigs = BTreeSet::default();
        populate_set_of_reconfings(signed_vote, &mut reconfigs);
        warn!(
            ">>> Checking if we agree with {} reconfigs before voting: {:?}",
            reconfigs.len(),
            signed_vote
        );

        // Are we accepting joins now?
        let joins_allowed = *self.joins_allowed.read().await;

        for reconfig in reconfigs {
            match reconfig {
                Reconfig::Join(_) if !joins_allowed => {
                    warn!(">>> Dropping membership Join vote msg since joins are currently not allowed: {:?}", signed_vote);
                    return false;
                }
                Reconfig::Join(node_state_msg) => {
                    let node_state = node_state_msg.into_state();
                    // Check if node wasn't relocated or perhaps was a member in the past and left
                    if self
                        .network_knowledge
                        .is_either_member_or_archived(&node_state.name())
                        .await
                        .is_some()
                    {
                        warn!(">>> Dropping membership Join vote msg since node is rejoining to our section: {:?}", signed_vote);
                        return false;
                    }

                    // Check section key matches joining node's name
                    if !self
                        .network_knowledge
                        .prefix()
                        .await
                        .matches(&node_state.name())
                    {
                        debug!(">>> Dropping membershp Join vote since node name ({}) doesn't match our prefix: {:?}",
                            node_state.name(), signed_vote);
                        return false;
                    }

                    // Joining node's age check
                    let current_section_size = self.network_knowledge.section_size().await;
                    let members = self.network_knowledge.section_members().await;
                    let (is_age_invalid, expected_age) =
                        self.verify_joining_node_age(node_state.peer()).await;

                    if is_age_invalid {
                        let adults = self.network_knowledge.adults().await.len();
                        let elders = self
                            .network_knowledge
                            .authority_provider()
                            .await
                            .elder_count();
                        debug!(">>> Dropping membershp Join vote for {:?} since node age ({}) is not the expected ({}), local section knowledge: {:?} Elders + {:?} Adults, section size {}, members: {:?}",
                            node_state.name(), node_state.age(), expected_age, elders, adults,current_section_size,members);
                        return false;
                    }
                }
                Reconfig::Leave(node_state) => {
                    unimplemented!();
                    /*
                    let churn_name = node_state.value.name;
                    let churn_signature = node_state.sig;
                    if let Some((node_state, relocate_details)) =
                        find_nodes_to_relocate(&self.network_knowledge, churn_name, churn_signature)
                            .await
                    {
                        // if the node_state that is being vote is the same as the one
                        // we see that needs to be relocated then we vote for it
                        // TODO!!
                    } else {
                        // We don't see any member needs to be relocated, so we don't vote
                        return false;
                    }
                    */
                }
            }
        }

        true
    }

    async fn handle_new_membership_consensus(
        &self,
        changes: BTreeMap<Reconfig<NodeState>, Signature>,
    ) -> Vec<Cmd> {
        // All joining nodes will be excluded from relocation.
        let mut excluded_from_relocation = BTreeSet::default();

        // All the NodeState signatures will be hashed together to form the ChurnId,
        // thus we accumulate them to have a deterministic order.
        let mut signatures = BTreeSet::new();

        let mut cmds = vec![];
        for (reconfig, signature) in changes {
            match reconfig {
                Reconfig::Join(node_state) => {
                    let node_name = node_state.name;
                    info!(">>> JOIN RECONFIG AFTER HANDLING for: {:?}", node_name);

                    let keyed_sig = KeyedSig {
                        public_key: self.network_knowledge.section_key().await,
                        signature: signature.clone(),
                    };

                    cmds.extend(self.handle_join_agreement(node_state, keyed_sig).await);

                    let _inserted = excluded_from_relocation.insert(node_name);
                    let _inserted = signatures.insert(signature.to_bytes());
                }
                Reconfig::Leave(node_state) => {
                    info!(
                        ">>> LEAVE RECONFIG AFTER HANDLING for: {:?}",
                        node_state.name
                    );

                    let peer = Peer::new(node_state.name, node_state.addr);

                    let sig = KeyedSig {
                        public_key: self.network_knowledge.section_key().await,
                        signature: signature.clone(),
                    };
                    let signed_node_state = SectionAuth {
                        value: node_state,
                        sig,
                    };

                    // If this is a Leave agreement where the new node state is Relocated,
                    // we then need to send the Relocate msg to the peer, attaching the
                    // signed NodeState with the relocation details.
                    if matches!(signed_node_state.value.state, MembershipState::Relocated(_)) {
                        match self.send_relocate(peer.clone(), signed_node_state).await {
                            Ok(relocate_cmds) => cmds.push(relocate_cmds),
                            Err(err) => {
                                warn!(
                                    ">>> Failed to generate Relocate msg for {:?}: {}",
                                    peer, err
                                )
                            }
                        }
                    }

                    if let Err(err) = self
                        .liveness_retain_only(
                            self.network_knowledge
                                .adults()
                                .await
                                .iter()
                                .map(|peer| peer.name())
                                .collect(),
                        )
                        .await
                    {
                        warn!(
                            ">>> Failed to update our Adults liveness check records: {}",
                            err
                        );
                    }

                    let _inserted = signatures.insert(signature.to_bytes());
                }
            };
        }

        // Generate cmds to relocate peers if necessary.
        // All the NodeState signatures will be hashed together to form the ChurnId.
        let mut signatures_hasher = Sha3::v256();
        let mut hash = Digest256::default();
        signatures
            .iter()
            .for_each(|sig| signatures_hasher.update(sig));
        signatures_hasher.finalize(&mut hash);
        let churn_id = ChurnId(hash.to_vec());
        cmds.extend(
            self.relocate_peers(churn_id, excluded_from_relocation)
                .await,
        );

        self.log_section_stats().await;

        // Using new membership information, make sure our flag to allow joins is up to date
        self.update_joins_allowed_flag().await;

        // Send AE-Update to Adults of our section
        let our_adults = self.network_knowledge.adults().await;
        let our_prefix = self.network_knowledge.prefix().await;
        let our_section_pk = self.network_knowledge.section_key().await;
        cmds.extend(
            self.send_ae_update_to_nodes(our_adults, &our_prefix, our_section_pk)
                .await,
        );

        // Check up to date members to generate cmds
        // to promote and/or demote peers accordingly.
        match self.promote_and_demote_elders().await {
            Ok(new_cmds) => cmds.extend(new_cmds),
            Err(err) => warn!(
                ">>> An error occurred when trying to check for promotions/demotions: {:?}",
                err
            ),
        }

        info!(
            ">>> Commands in queue for accepting joining node {:?}",
            cmds
        );
        self.print_network_stats().await;

        cmds
    }

    async fn handle_join_agreement(&self, new_node_state: NodeState, sig: KeyedSig) -> Vec<Cmd> {
        debug!("{}", LogMarker::AgreementOfJoin);
        /*
        FIXME !!!!!!!: at this point we've already updated our section_peers set
        if let Some(old_node_state) = self
            .network_knowledge
            .is_either_member_or_archived(&new_node_state.name)
            .await
        {
            return self
                .handle_rejoining_node(old_node_state.into_authed_msg(), new_node_state)
                .await;
        }
        */

        self.add_new_adult_to_trackers(new_node_state.name).await;

        info!(
            ">>> Joining node has been approved: {} at {}",
            new_node_state.name, new_node_state.addr
        );

        // still used for testing
        self.send_event(Event::MemberJoined {
            name: new_node_state.name,
            previous_name: new_node_state.previous_name,
            age: new_node_state.age(),
        })
        .await;

        // Generate the approval to be sent to the joining node
        let signed_node_state = SectionAuth {
            value: new_node_state,
            sig,
        };

        self.send_node_approval(signed_node_state).await
    }

    // TODO !!!: make these validations before voting
    async fn handle_rejoining_node(
        &self,
        old_node_state: SectionAuth<NodeState>,
        new_node_state: NodeState,
    ) -> Vec<Cmd> {
        // This node is rejoining with same name. We allow it only if we are aware it
        // previously left, and only if halving its previous age would still be over
        // the MIN_ADULT_AGE, in which case we'll relocate it immediatelly with half its age.
        if old_node_state.state != MembershipState::Left {
            debug!(
                ">>> Ignoring Joining node {} - previous state ('{:?}') is not 'Left'.",
                new_node_state.name, old_node_state.state,
            );

            return vec![];
        }

        let new_age = old_node_state.age() / 2;
        if new_age >= MIN_ADULT_AGE {
            let mut cmds = vec![];
            cmds.extend(self.send_node_approval(old_node_state.clone()).await);

            let peer = Peer::new(new_node_state.name, new_node_state.addr);
            let details =
                RelocateDetails::with_age(&self.network_knowledge, &peer, peer.name(), new_age);
            trace!(
                ">>> Relocating {:?} to {} with age {} due to rejoin",
                peer,
                details.dst,
                details.age
            );

            cmds.extend(
                self.propose_remove_from_membership(
                    old_node_state.value.into_state().relocate(details),
                )
                .await,
            );

            cmds
        } else {
            debug!(
                ">>> Ignoring Joining node {} - halving its previous age ({}) goes below MIN_ADULT_AGE ({}).",
                new_node_state.name,
                old_node_state.age(),
                MIN_ADULT_AGE
            );
            vec![]
        }
    }

    async fn relocate_peers(&self, churn_id: ChurnId, excluded: BTreeSet<XorName>) -> Vec<Cmd> {
        // Do not carry out relocations in the first section
        if self.network_knowledge.prefix().await.is_empty() {
            return vec![];
        }

        // Do not carry out relocation when there is not enough elder nodes.
        if self
            .network_knowledge
            .authority_provider()
            .await
            .elder_count()
            < elder_count()
        {
            return vec![];
        }

        let mut cmds = vec![];
        for (node_state, relocate_details) in
            find_nodes_to_relocate(&self.network_knowledge, &churn_id, excluded).await
        {
            debug!(
                ">>> Relocating {:?} to {} (on churn of {})",
                node_state.peer(),
                relocate_details.dst,
                churn_id
            );

            cmds.extend(
                self.propose_remove_from_membership(node_state.relocate(relocate_details))
                    .await,
            )
        }

        cmds
    }
}

fn populate_set_of_reconfings(
    signed_vote: &SignedVote<Reconfig<NodeState>>,
    reconfigs: &mut BTreeSet<Reconfig<NodeState>>,
) {
    match &signed_vote.vote.ballot {
        Ballot::Propose(reconfig) => {
            let _ = reconfigs.insert(reconfig.clone());
        }
        Ballot::Merge(votes) | Ballot::SuperMajority { votes, .. } => {
            votes.iter().for_each(|signed_vote| {
                populate_set_of_reconfings(signed_vote, reconfigs);
            })
        }
    }
}
