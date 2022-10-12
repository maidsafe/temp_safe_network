// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{
    messaging::system::{MembershipState, SectionSigned},
    network_knowledge::{errors::Result, NodeState, SectionsDAG},
};
use dashmap::{mapref::entry::Entry, DashMap};
use std::{collections::BTreeSet, sync::Arc};
use xor_name::{Prefix, XorName};

// Number of Elder churn events before a Left/Relocated member
// can be removed from the section members archive.
#[cfg(not(test))]
const ELDER_CHURN_EVENTS_TO_PRUNE_ARCHIVE: usize = 5;
#[cfg(test)]
const ELDER_CHURN_EVENTS_TO_PRUNE_ARCHIVE: usize = 3;

/// Container for storing information about (current and archived) members of our section.
#[derive(Clone, Default, Debug)]
pub(super) struct SectionPeers {
    members: Arc<DashMap<XorName, SectionSigned<NodeState>>>,
    archive: Arc<DashMap<XorName, SectionSigned<NodeState>>>,
}

impl SectionPeers {
    /// Returns set of current members, i.e. those with state == `Joined`.
    pub(super) fn members(&self) -> BTreeSet<SectionSigned<NodeState>> {
        self.members
            .iter()
            .map(|entry| {
                let (_, state) = entry.pair();
                state.clone()
            })
            .collect()
    }

    /// Returns the number of current members.
    pub(super) fn num_of_members(&self) -> usize {
        self.members.len()
    }

    /// Get the `NodeState` for the member with the given name.
    pub(super) fn get(&self, name: &XorName) -> Option<NodeState> {
        self.members.get(name).map(|state| state.value.clone())
    }

    /// Returns whether the given peer is currently a member of our section.
    pub(super) fn is_member(&self, name: &XorName) -> bool {
        self.members.get(name).is_some()
    }

    /// Get section signed `NodeState` for the member with the given name.
    pub(super) fn is_either_member_or_archived(
        &self,
        name: &XorName,
    ) -> Option<SectionSigned<NodeState>> {
        if let Some(member) = self.members.get(name).map(|state| state.value().clone()) {
            Some(member)
        } else {
            self.archive.get(name).map(|state| state.value().clone())
        }
    }

    /// Update a member of our section.
    /// Returns whether anything actually changed.
    /// To maintain commutativity, the only allowed transitions are:
    /// - Joined -> Joined if the new age is greater than the old age
    /// - Joined -> Left
    /// - Joined -> Relocated
    /// - Relocated <--> Left (should not happen, but needed for consistency)
    pub(super) fn update(&self, new_state: SectionSigned<NodeState>) -> bool {
        let node_name = new_state.name();
        // do ops on the dashmap _after_ matching, so we can drop any refs to prevent deadlocking
        let mut should_insert = false;
        let mut should_remove = false;

        let updating_something = match (self.members.entry(node_name), new_state.state()) {
            (Entry::Vacant(_entry), MembershipState::Joined) => {
                // unless it was already archived, insert it as current member
                if self.archive.get(&node_name).is_none() {
                    should_insert = true;
                    true
                } else {
                    false
                }
            }
            (Entry::Vacant(_), MembershipState::Left | MembershipState::Relocated(_)) => {
                // insert it in our archive regardless it was there with another state
                let _prev = self.archive.insert(node_name, new_state.clone());
                true
            }
            (Entry::Occupied(_), MembershipState::Joined) => false,
            (Entry::Occupied(_entry), MembershipState::Left | MembershipState::Relocated(_)) => {
                //  remove it from our current members, and insert it into our archive
                should_remove = true;
                let _prev = self.archive.insert(node_name, new_state.clone());
                true
            }
        };

        // now we have dropped the entry ref
        if should_insert {
            let _prev = self.members.insert(node_name, new_state);
        }
        if should_remove {
            let _prev = self.members.remove(&node_name);
        }

        updating_something
    }

    /// Remove all members whose name does not match `prefix`.
    pub(super) fn retain(&self, prefix: &Prefix) {
        self.members.retain(|name, _| prefix.matches(name))
    }

    // Remove any member which Left, or was Relocated, more
    // than ELDER_CHURN_EVENTS_TO_PRUNE_ARCHIVE section keys ago from `last_key`
    pub(super) fn prune_members_archive(
        &self,
        proof_chain: &SectionsDAG,
        last_key: &bls::PublicKey,
    ) -> Result<()> {
        let mut latest_section_keys = proof_chain.get_ancestors(last_key)?;
        latest_section_keys.truncate(ELDER_CHURN_EVENTS_TO_PRUNE_ARCHIVE - 1);
        latest_section_keys.push(*last_key);
        self.archive
            .retain(|_, node_state| latest_section_keys.contains(&node_state.sig.public_key));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{SectionPeers, SectionsDAG};
    use crate::{
        messaging::system::{MembershipState, RelocateDetails, SectionSigned},
        network_knowledge::{
            test_utils::{assert_lists, gen_addr, section_signed},
            NodeState,
        },
        types::{Peer, SecretKeySet},
    };
    use eyre::Result;
    use rand::thread_rng;
    use xor_name::XorName;

    #[test]
    fn retain_archived_members_of_the_latest_sections_while_pruning() -> Result<()> {
        let mut rng = thread_rng();
        let section_peers = SectionPeers::default();

        // adding node set 1
        let sk_1 = SecretKeySet::random(None).secret_key().clone();
        let nodes_1 = gen_random_signed_node_states(1, MembershipState::Left, &sk_1)?;
        nodes_1.iter().for_each(|node| {
            section_peers.update(node.clone());
        });
        let mut proof_chain = SectionsDAG::new(sk_1.public_key());
        // 1 should be retained
        section_peers.prune_members_archive(&proof_chain, &sk_1.public_key())?;
        assert_lists(
            section_peers
                .archive
                .iter()
                .map(|item| item.value().clone()),
            nodes_1.clone(),
        )?;

        // adding node set 2 as MembershipState::Relocated
        let sk_2 = SecretKeySet::random(None).secret_key().clone();
        let relocate = RelocateDetails {
            previous_name: XorName::random(&mut rng),
            dst: XorName::random(&mut rng),
            dst_section_key: bls::SecretKey::random().public_key(),
            age: 10,
        };
        let nodes_2 = gen_random_signed_node_states(
            1,
            MembershipState::Relocated(Box::new(relocate)),
            &sk_2,
        )?;
        nodes_2.iter().for_each(|node| {
            section_peers.update(node.clone());
        });
        let sig = bincode::serialize(&sk_2.public_key()).map(|bytes| sk_1.sign(&bytes))?;
        proof_chain.insert(&sk_1.public_key(), sk_2.public_key(), sig)?;
        // 1 -> 2 should be retained
        section_peers.prune_members_archive(&proof_chain, &sk_2.public_key())?;
        assert_lists(
            section_peers
                .archive
                .iter()
                .map(|item| item.value().clone()),
            nodes_1.iter().cloned().chain(nodes_2.clone()),
        )?;

        // adding node set 3
        let sk_3 = SecretKeySet::random(None).secret_key().clone();
        let nodes_3 = gen_random_signed_node_states(1, MembershipState::Left, &sk_3)?;
        nodes_3.iter().for_each(|node| {
            section_peers.update(node.clone());
        });
        let sig = bincode::serialize(&sk_3.public_key()).map(|bytes| sk_2.sign(&bytes))?;
        proof_chain.insert(&sk_2.public_key(), sk_3.public_key(), sig)?;
        // 1 -> 2 -> 3 should be retained
        section_peers.prune_members_archive(&proof_chain, &sk_3.public_key())?;
        assert_lists(
            section_peers
                .archive
                .iter()
                .map(|item| item.value().clone()),
            nodes_1
                .iter()
                .cloned()
                .chain(nodes_2.clone())
                .chain(nodes_3.clone()),
        )?;

        // adding node set 4
        let sk_4 = SecretKeySet::random(None).secret_key().clone();
        let nodes_4 = gen_random_signed_node_states(1, MembershipState::Left, &sk_4)?;
        nodes_4.iter().for_each(|node| {
            section_peers.update(node.clone());
        });
        let sig = bincode::serialize(&sk_4.public_key()).map(|bytes| sk_3.sign(&bytes))?;
        proof_chain.insert(&sk_3.public_key(), sk_4.public_key(), sig)?;
        //  2 -> 3 -> 4 should be retained
        section_peers.prune_members_archive(&proof_chain, &sk_4.public_key())?;
        assert_lists(
            section_peers
                .archive
                .iter()
                .map(|item| item.value().clone()),
            nodes_2
                .iter()
                .cloned()
                .chain(nodes_3.clone())
                .chain(nodes_4),
        )?;

        // adding node set 5 as a branch to 3
        // 1 -> 2 -> 3 -> 4
        //              |
        //              -> 5
        let sk_5 = SecretKeySet::random(None).secret_key().clone();
        let nodes_5 = gen_random_signed_node_states(1, MembershipState::Left, &sk_5)?;
        nodes_5.iter().for_each(|node| {
            section_peers.update(node.clone());
        });
        let sig = bincode::serialize(&sk_5.public_key()).map(|bytes| sk_3.sign(&bytes))?;
        proof_chain.insert(&sk_3.public_key(), sk_5.public_key(), sig)?;
        // 2 -> 3 -> 5 should be retained
        section_peers.prune_members_archive(&proof_chain, &sk_5.public_key())?;
        assert_lists(
            section_peers
                .archive
                .iter()
                .map(|item| item.value().clone()),
            nodes_2.iter().cloned().chain(nodes_3).chain(nodes_5),
        )?;

        Ok(())
    }

    #[test]
    fn archived_members_should_not_be_moved_to_members_list() -> Result<()> {
        let section_peers = SectionPeers::default();
        let sk = SecretKeySet::random(None).secret_key().clone();
        let node_left = gen_random_signed_node_states(1, MembershipState::Left, &sk)?[0].clone();
        assert!(section_peers.update(node_left.clone()));

        let node_joined = section_signed(&sk, NodeState::joined(*node_left.peer(), None))?;
        assert!(!section_peers.update(node_joined));

        assert_lists(
            section_peers
                .archive
                .iter()
                .map(|item| item.value().clone()),
            [node_left],
        )?;
        assert!(section_peers.members().is_empty());
        Ok(())
    }

    #[test]
    fn members_should_be_archived_if_they_leave_or_relocate() -> Result<()> {
        let mut rng = thread_rng();
        let section_peers = SectionPeers::default();
        let sk = SecretKeySet::random(None).secret_key().clone();

        let node_1 = gen_random_signed_node_states(1, MembershipState::Joined, &sk)?[0].clone();
        let relocate = RelocateDetails {
            previous_name: XorName::random(&mut rng),
            dst: XorName::random(&mut rng),
            dst_section_key: bls::SecretKey::random().public_key(),
            age: 10,
        };
        let node_2 =
            gen_random_signed_node_states(1, MembershipState::Relocated(Box::new(relocate)), &sk)?
                [0]
            .clone();
        assert!(section_peers.update(node_1.clone()));
        assert!(section_peers.update(node_2.clone()));

        let node_1 = NodeState::left(*node_1.peer(), Some(node_1.name()));
        let node_1 = section_signed(&sk, node_1)?;
        let node_2 = NodeState::left(*node_2.peer(), Some(node_2.name()));
        let node_2 = section_signed(&sk, node_2)?;
        assert!(section_peers.update(node_1.clone()));
        assert!(section_peers.update(node_2.clone()));

        assert!(section_peers.members().is_empty());
        assert_lists(
            section_peers
                .archive
                .iter()
                .map(|item| item.value().clone()),
            [node_1, node_2],
        )?;
        Ok(())
    }

    // Test helpers
    // generate node states signed by a section's sk
    fn gen_random_signed_node_states(
        num_nodes: usize,
        membership_state: MembershipState,
        secret_key: &bls::SecretKey,
    ) -> Result<Vec<SectionSigned<NodeState>>> {
        let mut rng = thread_rng();
        let mut signed_node_states = Vec::new();
        for _ in 0..num_nodes {
            let addr = gen_addr();
            let name = XorName::random(&mut rng);
            let peer = Peer::new(name, addr);
            let node_state = match membership_state {
                MembershipState::Joined => NodeState::joined(peer, None),
                MembershipState::Left => NodeState::left(peer, None),
                MembershipState::Relocated(ref details) => {
                    NodeState::relocated(peer, None, (**details).clone())
                }
            };
            let sig = section_signed(secret_key, node_state)?;
            signed_node_states.push(sig);
        }
        Ok(signed_node_states)
    }
}
