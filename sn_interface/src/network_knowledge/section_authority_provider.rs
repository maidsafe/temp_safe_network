// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::NodeState;
use crate::{
    messaging::system::{DkgSessionId, SectionSig, SectionSigned},
    network_knowledge::SectionsDAG,
    types::Peer,
};
use bls::{PublicKey, PublicKeySet};
use serde::{Deserialize, Serialize};
use sn_consensus::Generation;
use std::{
    collections::BTreeSet,
    fmt::{self, Debug, Display, Formatter},
    net::SocketAddr,
};
use xor_name::{Prefix, XorName};

///
pub trait SectionAuthUtils<T: Serialize> {
    ///
    fn new(value: T, sig: SectionSig) -> Self;

    ///
    fn verify(&self, section_dag: &SectionsDAG) -> bool;

    ///
    fn self_verify(&self) -> bool;
}

impl<T: Serialize> SectionAuthUtils<T> for SectionSigned<T> {
    fn new(value: T, sig: SectionSig) -> Self {
        Self { value, sig }
    }

    fn verify(&self, section_dag: &SectionsDAG) -> bool {
        section_dag.has_key(&self.sig.public_key) && self.self_verify()
    }

    fn self_verify(&self) -> bool {
        // verify_sig(&self.sig, &self.value)
        bincode::serialize(&self.value)
            .map(|bytes| self.sig.verify(&bytes))
            .unwrap_or(false)
    }
}

/// Details of section authority.
///
/// A new `SectionAuthorityProvider` is created whenever the elders change, due to an elder being
/// added or removed, or the section splitting or merging.
#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct SectionAuthorityProvider {
    /// The section prefix. It matches all the members' names.
    prefix: Prefix,
    /// Public key set of the section.
    public_key_set: PublicKeySet,
    /// The section's complete set of elders.
    elders: BTreeSet<Peer>,
    /// The section members at the time of this elder churn.
    members: BTreeSet<NodeState>,
    /// The membership generation this SAP was instantiated on
    membership_gen: Generation,
}

/// `SectionAuthorityProvider` candidates for handover consensus to vote on
#[allow(clippy::large_enum_variant)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize, Deserialize)]
pub enum SapCandidate {
    ElderHandover(SectionSigned<SectionAuthorityProvider>),
    SectionSplit(
        SectionSigned<SectionAuthorityProvider>,
        SectionSigned<SectionAuthorityProvider>,
    ),
}

impl Debug for SectionAuthorityProvider {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        #[derive(Debug)]
        enum PeerStatus {
            Elder,
            Member,
        }
        let elders: BTreeSet<_> = self.elders.iter().map(|peer| peer.name()).collect();
        let mut elder_count = 0;
        let mut peers: Vec<_> = self
            .members()
            .map(|peer| {
                let status = if elders.contains(&peer.name()) {
                    elder_count += 1;
                    PeerStatus::Elder
                } else {
                    PeerStatus::Member
                };
                (peer, status)
            })
            .collect();
        peers.sort_by_key(|(_, is_elder)| !matches!(is_elder, PeerStatus::Elder));

        let mut f = f.debug_struct(format!("SAP {:?}", self.prefix).as_str());
        let f = f
            .field("elders", &elders.len())
            .field("members", &self.members.len())
            .field("gen", &self.membership_gen);
        // something went wrong, some `elders` are not part of the `members` list.
        if elder_count != elders.len() {
            f.field(
                "elders (error: some elders are not part of members)",
                &elders,
            )
            .field("members", &self.members().collect::<Vec<_>>())
            .finish()
        } else {
            f.field("peers", &peers).finish()
        }
    }
}

impl Display for SectionAuthorityProvider {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl SectionAuthorityProvider {
    /// Creates a new `SectionAuthorityProvider` with the given members, prefix and public keyset.
    pub fn new<E, M>(
        elders: E,
        prefix: Prefix,
        members: M,
        pk_set: PublicKeySet,
        membership_gen: Generation,
    ) -> Self
    where
        E: IntoIterator<Item = Peer>,
        M: IntoIterator<Item = NodeState>,
    {
        Self {
            prefix,
            public_key_set: pk_set,
            elders: elders.into_iter().collect(),
            members: members.into_iter().collect(),
            membership_gen,
        }
    }

    pub fn from_dkg_session(session_id: &DkgSessionId, pk_set: PublicKeySet) -> Self {
        Self::new(
            session_id.elder_peers(),
            session_id.prefix,
            session_id.bootstrap_members.clone(),
            pk_set,
            session_id.membership_gen,
        )
    }

    pub fn prefix(&self) -> Prefix {
        self.prefix
    }

    // TODO: this should return &BTreeSet<Peer>, let the caller turn it into an iter
    pub fn elders(&self) -> impl Iterator<Item = &Peer> + '_ {
        self.elders.iter()
    }

    pub fn members(&self) -> impl Iterator<Item = &NodeState> + '_ {
        self.members.iter()
    }

    pub fn membership_gen(&self) -> Generation {
        self.membership_gen
    }

    /// A convenience function since we often use SAP elders as recipients.
    pub fn elders_vec(&self) -> Vec<Peer> {
        self.elders.iter().cloned().collect()
    }

    /// A convenience function since we often use SAP elders as recipients.
    pub fn elders_set(&self) -> BTreeSet<Peer> {
        self.elders.iter().cloned().collect()
    }

    // Returns a copy of the public key set
    pub fn public_key_set(&self) -> PublicKeySet {
        self.public_key_set.clone()
    }

    /// Returns the number of elders in the section.
    pub fn elder_count(&self) -> usize {
        self.elders.len()
    }

    /// Returns a map of name to `socket_addr`.
    pub fn contains_elder(&self, name: &XorName) -> bool {
        self.elders.iter().any(|elder| &elder.name() == name)
    }

    /// Returns the elder `Peer` with the given `name`.
    pub fn get_elder(&self, name: &XorName) -> Option<&Peer> {
        self.elders.iter().find(|elder| elder.name() == *name)
    }

    /// Returns the set of elder names.
    pub fn names(&self) -> BTreeSet<XorName> {
        self.elders.iter().map(Peer::name).collect()
    }

    pub fn addresses(&self) -> Vec<SocketAddr> {
        self.elders.iter().map(Peer::addr).collect()
    }

    /// Key of the section.
    pub fn section_key(&self) -> PublicKey {
        self.public_key_set.public_key()
    }
}
