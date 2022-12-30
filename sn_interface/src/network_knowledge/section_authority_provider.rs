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
/// Each is signed by their own section key (the one in the SectionAuthorityProvider)
#[allow(clippy::large_enum_variant)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize, Deserialize)]
pub enum SapCandidate {
    ElderHandover(SectionSigned<SectionAuthorityProvider>),
    SectionSplit(
        SectionSigned<SectionAuthorityProvider>,
        SectionSigned<SectionAuthorityProvider>,
    ),
}

impl SapCandidate {
    pub fn elders(&self) -> Vec<Peer> {
        match self {
            SapCandidate::ElderHandover(sap) => sap.elders_vec(),
            SapCandidate::SectionSplit(sap1, sap2) => {
                [sap1.elders_vec(), sap2.elders_vec()].concat().to_vec()
            }
        }
    }
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

#[cfg(any(test, feature = "test-utils"))]
pub mod test_utils {
    use crate::{
        elder_count,
        network_knowledge::{supermajority, MyNodeInfo, NodeState, SectionAuthorityProvider},
        test_utils::gen_sorted_nodes,
    };
    use rand::{thread_rng, RngCore};
    use xor_name::Prefix;

    /// Generate sk_set with the provided threshold, else use sup(elder_count)-1. It's because,
    /// generally we need sup(elder_count) shares to be valid. Thus threshold should be 1
    /// less than that
    pub fn gen_sk_set(
        mut rng: impl RngCore,
        elder_count: usize,
        sk_threshold_size: Option<usize>,
    ) -> bls::SecretKeySet {
        bls::SecretKeySet::random(
            sk_threshold_size.unwrap_or_else(|| supermajority(elder_count).saturating_sub(1)),
            &mut rng,
        )
    }

    /// Builder to generate a `SectionAuthorityProvider`
    pub struct TestSapBuilder {
        prefix: Prefix,
        elder_count: usize,
        adult_count: usize,
        membership_gen: usize,
        elder_nodes: Option<Vec<MyNodeInfo>>,
        adult_nodes: Option<Vec<MyNodeInfo>>,
        sk_set: Option<bls::SecretKeySet>,
        sk_threshold_size: Option<usize>,
        elder_age_pattern: Option<Vec<u8>>,
    }

    impl TestSapBuilder {
        /// Set the `Prefix` of the SAP. Also initiates the SAP builder by providing default values
        /// to the configs.
        pub fn new(prefix: Prefix) -> Self {
            Self {
                prefix,
                elder_count: elder_count(),
                adult_count: 0,
                membership_gen: 0,
                sk_threshold_size: None,
                elder_nodes: None,
                adult_nodes: None,
                sk_set: None,
                elder_age_pattern: None,
            }
        }

        /// Set the number of elders in the SAP. Will be overriden by `elder_nodes.len()` if
        /// the nodes are provided.
        pub fn elder_count(mut self, elder_count: usize) -> Self {
            self.elder_count = elder_count;
            self
        }

        /// Set the number of adults in the SAP. Will be overriden by `adult_nodes.len()` if
        /// the nodes are provided.
        ///
        /// A lot of tests don't require adults in the section, so zero is an acceptable value
        /// for `adult_count`.
        pub fn adult_count(mut self, adult_count: usize) -> Self {
            self.adult_count = adult_count;
            self
        }

        /// Set the membership generation of the SAP
        pub fn membership_gen(mut self, gen: usize) -> Self {
            self.membership_gen = gen;
            self
        }

        /// Use the provided set of nodes as elders
        pub fn elder_nodes(mut self, nodes: Vec<MyNodeInfo>) -> Self {
            self.elder_count = nodes.len();
            self.elder_nodes = Some(nodes);
            self
        }

        /// Use the provided set of nodes as adults
        pub fn adult_nodes(mut self, nodes: Vec<MyNodeInfo>) -> Self {
            self.adult_count = nodes.len();
            self.adult_nodes = Some(nodes);
            self
        }

        /// Use custom `SecretKeySet` for the SAP
        pub fn sk_set(mut self, sk_set: &bls::SecretKeySet) -> Self {
            self.sk_set = Some(sk_set.clone());
            self
        }

        /// Provide a threshold_size for the generated `SecretKeySet`. Will be overriden
        /// if a `sk_set` is provided.
        ///
        /// Some tests require a low threshold.
        pub fn sk_threshold_size(mut self, sk_threshold_size: usize) -> Self {
            self.sk_threshold_size = Some(sk_threshold_size);
            self
        }
        /// Provide `age_pattern` to create elders with specific ages. Will be overriden if you
        /// provide pre crafted elder nodes.
        /// e.g., vec![10, 20] will generate elders with the following age (10, 20, 20, 20...)
        ///
        /// If None = elder's age is set to `MIN_ADULT_AGE`
        /// If age_pattern.len() == elder, then apply the respective ages to each node
        /// If age_pattern.len() < elder, then the last element's value is taken as the age for the remaining nodes.
        /// If age_pattern.len() > elder, then the extra elements after `count` are ignored.
        pub fn elder_age_pattern(mut self, pattern: Vec<u8>) -> Self {
            self.elder_age_pattern = Some(pattern);
            self
        }

        /// Build the final SAP with the provided rng. Also returns the `SecretKeySet` used by the SAP along with the
        /// set of elder, adult nodes.
        ///
        ///  Use `build` if you don't want to provide rng
        pub fn build_rng(
            self,
            rng: impl RngCore,
        ) -> (
            SectionAuthorityProvider,
            bls::SecretKeySet,
            Vec<MyNodeInfo>,
            Vec<MyNodeInfo>,
        ) {
            // Todo: use custom rng to generate the random nodes. `gen_keypair` requires `rand-0.7`
            // version and `SecretKeySet` requires `rand-0.8`; wait for the other one to be bumped.
            let members = gen_sorted_nodes(
                &self.prefix,
                self.elder_count,
                self.adult_count,
                self.elder_age_pattern.as_deref(),
            );
            let elder_nodes = if let Some(elders) = self.elder_nodes {
                elders
            } else {
                members.iter().take(self.elder_count).cloned().collect()
            };
            let adult_nodes = if let Some(adults) = self.adult_nodes {
                adults
            } else {
                members.iter().skip(self.elder_count).cloned().collect()
            };
            let members = elder_nodes
                .iter()
                .chain(adult_nodes.iter())
                .map(|i| NodeState::joined(i.peer(), None));

            let sk_set = if let Some(sk) = self.sk_set {
                sk
            } else {
                gen_sk_set(rng, self.elder_count, self.sk_threshold_size)
            };

            let sap = SectionAuthorityProvider::new(
                elder_nodes.iter().map(|i| i.peer()),
                self.prefix,
                members,
                sk_set.public_keys(),
                self.membership_gen as u64,
            );
            (sap, sk_set, elder_nodes, adult_nodes)
        }

        /// Build the final SAP from the configs. Also returns the `SecretKeySet` used by the SAP along with the
        /// set of elder, adult nodes.
        ///
        /// Use `build_rng` if you want to provide custom rng.
        pub fn build(
            self,
        ) -> (
            SectionAuthorityProvider,
            bls::SecretKeySet,
            Vec<MyNodeInfo>,
            Vec<MyNodeInfo>,
        ) {
            self.build_rng(thread_rng())
        }
    }
}
