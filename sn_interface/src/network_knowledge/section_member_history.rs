// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::network_knowledge::{
    errors::{Error, Result},
    MembershipState, NodeState, SectionsDAG,
};
use sn_consensus::Decision;
use std::collections::{BTreeMap, BTreeSet};
use xor_name::{Prefix, XorName};

// Number of Elder churn events before a Left/Relocated member
// can be removed from the section members archive.
#[cfg(not(test))]
const ELDER_CHURN_EVENTS_TO_PRUNE_ARCHIVE: usize = 5;
#[cfg(test)]
const ELDER_CHURN_EVENTS_TO_PRUNE_ARCHIVE: usize = 3;

/// Container for storing information about (current and archived) members of our section.
#[derive(Clone, Default, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(super) struct SectionMemberHistory {
    /// Initial members snapshot at the time of SAP change
    initial_members: BTreeSet<NodeState>,
    /// Decisions received during the current SAP so far
    decisions: Vec<Decision<NodeState>>,
    /// Archived decisions containing left nodes, to avoid re-join with same id.
    archive: BTreeMap<XorName, Decision<NodeState>>,
}

impl SectionMemberHistory {
    /// Reset the bootstrap members when SAP changed.
    pub(crate) fn reset_initial_members(&mut self, new_initial_members: BTreeSet<NodeState>) {
        self.initial_members = new_initial_members;
        self.decisions = Default::default();
    }

    /// Returns set of current members, i.e. those with state == `Joined`.
    pub(crate) fn members(&self) -> BTreeSet<NodeState> {
        self.members_at_gen(self.decisions.len() as u64)
            .values()
            .cloned()
            .collect()
    }

    /// Returns section decisions since last SAP change
    pub(crate) fn section_decisions(&self) -> Vec<Decision<NodeState>> {
        self.decisions.clone()
    }

    /// Returns joined members at the specific generation.
    pub(crate) fn members_at_gen(&self, gen: u64) -> BTreeMap<XorName, NodeState> {
        if gen as usize > self.decisions.len() {
            return BTreeMap::new();
        }

        let mut members = BTreeMap::from_iter(
            self.initial_members
                .iter()
                .filter(|n| matches!(n.state(), MembershipState::Joined))
                .map(|n| (n.name(), n.clone())),
        );

        if gen == 0 {
            return members;
        }

        for i in 0..gen as usize {
            for node_state in self.decisions[i].proposals.keys() {
                trace!("SectionPeers::members checking against {node_state:?}");
                match node_state.state() {
                    MembershipState::Joined => {
                        let _ = members.insert(node_state.name(), node_state.clone());
                    }
                    MembershipState::Left | MembershipState::Relocated(_) => {
                        let _ = members.remove(&node_state.name());
                    }
                }
            }
        }

        members
    }

    /// Returns set of archived members, i.e those that've left our section
    pub(super) fn archived_members(&self) -> BTreeSet<NodeState> {
        let mut node_state_list = BTreeSet::new();
        for (name, decision) in self.archive.iter() {
            if let Some(node_state) = decision
                .proposals
                .keys()
                .find(|state| state.name() == *name)
            {
                let _ = node_state_list.insert(node_state.clone());
            }
        }
        node_state_list
    }

    /// Get the `NodeState` for the member with the given name.
    pub(super) fn get(&self, name: &XorName) -> Option<NodeState> {
        self.members()
            .iter()
            .find(|node_state| node_state.name() == *name)
            .cloned()
    }

    /// Returns whether the given node is currently a member of our section.
    pub(super) fn is_member(&self, name: &XorName) -> bool {
        self.get(name).is_some()
    }

    /// Update a member of our section.
    /// Returns whether anything actually changed.
    /// TODO: shall we still need to carry out checks to maintain commutativity?
    ///       the only allowed transitions are:
    ///         - Joined -> Left
    ///         - Joined -> Relocated
    ///         - Relocated <--> Left (should not happen, but needed for consistency)
    pub(super) fn update(&mut self, new_decision: Decision<NodeState>) -> Result<bool> {
        let incoming_generation = match new_decision.generation() {
            Ok(generation) => generation as usize,
            Err(err) => {
                trace!("Failed to get generation from {new_decision:?} with error {err:?}");
                return Err(Error::Consensus(err));
            }
        };

        trace!(
            "incoming_generation {incoming_generation:?} self.decisions.len() {:?}",
            self.decisions.len()
        );

        if incoming_generation == self.decisions.len() + 1 {
            self.decisions.push(new_decision.clone());
            trace!("Pushed decision {new_decision:?}");

            for (node, _) in new_decision.proposals.iter().filter(|(n, _)| {
                matches!(
                    n.state(),
                    MembershipState::Left | MembershipState::Relocated(_)
                )
            }) {
                trace!("Archived node {:?} - {:?}", node.name(), new_decision);
                self.archive.insert(node.name(), new_decision.clone());
            }

            Ok(true)
        } else if incoming_generation <= self.decisions.len() {
            // TODO: The sender seems behind us, force it to be updated?
            Ok(false)
        } else {
            // We are behind, return with error to trigger AE update
            Err(Error::AEOutdated)
        }
    }

    pub(super) fn update_peers(&mut self, peers: Vec<Decision<NodeState>>) -> bool {
        let mut there_was_an_update = false;

        for peer in peers {
            if let Ok(updated) = self.update(peer) {
                there_was_an_update |= updated;
            }
        }

        there_was_an_update
    }

    /// Remove all archived members whose name does not match `prefix`.
    pub(super) fn retain(&mut self, prefix: &Prefix) {
        self.archive.retain(|name, _| prefix.matches(name));
    }

    // Remove any member which Left, or was Relocated, more
    // than ELDER_CHURN_EVENTS_TO_PRUNE_ARCHIVE section keys ago from `last_key`
    pub(super) fn prune_members_archive(
        &mut self,
        proof_chain: &SectionsDAG,
        last_key: &bls::PublicKey,
    ) -> Result<()> {
        let mut latest_section_keys = proof_chain.get_ancestors(last_key)?;
        latest_section_keys.truncate(ELDER_CHURN_EVENTS_TO_PRUNE_ARCHIVE - 1);
        latest_section_keys.push(*last_key);
        self.archive.retain(|_, decision| {
            latest_section_keys.iter().any(|section_key| {
                for (node_state, sig) in decision.proposals.iter() {
                    if bincode::serialize(node_state)
                        .map(|bytes| section_key.verify(sig, bytes))
                        .unwrap_or(false)
                    {
                        return true;
                    }
                }
                false
            })
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{SectionMemberHistory, SectionsDAG};
    use crate::{
        network_knowledge::{MembershipState, NodeState},
        test_utils::{
            assert_lists, create_relocation_trigger, gen_addr, section_decision, TestKeys,
        },
        types::NodeId,
    };
    use eyre::Result;
    use rand::thread_rng;
    use sn_consensus::Decision;
    use xor_name::XorName;

    #[test]
    fn retain_archived_members_of_the_latest_sections_while_pruning() -> Result<()> {
        let _rng = thread_rng();
        let mut section_members = SectionMemberHistory::default();

        // adding node set 1
        let sk_set_1 = bls::SecretKeySet::random(0, &mut thread_rng());
        let nodes_1 = gen_random_signed_node_states(1, 1, MembershipState::Left, &sk_set_1)?;
        nodes_1.iter().for_each(|node| {
            let _ = section_members.update(node.clone());
        });
        let mut proof_chain = SectionsDAG::new(sk_set_1.secret_key().public_key());
        // 1 should be retained
        section_members.prune_members_archive(&proof_chain, &sk_set_1.secret_key().public_key())?;
        assert_lists(section_members.archive.values(), &nodes_1);

        // adding node set 2 as MembershipState::Relocated
        let sk_set_2 = bls::SecretKeySet::random(0, &mut thread_rng());
        let (trigger, _) = create_relocation_trigger(&sk_set_2, 2, 4)?;

        let nodes_2 =
            gen_random_signed_node_states(2, 1, MembershipState::Relocated(trigger), &sk_set_2)?;
        nodes_2.iter().for_each(|node| {
            let _ = section_members.update(node.clone());
        });
        let sig = TestKeys::sign(&sk_set_1.secret_key(), &sk_set_2.secret_key().public_key())?;
        proof_chain.verify_and_insert(
            &sk_set_1.secret_key().public_key(),
            sk_set_2.secret_key().public_key(),
            sig,
        )?;
        // 1 -> 2 should be retained
        section_members.prune_members_archive(&proof_chain, &sk_set_2.secret_key().public_key())?;
        assert_lists(
            section_members.archive.values(),
            nodes_1.iter().chain(&nodes_2),
        );

        // adding node set 3
        let sk_set_3 = bls::SecretKeySet::random(0, &mut thread_rng());
        let nodes_3 = gen_random_signed_node_states(3, 1, MembershipState::Left, &sk_set_3)?;
        nodes_3.iter().for_each(|node| {
            let _ = section_members.update(node.clone());
        });
        let sig = TestKeys::sign(&sk_set_2.secret_key(), &sk_set_3.secret_key().public_key())?;
        proof_chain.verify_and_insert(
            &sk_set_2.secret_key().public_key(),
            sk_set_3.secret_key().public_key(),
            sig,
        )?;
        // 1 -> 2 -> 3 should be retained
        section_members.prune_members_archive(&proof_chain, &sk_set_3.secret_key().public_key())?;
        assert_lists(
            section_members.archive.values(),
            nodes_1.iter().chain(&nodes_2).chain(&nodes_3),
        );

        // adding node set 4
        let sk_set_4 = bls::SecretKeySet::random(0, &mut thread_rng());
        let nodes_4 = gen_random_signed_node_states(4, 1, MembershipState::Left, &sk_set_4)?;
        nodes_4.iter().for_each(|node| {
            let _ = section_members.update(node.clone());
        });
        let sig = TestKeys::sign(&sk_set_3.secret_key(), &sk_set_4.secret_key().public_key())?;
        proof_chain.verify_and_insert(
            &sk_set_3.secret_key().public_key(),
            sk_set_4.secret_key().public_key(),
            sig,
        )?;
        //  2 -> 3 -> 4 should be retained
        section_members.prune_members_archive(&proof_chain, &sk_set_4.secret_key().public_key())?;
        assert_lists(
            section_members.archive.values(),
            nodes_2.iter().chain(&nodes_3).chain(&nodes_4),
        );

        // adding node set 5 as a branch to 3
        // 1 -> 2 -> 3 -> 4
        //              |
        //              -> 5
        let sk_set_5 = bls::SecretKeySet::random(0, &mut thread_rng());
        let nodes_5 = gen_random_signed_node_states(5, 1, MembershipState::Left, &sk_set_5)?;
        nodes_5.iter().for_each(|node| {
            let _ = section_members.update(node.clone());
        });
        let sig = TestKeys::sign(&sk_set_3.secret_key(), &sk_set_5.secret_key().public_key())?;
        proof_chain.verify_and_insert(
            &sk_set_3.secret_key().public_key(),
            sk_set_5.secret_key().public_key(),
            sig,
        )?;
        // 2 -> 3 -> 5 should be retained
        section_members.prune_members_archive(&proof_chain, &sk_set_5.secret_key().public_key())?;
        assert_lists(
            section_members.archive.values(),
            nodes_2.iter().chain(&nodes_3).chain(&nodes_5),
        );

        Ok(())
    }

    #[test]
    fn archived_members_should_not_be_moved_to_members_list() -> Result<()> {
        let mut _rng = thread_rng();
        let mut section_members = SectionMemberHistory::default();
        let sk_set = bls::SecretKeySet::random(0, &mut thread_rng());
        let node_left =
            gen_random_signed_node_states(1, 1, MembershipState::Left, &sk_set)?[0].clone();
        let (trigger, _) = create_relocation_trigger(&sk_set, 2, 4)?;
        let node_relocated =
            gen_random_signed_node_states(2, 1, MembershipState::Relocated(trigger), &sk_set)?[0]
                .clone();

        assert!(section_members.update(node_left.clone())?);
        assert!(section_members.update(node_relocated.clone())?);

        let (node_left_state, _) = node_left
            .proposals
            .first_key_value()
            .unwrap_or_else(|| panic!("Proposal of Decision is empty"));
        let (node_relocated_state, _) = node_relocated
            .proposals
            .first_key_value()
            .unwrap_or_else(|| panic!("Proposal of Decision is empty"));

        let _node_left_joins = section_decision(
            &sk_set,
            3,
            NodeState::joined(*node_left_state.node_id(), None),
        );

        let _node_relocated_joins = section_decision(
            &sk_set,
            4,
            NodeState::joined(*node_relocated_state.node_id(), None),
        )?;

        // TODO: `SectionPeers::update` function no longer carry out transition checks.
        //       Hence the following two assertion will be failed.
        // assert!(!section_members.update(node_left_joins)?);
        // assert!(!section_members.update(node_relocated_joins)?);

        assert_lists(
            section_members.archive.values(),
            &[node_left, node_relocated],
        );
        assert!(section_members.members().is_empty());

        Ok(())
    }

    #[test]
    fn members_should_be_archived_if_they_leave_or_relocate() -> Result<()> {
        let mut _rng = thread_rng();
        let mut section_members = SectionMemberHistory::default();
        let sk_set = bls::SecretKeySet::random(0, &mut thread_rng());

        let node_1 =
            gen_random_signed_node_states(1, 1, MembershipState::Joined, &sk_set)?[0].clone();
        let (trigger, _) = create_relocation_trigger(&sk_set, 2, 4)?;
        let node_2 =
            gen_random_signed_node_states(2, 1, MembershipState::Relocated(trigger), &sk_set)?[0]
                .clone();
        assert!(section_members.update(node_1.clone())?);
        assert!(section_members.update(node_2.clone())?);

        let (node_state_1, _) = node_1
            .proposals
            .first_key_value()
            .unwrap_or_else(|| panic!("Proposal of Decision is empty"));
        let (node_state_2, _) = node_2
            .proposals
            .first_key_value()
            .unwrap_or_else(|| panic!("Proposal of Decision is empty"));

        let node_1 = NodeState::left(*node_state_1.node_id(), Some(node_state_1.name()));
        let node_1 = section_decision(&sk_set, 3, node_1)?;
        let node_2 = NodeState::left(*node_state_2.node_id(), Some(node_state_2.name()));
        let node_2 = section_decision(&sk_set, 4, node_2)?;
        assert!(section_members.update(node_1.clone())?);
        assert!(section_members.update(node_2.clone())?);

        assert!(section_members.members().is_empty());
        assert_lists(section_members.archive.values(), &[node_1, node_2]);

        Ok(())
    }

    // Test helpers
    // generate node states signed by a section's sk
    fn gen_random_signed_node_states(
        start_gen: u64,
        num_nodes: usize,
        membership_state: MembershipState,
        secret_key_set: &bls::SecretKeySet,
    ) -> Result<Vec<Decision<NodeState>>> {
        let mut rng = thread_rng();
        let mut decisions = Vec::new();
        for gen in start_gen..(start_gen + num_nodes as u64) {
            let addr = gen_addr();
            let name = XorName::random(&mut rng);
            let node_id = NodeId::new(name, addr);
            let node_state = match membership_state {
                MembershipState::Joined => NodeState::joined(node_id, None),
                MembershipState::Left => NodeState::left(node_id, None),
                MembershipState::Relocated(ref trigger) => {
                    NodeState::relocated(node_id, None, trigger.clone())
                }
            };
            decisions.push(section_decision(secret_key_set, gen, node_state)?);
        }
        Ok(decisions)
    }
}
