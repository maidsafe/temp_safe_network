// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::NodeState;

use crate::messaging::system::DkgSessionId;
use crate::messaging::system::{SectionSig, SectionSigned};
use crate::types::Peer;
use sn_consensus::Generation;
use xor_name::{Prefix, XorName};

use crate::network_knowledge::SectionsDAG;
use bls::{PublicKey, PublicKeySet};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fmt::{self, Display, Formatter},
    net::SocketAddr,
};

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
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
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

impl Display for SectionAuthorityProvider {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let elders_info: Vec<_> = self
            .elders
            .iter()
            .map(|peer| (peer.addr(), peer.name()))
            .collect();
        write!(
            f,
            "Sap {:?}  elder len:{} gen:{} contains: {{{:?}}})",
            self.prefix,
            self.elders.len(),
            self.membership_gen,
            elders_info,
        )
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
        messaging::system::SectionSigned,
        network_knowledge::{MyNodeInfo, NodeState, SectionAuthorityProvider},
        test_utils::{gen_sorted_nodes, TestKeys},
    };
    use eyre::Result;
    use rand::RngCore;
    use xor_name::Prefix;

    /// `SectionAuthorityProvider` related utils for testing
    pub struct TestSAP {}

    impl TestSAP {
        /// Generate a random `SectionAuthorityProvider` for testing.
        ///
        /// The total number of members in the section will be `elder_count` + `adult_count`. A lot of
        /// tests don't require adults in the section, so zero is an acceptable value for
        /// `adult_count`.
        ///
        /// The rng is used to generate the `SecretKeySet`
        ///
        /// An optional `sk_threshold_size` can be passed to specify the threshold when the secret key
        /// set is generated for the section key. Some tests require a low threshold.
        pub fn random_sap_with_rng<R: RngCore>(
            rng: &mut R,
            prefix: Prefix,
            elder_count: usize,
            adult_count: usize,
            sk_threshold_size: Option<usize>,
        ) -> (SectionAuthorityProvider, Vec<MyNodeInfo>, bls::SecretKeySet) {
            let nodes = gen_sorted_nodes(&prefix, elder_count + adult_count, false);
            let elders = nodes.iter().map(MyNodeInfo::peer).take(elder_count);
            let members = nodes.iter().map(|i| NodeState::joined(i.peer(), None));
            let poly = bls::poly::Poly::random(sk_threshold_size.unwrap_or(0), rng);
            let sks = bls::SecretKeySet::from(poly);
            let section_auth =
                SectionAuthorityProvider::new(elders, prefix, members, sks.public_keys(), 0);
            (section_auth, nodes, sks)
        }

        /// Generate a random `SectionAuthorityProvider` for testing.
        ///
        /// Same as `random_sap_with_rng` but with `thread_rng`.
        pub fn random_sap(
            prefix: Prefix,
            elder_count: usize,
            adult_count: usize,
            sk_threshold_size: Option<usize>,
        ) -> (SectionAuthorityProvider, Vec<MyNodeInfo>, bls::SecretKeySet) {
            Self::random_sap_with_rng(
                &mut rand::thread_rng(),
                prefix,
                elder_count,
                adult_count,
                sk_threshold_size,
            )
        }

        /// Generate a random `SectionAuthorityProvider` for testing.
        ///
        /// Same as `random_sap`, but instead the secret key is provided. This can be useful for
        /// creating a section to share the same genesis key as another one.
        pub fn random_sap_with_key(
            prefix: Prefix,
            elder_count: usize,
            adult_count: usize,
            sk_set: &bls::SecretKeySet,
        ) -> (SectionAuthorityProvider, Vec<MyNodeInfo>) {
            let nodes = gen_sorted_nodes(&prefix, elder_count + adult_count, false);
            let elders = nodes.iter().map(MyNodeInfo::peer).take(elder_count);
            let members = nodes.iter().map(|i| NodeState::joined(i.peer(), None));
            let section_auth =
                SectionAuthorityProvider::new(elders, prefix, members, sk_set.public_keys(), 0);
            (section_auth, nodes)
        }

        /// Generate a random `SectionAuthorityProvider` which is signed by itself
        pub fn random_signed_sap(
            prefix: Prefix,
            elder_count: usize,
            adult_count: usize,
            sk_threshold_size: Option<usize>,
        ) -> Result<(
            SectionSigned<SectionAuthorityProvider>,
            Vec<MyNodeInfo>,
            bls::SecretKey,
        )> {
            let (section_auth, nodes, secret_key_set) =
                Self::random_sap(prefix, elder_count, adult_count, sk_threshold_size);
            let signed_sap =
                TestKeys::get_section_signed(&secret_key_set.secret_key(), section_auth)?;

            Ok((signed_sap, nodes, secret_key_set.secret_key()))
        }
    }
}
