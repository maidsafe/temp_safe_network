// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::system::{MembershipState, NodeState as NodeStateMsg};
use crate::network_knowledge::NodeState;

use bls_dkg::PublicKeySet;
use dashmap::{mapref::entry::Entry, DashMap};
use secured_linked_list::SecuredLinkedList;
use sn_consensus::Decision;
use std::{collections::BTreeSet, sync::Arc};
use xor_name::{Prefix, XorName};

// Number of Elder churn events before a Left/Relocated member
// can be removed from the section members archive.
const ELDER_CHURN_EVENTS_TO_PRUNE_ARCHIVE: usize = 5;

type NodeProof = (PublicKeySet, Decision<NodeStateMsg>);

/// Container for storing information about (current and archived) members of our section.
#[derive(Clone, Default, Debug)]
pub(super) struct SectionPeers {
    members: Arc<DashMap<XorName, (NodeStateMsg, NodeProof)>>,
    archive: Arc<DashMap<XorName, (NodeStateMsg, NodeProof)>>,
}

impl SectionPeers {
    pub(super) fn decisions(&self) -> BTreeSet<(PublicKeySet, Decision<NodeStateMsg>)> {
        self.members
            .iter()
            .map(|entry| {
                let (_, (_, signed_decision)) = entry.pair();
                signed_decision.clone()
            })
            .collect()
    }

    /// Returns set of current members, i.e. those with state == `Joined`.
    pub(super) fn members(&self) -> BTreeSet<NodeState> {
        self.members
            .iter()
            .map(|entry| {
                let (_, (state, _)) = entry.pair();
                state.clone().into_state()
            })
            .collect()
    }

    /// Returns the number of current members.
    pub(super) fn num_of_members(&self) -> usize {
        self.members.len()
    }

    /// Get the `NodeState` for the member with the given name.
    pub(super) fn get(&self, name: &XorName) -> Option<NodeState> {
        self.members
            .get(name)
            .map(|state| state.0.clone().into_state())
    }

    /// Returns whether the given peer is currently a member of our section.
    pub(super) fn is_member(&self, name: &XorName) -> bool {
        self.members.get(name).is_some()
    }

    /// Get section signed `NodeState` for the member with the given name.
    pub(super) fn is_archived(&self, name: &XorName) -> Option<NodeState> {
        self.archive
            .get(name)
            .map(|state| state.value().0.clone().into_state())
    }

    /// Update members of our section given a decision.
    /// Returns whether anything actually changed.
    /// To maintain commutativity, the only allowed transitions are:
    /// - Joined -> Joined if the new age is greater than the old age
    /// - Joined -> Left
    /// - Joined -> Relocated
    /// - Relocated <--> Left (should not happen, but needed for consistency)
    pub(super) fn update(
        &self,
        section_key_set: &PublicKeySet,
        decision: Decision<NodeStateMsg>,
    ) -> bool {
        let mut updated_something = false;

        for new_state in decision.proposals() {
            let node_name = new_state.name;

            if self.archive.contains_key(&node_name) {
                trace!("Skipping archived node {node_name}");
                continue;
            }

            // do ops on the dashmap _after_ matching, so we can drop any refs to prevent deadlocking
            let mut should_insert = false;
            let mut should_remove = false;

            updated_something |= match (self.members.entry(node_name), new_state.state.clone()) {
                (Entry::Vacant(_entry), MembershipState::Joined) => {
                    should_insert = true;
                    true
                }
                (Entry::Vacant(_), MembershipState::Left | MembershipState::Relocated(_)) => {
                    // Node has left the section, insert it in our archive
                    let _prev = self.archive.insert(
                        node_name,
                        (
                            new_state.clone(),
                            (section_key_set.clone(), decision.clone()),
                        ),
                    );
                    true
                }
                (Entry::Occupied(entry), MembershipState::Joined)
                    if new_state.age() > entry.get().0.age() =>
                {
                    should_insert = true;
                    true
                }
                (Entry::Occupied(_), MembershipState::Joined) => false,
                (
                    Entry::Occupied(_entry),
                    MembershipState::Left | MembershipState::Relocated(_),
                ) => {
                    //  remove it from our current members, and insert it into our archive
                    should_remove = true;
                    let _prev = self.archive.insert(
                        node_name,
                        (
                            new_state.clone(),
                            (section_key_set.clone(), decision.clone()),
                        ),
                    );
                    true
                }
            };

            // now we have dropped the entry ref
            if should_insert {
                let _prev = self.members.insert(
                    node_name,
                    (new_state, (section_key_set.clone(), decision.clone())),
                );
            }

            if should_remove {
                let _prev = self.members.remove(&node_name);
            }
        }

        updated_something
    }

    /// Remove all members whose name does not match `prefix`.
    pub(super) fn retain(&self, prefix: &Prefix) {
        self.members.retain(|name, _| prefix.matches(name))
    }

    // Remove any member which Left, or was Relocated, more
    // than ELDER_CHURN_EVENTS_TO_PRUNE_ARCHIVE section keys ago.
    pub(super) async fn prune_members_archive(&self, section_chain: &SecuredLinkedList) {
        let last_section_keys = section_chain.truncate(ELDER_CHURN_EVENTS_TO_PRUNE_ARCHIVE);
        self.archive.retain(|_, (_, (section_key_set, _))| {
            last_section_keys.has_key(&section_key_set.public_key())
        })
    }
}
