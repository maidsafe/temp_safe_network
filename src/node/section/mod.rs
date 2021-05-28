// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod member_info;
mod peer;
mod section_authority_provider;

pub use member_info::{MemberInfo, PeerState};
pub use peer::Peer;
pub use section_authority_provider::{ElderCandidates, SectionAuthorityProvider};

use crate::node::agreement::Proven;
use secured_linked_list::SecuredLinkedList;
use serde::{Deserialize, Serialize};
use std::{
    collections::{btree_map, BTreeMap},
    hash::{Hash, Hasher},
};
use threshold_crypto::PublicKey as BlsPublicKey;
use xor_name::XorName;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Section {
    pub genesis_key: BlsPublicKey,
    pub chain: SecuredLinkedList,
    pub section_auth: Proven<SectionAuthorityProvider>,
    pub members: SectionPeers,
}

/// Container for storing information about members of our section.
#[derive(Clone, Default, Debug, Eq, Serialize, Deserialize)]
pub struct SectionPeers {
    pub members: BTreeMap<XorName, Proven<MemberInfo>>,
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

pub struct IntoIter(btree_map::IntoIter<XorName, Proven<MemberInfo>>);

impl Iterator for IntoIter {
    type Item = Proven<MemberInfo>;

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
