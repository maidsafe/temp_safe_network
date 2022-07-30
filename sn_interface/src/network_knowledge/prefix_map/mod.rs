// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! Container that acts as a map whose keys are section Prefixes, and values
//! are the Section Authority Provider of the section.
//!
//! It automatically prunes redundant entries. That is, when the prefix of an entry is fully
//! covered by other prefixes, that entry is removed. For example, when there is entry with
//! prefix (00) and we insert entries with (000) and (001), the (00) prefix becomes fully
//! covered and is automatically removed.
//!

mod stats;

use self::stats::NetworkStats;

use crate::messaging::system::SectionAuth;
use crate::network_knowledge::{Error, Result, SectionAuthUtils, SectionAuthorityProvider};

use bls::PublicKey as BlsPublicKey;
use secured_linked_list::SecuredLinkedList;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::{
    cmp::Ordering,
    iter::{self, Iterator},
};
use xor_name::{Prefix, XorName};

/// Container for storing information about other sections in the network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPrefixMap {
    /// Map of sections prefixes to their latest signed section authority providers.
    sections: BTreeMap<Prefix, SectionAuth<SectionAuthorityProvider>>,
    /// A DAG containing all section chains of the whole network that we are aware of
    sections_dag: SecuredLinkedList,
}

impl NetworkPrefixMap {
    /// Create an empty container
    pub fn new(genesis_pk: BlsPublicKey) -> Self {
        Self {
            sections: BTreeMap::new(),
            sections_dag: SecuredLinkedList::new(genesis_pk),
        }
    }

    /// Returns the genesis key of the Network
    pub fn genesis_key(&self) -> &BlsPublicKey {
        self.sections_dag.root_key()
    }

    pub fn get_sections_dag(&self) -> &SecuredLinkedList {
        &self.sections_dag
    }

    /// Inserts new entry into the map. Replaces previous entry at the same prefix.
    /// Removes those ancestors of the inserted prefix.
    /// Does not insert anything if any descendant of the prefix of `entry` is already present in
    /// the map.
    /// Returns a boolean indicating whether anything changed.
    //
    // This is not a public API since we shall not allow any insert/update without a
    // proof chain, users shall call the `update` API.
    fn insert(&mut self, sap: SectionAuth<SectionAuthorityProvider>) -> bool {
        let prefix = sap.prefix();
        // Don't insert if any descendant is already present in the map.
        if self
            .sections
            .iter()
            .any(|(p, _)| p.is_extension_of(&prefix))
        {
            return false;
        }

        let _prev = self.sections.insert(prefix, sap);

        if prefix.is_empty() {
            return true;
        }
        let parent_prefix = prefix.popped();

        self.prune(parent_prefix);
        true
    }

    /// For testing purpose, we may need to populate a `prefix_map` without a proof chain.
    #[cfg(any(test, feature = "test-utils"))]
    pub fn insert_without_chain(&mut self, sap: SectionAuth<SectionAuthorityProvider>) -> bool {
        self.insert(sap)
    }

    /// Returns the known section that is closest to the given name,
    /// regardless of whether `name` belongs in that section or not.
    /// If provided, it excludes any section matching the passed prefix.
    fn closest(
        &self,
        name: &XorName,
        exclude: Option<&Prefix>,
    ) -> Option<SectionAuth<SectionAuthorityProvider>> {
        self.sections
            .iter()
            .filter(|&(prefix, _)| {
                if let Some(p) = exclude {
                    prefix != p
                } else {
                    true
                }
            })
            .min_by(|&(prefix_lhs, _), &(prefix_rhs, _)| prefix_lhs.cmp_distance(prefix_rhs, name))
            .map(|(_, sap)| sap.clone())
    }

    /// Returns the known section that is closest to the given name,
    /// regardless of whether `name` belongs in that section or not.
    /// If there are no close matches in remote sections, return a SAP from an opposite prefix.
    /// If provided, it excludes any section matching the passed prefix.
    pub fn closest_or_opposite(
        &self,
        name: &XorName,
        exclude: Option<&Prefix>,
    ) -> Option<SectionAuth<SectionAuthorityProvider>> {
        self.closest(name, exclude).or_else(|| {
            self.sections
                .iter()
                .filter(|&(prefix, _)| prefix.matches(&name.with_bit(0, !name.bit(0))))
                .max_by_key(|&(prefix, _)| prefix.bit_count())
                .map(|(_, sap)| sap.clone())
        })
    }

    /// Returns all known sections SAP.
    pub fn all(&self) -> Vec<SectionAuthorityProvider> {
        self.sections
            .iter()
            .map(|(_, sap)| sap.value.clone())
            .collect()
    }

    /// Get `SectionAuthorityProvider` of a known section with the given prefix.
    #[allow(unused)]
    pub fn get(&self, prefix: &Prefix) -> Option<SectionAuthorityProvider> {
        self.sections.get(prefix).map(|sap| sap.value.clone())
    }

    /// Get signed `SectionAuthorityProvider` of a known section with the given prefix.
    pub fn get_signed(&self, prefix: &Prefix) -> Option<SectionAuth<SectionAuthorityProvider>> {
        self.sections.get(prefix).cloned()
    }

    /// Update our knowledge on the remote section's SAP and our sections DAG
    /// if it's verifiable with the provided proof chain.
    /// Returns true if an update was made
    pub fn update(
        &mut self,
        signed_sap: SectionAuth<SectionAuthorityProvider>,
        proof_chain: &SecuredLinkedList,
    ) -> Result<bool> {
        // Check if SAP signature is valid
        if !signed_sap.self_verify() {
            return Err(Error::UntrustedSectionAuthProvider(format!(
                "invalid signature: {:?}",
                signed_sap.value
            )));
        }

        // Check if SAP's section key matches SAP signature's key
        if signed_sap.sig.public_key != signed_sap.section_key() {
            return Err(Error::UntrustedSectionAuthProvider(format!(
                "section key doesn't match signature's key: {:?}",
                signed_sap.value
            )));
        }

        let incoming_prefix = &signed_sap.prefix();

        // Lets warn if we're in a SAP that's shrinking for some reason.
        // So we check the incoming elder count vs what we know of for
        // the incoming prefix. If elder_count() is smaller at _all_ we
        // should warn! something so we can track this.
        if !incoming_prefix.is_empty() {
            match self.section_by_prefix(incoming_prefix) {
                Ok(sap) => {
                    let current_sap_elder_count = sap.elder_count();
                    let proposed_sap_elder_count = signed_sap.elder_count();

                    if proposed_sap_elder_count < current_sap_elder_count {
                        warn!("Proposed SAP elder count is LESS than current... proposed: {proposed_sap_elder_count:?}, current: {current_sap_elder_count:?} (proposed is: {signed_sap:?})");
                    }
                }
                Err(e) => {
                    warn!("Could not find related section to {incoming_prefix:?} in order to validate SAP's section is not shrinking");
                    warn!("Error on prefix search: {e}");
                }
            };
        }

        match self.sections.get(incoming_prefix) {
            Some(sap) if sap == &signed_sap => {
                // It's the same SAP we are already aware of
                return Ok(false);
            }
            Some(sap) => {
                // We are then aware of the prefix, let's just verify the new SAP can
                // be trusted based on the SAP we aware of and the proof chain provided.
                if !proof_chain.has_key(&sap.section_key()) {
                    // This case may happen when both the sender and receiver is about to using
                    // a new SAP. The AE-Update was sent before sender switching to use new SAP,
                    // hence it only contains proof_chain covering the old SAP.
                    // When the update arrives after the receiver got switched to use new SAP,
                    // this error will be complained.
                    // As an outdated node will got updated via AE triggered by other messages,
                    // there is no need to bounce back here (assuming the sender is outdated) to
                    // avoid potential looping.
                    return Err(Error::UntrustedProofChain(format!(
                        "provided proof_chain doesn't cover the SAP's key we currently know: {:?}",
                        sap.value
                    )));
                }
            }
            None => {
                // We are not aware of the prefix, let's then verify it can be
                // trusted based on our own section chain and the provided proof chain.
                if !proof_chain.check_trust(self.sections_dag.keys()) {
                    return Err(Error::UntrustedProofChain(format!(
                        "none of the keys were found on our section chain: {:?}",
                        signed_sap.value
                    )));
                }
            }
        }

        // Make sure the proof chain can be trusted,
        // i.e. check each key is signed by its parent/predecessor key.
        if !proof_chain.self_verify() {
            return Err(Error::UntrustedProofChain(format!(
                "invalid proof chain: {:?}",
                proof_chain
            )));
        }

        // Check the SAP's key is the last key of the proof chain
        if proof_chain.last_key() != &signed_sap.section_key() {
            return Err(Error::UntrustedSectionAuthProvider(format!(
                "section key ({:?}, from prefix {:?}) isn't in the last key in the proof chain provided. (Which ends with ({:?}))",
                signed_sap.section_key(),
                signed_sap.prefix(),
                proof_chain.last_key()
            )));
        }

        // We can now update our knowledge of the remote section's SAP.
        // Note: we don't expect the same SAP to be found in our records
        // for the prefix since we've already checked that above.
        let changed = self.insert(signed_sap);

        // update our sections_dag with the proof_chain. Cannot be an error, since in cases where we
        // have outdated SAP (aware of prefix)/ not aware of the prefix, we have the proof_chains's
        // root/child key in our sections_dag. Checked in the above match statement.
        self.sections_dag.join(proof_chain.clone())?;

        for (prefix, section_key) in self.sections.iter() {
            debug!(
                "Known prefix,section_key after update: {:?} = {:?}",
                prefix, section_key
            );
        }
        debug!("updated sections_dag: {:?}", self.sections_dag);

        Ok(changed)
    }

    /// Returns the known section public keys.
    pub fn section_keys(&self) -> Vec<bls::PublicKey> {
        self.sections
            .iter()
            .map(|(_, sap)| sap.section_key())
            .collect()
    }

    /// Number of SAPs we know about.
    pub fn len(&self) -> usize {
        self.sections.len()
    }

    /// Is the prefix map empty?
    pub fn is_empty(&self) -> bool {
        self.sections.is_empty()
    }

    /// Returns the section authority provider for the prefix that matches `name`.
    /// In case there is no prefix matches the `name`, we shall return the one with longest
    /// common bits. i.e. for the name of `00xxx`, if we have `01` and `1`, then we shall return
    /// with `01`.
    pub fn section_by_name(&self, name: &XorName) -> Result<SectionAuthorityProvider> {
        self.sections
            .iter()
            .max_by_key(|&(prefix, _)| prefix.common_prefix(name))
            .ok_or(Error::NoMatchingSection)
            .map(|(_, sap)| sap.value.clone())
    }

    /// Get the section that matches `prefix`. In case of multiple matches, returns the
    /// one with the longest prefix.
    pub fn section_by_prefix(&self, prefix: &Prefix) -> Result<SectionAuthorityProvider> {
        self.section_by_name(&prefix.name())
    }

    /// Get total number of known sections
    pub fn known_sections_count(&self) -> usize {
        self.sections.len()
    }

    /// Returns network statistics.
    pub fn network_stats(&self, our: &SectionAuthorityProvider) -> NetworkStats {
        // Let's compute an estimate of the total number of elders in the network
        // from the size of our routing table.
        let section_prefixes = self.sections.iter().map(|(prefix, _)| *prefix);
        let known_prefixes: Vec<_> = section_prefixes.chain(iter::once(our.prefix())).collect();

        let total_elders_exact = Prefix::default().is_covered_by(&known_prefixes);

        // Estimated fraction of the network that we have in our RT.
        // Computed as the sum of 1 / 2^(prefix.bit_count) for all known section prefixes.
        let network_fraction: f64 = known_prefixes
            .iter()
            .map(|p| 1.0 / (p.bit_count() as f64).exp2())
            .sum();

        let network_elders_count: usize =
            self.sections.iter().map(|(_, sap)| sap.elder_count()).sum();
        let total = network_elders_count as f64 / network_fraction;

        // `total_elders_exact` indicates whether `total_elders` is
        // an exact number or an estimate.
        NetworkStats {
            known_elders: network_elders_count as u64,
            total_elders: total.ceil() as u64,
            total_elders_exact,
        }
    }

    /// Remove `prefix` and any of its ancestors.
    fn prune(&mut self, mut prefix: Prefix) {
        loop {
            let _prev = self.sections.remove(&prefix);

            if prefix.is_empty() {
                break;
            } else {
                prefix = prefix.popped();
            }
        }
    }
}

impl Ord for NetworkPrefixMap {
    fn cmp(&self, other: &Self) -> Ordering {
        self.len().cmp(&other.len())
    }
}

impl PartialOrd for NetworkPrefixMap {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for NetworkPrefixMap {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len()
    }
}

impl Eq for NetworkPrefixMap {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network_knowledge::test_utils::{random_sap, section_signed};
    use eyre::{eyre, Context, Result};
    use proptest::{
        prelude::{any, Strategy},
        prop_assert, proptest,
    };
    use rand::{
        prelude::{SeedableRng, SmallRng},
        Rng,
    };
    use std::{
        cell::RefCell,
        collections::{BTreeMap, BTreeSet},
        rc::Rc,
    };

    #[test]
    fn insert_existing_prefix() -> Result<()> {
        let (mut map, _, _) = new_network_prefix_map();
        let p0 = prefix("0")?;
        let sap0 = gen_section_auth(p0)?;
        let new_sap0 = gen_section_auth(p0)?;
        assert_ne!(sap0, new_sap0);

        assert!(map.insert(sap0));
        assert!(map.insert(new_sap0.clone()));
        assert_eq!(map.get(&p0), Some(new_sap0.value));

        Ok(())
    }

    #[test]
    fn insert_direct_descendants_of_existing_prefix() -> Result<()> {
        let (mut map, _, _) = new_network_prefix_map();
        let p0 = prefix("0")?;
        let p00 = prefix("00")?;
        let p01 = prefix("01")?;

        let sap0 = gen_section_auth(p0)?;
        assert!(map.insert(sap0));

        // Insert the first sibling. Parent get pruned in the map.
        let sap00 = gen_section_auth(p00)?;
        assert!(map.insert(sap00.clone()));

        assert_eq!(map.get(&p00), Some(sap00.value.clone()));
        assert_eq!(map.get(&p01), None);
        assert_eq!(map.get(&p0), None);

        // Insert the other sibling.
        let sap3 = gen_section_auth(p01)?;
        assert!(map.insert(sap3.clone()));

        assert_eq!(map.get(&p00), Some(sap00.value));
        assert_eq!(map.get(&p01), Some(sap3.value));
        assert_eq!(map.get(&p0), None);

        Ok(())
    }

    #[test]
    fn return_opposite_prefix_if_none_matching() -> Result<()> {
        let (mut map, _, _) = new_network_prefix_map();
        let p0 = prefix("0")?;
        let p1 = prefix("1")?;

        let sap0 = gen_section_auth(p0)?;

        let _changed = map.insert(sap0.clone());

        assert_eq!(
            map.section_by_name(&p1.substituted_in(xor_name::rand::random()))?,
            sap0.value
        );

        // There are no matching prefixes, so return an opposite prefix.
        assert_eq!(
            map.closest_or_opposite(&p1.substituted_in(xor_name::rand::random()), None)
                .ok_or(Error::NoMatchingSection)?,
            sap0
        );

        let _changed = map.insert(sap0.clone());
        assert_eq!(
            map.closest_or_opposite(&p1.substituted_in(xor_name::rand::random()), None)
                .ok_or(Error::NoMatchingSection)?,
            sap0
        );

        Ok(())
    }

    #[test]
    fn insert_indirect_descendants_of_existing_prefix() -> Result<()> {
        let (mut map, _, _) = new_network_prefix_map();
        let p0 = prefix("0")?;
        let p000 = prefix("000")?;
        let p001 = prefix("001")?;
        let p00 = prefix("00")?;
        let p01 = prefix("01")?;

        let sap0 = gen_section_auth(p0)?;
        let sap01 = gen_section_auth(p01)?;
        let sap000 = gen_section_auth(p000)?;
        let sap001 = gen_section_auth(p001)?;

        assert!(map.insert(sap0));

        assert!(map.insert(sap000.clone()));
        assert_eq!(map.get(&p000), Some(sap000.value.clone()));
        assert_eq!(map.get(&p001), None);
        assert_eq!(map.get(&p00), None);
        assert_eq!(map.get(&p01), None);
        assert_eq!(map.get(&p0), None);

        assert!(map.insert(sap001.clone()));
        assert_eq!(map.get(&p000), Some(sap000.value.clone()));
        assert_eq!(map.get(&p001), Some(sap001.value.clone()));
        assert_eq!(map.get(&p00), None);
        assert_eq!(map.get(&p01), None);
        assert_eq!(map.get(&p0), None);

        assert!(map.insert(sap01.clone()));
        assert_eq!(map.get(&p000), Some(sap000.value));
        assert_eq!(map.get(&p001), Some(sap001.value));
        assert_eq!(map.get(&p00), None);
        assert_eq!(map.get(&p01), Some(sap01.value));
        assert_eq!(map.get(&p0), None);

        Ok(())
    }

    #[test]
    fn insert_ancestor_of_existing_prefix() -> Result<()> {
        let (mut map, _, _) = new_network_prefix_map();
        let p0 = prefix("0")?;
        let p00 = prefix("00")?;

        let sap0 = gen_section_auth(p0)?;
        let sap00 = gen_section_auth(p00)?;
        let _changed = map.insert(sap00.clone());

        assert!(!map.insert(sap0));
        assert_eq!(map.get(&p0), None);
        assert_eq!(map.get(&p00), Some(sap00.value));

        Ok(())
    }

    #[test]
    fn get_matching() -> Result<()> {
        let (mut map, _, _) = new_network_prefix_map();
        let p = prefix("")?;
        let p0 = prefix("0")?;
        let p1 = prefix("1")?;
        let p10 = prefix("10")?;

        let sap = gen_section_auth(p)?;
        let sap0 = gen_section_auth(p0)?;
        let sap1 = gen_section_auth(p1)?;
        let sap10 = gen_section_auth(p10)?;

        let _changed = map.insert(sap.clone());

        assert_eq!(
            map.section_by_name(&p0.substituted_in(xor_name::rand::random()))?,
            sap.value
        );

        let _changed = map.insert(sap0.clone());

        assert_eq!(
            map.section_by_name(&p1.substituted_in(xor_name::rand::random()))?,
            sap0.value
        );

        let _changed = map.insert(sap1);
        let _changed = map.insert(sap10.clone());

        assert_eq!(
            map.section_by_name(&p0.substituted_in(xor_name::rand::random()))?,
            sap0.value
        );

        // sap1 get pruned once sap10 inserted.
        assert_eq!(
            map.section_by_name(&prefix("11")?.substituted_in(xor_name::rand::random()))?,
            sap10.value
        );

        assert_eq!(
            map.section_by_name(&p10.substituted_in(xor_name::rand::random()))?,
            sap10.value
        );

        Ok(())
    }

    #[test]
    fn get_matching_prefix() -> Result<()> {
        let (mut map, _, _) = new_network_prefix_map();
        let p0 = prefix("0")?;
        let p1 = prefix("1")?;
        let p10 = prefix("10")?;

        let sap0 = gen_section_auth(p0)?;
        let sap1 = gen_section_auth(p1)?;
        let sap10 = gen_section_auth(p10)?;

        let _changed = map.insert(sap0.clone());
        let _changed = map.insert(sap1);
        let _changed = map.insert(sap10.clone());

        assert_eq!(map.section_by_prefix(&p0)?, sap0.value);

        // sap1 get pruned once sap10 inserted.
        assert_eq!(map.section_by_prefix(&prefix("11")?)?, sap10.value);

        assert_eq!(map.section_by_prefix(&p10)?, sap10.value);

        assert_eq!(map.section_by_prefix(&prefix("101")?)?, sap10.value);

        Ok(())
    }

    #[test]
    fn closest() -> Result<()> {
        // Create map containing sections (00), (01) and (10)
        let (mut map, genesis_sk, genesis_pk) = new_network_prefix_map();
        let chain = SecuredLinkedList::new(genesis_pk);
        let p01 = prefix("01")?;
        let p10 = prefix("10")?;
        let p11 = prefix("11")?;

        let mut chain01 = chain.clone();
        let section_auth_01 = gen_section_auth(p01)?;
        let pk01 = section_auth_01.section_key();
        let sig01 = bincode::serialize(&pk01).map(|bytes| genesis_sk.sign(&bytes))?;
        chain01.insert(&genesis_pk, pk01, sig01)?;
        let _updated = map.update(section_auth_01, &chain01);

        let mut chain10 = chain;
        let section_auth_10 = gen_section_auth(p10)?;
        let pk10 = section_auth_10.section_key();
        let sig10 = bincode::serialize(&pk10).map(|bytes| genesis_sk.sign(&bytes))?;
        chain10.insert(&genesis_pk, pk10, sig10)?;
        let _updated = map.update(section_auth_10, &chain10);

        let n01 = p01.substituted_in(xor_name::rand::random());
        let n10 = p10.substituted_in(xor_name::rand::random());
        let n11 = p11.substituted_in(xor_name::rand::random());

        assert_eq!(map.closest(&n01, None).map(|sap| sap.prefix()), Some(p01));
        assert_eq!(map.closest(&n10, None).map(|sap| sap.prefix()), Some(p10));
        assert_eq!(map.closest(&n11, None).map(|sap| sap.prefix()), Some(p10));

        Ok(())
    }

    proptest! {
        #[test]
        fn proptest_prefix_map_fields_should_stay_in_sync((main_chain, update_variations_list) in arb_sll_and_proof_chains(100, 3)) {
            for variation in update_variations_list {
                let mut prefix_map = NetworkPrefixMap::new(*main_chain.root_key());
                for (proof_chain, sap) in &variation {
                    let _ = prefix_map.update(sap.clone(), proof_chain)?;
                }
                let synced = is_synced(&prefix_map).unwrap();
                prop_assert!(synced);
            }
        }
    }

    // Generate an arbitrary sized SecuredLinkedList and a List<list of proof_chains which inserted in
    // that order gives back the main_chain>; i.e., multiple variations of <list of proof_chains>
    fn arb_sll_and_proof_chains(
        max_sections: usize,
        update_variations: usize,
    ) -> impl Strategy<
        Value = (
            SecuredLinkedList,
            Vec<Vec<(SecuredLinkedList, SectionAuth<SectionAuthorityProvider>)>>,
        ),
    > {
        (any::<u64>(), 0..max_sections).prop_map(move |(seed, size)| {
            let (map, main_chain) = generate_random_prefix_map(Some(seed as u64), size).unwrap();
            let mut rng = SmallRng::seed_from_u64(seed);
            let mut update_variations_list = Vec::new();

            for _ in 0..update_variations {
                let mut leaves: Vec<(bls::PublicKey, Prefix)> = map
                    .iter()
                    .filter(|(_, (status, ..))| matches!(status, SectionStatus::None))
                    .map(|(section_key, (.., sap))| (*section_key, sap.prefix()))
                    .collect();
                let mut inserted_keys = BTreeSet::new();
                inserted_keys.insert(*main_chain.root_key());

                let mut list_of_proofs = Vec::new();
                while !leaves.is_empty() {
                    // get proof chain to a random leaf
                    let (random_leaf, _) = *leaves
                        .get(rng.gen::<usize>() % leaves.len())
                        .expect("leaves cannot be empty");
                    let proof_chain = main_chain
                        .get_proof_chain(main_chain.root_key(), &random_leaf)
                        .expect("cannot be error");

                    // create a sub-chain of the proof_chain by first selecting a random_root for the
                    // proof and then selecting a random last_key.
                    let possible_roots: Vec<bls::PublicKey> = proof_chain
                        .keys()
                        .filter(|proof_key| {
                            // keep the proof_key if it has been inserted
                            inserted_keys.iter().any(|ins_key| ins_key == *proof_key)
                        })
                        .cloned()
                        .collect();
                    let random_root = possible_roots
                        .get(rng.gen::<usize>() % possible_roots.len())
                        .expect("Cannot be None");

                    // Consider the prefix_map is aware of the prefix 011 with keys A->B->C
                    // Now, our proof_chain can be [A/B/C to C/D/E/F]
                    // i.e., B->C->D is valid, C->D is valid, C is also valid
                    // but A->B is invalid since we are providing old information to the prefix_map
                    // Since proof_chain is a single branch, we can skip directly to "C" by len - 1
                    let possible_last_keys: Vec<bls::PublicKey> = proof_chain
                        .keys()
                        .skip(possible_roots.len() - 1)
                        .cloned()
                        .collect();
                    let random_last_key = possible_last_keys
                        .get(rng.gen::<usize>() % possible_last_keys.len())
                        .expect("Cannot be None");

                    let proof_chain = proof_chain
                        .get_proof_chain(random_root, random_last_key)
                        .expect("cannot be error");

                    // remove last_key from leaves if it's a leaf
                    if let Some(index) = leaves.iter().position(|(key, _)| *key == *random_last_key)
                    {
                        leaves.swap_remove(index);
                    }
                    // update the inserted list
                    proof_chain.keys().for_each(|key| {
                        inserted_keys.insert(*key);
                    });

                    let (_, _, sap) = map.get(random_last_key).expect("cannot be error").clone();
                    list_of_proofs.push((proof_chain, sap));
                }
                update_variations_list.push(list_of_proofs);
            }
            (main_chain, update_variations_list)
        })
    }

    // Test helpers

    fn prefix(s: &str) -> Result<Prefix> {
        s.parse()
            .map_err(|err| eyre!("failed to parse Prefix '{}': {}", s, err))
    }

    fn gen_section_auth(prefix: Prefix) -> Result<SectionAuth<SectionAuthorityProvider>> {
        let (section_auth, _, secret_key_set) = random_sap(prefix, 5);
        section_signed(secret_key_set.secret_key(), section_auth)
            .context(format!("Failed to generate SAP for prefix {:?}", prefix))
    }

    fn new_network_prefix_map() -> (NetworkPrefixMap, bls::SecretKey, BlsPublicKey) {
        let genesis_sk = bls::SecretKey::random();
        let genesis_pk = genesis_sk.public_key();

        let map = NetworkPrefixMap::new(genesis_pk);

        (map, genesis_sk, genesis_pk)
    }

    // Check if the 'sections' and 'sections_dag' fields are in sync
    // Get the leaves of the 'sections_dag' from 'sections' and verify that the added
    // lengths from the leaves matches the actual length of the 'sections_dag'
    fn is_synced(prefix_map: &NetworkPrefixMap) -> Result<bool> {
        let mut count: BTreeSet<bls::PublicKey> = BTreeSet::new();

        for leaf_sap in prefix_map.all() {
            let chain = prefix_map
                .sections_dag
                .get_proof_chain(prefix_map.sections_dag.root_key(), &leaf_sap.section_key())?;

            for key in chain.keys() {
                count.insert(*key);
            }
        }

        Ok(prefix_map.sections_dag.len() == count.len())
    }

    type SectionInfo = (
        SectionStatus,
        bls::SecretKey,
        SectionAuth<SectionAuthorityProvider>,
    );

    // Generate a random SectionChain and the sap for all the sections
    fn generate_random_prefix_map(
        seed: Option<u64>,
        n_sections: usize,
    ) -> Result<(BTreeMap<bls::PublicKey, SectionInfo>, SecuredLinkedList)> {
        let mut rng = match seed {
            Some(seed) => SmallRng::seed_from_u64(seed),
            None => SmallRng::from_entropy(),
        };
        // map of each section key to its Status, SecretKey and SAP. A seection can have
        // children only if its SectionStatus is None. i.e., without Churn/Split
        let sections_map: Rc<RefCell<BTreeMap<bls::PublicKey, SectionInfo>>> =
            Rc::new(RefCell::new(BTreeMap::new()));

        // genesis section; inserted at the end
        let (gen_sap, _, gen_sk) = random_sap(Prefix::default(), 0);
        let gen_sap_signed = section_signed(gen_sk.secret_key(), gen_sap.clone())?;
        let mut chain = SecuredLinkedList::new(gen_sap.section_key());

        //  if n_sections == 0, return early with just the genesis attached
        if n_sections == 0 {
            sections_map.borrow_mut().insert(
                gen_sap.section_key(),
                (
                    SectionStatus::None,
                    gen_sk.secret_key().clone(),
                    gen_sap_signed,
                ),
            );
            return Ok((sections_map.borrow().clone(), chain));
        }

        // insert a new section
        let mut insert = |prefix: Prefix, parent_sk: &bls::SecretKey| -> Result<()> {
            let (sap, _, sk) = random_sap(prefix, 0);
            let section_key_signed =
                bincode::serialize(&sap.section_key()).map(|bytes| parent_sk.sign(&bytes))?;
            let sap_signed = section_signed(sk.secret_key(), sap.clone())?;

            sections_map.borrow_mut().insert(
                sap.section_key(),
                (SectionStatus::None, sk.secret_key().clone(), sap_signed),
            );
            chain.insert(
                &parent_sk.public_key(),
                sap.section_key(),
                section_key_signed,
            )?;
            Ok(())
        };

        // initial sections
        insert(prefix("0")?, gen_sk.secret_key())?;
        insert(prefix("1")?, gen_sk.secret_key())?;

        // +1 for genesis
        while sections_map.borrow().len() + 1 < n_sections {
            // get a random section only if its SectionStatus is None
            let (_, p_sk, p_sap) = match sections_map
                .borrow()
                .iter()
                .nth(rng.gen::<usize>() % sections_map.borrow().len())
            {
                Some((_, (status, _, _))) if !matches!(status, SectionStatus::None) => continue,
                Some((pk, (_, sk, sap))) => (*pk, sk.clone(), sap.clone()),
                None => continue,
            };

            let switch: i8 = rng.gen_range(0..2);
            if switch % 2 == 0 {
                // Churn event for the parent, so leads to single child with same prefix
                insert(p_sap.prefix(), &p_sk)?;
                // update parent
                assert!(sections_map
                    .borrow_mut()
                    .insert(p_sap.section_key(), (SectionStatus::Churn, p_sk, p_sap))
                    .is_some());
            } else {
                // Split event for the parent, so leads to 2 children
                insert(prefix(format!("{:b}0", p_sap.prefix()).as_str())?, &p_sk)?;
                insert(prefix(format!("{:b}1", p_sap.prefix()).as_str())?, &p_sk)?;
                // update parent
                assert!(sections_map
                    .borrow_mut()
                    .insert(p_sap.section_key(), (SectionStatus::Split, p_sk, p_sap))
                    .is_some());
            };
        }

        // insert genesis
        sections_map.borrow_mut().insert(
            gen_sap.section_key(),
            (
                SectionStatus::Split,
                gen_sk.secret_key().clone(),
                gen_sap_signed,
            ),
        );

        return Ok((sections_map.borrow().clone(), chain));
    }

    // Denotes if a section has gone through Split/Churn
    #[derive(Debug, Clone, Copy)]
    enum SectionStatus {
        Split,
        Churn,
        None,
    }
}
