// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::system::{MembershipState, SectionAuth};
use crate::node::network_knowledge::{NodeState, SectionAuthorityProvider};
use crate::peer::Peer;
use dashmap::{mapref::entry::Entry, DashMap};
use itertools::Itertools;
use std::{cmp::Ordering, collections::BTreeSet, ops::Deref, sync::Arc};
use xor_name::{Prefix, XorName};

/// Container for storing information about members of our section.
#[derive(Clone, Default, Debug)]
pub(super) struct SectionPeers {
    members: Arc<DashMap<XorName, SectionAuth<NodeState>>>,
}

impl<'a> IntoIterator for &'a SectionPeers {
    type Item = dashmap::mapref::multiple::RefMulti<'a, XorName, SectionAuth<NodeState>>;

    type IntoIter = dashmap::iter::Iter<'a, XorName, SectionAuth<NodeState>>;

    fn into_iter(self) -> Self::IntoIter {
        self.members.iter()
    }
}

impl SectionPeers {
    pub(super) fn iter(
        &self,
    ) -> impl Iterator<Item = impl Deref<Target = SectionAuth<NodeState>> + '_> + '_ {
        IntoIterator::into_iter(self)
    }

    /// Returns members that have state == `Joined`.
    pub(super) fn joined(&self) -> BTreeSet<SectionAuth<NodeState>> {
        self.members
            .iter()
            .filter(|entry| {
                let (_, state) = entry.pair();
                state.state() == MembershipState::Joined
            })
            .map(|entry| {
                let (_, state) = entry.pair();
                state.clone()
            })
            .collect()
    }

    /// Returns number of members that have state == `Joined`.
    pub(super) fn num_of_joined(&self) -> usize {
        self.members
            .iter()
            .filter(|entry| {
                let (_, state) = entry.pair();
                state.state() == MembershipState::Joined
            })
            .count()
    }

    /// Get info for the member with the given name.
    pub(super) fn get(&self, name: &XorName) -> Option<NodeState> {
        self.members
            .get(name)
            .filter(|state| state.state() == MembershipState::Joined)
            .map(|state| state.value.clone())
    }

    /// Returns whether the given peer is a joined member of our section.
    pub(super) fn is_joined(&self, name: &XorName) -> bool {
        self.members
            .get(name)
            .map(|info| info.state() == MembershipState::Joined)
            .unwrap_or(false)
    }

    /// Returns whether the given peer is already relocated to our section.
    pub(super) fn is_relocated_to_our_section(&self, name: &XorName) -> bool {
        self.members
            .get(name)
            .map(|state| state.previous_name() == Some(*name))
            .unwrap_or(false)
    }

    /// Get section_signed info for the member with the given name.
    pub(super) fn get_section_signed(&self, name: &XorName) -> Option<SectionAuth<NodeState>> {
        self.members.get(name).map(|oneref| oneref.value().clone())
    }

    /// Returns the candidates for elders out of all the nodes in this section.
    pub(super) fn elder_candidates(
        &self,
        elder_size: usize,
        current_elders: &SectionAuthorityProvider,
        excluded_names: &BTreeSet<XorName>,
    ) -> Vec<Peer> {
        let mut candidates = vec![];
        let members = &*self.members;

        for entry in members.into_iter() {
            let (name, info) = entry.pair();

            if is_active(info, current_elders) && !excluded_names.contains(name) {
                candidates.push(info.clone())
            }
        }

        elder_candidates(elder_size, current_elders, candidates)
    }

    /// Returns the candidates for elders out of all nodes matching the prefix.
    pub(super) fn elder_candidates_matching_prefix(
        &self,
        prefix: &Prefix,
        elder_size: usize,
        current_elders: &SectionAuthorityProvider,
        excluded_names: &BTreeSet<XorName>,
    ) -> Vec<Peer> {
        let mut candidates = vec![];
        let members = &*self.members;

        for entry in members.into_iter() {
            let (name, info) = entry.pair();

            if info.state() == MembershipState::Joined
                && prefix.matches(name)
                && !excluded_names.contains(name)
            {
                candidates.push(info.clone())
            }
        }

        elder_candidates(elder_size, current_elders, candidates)
    }

    /// Update a member of our section.
    /// Returns whether anything actually changed.
    pub(super) fn update(&self, new_info: SectionAuth<NodeState>) -> bool {
        match self.members.entry(new_info.name()) {
            Entry::Vacant(entry) => {
                let _prev = entry.insert(new_info);
                true
            }
            Entry::Occupied(mut entry) => {
                // To maintain commutativity, the only allowed transitions are:
                // - Joined -> Joined if the new age is greater than the old age
                // - Joined -> Left
                // - Joined -> Relocated
                // - Relocated -> Left (should not happen, but needed for consistency)
                match (entry.get().state(), new_info.state()) {
                    (MembershipState::Joined, MembershipState::Joined)
                        if new_info.age() > entry.get().age() => {}
                    (MembershipState::Joined, MembershipState::Left)
                    | (MembershipState::Joined, MembershipState::Relocated(_))
                    | (MembershipState::Relocated(_), MembershipState::Left) => {}
                    _ => return false,
                };

                let _prev = entry.insert(new_info);
                true
            }
        }
    }

    /// Remove all members whose name does not match `prefix`.
    pub(super) fn retain(&self, prefix: &Prefix) {
        self.members.retain(|name, _value| prefix.matches(name))
    }
}

// Returns the nodes that should become the next elders out of the given members, sorted by names.
// It is assumed that `members` contains only "active" peers (see the `is_active` function below
// for explanation)
fn elder_candidates(
    elder_size: usize,
    current_elders: &SectionAuthorityProvider,
    members: Vec<SectionAuth<NodeState>>,
) -> Vec<Peer> {
    members
        .into_iter()
        .sorted_by(|lhs, rhs| cmp_elder_candidates(lhs, rhs, current_elders))
        .map(|auth| auth.peer().clone())
        .take(elder_size)
        .collect()
}

// Compare candidates for the next elders. The one comparing `Less` wins.
fn cmp_elder_candidates(
    lhs: &SectionAuth<NodeState>,
    rhs: &SectionAuth<NodeState>,
    current_elders: &SectionAuthorityProvider,
) -> Ordering {
    // Older nodes are preferred. In case of a tie, prefer current elders. If still a tie, break
    // it comparing by the signed signatures because it's impossible for a node to predict its
    // signature and therefore game its chances of promotion.
    cmp_elder_candidates_by_membership_state(&lhs.state(), &rhs.state())
        .then_with(|| rhs.age().cmp(&lhs.age()))
        .then_with(|| {
            let lhs_is_elder = is_elder(lhs, current_elders);
            let rhs_is_elder = is_elder(rhs, current_elders);

            match (lhs_is_elder, rhs_is_elder) {
                (true, false) => Ordering::Less,
                (false, true) => Ordering::Greater,
                _ => Ordering::Equal,
            }
        })
        .then_with(|| lhs.sig.signature.cmp(&rhs.sig.signature))
}

// Compare candidates for the next elders according to their peer state. The one comparing `Less`
// wins. `Joined` is preferred over `Relocated` which is preferred over `Left`.
// NOTE: we only consider `Relocated` peers as elder candidates if we don't have enough `Joined`
// members to reach `elder_count()`.
fn cmp_elder_candidates_by_membership_state(
    lhs: &MembershipState,
    rhs: &MembershipState,
) -> Ordering {
    use MembershipState::*;

    match (lhs, rhs) {
        (Joined, Joined) | (Relocated(_), Relocated(_)) => Ordering::Equal,
        (Joined, Relocated(_)) | (_, Left) => Ordering::Less,
        (Relocated(_), Joined) | (Left, _) => Ordering::Greater,
    }
}

// A peer is considered active if either it is joined or it is a current elder who is being
// relocated. This is because such elder still fulfils its duties and only when demoted can it
// leave.
fn is_active(info: &NodeState, current_elders: &SectionAuthorityProvider) -> bool {
    match info.state() {
        MembershipState::Joined => true,
        MembershipState::Relocated(_) if is_elder(info, current_elders) => true,
        _ => false,
    }
}

fn is_elder(info: &NodeState, current_elders: &SectionAuthorityProvider) -> bool {
    current_elders.contains_elder(&info.name())
}
