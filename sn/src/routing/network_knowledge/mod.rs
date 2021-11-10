// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

pub(super) mod node_state;
pub(crate) mod section_authority_provider;
pub(super) mod section_keys;
mod section_peers;

#[cfg(test)]
pub(crate) use self::section_authority_provider::test_utils;

pub(super) use self::section_keys::{SectionKeyShare, SectionKeysProvider};

use crate::messaging::{
    system::{KeyedSig, NodeState, SectionAuth, SectionPeers},
    SectionAuthorityProvider,
};
use crate::prefix_map::NetworkPrefixMap;
use crate::routing::{
    dkg::SectionAuthUtils,
    error::{Error, Result},
    log_markers::LogMarker,
    ElderCandidates, Peer, ELDER_SIZE, RECOMMENDED_SECTION_SIZE,
};
use bls::PublicKey as BlsPublicKey;
pub(crate) use node_state::NodeStateUtils;
use section_authority_provider::SectionAuthorityProviderUtils;
use secured_linked_list::SecuredLinkedList;
use serde::Serialize;
use std::{collections::BTreeSet, convert::TryInto, iter, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use xor_name::{Prefix, XorName};

/// Container for storing information about the network, including our own section.
#[derive(Clone, Debug)]
pub(crate) struct NetworkKnowledge {
    /// Network genesis key
    genesis_key: BlsPublicKey,
    /// The secured linked list of previous section keys, starting from genesis key
    chain: Arc<RwLock<SecuredLinkedList>>,
    /// Signed Section Authority Provider
    signed_sap: Arc<RwLock<SectionAuth<SectionAuthorityProvider>>>,
    /// Members of our section
    section_peers: SectionPeers,
    /// The network prefix map, i.e. a map from prefix to SAPs
    prefix_map: NetworkPrefixMap,
}

impl NetworkKnowledge {
    /// Creates a minimal `NetworkKnowledge` initially containing only info about our elders
    /// (`SAP`).
    ///
    /// Returns error if the `signed_sap` is not verifiable with the `chain`.
    pub(super) fn new(
        genesis_key: bls::PublicKey,
        chain: SecuredLinkedList,
        signed_sap: SectionAuth<SectionAuthorityProvider>,
        passed_prefix_map: Option<NetworkPrefixMap>,
    ) -> Result<Self, Error> {
        // Let's check the section chain's genesis key matches ours.
        if genesis_key != *chain.root_key() {
            return Err(Error::UntrustedProofChain(format!(
                "genesis key doesn't match first key in proof chain: {:?}",
                chain.root_key()
            )));
        }

        // Check the SAP's key is the last key of the section chain
        if signed_sap.sig.public_key != *chain.last_key() {
            error!("can't create section: SAP signed with incorrect key");
            return Err(Error::UntrustedSectionAuthProvider(format!(
                "section key doesn't match last key in proof chain: {:?}",
                signed_sap.value
            )));
        }

        // Check if SAP signature is valid
        if !signed_sap.self_verify() {
            return Err(Error::UntrustedSectionAuthProvider(format!(
                "invalid signature: {:?}",
                signed_sap.value
            )));
        }

        // Check if SAP's section key matches SAP signature's key
        if signed_sap.sig.public_key != signed_sap.public_key_set.public_key() {
            return Err(Error::UntrustedSectionAuthProvider(format!(
                "section key doesn't match signature's key: {:?}",
                signed_sap.value
            )));
        }

        // Make sure the section chain can be trusted, i.e. check that
        // each key is signed by its parent/predecesor key.
        if !chain.self_verify() {
            return Err(Error::UntrustedProofChain(format!(
                "invalid chain: {:?}",
                chain
            )));
        }

        // Check if the genesis key in the provided prefix_map matches ours.
        // If no prefix map was provided, start afresh.
        let prefix_map = match passed_prefix_map {
            Some(prefix_map) => {
                if prefix_map.genesis_key() != genesis_key {
                    return Err(Error::InvalidGenesisKey(prefix_map.genesis_key()));
                } else {
                    prefix_map
                }
            }
            None => NetworkPrefixMap::new(genesis_key),
        };

        // At this point we know the prefix map corresponds to the correct genesis key,
        // let's make sure the prefix map contains also our own pefix and SAP,
        if let Err(err) = prefix_map.update(signed_sap.clone(), &chain) {
            debug!("Failed to update NetworkPrefixMap with SAP {:?} and chain {:?} upon creating new NetworkKnowledge intance: {:?}", signed_sap, chain, err);
        }

        Ok(Self {
            genesis_key,
            chain: Arc::new(RwLock::new(chain)),
            signed_sap: Arc::new(RwLock::new(signed_sap)),
            section_peers: SectionPeers::default(),
            prefix_map,
        })
    }

    /// update all section info for our new section
    pub(super) async fn relocated_to(&self, new_network_nowledge: Self) -> Result<()> {
        debug!("Node was relocated to {:?}", new_network_nowledge);

        let mut chain = self.chain.write().await;
        *chain = new_network_nowledge.chain().await;
        // don't hold write lock
        drop(chain);

        let mut signed_sap = self.signed_sap.write().await;
        *signed_sap = new_network_nowledge.signed_sap.read().await.clone();
        // don't hold write lock
        drop(signed_sap);

        let _updated = self
            .merge_members(new_network_nowledge.members().clone())
            .await?;

        Ok(())
    }

    /// Creates `NetworkKnowledge` for the first node in the network
    pub(super) async fn first_node(
        peer: Peer,
        genesis_sk_set: bls::SecretKeySet,
    ) -> Result<(NetworkKnowledge, SectionKeyShare)> {
        let public_key_set = genesis_sk_set.public_keys();
        let secret_key_share = genesis_sk_set.secret_key_share(0);
        let genesis_key = public_key_set.public_key();

        let section_auth =
            create_first_section_authority_provider(&public_key_set, &secret_key_share, peer)?;

        let network_knowledge = NetworkKnowledge::new(
            genesis_key,
            SecuredLinkedList::new(genesis_key),
            section_auth,
            None,
        )?;

        for peer in network_knowledge.signed_sap.read().await.peers() {
            let node_state = NodeState::joined(peer, None);
            let sig = create_first_sig(&public_key_set, &secret_key_share, &node_state)?;
            let _changed = network_knowledge.section_peers.update(SectionAuth {
                value: node_state,
                sig,
            });
        }

        let section_key_share = SectionKeyShare {
            public_key_set,
            index: 0,
            secret_key_share,
        };

        Ok((network_knowledge, section_key_share))
    }

    pub(super) async fn update_knowledge_if_valid(
        &self,
        signed_sap: SectionAuth<SectionAuthorityProvider>,
        proof_chain: &SecuredLinkedList,
        updated_members: Option<SectionPeers>,
        our_name: &XorName,
        update_sap: bool,
    ) -> Result<bool> {
        let provided_sap = signed_sap.value.clone();

        // TODO: once we have a proper DAG implementation we won't
        // need proof chains from genesis but from any other key in the chain
        if proof_chain.root_key() != self.genesis_key() {
            println!(
                "Proof chain's first key is not the genesis key ({:?}): {:?}",
                self.genesis_key(),
                proof_chain.root_key()
            );

            return Ok(false);
        }

        let mut there_was_an_update = false;

        // 1. Update the network prefix map
        match self.prefix_map.verify_with_chain_and_update(
            signed_sap.clone(),
            proof_chain,
            &self.chain().await,
        ) {
            Ok(true) => {
                there_was_an_update = true;
                debug!(
                    "Anti-Entropy: updated network prefix map with SAP for {:?}",
                    provided_sap.prefix
                );

                // We try to update our SAP and own chain only if we were flagged to,
                // otherwise this update could be due to an AE message and we still don't have
                // the key share for the new SAP, making this node unable to sign section messages
                // and possibly being kicked out of the group of Elders.
                if update_sap && provided_sap.prefix.matches(our_name) {
                    let our_prev_prefix = self.prefix().await;
                    *self.signed_sap.write().await = signed_sap.clone();
                    *self.chain.write().await = proof_chain.clone();

                    // Remove any peer which doesn't belong to our new section's prefix
                    self.section_peers.retain(&provided_sap.prefix);
                    info!(
                        "Updated our section's SAP ({:?} to {:?}) with new one: {:?}",
                        our_prev_prefix, provided_sap.prefix, provided_sap
                    );
                }
            }
            Ok(false) => {
                debug!(
                    "Anti-Entropy: discarded SAP for {:?} since it's the same as the one in our records: {:?}",
                    provided_sap.prefix, provided_sap
                );
            }
            Err(err) => {
                debug!(
                    "Anti-Entropy: discarded SAP for {:?} since we failed to update prefix map with: {:?}",
                    provided_sap.prefix, err
                );
            }
        }

        // TODO:
        // 2. Update the chains DAG with provided knowledge so we have all sections chains,
        // inlcuding our own, to be able to provide proof chains when required, e.g. in AE responses.

        // 3. Update members if changes were provided
        if let Some(peers) = updated_members {
            if self.merge_members(peers).await? {
                info!(
                    "Updated our section's members ({:?}): {:?}",
                    self.prefix().await,
                    self.members()
                );
                there_was_an_update = true;
            }
        }

        Ok(there_was_an_update)
    }

    // Returns reference to network prefix map
    pub(crate) fn prefix_map(&self) -> &NetworkPrefixMap {
        &self.prefix_map
    }

    // Returns the section authority provider for the prefix that matches name, excluding self section.
    pub(super) fn section_by_name(&self, name: &XorName) -> Result<SectionAuthorityProvider> {
        self.prefix_map.section_by_name(name)
    }

    // Returns the section authority provider for the prefix that matches name, excluding self section.
    pub(super) fn section_by_prefix(&self, prefix: &Prefix) -> Result<SectionAuthorityProvider> {
        self.prefix_map.section_by_prefix(prefix)
    }

    // Get SectionAuthorityProvider of a known section with the given prefix.
    pub(super) fn get_sap(&self, prefix: &Prefix) -> Option<SectionAuthorityProvider> {
        self.prefix_map.get(prefix)
    }

    // Get SectionAuthorityProvider of a known section with the given prefix.
    pub(super) async fn get_closest_or_opposite_signed_sap(
        &self,
        name: &XorName,
    ) -> Option<SectionAuth<SectionAuthorityProvider>> {
        self.prefix_map
            .closest_or_opposite(name, Some(&self.prefix().await))
    }

    pub(super) fn genesis_key(&self) -> &bls::PublicKey {
        &self.genesis_key
    }

    // Try to merge this `NetworkKnowledge` members with `peers`. .
    async fn merge_members(&self, peers: SectionPeers) -> Result<bool> {
        let mut there_was_an_update = false;
        let chain = self.chain.read().await.clone();

        for refmulti in peers.members.iter() {
            let node_state = refmulti.value().clone();
            if !node_state.verify(&chain) {
                error!("can't merge member {:?}", node_state.value);
            } else if self.section_peers.update(node_state) {
                there_was_an_update = true;
            }
        }

        self.section_peers.retain(&self.prefix().await);

        Ok(there_was_an_update)
    }

    /// Update the member. Returns whether it actually updated it.
    pub(super) async fn update_member(&self, node_state: SectionAuth<NodeState>) -> bool {
        // let's check the node state is properly signed by one of the keys in our chain
        if !node_state.verify(&*self.chain.read().await) {
            error!("can't merge member {:?}", node_state.value);
            return false;
        }

        self.section_peers.update(node_state)
    }

    /// Return a copy of the section chain
    pub(super) async fn chain(&self) -> SecuredLinkedList {
        self.chain.read().await.clone()
    }

    /// Return current section key
    pub(super) async fn section_key(&self) -> bls::PublicKey {
        self.signed_sap.read().await.section_key()
    }

    /// Return current section chain length
    pub(crate) async fn chain_len(&self) -> u64 {
        self.chain.read().await.main_branch_len() as u64
    }

    /// Return weather current section chain has the provided key
    pub(crate) async fn has_chain_key(&self, key: &bls::PublicKey) -> bool {
        self.chain.read().await.has_key(key)
    }

    /// Return a copy of current SAP
    pub(super) async fn authority_provider(&self) -> SectionAuthorityProvider {
        self.signed_sap.read().await.value.clone()
    }

    /// Return a copy of current SAP with corresponding section authority
    pub(super) async fn section_signed_authority_provider(
        &self,
    ) -> SectionAuth<SectionAuthorityProvider> {
        self.signed_sap.read().await.clone()
    }

    /// Return weather the name provided belongs to an Elder, by checking if
    /// it is one of the current section's SAP member,
    pub(super) async fn is_elder(&self, name: &XorName) -> bool {
        self.signed_sap.read().await.contains_elder(name)
    }

    /// Generate a new section info(s) based on the current set of members,
    /// excluding any member matching a name in the provided `excluded_names` set.
    /// Returns a set of candidate SectionAuthorityProviders.
    pub(super) async fn promote_and_demote_elders(
        &self,
        our_name: &XorName,
        excluded_names: &BTreeSet<XorName>,
    ) -> Vec<ElderCandidates> {
        if let Some((our_elder_candidates, other_elder_candidates)) =
            self.try_split(our_name, excluded_names).await
        {
            return vec![our_elder_candidates, other_elder_candidates];
        }

        // Candidates for elders out of all the nodes in the section, even out of the
        // relocating nodes if there would not be enough instead.
        let sap = self.authority_provider().await;
        let expected_peers = self
            .section_peers
            .elder_candidates(ELDER_SIZE, &sap, excluded_names);

        let expected_names: BTreeSet<_> = expected_peers.iter().map(Peer::name).collect();
        let current_names: BTreeSet<_> = sap.names();

        if expected_names == current_names {
            vec![]
        } else if expected_names.len() < crate::routing::supermajority(current_names.len()) {
            warn!("ignore attempt to reduce the number of elders too much");
            vec![]
        } else {
            let elder_candidates = ElderCandidates::from_peers(sap.prefix(), expected_peers);
            vec![elder_candidates]
        }
    }

    /// Prefix of our section.
    pub(super) async fn prefix(&self) -> Prefix {
        self.signed_sap.read().await.prefix
    }

    /// Return the list of our section's members
    pub(super) fn members(&self) -> &SectionPeers {
        &self.section_peers
    }

    /// Returns members that are either joined or are left but still elders.
    pub(super) async fn active_members(&self) -> Vec<Peer> {
        let mut active_members = vec![];
        let nodes = self.section_peers.all_members();
        for peer in nodes {
            if self.section_peers.is_joined(&peer.name()) || self.is_elder(&peer.name()).await {
                active_members.push(peer);
            }
        }

        active_members
    }

    /// Returns adults from our section.
    pub(super) async fn adults(&self) -> Vec<Peer> {
        let mut adults = vec![];
        let nodes = self.section_peers.mature();
        for peer in nodes {
            if !self.is_elder(&peer.name()).await {
                adults.push(peer);
            }
        }

        adults
    }

    /// Returns live adults from our section.
    pub(super) async fn live_adults(&self) -> Vec<Peer> {
        let mut live_adults = vec![];

        for node_state in self.section_peers.joined() {
            if !self.is_elder(&node_state.name).await {
                live_adults.push(node_state.to_peer())
            }
        }
        live_adults
    }

    pub(super) fn find_joined_member_by_addr(&self, addr: &SocketAddr) -> Option<Peer> {
        self.section_peers
            .joined()
            .into_iter()
            .find(|info| &info.addr == addr)
            .map(|info| info.to_peer())
    }

    // Tries to split our section.
    // If we have enough mature nodes for both subsections, returns the SectionAuthorityProviders
    // of the two subsections. Otherwise returns `None`.
    async fn try_split(
        &self,
        our_name: &XorName,
        excluded_names: &BTreeSet<XorName>,
    ) -> Option<(ElderCandidates, ElderCandidates)> {
        trace!("{}", LogMarker::SplitAttempt);
        if self.authority_provider().await.elders.len() < ELDER_SIZE {
            trace!("No attempt to split as our section does not have enough elders.");
            return None;
        }

        let next_bit_index = if let Ok(index) = self.prefix().await.bit_count().try_into() {
            index
        } else {
            debug!("at longest prefix");
            // Already at the longest prefix, can't split further.
            return None;
        };

        let next_bit = our_name.bit(next_bit_index);

        let (our_new_size, sibling_new_size) = self
            .section_peers
            .mature()
            .iter()
            .filter(|peer| !excluded_names.contains(&peer.name()))
            .map(|peer| peer.name().bit(next_bit_index) == next_bit)
            .fold((0, 0), |(ours, siblings), is_our_prefix| {
                if is_our_prefix {
                    (ours + 1, siblings)
                } else {
                    (ours, siblings + 1)
                }
            });

        debug!(
            ">>>> our size {:?}, theirs {:?}",
            our_new_size, sibling_new_size
        );

        // If none of the two new sections would contain enough entries, return `None`.
        if our_new_size < RECOMMENDED_SECTION_SIZE || sibling_new_size < RECOMMENDED_SECTION_SIZE {
            debug!(">>>> returning here TOO SMALLLLLLLLL hmmmmmm");
            return None;
        }

        let our_prefix = self.prefix().await.pushed(next_bit);
        let other_prefix = self.prefix().await.pushed(!next_bit);

        let our_elders = self.section_peers.elder_candidates_matching_prefix(
            &our_prefix,
            ELDER_SIZE,
            &self.authority_provider().await,
            excluded_names,
        );
        let other_elders = self.section_peers.elder_candidates_matching_prefix(
            &other_prefix,
            ELDER_SIZE,
            &self.authority_provider().await,
            excluded_names,
        );

        let our_elder_candidates = ElderCandidates::from_peers(our_prefix, our_elders);
        let other_elder_candidates = ElderCandidates::from_peers(other_prefix, other_elders);

        debug!(">>>>> end of split attempt");
        Some((our_elder_candidates, other_elder_candidates))
    }
}

// Create `SectionAuthorityProvider` for the first node.
fn create_first_section_authority_provider(
    pk_set: &bls::PublicKeySet,
    sk_share: &bls::SecretKeyShare,
    peer: Peer,
) -> Result<SectionAuth<SectionAuthorityProvider>> {
    let section_auth =
        SectionAuthorityProvider::new(iter::once(peer), Prefix::default(), pk_set.clone());
    let sig = create_first_sig(pk_set, sk_share, &section_auth)?;
    Ok(SectionAuth::new(section_auth, sig))
}

fn create_first_sig<T: Serialize>(
    pk_set: &bls::PublicKeySet,
    sk_share: &bls::SecretKeyShare,
    payload: &T,
) -> Result<KeyedSig> {
    let bytes = bincode::serialize(payload).map_err(|_| Error::InvalidPayload)?;
    let signature_share = sk_share.sign(&bytes);
    let signature = pk_set
        .combine_signatures(iter::once((0, &signature_share)))
        .map_err(|_| Error::InvalidSignatureShare)?;

    Ok(KeyedSig {
        public_key: pk_set.public_key(),
        signature,
    })
}
