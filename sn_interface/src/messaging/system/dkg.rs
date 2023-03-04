// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::NodeState;
use crate::types::{keys::ed25519::Digest256, NodeId, RewardPeer};

use sn_consensus::Generation;

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use tiny_keccak::{Hasher, Sha3};
use xor_name::{Prefix, XorName};

/// Unique identifier of a DKG session.
#[derive(Clone, Eq, PartialEq, Hash, Serialize, Deserialize, custom_debug::Debug)]
pub struct DkgSessionId {
    /// Prefix of the session we are elder candidates for
    pub prefix: Prefix,
    /// Other Elders in this dkg session
    pub elders: BTreeSet<RewardPeer>,
    /// The length of the section chain main branch.
    pub section_chain_len: u64,
    /// The bootstrap members for the next Membership instance.
    pub bootstrap_members: BTreeSet<NodeState>,
    /// The membership generation this SAP was instantiated at
    pub membership_gen: Generation,
}
impl DkgSessionId {
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
        self.elders.iter().map(|p| p.name())
    }

    pub fn elder_ids(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.elders.iter().map(|e| e.node_id())
    }

    pub fn elders(&self) -> impl Iterator<Item = RewardPeer> + '_ {
        self.elders.iter().cloned()
    }

    pub fn elder_index(&self, elder: XorName) -> Option<usize> {
        self.elder_names().sorted().position(|p| p == elder)
    }
}
