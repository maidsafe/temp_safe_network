// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod stats;

use self::stats::NetworkStats;
use crate::routing::{
    dkg::SectionAuthUtils, peer::PeerUtils, Error, Result, SectionAuthorityProviderUtils,
};

use crate::messaging::{
    node::{Network, Peer, SectionAuth},
    SectionAuthorityProvider,
};
use crate::types::PrefixMap;
use secured_linked_list::SecuredLinkedList;
use std::iter;
use xor_name::{Prefix, XorName};

pub(super) trait NetworkUtils {
    fn new() -> Self;

    fn closest(&self, name: &XorName) -> Option<&SectionAuthorityProvider>;

    /// Returns iterator over all known sections.
    fn all(&self) -> Box<dyn Iterator<Item = &SectionAuthorityProvider> + '_>;

    /// Get `SectionAuthorityProvider` of a known section with the given prefix.
    fn get(&self, prefix: &Prefix) -> Option<&SectionAuthorityProvider>;

    /// Returns all elders from all known sections.
    fn elders(&'_ self) -> Box<dyn Iterator<Item = Peer> + '_>;

    /// Returns a `Peer` of an elder from a known section.
    fn get_elder(&self, name: &XorName) -> Option<Peer>;

    /// Merge two `Network`s into one.
    /// TODO: make this operation commutative, associative and idempotent (CRDT)
    /// TODO: return bool indicating whether anything changed.
    fn merge(&mut self, other: Network, section_chain: &SecuredLinkedList);

    /// Update our knowledge of a remote section's SAP only
    /// if it's verifiable with the provided proof chain.
    fn update_remote_section_sap(
        &mut self,
        signed_section_auth: SectionAuth<SectionAuthorityProvider>,
        proof_chain: &SecuredLinkedList,
        our_section_chain: &SecuredLinkedList,
    ) -> bool;

    /// Returns the known section keys.
    fn keys(&self) -> Box<dyn Iterator<Item = (Prefix, bls::PublicKey)> + '_>;

    /// Returns the latest known key for the prefix that matches `name`.
    fn key_by_name(&self, name: &XorName) -> Result<bls::PublicKey>;

    /// Returns the latest known key for a section with `prefix`.
    /// If this returns `None` that means the latest known key is the genesis key.
    fn key_by_prefix(&self, prefix: &Prefix) -> Option<bls::PublicKey>;

    /// Returns the section_auth and the latest known key for the prefix that matches `name`,
    /// excluding self section.
    fn section_by_name(&self, name: &XorName) -> Result<SectionAuthorityProvider>;

    /// Returns network statistics.
    fn network_stats(&self, our: &SectionAuthorityProvider) -> NetworkStats;

    fn network_elder_counts(&self, our: &SectionAuthorityProvider) -> (u64, u64, bool);
}

impl NetworkUtils for Network {
    fn new() -> Self {
        Self {
            sections: PrefixMap::new(),
        }
    }

    /// Returns the known section that is closest to the given name, regardless of whether `name`
    /// belongs in that section or not.
    fn closest(&self, name: &XorName) -> Option<&SectionAuthorityProvider> {
        self.all()
            .min_by(|lhs, rhs| lhs.prefix.cmp_distance(&rhs.prefix, name))
    }

    /// Returns iterator over all known sections.
    fn all(&self) -> Box<dyn Iterator<Item = &SectionAuthorityProvider> + '_> {
        Box::new(self.sections.iter().map(|section_auth| &section_auth.value))
    }

    /// Get `SectionAuthorityProvider` of a known section with the given prefix.
    fn get(&self, prefix: &Prefix) -> Option<&SectionAuthorityProvider> {
        self.sections
            .get(prefix)
            .map(|section_auth| &section_auth.value)
    }

    /// Returns all elders from all known sections.
    fn elders(&'_ self) -> Box<dyn Iterator<Item = Peer> + '_> {
        Box::new(self.all().flat_map(|info| info.peers()))
    }

    /// Returns a `Peer` of an elder from a known section.
    fn get_elder(&self, name: &XorName) -> Option<Peer> {
        self.sections
            .get_matching(name)?
            .value
            .get_addr(name)
            .map(|addr| {
                let mut peer = Peer::new(*name, addr);
                peer.set_reachable(true);
                peer
            })
    }

    /// Merge two `Network`s into one.
    /// TODO: make this operation commutative, associative and idempotent (CRDT)
    /// TODO: return bool indicating whether anything changed.
    fn merge(&mut self, other: Network, section_chain: &SecuredLinkedList) {
        // FIXME: these operations are not commutative:

        for entry in other.sections {
            if entry.verify(section_chain) {
                let _ = self.sections.insert(entry);
            }
        }
    }

    /// Update our knowledge of a remote section's SAP only
    /// if it's verifiable with the provided proof chain.
    fn update_remote_section_sap(
        &mut self,
        signed_section_auth: SectionAuth<SectionAuthorityProvider>,
        proof_chain: &SecuredLinkedList,
        our_section_chain: &SecuredLinkedList,
    ) -> bool {
        // Check if SAP signature is valid
        if !signed_section_auth.self_verify() {
            trace!(
                "Failed to update remote section knowledge, SAP signature invalid: {:?}",
                signed_section_auth.value
            );
            return false;
        }

        // TODO: Let's make sure the proof chain can be trusted,
        // i.e. check each key is signed by its parent/predecesor key.

        // Check the SAP's key is the last key of the proof chain
        if proof_chain.last_key() != &signed_section_auth.value.public_key_set.public_key() {
            trace!(
                "Failed to update remote section knowledge, SAP's key ({:?}) doesn't match proof chain last key ({:?})",
                 signed_section_auth.value.public_key_set.public_key(), proof_chain.last_key()
            );
            return false;
        }

        // We currently don't keep the complete chain of remote sections (TODO??),
        // **but** the SAPs of remote sections we keep were already verified by us
        // as trusted before we store them in our local records.
        // Thus, we just need to check our knowledge of the remote section's key
        // is part of the proof chain received.
        let is_sap_trusted = match self.sections.get(&signed_section_auth.value.prefix) {
            Some(sap) if sap == &signed_section_auth => {
                // It's the same SAP we are already aware of
                return false;
            }
            Some(sap) => {
                // We are then aware of the prefix, let's just verify
                // the new SAP can be trusted based on the SAP we
                // aware of and the proof chain provided.
                proof_chain.has_key(&sap.value.public_key_set.public_key())
            }
            None => {
                // We are not aware of the prefix, let's then verify
                // it can be trusted based on our own section chain and the
                // provided proof chain.
                our_section_chain.check_trust(proof_chain.keys())
            }
        };

        if !is_sap_trusted {
            trace!(
                "Failed to update remote section knowledge, SAP cannot be trusted: {:?}",
                signed_section_auth.value
            );
            return false;
        }

        // We can now update our knowledge of the remote section's SAP.
        // Note: we don't expect the the same SAP to be found in our records
        // for the prefix since we've already checked that above.
        let _ = self.sections.insert(signed_section_auth);

        true
    }

    /// Returns the known section keys.
    fn keys(&self) -> Box<dyn Iterator<Item = (Prefix, bls::PublicKey)> + '_> {
        Box::new(
            self.sections
                .iter()
                .map(|section_auth| (section_auth.value.prefix, section_auth.value.section_key())),
        )
    }

    /// Returns the latest known key for the prefix that matches `name`.
    fn key_by_name(&self, name: &XorName) -> Result<bls::PublicKey> {
        self.sections
            .get_matching(name)
            .ok_or(Error::NoMatchingSection)
            .map(|section_auth| section_auth.value.section_key())
    }

    /// Returns the latest known key for a section with `prefix`.
    /// If this returns `None` that means the latest known key is the genesis key.
    fn key_by_prefix(&self, prefix: &Prefix) -> Option<bls::PublicKey> {
        self.sections
            .get_equal_or_ancestor(prefix)
            .map(|section_auth| section_auth.value.section_key())
    }

    /// Returns the section_auth and the latest known key for the prefix that matches `name`,
    /// excluding self section.
    fn section_by_name(&self, name: &XorName) -> Result<SectionAuthorityProvider> {
        self.sections
            .get_matching(name)
            .ok_or(Error::NoMatchingSection)
            .map(|section_auth| section_auth.value.clone())
    }

    /// Returns network statistics.
    fn network_stats(&self, our: &SectionAuthorityProvider) -> NetworkStats {
        let (known_elders, total_elders, total_elders_exact) = self.network_elder_counts(our);

        NetworkStats {
            known_elders,
            total_elders,
            total_elders_exact,
        }
    }

    // Compute an estimate of the total number of elders in the network from the size of our
    // routing table.
    //
    // Return (known, total, exact), where `exact` indicates whether `total` is an exact number of
    // an estimate.
    fn network_elder_counts(&self, our: &SectionAuthorityProvider) -> (u64, u64, bool) {
        let known_prefixes = iter::once(&our.prefix).chain(
            self.sections
                .iter()
                .map(|section_auth| &section_auth.value.prefix),
        );
        let is_exact = Prefix::default().is_covered_by(known_prefixes.clone());

        // Estimated fraction of the network that we have in our RT.
        // Computed as the sum of 1 / 2^(prefix.bit_count) for all known section prefixes.
        let network_fraction: f64 = known_prefixes
            .map(|p| 1.0 / (p.bit_count() as f64).exp2())
            .sum();

        let known = our.elder_count() + self.elders().count();
        let total = known as f64 / network_fraction;

        (known as u64, total.ceil() as u64, is_exact)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routing::{dkg, section};
    use rand::Rng;

    #[test]
    fn closest() -> Result<()> {
        let sk = bls::SecretKey::random();
        let chain = SecuredLinkedList::new(sk.public_key());

        let p01: Prefix = "01".parse().unwrap();
        let p10: Prefix = "10".parse().unwrap();
        let p11: Prefix = "11".parse().unwrap();

        // Create map containing sections (00), (01) and (10)
        let mut map = Network::new();
        let _ = map.update_remote_section_sap(gen_section_auth(&sk, p01)?, &chain, &chain);
        let _ = map.update_remote_section_sap(gen_section_auth(&sk, p10)?, &chain, &chain);

        let mut rng = rand::thread_rng();
        let n01 = p01.substituted_in(rng.gen());
        let n10 = p10.substituted_in(rng.gen());
        let n11 = p11.substituted_in(rng.gen());

        assert_eq!(map.closest(&n01).map(|i| &i.prefix), Some(&p01));
        assert_eq!(map.closest(&n10).map(|i| &i.prefix), Some(&p10));
        assert_eq!(map.closest(&n11).map(|i| &i.prefix), Some(&p10));

        Ok(())
    }

    fn gen_section_auth(
        sk: &bls::SecretKey,
        prefix: Prefix,
    ) -> Result<SectionAuth<SectionAuthorityProvider>> {
        let (section_auth, _, _) = section::test_utils::gen_section_authority_provider(prefix, 5);
        dkg::test_utils::section_signed(sk, section_auth)
    }
}
