// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::NodeState;
use crate::types::keys::ed25519::Digest256;
use crate::types::Peer;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sn_consensus::Generation;
use std::{
    collections::{BTreeMap, BTreeSet},
    net::SocketAddr,
};
use tiny_keccak::{Hasher, Sha3};
use xor_name::{Prefix, XorName};

/// Unique identifier of a DKG session.
#[derive(Clone, Eq, PartialEq, Hash, Serialize, Deserialize, custom_debug::Debug)]
pub struct DkgSessionId {
    /// Prefix of the session we are elder candidates for
    pub prefix: Prefix,
    /// Other Elders in this dkg session
    pub elders: BTreeMap<XorName, SocketAddr>,
    /// The length of the section chain main branch.
    pub section_chain_len: u64,
    /// The bootstrap members for the next Membership instance.
    pub bootstrap_members: BTreeSet<NodeState>,
    /// The membership generation this SAP was instantiated at
    pub membership_gen: Generation,
}
impl DkgSessionId {
    pub fn new(
        prefix: Prefix,
        elders: BTreeMap<XorName, SocketAddr>,
        section_chain_len: u64,
        bootstrap_members: BTreeSet<NodeState>,
        membership_gen: Generation,
    ) -> Self {
        assert!(elders
            .keys()
            .all(|e| bootstrap_members.iter().any(|m| &m.name() == e)));

        // Calculate the hash without involving serialization to avoid having to return `Result`.
        Self {
            prefix,
            elders,
            section_chain_len,
            bootstrap_members,
            membership_gen,
        }
    }

    pub fn hash(&self) -> Digest256 {
        let mut hasher = Sha3::v256();
        self.hash_update(&mut hasher);
        let mut hash = Digest256::default();
        hasher.finalize(&mut hash);
        hash
    }

    /// Short Hash: a small chunk of the session id's hash used for logging as it is very short
    pub fn sh(&self) -> u16 {
        let h = self.hash();
        u16::from_le_bytes([h[0], h[1]])
    }

    pub fn hash_update(&self, hasher: &mut Sha3) {
        hasher.update(&self.prefix.name());

        for elder in self.elder_names() {
            hasher.update(&elder);
        }

        hasher.update(&self.section_chain_len.to_le_bytes());

        for member in &self.bootstrap_members {
            hasher.update(&member.name());
        }
    }

    pub fn elder_names(&self) -> impl Iterator<Item = XorName> + '_ {
        self.elders.keys().copied()
    }

    pub fn elder_peers(&self) -> impl Iterator<Item = Peer> + '_ {
        self.elders
            .iter()
            .map(|(name, addr)| Peer::new(*name, *addr))
    }

    pub fn elder_index(&self, elder: XorName) -> Option<usize> {
        self.elder_names().sorted().position(|p| p == elder)
    }

    pub fn contains_elder(&self, elder: XorName) -> bool {
        self.elder_names().any(|e| e == elder)
    }
}
