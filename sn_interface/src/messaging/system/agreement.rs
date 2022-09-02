// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{KeyedSig, NodeState};
use crate::types::keys::ed25519::{self, Digest256, Keypair, Verifier};
use crate::{messaging::SectionAuthorityProvider, network_knowledge::supermajority, types::Peer};
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
            .all(|e| bootstrap_members.iter().any(|m| &m.name == e)));

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

/// One signed failure for a DKG round by a given `PublicKey`
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

impl DkgFailureSig {
    pub fn new(
        keypair: &Keypair,
        failed_participants: &BTreeSet<XorName>,
        session_id: DkgSessionId,
    ) -> Self {
        Self {
            public_key: keypair.public,
            signature: ed25519::sign(&hashed_failure(&session_id, failed_participants), keypair),
            session_id,
        }
    }

    pub fn verify(&self, dkg_key: &DkgSessionId, failed_participants: &BTreeSet<XorName>) -> bool {
        let hash = hashed_failure(dkg_key, failed_participants);
        self.public_key.verify(&hash, &self.signature).is_ok()
    }
}

impl DkgFailureSigSet {
    // Insert a signature into this set. The signature is assumed valid. Returns `true` if the signature was
    // not already present in the set and `false` otherwise.
    pub fn insert(&mut self, sig: DkgFailureSig, failed_participants: &BTreeSet<XorName>) -> bool {
        if self.failed_participants.is_empty() {
            self.failed_participants = failed_participants.clone();
        }
        if self
            .sigs
            .iter()
            .all(|existing_sig| existing_sig.public_key != sig.public_key)
        {
            self.sigs.push(sig);
            true
        } else {
            false
        }
    }

    // Check whether we have enough signatures to reach agreement on the failure. The contained signatures
    // are assumed valid.
    pub fn has_agreement(&self, session_id: &DkgSessionId) -> bool {
        has_failure_agreement(session_id.elders.len(), self.sigs.len())
    }

    pub fn verify(&self, reference_session_id: &DkgSessionId) -> bool {
        let hash = hashed_failure(reference_session_id, &self.failed_participants);
        let votes = self
            .sigs
            .iter()
            .filter(|sig| {
                let sig_name = ed25519::name(&sig.public_key);
                reference_session_id.contains_elder(sig_name)
                    && sig.public_key.verify(&hash, &sig.signature).is_ok()
            })
            .count();

        has_failure_agreement(reference_session_id.elders.len(), votes)
    }
}

// Check whether we have enough signeds to reach agreement on the failure. We only need
// `N - supermajority(N) + 1` signeds, because that already makes a supermajority agreement on a
// successful outcome impossible.
fn has_failure_agreement(num_participants: usize, num_votes: usize) -> bool {
    num_votes > num_participants - supermajority(num_participants)
}

// Create a value whose signature serves as proof that a failure of a DKG session with the given
// `dkg_key` was observed.
fn hashed_failure(dkg_key: &DkgSessionId, failed_participants: &BTreeSet<XorName>) -> Digest256 {
    let mut hasher = Sha3::v256();
    let mut hash = Digest256::default();

    dkg_key.hash_update(&mut hasher);

    for name in failed_participants.iter() {
        hasher.update(&name.0);
    }

    hasher.update(b"failure");

    hasher.finalize(&mut hash);
    hash
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
    NewElders(SectionAuth<SectionAuthorityProvider>),
    /// Proposal to change whether new nodes are allowed to join our section.
    JoinsAllowed(bool),
}
