// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::ElderCandidates;
use crate::messaging::{
    system::SectionAuth, SectionAuthorityProvider as SectionAuthorityProviderMsg,
};
use crate::routing::{Peer, Prefix, XorName};
use bls::{PublicKey, PublicKeySet};
use std::{
    collections::{BTreeMap, BTreeSet},
    net::SocketAddr,
};

/// Details of section authority.
///
/// A new `SectionAuthorityProvider` is created whenever the elders change, due to an elder being
/// added or removed, or the section splitting or merging.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize)]
// TODO: make pub(crate) - but it's used by routing stress_example atm
pub struct SectionAuthorityProvider {
    prefix: Prefix,
    public_key_set: PublicKeySet,
    elders: BTreeMap<XorName, SocketAddr>,
}

impl SectionAuthorityProvider {
    /// Creates a new `SectionAuthorityProvider` with the given members, prefix and public keyset.
    pub(crate) fn new<I>(elders: I, prefix: Prefix, pk_set: PublicKeySet) -> Self
    where
        I: IntoIterator<Item = Peer>,
    {
        let elders = elders
            .into_iter()
            .map(|peer| (peer.name(), peer.addr()))
            .collect();

        Self {
            prefix,
            public_key_set: pk_set,
            elders,
        }
    }

    /// Creates a new `SectionAuthorityProvider` from ElderCandidates and public keyset.
    pub(crate) fn from_elder_candidates(
        elder_candidates: ElderCandidates,
        pk_set: PublicKeySet,
    ) -> SectionAuthorityProvider {
        SectionAuthorityProvider {
            prefix: elder_candidates.prefix(),
            public_key_set: pk_set,
            elders: elder_candidates
                .elders()
                .map(|peer| (peer.name(), peer.addr()))
                .collect(),
        }
    }

    pub(crate) fn prefix(&self) -> Prefix {
        self.prefix
    }

    pub(crate) fn elders(&self) -> &BTreeMap<XorName, SocketAddr> {
        &self.elders
    }

    /// Returns `ElderCandidates`, which doesn't have key related infos.
    pub(crate) fn elder_candidates(&self) -> ElderCandidates {
        ElderCandidates::new(self.prefix, self.peers())
    }

    pub(crate) fn peers(&'_ self) -> Vec<Peer> {
        self.elders
            .iter()
            .map(|(name, addr)| Peer::new(*name, *addr))
            .collect()
    }

    /// Returns the number of elders in the section.
    pub(crate) fn elder_count(&self) -> usize {
        self.elders.len()
    }

    /// Returns a map of name to socket_addr.
    pub(crate) fn contains_elder(&self, name: &XorName) -> bool {
        self.elders.contains_key(name)
    }

    /// Returns a socket_addr of an elder.
    pub(crate) fn get_addr(&self, name: &XorName) -> Option<SocketAddr> {
        self.elders.get(name).copied()
    }

    /// Returns the set of elder names.
    pub(crate) fn names(&self) -> BTreeSet<XorName> {
        self.elders.keys().copied().collect()
    }

    pub(crate) fn addresses(&self) -> Vec<SocketAddr> {
        self.elders.values().copied().collect()
    }

    /// Key of the section.
    // TODO: make pub(crate) - but it's used by routing stress_example atm
    pub fn section_key(&self) -> PublicKey {
        self.public_key_set.public_key()
    }
}

// Add conversion methods to/from `messaging::...::NodeState`
// We prefer this over `From<...>` to make it easier to read the conversion.

impl SectionAuthorityProvider {
    pub(crate) fn into_msg(self) -> SectionAuthorityProviderMsg {
        SectionAuthorityProviderMsg {
            prefix: self.prefix,
            public_key_set: self.public_key_set,
            elders: self.elders,
        }
    }
}

impl SectionAuth<SectionAuthorityProvider> {
    pub(crate) fn into_authed_msg(self) -> SectionAuth<SectionAuthorityProviderMsg> {
        SectionAuth {
            value: self.value.into_msg(),
            sig: self.sig,
        }
    }
}

impl SectionAuthorityProviderMsg {
    pub(crate) fn into_state(self) -> SectionAuthorityProvider {
        SectionAuthorityProvider {
            prefix: self.prefix,
            public_key_set: self.public_key_set,
            elders: self.elders,
        }
    }
}

impl SectionAuth<SectionAuthorityProviderMsg> {
    pub(crate) fn into_authed_state(self) -> SectionAuth<SectionAuthorityProvider> {
        SectionAuth {
            value: self.value.into_state(),
            sig: self.sig,
        }
    }
}

#[cfg(test)]
pub(crate) mod test_utils {
    use super::*;
    use crate::routing::routing_api::tests::SecretKeySet;
    use crate::routing::{ed25519, node::Node, MIN_ADULT_AGE, MIN_AGE};
    use itertools::Itertools;
    use std::{cell::Cell, net::SocketAddr};
    use xor_name::Prefix;

    // Generate unique SocketAddr for testing purposes
    pub(crate) fn gen_addr() -> SocketAddr {
        thread_local! {
            static NEXT_PORT: Cell<u16> = Cell::new(1000);
        }

        let port = NEXT_PORT.with(|cell| cell.replace(cell.get().wrapping_add(1)));

        ([192, 0, 2, 0], port).into()
    }

    // Create `count` Nodes sorted by their names.
    // The `age_diff` flag is used to trigger nodes being generated with different age pattern.
    // The test of `handle_agreement_on_online_of_elder_candidate` requires most nodes to be with
    // age of MIN_AGE + 2 and one node with age of MIN_ADULT_AGE.
    pub(crate) fn gen_sorted_nodes(prefix: &Prefix, count: usize, age_diff: bool) -> Vec<Node> {
        (0..count)
            .map(|index| {
                let age = if age_diff && index < count - 1 {
                    MIN_AGE + 2
                } else {
                    MIN_ADULT_AGE
                };
                Node::new(
                    ed25519::gen_keypair(&prefix.range_inclusive(), age),
                    gen_addr(),
                )
            })
            .sorted_by_key(|node| node.name())
            .collect()
    }

    // Generate random `SectionAuthorityProvider` for testing purposes.
    pub(crate) fn gen_section_authority_provider(
        prefix: Prefix,
        count: usize,
    ) -> (SectionAuthorityProvider, Vec<Node>, SecretKeySet) {
        let nodes = gen_sorted_nodes(&prefix, count, false);
        let elders = nodes.iter().map(Node::peer);

        let secret_key_set = SecretKeySet::random();
        let section_auth = SectionAuthorityProvider::from_elder_candidates(
            ElderCandidates::new(prefix, elders),
            secret_key_set.public_keys(),
        );

        (section_auth, nodes, secret_key_set)
    }
}
