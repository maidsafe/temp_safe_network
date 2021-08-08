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
    node::{NetworkDto, Peer, SectionAuth},
    SectionAuthorityProvider,
};
use async_trait::async_trait;
use secured_linked_list::SecuredLinkedList;
use std::iter;
use std::sync::Arc;
use tokio::sync::RwLock;
use xor_name::{Prefix, PrefixMap, XorName};

#[async_trait]
pub(super) trait NetworkLogic {
    fn new() -> Self;

    async fn insert(&self, prefix: Prefix, sap: SectionAuth<SectionAuthorityProvider>) -> bool;

    async fn closest(&self, name: &XorName) -> Option<SectionAuth<SectionAuthorityProvider>>;

    /// Returns iterator over all known sections.
    async fn all(&self) -> Vec<SectionAuthorityProvider>;

    /// Get `SectionAuthorityProvider` of a known section with the given prefix.
    async fn get(&self, prefix: &Prefix) -> Option<SectionAuthorityProvider>;

    /// Returns a `Peer` of an elder from a known section.
    async fn get_elder(&self, name: &XorName) -> Option<Peer>;

    /// Merge two `Network`s into one.
    /// TODO: make this operation commutative, associative and idempotent (CRDT)
    /// TODO: return bool indicating whether anything changed.
    async fn merge(&self, other: NetworkDto, section_chain: SecuredLinkedList);

    /// Update our knowledge of a remote section's SAP only
    /// if it's verifiable with the provided proof chain.
    async fn update_remote_section_sap(
        &self,
        signed_section_auth: SectionAuth<SectionAuthorityProvider>,
        proof_chain: &SecuredLinkedList,
        our_section_chain: &SecuredLinkedList,
    ) -> Result<bool>;

    /// Returns the known section keys.
    async fn keys(&self) -> Vec<(Prefix, bls::PublicKey)>;

    /// Returns the latest known key for the prefix that matches `name`.
    async fn key_by_name(&self, name: &XorName) -> Result<bls::PublicKey>;

    /// Returns the section authority provider for the prefix that matches `name`,
    /// excluding self section.
    async fn section_by_name(&self, name: &XorName) -> Result<SectionAuthorityProvider>;

    /// Returns network statistics.
    async fn network_stats(&self, our: &SectionAuthorityProvider) -> NetworkStats;

    /// Returns a dto, for sending on the wire
    async fn as_dto(&self) -> NetworkDto;

    /// Clears the state
    async fn clear(&self);
}

/// Container for storing information about other sections in the network.
#[derive(Debug, Clone)]
pub(crate) struct Network {
    /// Other sections: maps section prefixes to their latest signed section authority providers.
    sections: Arc<RwLock<PrefixMap<SectionAuth<SectionAuthorityProvider>>>>,
}

impl From<NetworkDto> for Network {
    fn from(dto: NetworkDto) -> Self {
        Self {
            sections: Arc::new(RwLock::new(dto.sections)),
        }
    }
}

#[async_trait]
impl NetworkLogic for Network {
    ///
    fn new() -> Self {
        Self {
            sections: Arc::new(RwLock::new(PrefixMap::new())),
        }
    }

    /// Returns a dto, for sending on the wire
    async fn as_dto(&self) -> NetworkDto {
        NetworkDto {
            sections: self.sections.read().await.clone(),
        }
    }

    /// Clears the state
    async fn clear(&self) {
        *self.sections.write().await = PrefixMap::new();
    }

    /// Inserts sap
    async fn insert(&self, prefix: Prefix, sap: SectionAuth<SectionAuthorityProvider>) -> bool {
        self.sections.write().await.insert(prefix, sap)
    }

    /// Returns the known section that is closest to the given name, regardless of whether `name`
    /// belongs in that section or not.
    async fn closest(&self, name: &XorName) -> Option<SectionAuth<SectionAuthorityProvider>> {
        self.sections
            .read()
            .await
            .iter()
            .min_by(|(lhs_prefix, _), (rhs_prefix, _)| lhs_prefix.cmp_distance(rhs_prefix, name))
            .map(|(_, sap)| sap.clone())
    }

    /// Returns iterator over all known sections.
    async fn all(&self) -> Vec<SectionAuthorityProvider> {
        self.sections
            .read()
            .await
            .iter()
            .map(|(_, section_auth)| section_auth.value.clone())
            .collect()
    }

    /// Get `SectionAuthorityProvider` of a known section with the given prefix.
    async fn get(&self, prefix: &Prefix) -> Option<SectionAuthorityProvider> {
        self.sections
            .read()
            .await
            .get(prefix)
            .map(|(_, section_auth)| section_auth.value.clone())
    }

    /// Returns a `Peer` of an elder from a known section.
    async fn get_elder(&self, name: &XorName) -> Option<Peer> {
        self.sections
            .read()
            .await
            .get_matching(name)?
            .1
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
    async fn merge(&self, other: NetworkDto, section_chain: SecuredLinkedList) {
        // FIXME: these operations are not commutative:

        for (prefix, sap) in other.sections.iter() {
            if sap.verify(&section_chain) {
                let _ = self.sections.write().await.insert(*prefix, sap.clone());
            }
        }
    }

    /// Update our knowledge of a remote section's SAP only
    /// if it's verifiable with the provided proof chain.
    async fn update_remote_section_sap(
        &self,
        signed_section_auth: SectionAuth<SectionAuthorityProvider>,
        proof_chain: &SecuredLinkedList,
        our_section_chain: &SecuredLinkedList,
    ) -> Result<bool> {
        // Check if SAP signature is valid
        if !signed_section_auth.self_verify() {
            return Err(Error::UntrustedSectionAuthProvider(format!(
                "invalid signature: {:?}",
                signed_section_auth.value
            )));
        }

        // Check if SAP's section key matches SAP signature's key
        if signed_section_auth.sig.public_key
            != signed_section_auth.value.public_key_set.public_key()
        {
            return Err(Error::UntrustedSectionAuthProvider(format!(
                "section key doesn't match signature's key: {:?}",
                signed_section_auth.value
            )));
        }

        // We currently don't keep the complete chain of remote sections,
        // **but** the SAPs of remote sections we keep were already verified by us
        // as trusted before we store them in our local records.
        // Thus, we just need to check our knowledge of the remote section's key
        // is part of the proof chain received.
        let prefix = signed_section_auth.value.prefix;
        match self.sections.read().await.get(&prefix) {
            Some((_, sap)) if sap == &signed_section_auth => {
                // It's the same SAP we are already aware of
                return Ok(false);
            }
            Some((_, sap)) => {
                // We are then aware of the prefix, let's just verify the new SAP can
                // be trusted based on the SAP we aware of and the proof chain provided.
                if !proof_chain.has_key(&sap.value.public_key_set.public_key()) {
                    return Err(Error::UntrustedProofChain(format!(
                        "none of the keys match the SAP's key we currently know: {:?}",
                        signed_section_auth.value
                    )));
                }
            }
            None => {
                // We are not aware of the prefix, let's then verify it can be
                // trusted based on our own section chain and the provided proof chain.
                if !proof_chain.check_trust(our_section_chain.keys()) {
                    return Err(Error::UntrustedProofChain(format!(
                        "none of the keys were found on our section chain: {:?}",
                        signed_section_auth.value
                    )));
                }
            }
        }

        // Make sure the proof chain can be trusted,
        // i.e. check each key is signed by its parent/predecesor key.
        if !proof_chain.self_verify() {
            return Err(Error::UntrustedProofChain(format!(
                "invalid proof chain: {:?}",
                proof_chain
            )));
        }

        // Check the SAP's key is the last key of the proof chain
        if proof_chain.last_key() != &signed_section_auth.value.public_key_set.public_key() {
            return Err(Error::UntrustedSectionAuthProvider(format!(
                "section key ({:?}) doesn't match proof chain last key ({:?})",
                signed_section_auth.value.public_key_set.public_key(),
                proof_chain.last_key()
            )));
        }

        // We can now update our knowledge of the remote section's SAP.
        // Note: we don't expect the same SAP to be found in our records
        // for the prefix since we've already checked that above.
        let _ = self
            .sections
            .write()
            .await
            .insert(prefix, signed_section_auth);

        Ok(true)
    }

    /// Returns the known section keys.
    async fn keys(&self) -> Vec<(Prefix, bls::PublicKey)> {
        self.sections
            .read()
            .await
            .iter()
            .map(|(_, section_auth)| (section_auth.value.prefix, section_auth.value.section_key()))
            .collect()
    }

    /// Returns the latest known key for the prefix that matches `name`.
    async fn key_by_name(&self, name: &XorName) -> Result<bls::PublicKey> {
        self.sections
            .read()
            .await
            .get_matching(name)
            .ok_or(Error::NoMatchingSection)
            .map(|(_, section_auth)| section_auth.value.section_key())
    }

    /// Returns the section authority provider for the prefix that matches `name`,
    /// excluding self section.
    async fn section_by_name(&self, name: &XorName) -> Result<SectionAuthorityProvider> {
        self.sections
            .read()
            .await
            .get_matching(name)
            .ok_or(Error::NoMatchingSection)
            .map(|(_, section_auth)| section_auth.value.clone())
    }

    /// Returns network statistics.
    async fn network_stats(&self, our: &SectionAuthorityProvider) -> NetworkStats {
        // Let's compute an estimate of the total number of elders in the network
        // from the size of our routing table.
        let sections = self.sections.read().await;
        let known_prefixes = sections
            .iter()
            .map(|(prefix, _)| prefix)
            .chain(iter::once(&our.prefix));
        let total_elders_exact = Prefix::default().is_covered_by(known_prefixes.clone());

        // Estimated fraction of the network that we have in our RT.
        // Computed as the sum of 1 / 2^(prefix.bit_count) for all known section prefixes.
        let network_fraction: f64 = known_prefixes
            .map(|p| 1.0 / (p.bit_count() as f64).exp2())
            .sum();

        let network_elders_count: usize = self
            .sections
            .read()
            .await
            .iter()
            .map(|(_, info)| info.value.elder_count())
            .sum();
        let known = our.elder_count() + network_elders_count;
        let total = known as f64 / network_fraction;

        // `total_elders_exact` indicates whether `total_elders` is
        // an exact number or an estimate.
        NetworkStats {
            known_elders: known as u64,
            total_elders: total.ceil() as u64,
            total_elders_exact,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routing::{dkg, section};
    use eyre::{Context, Result};
    use rand::Rng;

    #[tokio::test]
    async fn closest() -> Result<()> {
        let genesis_sk = bls::SecretKey::random();
        let genesis_pk = genesis_sk.public_key();

        let chain = SecuredLinkedList::new(genesis_pk);
        let p01: Prefix = "01".parse().unwrap();
        let p10: Prefix = "10".parse().unwrap();
        let p11: Prefix = "11".parse().unwrap();

        // Create map containing sections (00), (01) and (10)
        let map = Network::new();

        let mut chain01 = chain.clone();
        let section_auth_01 = gen_section_auth(p01)?;
        let pk01 = section_auth_01.value.public_key_set.public_key();
        let sig01 = bincode::serialize(&pk01).map(|bytes| genesis_sk.sign(&bytes))?;
        chain01.insert(&genesis_pk, pk01, sig01)?;
        let _ = map
            .update_remote_section_sap(section_auth_01, &chain01, &chain)
            .await;

        let mut chain10 = chain.clone();
        let section_auth_10 = gen_section_auth(p10)?;
        let pk10 = section_auth_10.value.public_key_set.public_key();
        let sig10 = bincode::serialize(&pk10).map(|bytes| genesis_sk.sign(&bytes))?;
        chain10.insert(&genesis_pk, pk10, sig10)?;
        let _ = map
            .update_remote_section_sap(section_auth_10, &chain10, &chain)
            .await;

        let mut rng = rand::thread_rng();
        let n01 = p01.substituted_in(rng.gen());
        let n10 = p10.substituted_in(rng.gen());
        let n11 = p11.substituted_in(rng.gen());

        assert_eq!(map.closest(&n01).await.map(|i| i.value.prefix), Some(p01));
        assert_eq!(map.closest(&n10).await.map(|i| i.value.prefix), Some(p10));
        assert_eq!(map.closest(&n11).await.map(|i| i.value.prefix), Some(p10));

        Ok(())
    }

    fn gen_section_auth(prefix: Prefix) -> Result<SectionAuth<SectionAuthorityProvider>> {
        let (section_auth, _, secret_key_set) =
            section::test_utils::gen_section_authority_provider(prefix, 5);
        dkg::test_utils::section_signed(secret_key_set.secret_key(), section_auth)
            .context("Failed to generate SAP")
    }
}
