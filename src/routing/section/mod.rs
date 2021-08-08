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

pub(super) use self::section_keys::{SectionKeyShare, SectionKeysProvider, Signer};
use self::section_peers::SectionPeers;

use super::dkg::SectionDkgOutcome;
use crate::routing::{
    dkg::SectionAuthUtils,
    error::{Error, Result},
    peer::PeerUtils,
    ELDER_SIZE, RECOMMENDED_SECTION_SIZE,
};
use crate::{
    messaging::{
        node::{ElderCandidates, KeyedSig, NodeState, Peer, SectionAuth, SectionDto},
        SectionAuthorityProvider,
    },
    types::CFValue,
};
use async_trait::async_trait;
pub(crate) use node_state::NodeStateUtils;
pub(crate) use section_authority_provider::ElderCandidatesUtils;
use section_authority_provider::SectionAuthorityProviderUtils;
pub(super) use section_peers::SectionPeersLogic;
use secured_linked_list::{error::Error as SecuredLinkedListError, SecuredLinkedList};
use serde::Serialize;
use std::{
    cmp::Ordering, collections::BTreeSet, convert::TryInto, iter, marker::Sized, net::SocketAddr,
};
use tokio::sync::RwLock;
use xor_name::{Prefix, XorName};

#[async_trait]
pub(super) trait SectionLogic {
    /// Creates a minimal `Section` initially containing only info about our elders
    /// (`section_auth`).
    ///
    /// Returns error if `section_auth` is not signed with the last key of `chain`.
    fn new(
        genesis_key: bls::PublicKey,
        chain: SecuredLinkedList,
        section_auth: SectionAuth<SectionAuthorityProvider>,
    ) -> Result<Self, Error>
    where
        Self: Sized;

    /// Creates `Section` for the first node in the network
    async fn first_node(peer: Peer) -> Result<(Section, SectionDkgOutcome)>;

    fn genesis_key(&self) -> &bls::PublicKey;

    /// Try to merge this `Section` with `other`. Returns `InvalidMessage` if `other` is invalid or
    /// its chain is not compatible with the chain of `self`.
    async fn merge(&self, other: SectionDto) -> Result<()>;

    /// Update the `SectionAuthorityProvider` of our section.
    async fn update_elders(
        &self,
        new_section_auth: SectionAuth<SectionAuthorityProvider>,
        new_key_sig: KeyedSig,
    ) -> bool;

    /// Update the member. Returns whether it actually changed anything.
    async fn update_member(&self, node_state: SectionAuth<NodeState>) -> bool;

    async fn last_key(&self) -> bls::PublicKey;
    async fn prev_key(&self) -> bls::PublicKey;
    async fn root_key(&self) -> bls::PublicKey;
    async fn has_key(&self, section_key: &bls::PublicKey) -> bool;
    async fn main_branch_len(&self) -> usize;
    async fn cmp_by_position(&self, lhs: &bls::PublicKey, rhs: &bls::PublicKey) -> Ordering;
    async fn get_proof_chain_to_current(
        &self,
        from_key: &bls::PublicKey,
    ) -> Result<SecuredLinkedList>;
    async fn check_trust<'a, I>(&self, trusted_keys: I) -> bool
    where
        I: Send + IntoIterator<Item = &'a bls::PublicKey>;
    async fn keys(&self) -> Box<dyn DoubleEndedIterator<Item = bls::PublicKey>>;
    async fn chain_clone(&self) -> SecuredLinkedList;

    // Extend the section chain so it starts at `trusted_key` while keeping the last key intact.
    async fn extend_chain(
        &self,
        trusted_key: &bls::PublicKey,
        full_chain: &SecuredLinkedList,
    ) -> Result<SectionDto, SecuredLinkedListError>;

    async fn authority_provider(&self) -> SectionAuthorityProvider;

    async fn section_signed_authority_provider(&self) -> SectionAuth<SectionAuthorityProvider>;

    fn is_elder(&self, name: &XorName) -> bool;

    /// Generate a new section info(s) based on the current set of members.
    /// Returns a set of candidate SectionAuthorityProviders.
    async fn promote_and_demote_elders(&self, our_name: &XorName) -> Vec<ElderCandidates>;

    // Prefix of our section.
    async fn prefix(&self) -> Prefix;

    fn members(&self) -> &SectionPeers;

    /// Returns members that are either joined or are left but still elders.
    async fn active_members(&self) -> Box<dyn Iterator<Item = Peer> + '_>;

    /// Returns adults from our section.
    async fn adults(&self) -> Box<dyn Iterator<Item = Peer> + '_>;

    /// Returns live adults from our section.
    async fn live_adults(&self) -> Box<dyn Iterator<Item = Peer> + '_>;

    async fn find_joined_member_by_addr(&self, addr: &SocketAddr) -> Option<Peer>;

    // Tries to split our section.
    // If we have enough mature nodes for both subsections, returns the SectionAuthorityProviders
    // of the two subsections. Otherwise returns `None`.
    async fn try_split(&self, our_name: &XorName) -> Option<(ElderCandidates, ElderCandidates)>;

    // Returns the candidates for elders out of all the nodes in the section, even out of the
    // relocating nodes if there would not be enough instead.
    async fn elder_candidates(&self, elder_size: usize) -> Vec<Peer>;

    //
    async fn clone(&self) -> SectionDto;
}

/// Container for storing information about a section.
#[derive(Debug)]
/// All information about a section
pub(crate) struct Section {
    /// network genesis key
    pub(crate) genesis_key: bls::PublicKey,
    /// The secured linked list of previous section keys
    pub(crate) chain: RwLock<SecuredLinkedList>,
    /// Signed section authority
    pub(crate) section_auth: CFValue<SectionAuth<SectionAuthorityProvider>>,
    /// memebers of the section
    pub(crate) members: SectionPeers,
}

impl From<SectionDto> for Section {
    fn from(section: SectionDto) -> Self {
        let SectionDto {
            genesis_key,
            chain,
            section_auth,
            members,
        } = section;
        Section {
            genesis_key,
            chain: RwLock::new(chain),
            section_auth: CFValue::new(section_auth),
            members: SectionPeers::from(members),
        }
    }
}

#[async_trait]
impl SectionLogic for Section {
    /// rename to "to_wire"?
    async fn clone(&self) -> SectionDto {
        SectionDto {
            genesis_key: self.genesis_key,
            chain: self.chain_clone().await,
            section_auth: self.section_auth.clone().await,
            members: self.members.clone().await,
        }
    }

    /// Creates a minimal `Section` initially containing only info about our elders
    /// (`section_auth`).
    ///
    /// Returns error if `section_auth` is not signed with the last key of `chain`.
    fn new(
        genesis_key: bls::PublicKey,
        chain: SecuredLinkedList,
        section_auth: SectionAuth<SectionAuthorityProvider>,
    ) -> Result<Self, Error> {
        if section_auth.sig.public_key != *chain.last_key() {
            error!("can't create section: section_auth signed with incorrect key");
            // TODO: consider more specific error here.
            return Err(Error::InvalidMessage);
        }

        Ok(Self {
            genesis_key,
            chain: RwLock::new(chain),
            section_auth: CFValue::new(section_auth),
            members: SectionPeers::new(),
        })
    }

    /// Creates `Section` for the first node in the network
    async fn first_node(peer: Peer) -> Result<(Section, SectionDkgOutcome)> {
        let secret_key_set = bls::SecretKeySet::random(0, &mut rand::thread_rng());
        let public_key_set = secret_key_set.public_keys();
        let secret_key_share = secret_key_set.secret_key_share(0);

        let section_auth =
            create_first_section_authority_provider(&public_key_set, &secret_key_share, peer)?;

        let section = Section::new(
            section_auth.sig.public_key,
            SecuredLinkedList::new(section_auth.sig.public_key),
            section_auth,
        )?;

        for peer in section.section_auth.get().await.value.peers() {
            let node_state = NodeState::joined(peer, None);
            let sig = create_first_sig(&public_key_set, &secret_key_share, &node_state)?;
            let _ = section.members.update(SectionAuth {
                value: node_state,
                sig,
            });
        }

        Ok((
            section,
            SectionDkgOutcome::new(public_key_set, 0, secret_key_share),
        ))
    }

    fn genesis_key(&self) -> &bls::PublicKey {
        &self.genesis_key
    }

    /// Try to merge this `Section` with `other`. Returns `InvalidMessage` if `other` is invalid or
    /// its chain is not compatible with the chain of `self`.
    async fn merge(&self, other: SectionDto) -> Result<()> {
        if !other.section_auth.self_verify() {
            error!("can't merge sections: other section_auth failed self-verification");
            return Err(Error::InvalidMessage);
        }
        if &other.section_auth.sig.public_key != other.chain.last_key() {
            // TODO: use more specific error variant.
            error!("can't merge sections: other section_auth signed with incorrect key");
            return Err(Error::InvalidMessage);
        }

        self.chain.write().await.merge(other.chain.clone())?;

        if &other.section_auth.sig.public_key == self.chain.read().await.last_key() {
            self.section_auth.set(other.section_auth).await;
        }

        for info in other.members {
            let _ = self.update_member(info);
        }

        self.members
            .prune_not_matching(&self.section_auth.get().await.value.prefix());

        Ok(())
    }

    /// Update the `SectionAuthorityProvider` of our section.
    async fn update_elders(
        &self,
        new_section_auth: SectionAuth<SectionAuthorityProvider>,
        new_key_sig: KeyedSig,
    ) -> bool {
        if new_section_auth.value.prefix() != self.prefix().await
            && !new_section_auth
                .value
                .prefix()
                .is_extension_of(&self.prefix().await)
        {
            return false;
        }

        if !new_section_auth.self_verify() {
            return false;
        }

        if let Err(error) = self.chain.write().await.insert(
            &new_key_sig.public_key,
            new_section_auth.sig.public_key,
            new_key_sig.signature,
        ) {
            error!(
                "failed to insert key {:?} (signed with {:?}) into the section chain: {}",
                new_section_auth.sig.public_key, new_key_sig.public_key, error,
            );
            return false;
        }

        if &new_section_auth.sig.public_key == self.chain.read().await.last_key() {
            self.section_auth.set(new_section_auth).await;
        }

        self.members
            .prune_not_matching(&self.section_auth.get().await.value.prefix());

        true
    }

    /// Update the member. Returns whether it actually changed anything.
    async fn update_member(&self, node_state: SectionAuth<NodeState>) -> bool {
        if !node_state.verify(&self.chain.read().await.clone()) {
            error!("can't merge member {:?}", node_state.value);
            return false;
        }

        self.members.update(node_state).await
    }

    async fn last_key(&self) -> bls::PublicKey {
        *self.chain.read().await.last_key()
    }

    async fn prev_key(&self) -> bls::PublicKey {
        *self.chain.read().await.prev_key()
    }

    async fn root_key(&self) -> bls::PublicKey {
        *self.chain.read().await.root_key()
    }

    async fn has_key(&self, section_key: &bls::PublicKey) -> bool {
        self.chain.read().await.has_key(section_key)
    }

    async fn main_branch_len(&self) -> usize {
        self.chain.read().await.main_branch_len()
    }

    async fn cmp_by_position(&self, lhs: &bls::PublicKey, rhs: &bls::PublicKey) -> Ordering {
        self.chain.read().await.cmp_by_position(lhs, rhs)
    }

    async fn get_proof_chain_to_current(
        &self,
        from_key: &bls::PublicKey,
    ) -> Result<SecuredLinkedList> {
        Ok(self
            .chain
            .read()
            .await
            .get_proof_chain_to_current(from_key)?)
    }

    async fn check_trust<'a, I>(&self, trusted_keys: I) -> bool
    where
        I: Send + IntoIterator<Item = &'a bls::PublicKey>,
    {
        self.chain.read().await.check_trust(trusted_keys)
    }

    async fn keys(&self) -> Box<dyn DoubleEndedIterator<Item = bls::PublicKey>> {
        let keys: Vec<_> = self.chain.read().await.keys().map(|c| *c).collect();
        Box::new(keys.into_iter())
    }

    async fn chain_clone(&self) -> SecuredLinkedList {
        self.chain.read().await.clone()
    }

    // Extend the section chain so it starts at `trusted_key` while keeping the last key intact.
    async fn extend_chain(
        &self,
        trusted_key: &bls::PublicKey,
        full_chain: &SecuredLinkedList,
    ) -> Result<SectionDto, SecuredLinkedListError> {
        let chain = match self.chain.write().await.extend(trusted_key, full_chain) {
            Ok(chain) => chain,
            Err(SecuredLinkedListError::InvalidOperation) => {
                // This means the tip of the chain is not reachable from `trusted_key`.
                // Use the full chain instead as it is always trusted.
                self.chain.read().await.clone()
            }
            Err(error) => return Err(error),
        };

        Ok(SectionDto {
            genesis_key: self.genesis_key,
            section_auth: self.section_auth.clone().await,
            chain,
            members: self.members.clone().await,
        })
    }

    async fn authority_provider(&self) -> SectionAuthorityProvider {
        self.section_auth.get().await.value.clone()
    }

    async fn section_signed_authority_provider(&self) -> SectionAuth<SectionAuthorityProvider> {
        self.section_auth.get().await.as_ref().clone()
    }

    fn is_elder(&self, name: &XorName) -> bool {
        futures::executor::block_on(self.authority_provider()).contains_elder(name)
    }

    /// Generate a new section info(s) based on the current set of members.
    /// Returns a set of candidate SectionAuthorityProviders.
    async fn promote_and_demote_elders(&self, our_name: &XorName) -> Vec<ElderCandidates> {
        if let Some((our_elder_candidates, other_elder_candidates)) = self.try_split(our_name).await
        {
            return vec![our_elder_candidates, other_elder_candidates];
        }

        let expected_peers = self.elder_candidates(ELDER_SIZE).await;
        let expected_names: BTreeSet<_> = expected_peers.iter().map(Peer::name).cloned().collect();
        let current_names: BTreeSet<_> = self.authority_provider().await.names();

        if expected_names == current_names {
            vec![]
        } else if expected_names.len() < crate::routing::supermajority(current_names.len()) {
            warn!("ignore attempt to reduce the number of elders too much");
            vec![]
        } else {
            let elder_candidates = ElderCandidates::new(expected_peers, self.prefix().await);
            vec![elder_candidates]
        }
    }

    // Prefix of our section.
    async fn prefix(&self) -> Prefix {
        self.authority_provider().await.prefix.clone()
    }

    fn members(&self) -> &SectionPeers {
        &self.members
    }

    /// Returns members that are either joined or are left but still elders.
    async fn active_members(&self) -> Box<dyn Iterator<Item = Peer> + '_> {
        use futures::executor::block_on;
        Box::new(
            self.members
                .all()
                .await
                .filter(move |info| {
                    block_on(self.members.is_joined(info.peer.name()))
                        || self.is_elder(info.peer.name())
                })
                .map(|info| info.peer),
        )
    }

    /// Returns adults from our section.
    async fn adults(&self) -> Box<dyn Iterator<Item = Peer> + '_> {
        Box::new(
            self.members
                .mature()
                .await
                .filter(move |peer| !self.is_elder(peer.name())),
        )
    }

    /// Returns live adults from our section.
    async fn live_adults(&self) -> Box<dyn Iterator<Item = Peer> + '_> {
        Box::new(self.members.joined().await.filter_map(move |info| {
            if !self.is_elder(info.peer.name()) {
                Some(info.peer)
            } else {
                None
            }
        }))
    }

    async fn find_joined_member_by_addr(&self, addr: &SocketAddr) -> Option<Peer> {
        self.members
            .joined()
            .await
            .find(|info| info.peer.addr() == addr)
            .map(|info| info.peer)
    }

    // Tries to split our section.
    // If we have enough mature nodes for both subsections, returns the SectionAuthorityProviders
    // of the two subsections. Otherwise returns `None`.
    async fn try_split(&self, our_name: &XorName) -> Option<(ElderCandidates, ElderCandidates)> {
        let next_bit_index = if let Ok(index) = self.prefix().await.bit_count().try_into() {
            index
        } else {
            // Already at the longest prefix, can't split further.
            return None;
        };

        let next_bit = our_name.bit(next_bit_index);

        let (our_new_size, sibling_new_size) = self
            .members
            .mature()
            .await
            .map(|peer| peer.name().bit(next_bit_index) == next_bit)
            .fold((0, 0), |(ours, siblings), is_our_prefix| {
                if is_our_prefix {
                    (ours + 1, siblings)
                } else {
                    (ours, siblings + 1)
                }
            });

        // If none of the two new sections would contain enough entries, return `None`.
        if our_new_size < RECOMMENDED_SECTION_SIZE || sibling_new_size < RECOMMENDED_SECTION_SIZE {
            return None;
        }

        let our_prefix = self.prefix().await.pushed(next_bit);
        let other_prefix = self.prefix().await.pushed(!next_bit);

        let our_elders = self
            .members
            .elder_candidates_matching_prefix(
                &our_prefix,
                ELDER_SIZE,
                &self.authority_provider().await,
            )
            .await;
        let other_elders = self
            .members
            .elder_candidates_matching_prefix(
                &other_prefix,
                ELDER_SIZE,
                &self.authority_provider().await,
            )
            .await;

        let our_elder_candidates = ElderCandidates::new(our_elders, our_prefix);
        let other_elder_candidates = ElderCandidates::new(other_elders, other_prefix);

        Some((our_elder_candidates, other_elder_candidates))
    }

    // Returns the candidates for elders out of all the nodes in the section, even out of the
    // relocating nodes if there would not be enough instead.
    async fn elder_candidates(&self, elder_size: usize) -> Vec<Peer> {
        self.members
            .elder_candidates(elder_size, &self.authority_provider().await)
            .await
    }
}

// Create `SectionAuthorityProvider` for the first node.
fn create_first_section_authority_provider(
    pk_set: &bls::PublicKeySet,
    sk_share: &bls::SecretKeyShare,
    mut peer: Peer,
) -> Result<SectionAuth<SectionAuthorityProvider>> {
    peer.set_reachable(true);
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
