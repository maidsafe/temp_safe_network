// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::system::SectionAuth;
use crate::node::network_knowledge::{NodeState, SectionAuthorityProvider};
use crate::types::Peer;

use dashmap::DashMap;
use itertools::Itertools;
use secured_linked_list::SecuredLinkedList;
use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
    net::SocketAddr,
    sync::Arc,
};
use tokio::sync::RwLock;
use xor_name::{Prefix, XorName};

// Number of Elder churn events before a Left/Relocated member
// can be removed from the section members archive.
const ELDER_CHURN_EVENTS_TO_PRUNE_ARCHIVE: usize = 5;

/// Container for storing information about (current and archived) members of our section.
#[derive(Clone, Default, Debug)]
pub(super) struct SectionPeers {
    members: Arc<RwLock<BTreeMap<XorName, SectionAuth<NodeState>>>>,
    archive: Arc<DashMap<XorName, SectionAuth<NodeState>>>,
}

impl SectionPeers {
    /// Returns set of current members, i.e. those with state == `Joined`.
    pub(super) async fn members(&self) -> BTreeSet<SectionAuth<NodeState>> {
        self.members
            .read()
            .await
            .iter()
            .map(|(_, state)| state.clone())
            .collect()
    }

    /// Returns the number of current members.
    pub(super) async fn num_of_members(&self) -> usize {
        self.members.read().await.len()
    }

    /// Get the `NodeState` for the member with the given name.
    pub(super) async fn get(&self, name: &XorName) -> Option<NodeState> {
        self.members
            .read()
            .await
            .get(name)
            .map(|state| state.value.clone())
    }

    /// Returns whether the given peer is currently a member of our section.
    pub(super) async fn is_member(&self, name: &XorName) -> bool {
        self.members.read().await.get(name).is_some()
    }

    /// Returns whether the given peer is already relocated to our section.
    pub(super) async fn is_relocated_to_our_section(&self, name: &XorName) -> bool {
        let is_previous_name_of_member = self
            .members
            .read()
            .await
            .iter()
            .any(|(_, state)| state.previous_name() == Some(*name));

        is_previous_name_of_member
            || self.archive.iter().any(|entry| {
                let (_, state) = entry.pair();
                state.previous_name() == Some(*name)
            })
    }

    /// Get section signed `NodeState` for the member with the given name.
    pub(super) async fn is_either_member_or_archived(
        &self,
        name: &XorName,
    ) -> Option<SectionAuth<NodeState>> {
        if let Some(member) = self.members.read().await.get(name).cloned() {
            Some(member)
        } else {
            self.archive.get(name).map(|state| state.value().clone())
        }
    }

    /// Returns the nodes that should be candidates to become the next elders, sorted by names.
    pub(super) async fn elder_candidates(
        &self,
        elder_size: usize,
        current_elders: &SectionAuthorityProvider,
        excluded_names: &BTreeSet<XorName>,
        prefix: Option<&Prefix>,
    ) -> Vec<Peer> {
        self.members
            .read()
            .await
            .iter()
            .filter(|(name, _)| {
                prefix.map_or_else(|| true, |p| p.matches(name)) && !excluded_names.contains(name)
            })
            .map(|(_, node_state)| node_state.clone())
            .sorted_by(|lhs, rhs| cmp_elder_candidates(lhs, rhs, current_elders))
            .take(elder_size)
            .map(|node_state| node_state.peer().clone())
            .collect()
    }

    /// Set current list of members of our section.
    pub(super) async fn set_members(&self, members: BTreeSet<SectionAuth<NodeState>>) {
        let mut write_guard = self.members.write().await;
        write_guard.clear();
        for node_state in members.into_iter() {
            let _prev = write_guard.insert(node_state.name(), node_state);
        }
    }

    /// Merge connections into of our current members
    pub(super) async fn merge_connections(&self, sources: &BTreeMap<SocketAddr, &Peer>) {
        for (_, node_state) in self.members.read().await.iter() {
            if let Some(source) = sources.get(&node_state.addr()) {
                node_state.peer().merge_connection(source).await;
            }
        }
    }

    // Remove any member which Left, or was Relocated, more
    // than ELDER_CHURN_EVENTS_TO_PRUNE_ARCHIVE section keys ago.
    pub(super) async fn prune_members_archive(&self, section_chain: &SecuredLinkedList) {
        let last_section_keys = section_chain.truncate(ELDER_CHURN_EVENTS_TO_PRUNE_ARCHIVE);
        self.archive
            .retain(|_, node_state| last_section_keys.has_key(&node_state.sig.public_key))
    }
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
    rhs.age()
        .cmp(&lhs.age())
        .then_with(|| {
            let lhs_is_elder = current_elders.contains_elder(&lhs.name());
            let rhs_is_elder = current_elders.contains_elder(&rhs.name());

            match (lhs_is_elder, rhs_is_elder) {
                (true, false) => Ordering::Less,
                (false, true) => Ordering::Greater,
                _ => Ordering::Equal,
            }
        })
        .then_with(|| lhs.sig.signature.cmp(&rhs.sig.signature))
}
