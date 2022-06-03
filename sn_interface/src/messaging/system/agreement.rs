// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{KeyedSig, NodeState};
use crate::{messaging::SectionAuthorityProvider, types::Peer};
use ed25519_dalek::{PublicKey, Signature};
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

/// SHA3-256 hash digest.
type Digest256 = [u8; 32];

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
    pub fn hash(&self) -> Digest256 {
        let mut hasher = Sha3::v256();
        self.hash_update(&mut hasher);
        let mut hash = Digest256::default();
        hasher.finalize(&mut hash);
        hash
    }

    pub fn hash_update(&self, hasher: &mut Sha3) {
        hasher.update(&self.prefix.name());

        for elder in self.elder_names() {
            hasher.update(&elder);
        }

        hasher.update(&self.section_chain_len.to_le_bytes());

        for member in self.bootstrap_members.iter() {
            hasher.update(&member.name);
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

/// One signed failure for a DKG round by a given PublicKey
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, custom_debug::Debug)]
pub struct DkgFailureSig {
    #[allow(missing_docs)]
    #[debug(with = "crate::types::PublicKey::fmt_ed25519")]
    pub public_key: PublicKey,
    #[allow(missing_docs)]
    #[debug(with = "crate::types::Signature::fmt_ed25519")]
    pub signature: Signature,
    #[allow(missing_docs)]
    pub session_id: DkgSessionId,
}

/// Dkg failure info for a round
#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct DkgFailureSigSet {
    #[allow(missing_docs)]
    pub sigs: Vec<DkgFailureSig>,
    #[allow(missing_docs)]
    pub failed_participants: BTreeSet<XorName>,
    #[allow(missing_docs)]
    pub session_id: DkgSessionId,
}

impl From<DkgSessionId> for DkgFailureSigSet {
    fn from(session_id: DkgSessionId) -> Self {
        Self {
            session_id,
            sigs: Default::default(),
            failed_participants: Default::default(),
        }
    }
}

/// A value together with the signature that it was agreed on by the majority of the section elders.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct SectionAuth<T: Serialize> {
    /// some value to be agreed upon by elders
    pub value: T,
    /// signature over the value
    pub sig: KeyedSig,
}

impl<T> Borrow<Prefix> for SectionAuth<T>
where
    T: Borrow<Prefix> + Serialize,
{
    fn borrow(&self) -> &Prefix {
        self.value.borrow()
    }
}

impl<T: Serialize> Deref for SectionAuth<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

pub type MembershipGeneration = u64;

/// A step in the Propose-Broadcast-Aggregate-Execute workflow.
#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
/// A proposal about the state of the network
/// This can be a result of seeing a node come online, go offline, changes to section info etc.
/// Anything where we need section authority before action can be taken
pub enum Proposal {
    /// Proposal to remove a node from our section
    Offline(NodeState),
    /// Proposal to update info about a section.
    ///
    /// It signals the completion of a DKG by the elder candidates to the current elders.
    /// This proposal is then signed by the newly generated section key.
    SectionInfo {
        sap: SectionAuthorityProvider,
        generation: MembershipGeneration,
    },
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
    NewElders(SectionAuth<SectionAuthorityProvider>),
    /// Proposal to change whether new nodes are allowed to join our section.
    JoinsAllowed(bool),
}
