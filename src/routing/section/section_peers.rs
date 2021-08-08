// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::node_state::NodeStateUtils;
use crate::routing::{peer::PeerUtils, SectionAuthorityProviderUtils};
use crate::{
    messaging::{
        node::{MembershipState, NodeState, Peer, SectionAuth, SectionPeersDto},
        SectionAuthorityProvider,
    },
    types::CFMap,
};
use async_trait::async_trait;
use itertools::Itertools;
use std::cmp::Ordering;
use std::sync::Arc;
use xor_name::{Prefix, XorName};

/// Container for storing information about members of our section.
#[async_trait]
pub(crate) trait SectionPeersLogic {
    /// Returns an iterator over all current (joined) and past (left) members.
    async fn all(&self) -> Box<dyn Iterator<Item = NodeState> + '_>;

    /// Returns an iterator over the members that have state == `Joined`.
    async fn joined(&self) -> Box<dyn Iterator<Item = NodeState> + '_>;

    /// Returns joined nodes from our section with age greater than `MIN_AGE`
    async fn mature(&self) -> Box<dyn Iterator<Item = Peer> + '_>;

    /// Get info for the member with the given name.
    async fn get(&self, name: &XorName) -> Option<NodeState>;

    /// Get section_signed info for the member with the given name.
    async fn get_section_signed(&self, name: &XorName) -> Option<Arc<SectionAuth<NodeState>>>;

    /// Returns the candidates for elders out of all the nodes in this section.
    async fn elder_candidates(
        &self,
        elder_size: usize,
        current_elders: &SectionAuthorityProvider,
    ) -> Vec<Peer>;

    /// Returns the candidates for elders out of all nodes matching the prefix.
    async fn elder_candidates_matching_prefix(
        &self,
        prefix: &Prefix,
        elder_size: usize,
        current_elders: &SectionAuthorityProvider,
    ) -> Vec<Peer>;

    /// Returns whether the given peer is a joined member of our section.
    async fn is_joined(&self, name: &XorName) -> bool;

    /// Returns whether the given peer is already relocated to our section.
    async fn is_relocated(&self, name: &XorName) -> bool;

    /// Update a member of our section.
    /// Returns whether anything actually changed.
    async fn update(&self, new_info: SectionAuth<NodeState>) -> bool;

    /// Remove all members whose name does not match `prefix`.
    fn prune_not_matching(&self, prefix: &Prefix);

    ///
    async fn clone(&self) -> SectionPeersDto;
}

/// Container for storing information about members of our section.
#[derive(Debug)]
pub(crate) struct SectionPeers {
    /// members of the section
    members: CFMap<XorName, SectionAuth<NodeState>>,
}

impl From<SectionPeersDto> for SectionPeers {
    fn from(dto: SectionPeersDto) -> Self {
        SectionPeers {
            members: CFMap::from(dto.members),
        }
    }
}

impl SectionPeers {
    ///
    pub(crate) fn new() -> Self {
        SectionPeers {
            members: CFMap::new(),
        }
    }
}

#[async_trait]
impl SectionPeersLogic for SectionPeers {
    ///
    async fn clone(&self) -> SectionPeersDto {
        SectionPeersDto {
            members: self.members.clone().await,
        }
    }

    /// Returns an iterator over all current (joined) and past (left) members.
    async fn all(&self) -> Box<dyn Iterator<Item = NodeState> + '_> {
        Box::new(
            self.members
                .values()
                .await
                .into_iter()
                .map(|info| info.value.clone()),
        )
    }

    /// Returns an iterator over the members that have state == `Joined`.
    async fn joined(&self) -> Box<dyn Iterator<Item = NodeState> + '_> {
        Box::new(
            self.members
                .values()
                .await
                .into_iter()
                .map(|info| info.value.clone())
                .filter(|member| member.state == MembershipState::Joined),
        )
    }

    /// Returns joined nodes from our section with age greater than `MIN_AGE`
    async fn mature(&self) -> Box<dyn Iterator<Item = Peer> + '_> {
        Box::new(
            self.joined()
                .await
                .filter(|info| info.is_mature())
                .map(|info| info.peer),
        )
    }

    /// Get info for the member with the given name.
    async fn get(&self, name: &XorName) -> Option<NodeState> {
        self.members.get(name).await.map(|info| info.value.clone())
    }

    /// Get section_signed info for the member with the given name.
    async fn get_section_signed(&self, name: &XorName) -> Option<Arc<SectionAuth<NodeState>>> {
        self.members.get(name).await
    }

    /// Returns the candidates for elders out of all the nodes in this section.
    async fn elder_candidates(
        &self,
        elder_size: usize,
        current_elders: &SectionAuthorityProvider,
    ) -> Vec<Peer> {
        elder_candidates(
            elder_size,
            current_elders,
            self.members
                .values()
                .await
                .into_iter()
                .filter(|info| is_active(&info.value, current_elders))
                .filter(|info| info.value.peer.is_reachable()),
        )
    }

    /// Returns the candidates for elders out of all nodes matching the prefix.
    async fn elder_candidates_matching_prefix(
        &self,
        prefix: &Prefix,
        elder_size: usize,
        current_elders: &SectionAuthorityProvider,
    ) -> Vec<Peer> {
        elder_candidates(
            elder_size,
            current_elders,
            self.members.values().await.into_iter().filter(|info| {
                info.value.state == MembershipState::Joined
                    && prefix.matches(info.value.peer.name())
                    && info.value.peer.is_reachable()
            }),
        )
    }

    /// Returns whether the given peer is a joined member of our section.
    async fn is_joined(&self, name: &XorName) -> bool {
        self.members
            .get(name)
            .await
            .map(|info| info.value.state == MembershipState::Joined)
            .unwrap_or(false)
    }

    /// Returns whether the given peer is already relocated to our section.
    async fn is_relocated(&self, name: &XorName) -> bool {
        self.members
            .any_value(|info| info.value.previous_name == Some(*name))
            .await
    }

    /// Update a member of our section.
    /// Returns whether anything actually changed.
    async fn update(&self, new_info: SectionAuth<NodeState>) -> bool {
        let key = *new_info.value.peer.name();
        self.members
            .insert_if(key, new_info, |(old, new)| {
                // To maintain commutativity, the only allowed transitions are:
                // - Joined -> Joined if the new age is greater than the old age
                // - Joined -> Left
                // - Joined -> Relocated
                // - Relocated -> Left (should not happen, but needed for consistency)
                match (old.value.state, new.value.state) {
                    (MembershipState::Joined, MembershipState::Joined)
                        if new.value.peer.age() > old.value.peer.age() =>
                    {
                        true
                    }
                    (MembershipState::Joined, MembershipState::Left)
                    | (MembershipState::Joined, MembershipState::Relocated(_))
                    | (MembershipState::Relocated(_), MembershipState::Left) => true,
                    _ => false,
                }
            })
            .await
    }

    /// Remove all members whose name does not match `prefix`.
    fn prune_not_matching(&self, prefix: &Prefix) {
        self.members.retain(|name| prefix.matches(name));
    }
}

// Returns the nodes that should become the next elders out of the given members, sorted by names.
// It is assumed that `members` contains only "active" peers (see the `is_active` function below
// for explanation)
fn elder_candidates<I>(
    elder_size: usize,
    current_elders: &SectionAuthorityProvider,
    members: I,
) -> Vec<Peer>
where
    I: IntoIterator<Item = Arc<SectionAuth<NodeState>>>,
{
    members
        .into_iter()
        .sorted_by(|lhs, rhs| cmp_elder_candidates(lhs, rhs, current_elders))
        .map(|info| info.value.peer)
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
    cmp_elder_candidates_by_membership_state(&lhs.value.state, &rhs.value.state)
        .then_with(|| rhs.value.peer.age().cmp(&lhs.value.peer.age()))
        .then_with(|| {
            let lhs_is_elder = is_elder(&lhs.value, current_elders);
            let rhs_is_elder = is_elder(&rhs.value, current_elders);

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
// members to reach `ELDER_SIZE`.
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
    match info.state {
        MembershipState::Joined => true,
        MembershipState::Relocated(_) if is_elder(info, current_elders) => true,
        _ => false,
    }
}

fn is_elder(info: &NodeState, current_elders: &SectionAuthorityProvider) -> bool {
    current_elders.contains_elder(info.peer.name())
}
