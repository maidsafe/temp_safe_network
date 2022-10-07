// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{NodeState, SectionSig};
use crate::types::keys::ed25519::Digest256;
use crate::{messaging::SectionAuthorityProvider, types::Peer};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sn_consensus::Generation;
use std::{
    borrow::Borrow,
    collections::{BTreeMap, BTreeSet},
    net::SocketAddr,
    ops::Deref,
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

/// A section signed piece of data
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct SectionSigned<T: Serialize> {
    /// some value agreed upon by elders
    pub value: T,
    /// section signature over the value
    pub sig: SectionSig,
}

impl<T> Borrow<Prefix> for SectionSigned<T>
where
    T: Borrow<Prefix> + Serialize,
{
    fn borrow(&self) -> &Prefix {
        self.value.borrow()
    }
}

impl<T: Serialize> Deref for SectionSigned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

/// A step in the Propose-Broadcast-Aggregate-Execute workflow.
#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
/// A proposal about the state of the network
/// This can be a result of seeing a node come online, go offline, changes to section info etc.
/// Anything where we need section authority before action can be taken
pub enum Proposal {
    /// Proposal to remove a node from our section
    VoteNodeOffline(NodeState),
    /// Proposal to update info about a section.
    ///
    /// It signals the completion of a DKG by the elder candidates to the current elders.
    /// This proposal is then signed by the newly generated section key.
    SectionInfo(SectionAuthorityProvider),
    /// Proposal to change the elders (and possibly the prefix) of our section.
    /// NOTE: the `SectionAuthorityProvider` is already signed with the new key. This proposal is only to signs the
    /// new key with the current key. That way, when it aggregates, we obtain all the following
    /// pieces of information at the same time:
    ///   1. the new section authority provider
    ///   2. the new key
    ///   3. the signature of the new section authority provider using the new key
    ///   4. the signature of the new key using the current key
    /// Which we can use to update the section section authority provider and the section chain at
    /// the same time as a single atomic operation without needing to cache anything.
    NewElders(SectionSigned<SectionAuthorityProvider>),
    /// Proposal to change whether new nodes are allowed to join our section.
    JoinsAllowed(bool),
}
