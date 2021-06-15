// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod candidates;
mod node_state;
mod peer;

pub use candidates::ElderCandidates;
pub use node_state::{MembershipState, NodeState};
pub use peer::Peer;

use crate::messaging::{node::agreement::SectionSigned, SectionAuthorityProvider};
use secured_linked_list::SecuredLinkedList;
use serde::{Deserialize, Serialize};
use std::{
    collections::{btree_map, BTreeMap},
    hash::{Hash, Hasher},
};
use xor_name::XorName;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Section {
    pub genesis_key: BlsPublicKey,
    pub chain: SecuredLinkedList,
    pub section_auth: SectionSigned<SectionAuthorityProvider>,
    pub members: SectionPeers,
}

/// Container for storing information about members of our section.
#[derive(Clone, Default, Debug, Eq, Serialize, Deserialize)]
pub struct SectionPeers {
    pub members: BTreeMap<XorName, SectionSigned<NodeState>>,
}

impl PartialEq for SectionPeers {
    fn eq(&self, other: &Self) -> bool {
        self.members == other.members
    }
}

impl Hash for SectionPeers {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.members.hash(state)
    }
}

pub struct IntoIter(btree_map::IntoIter<XorName, SectionSigned<NodeState>>);

impl Iterator for IntoIter {
    type Item = SectionSigned<NodeState>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(_, info)| info)
    }
}

impl IntoIterator for SectionPeers {
    type IntoIter = IntoIter;
    type Item = <Self::IntoIter as Iterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.members.into_iter())
    }
}
