// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! Relocation related types and utilities.

use super::JoiningAsRelocated;

use crate::elder_count;
use crate::messaging::{
    system::{RelocateDetails, RelocatePayload, RelocatePromise, SystemMsg},
    AuthorityProof, SectionAuth,
};
use crate::node::error::Error;
use crate::node::routing::{
    ed25519::{self, Keypair, Verifier},
    network_knowledge::{NetworkKnowledge, NodeState},
    recommended_section_size, Peer,
};

use async_trait::async_trait;
use std::{cmp::min, collections::BTreeSet};
use xor_name::XorName;

/// Find all nodes to relocate after a churn event and create the relocate actions for them.
pub(crate) async fn actions(
    network_knowledge: &NetworkKnowledge,
    churn_name: &XorName,
    churn_signature: &bls::Signature,
) -> Vec<(NodeState, RelocateAction)> {
    // Find the peers that pass the relocation check and take only the oldest ones to avoid
    // relocating too many nodes at the same time.
    // Capped by criteria that cannot relocate too many node at once.
    let joined_nodes = network_knowledge.section_members();

    if joined_nodes.len() < recommended_section_size() {
        return vec![];
    }
    let max_reloctions = elder_count() / 2;
    let allowed_relocations = min(
        joined_nodes.len() - recommended_section_size(),
        max_reloctions,
    );

    let candidates: BTreeSet<_> = joined_nodes
        .into_iter()
        .filter(|info| check(info.age(), churn_signature))
        .collect();

    let max_age = if let Some(age) = candidates.iter().map(|info| info.age()).max() {
        age
    } else {
        return vec![];
    };

    let mut relocating_nodes = vec![];

    for node_state in candidates {
        if node_state.age() == max_age {
            let action =
                RelocateAction::new(network_knowledge, node_state.peer(), churn_name).await;
            relocating_nodes.push((node_state, action))
        }
    }

    relocating_nodes
        .into_iter()
        .take(allowed_relocations)
        .collect()
}

/// Details of a relocation: which node to relocate, where to relocate it to and what age it should
/// get once relocated.
#[async_trait]
pub(super) trait RelocateDetailsUtils {
    async fn new(network_knowledge: &NetworkKnowledge, peer: &Peer, dst: XorName) -> Self;

    async fn with_age(
        network_knowledge: &NetworkKnowledge,
        peer: &Peer,
        dst: XorName,
        age: u8,
    ) -> RelocateDetails;
}

#[async_trait]
impl RelocateDetailsUtils for RelocateDetails {
    async fn new(network_knowledge: &NetworkKnowledge, peer: &Peer, dst: XorName) -> Self {
        Self::with_age(network_knowledge, peer, dst, peer.age().saturating_add(1)).await
    }

    async fn with_age(
        network_knowledge: &NetworkKnowledge,
        peer: &Peer,
        dst: XorName,
        age: u8,
    ) -> RelocateDetails {
        let root_key = *network_knowledge.genesis_key();

        let dst_key = network_knowledge
            .section_by_name(&dst)
            .map_or_else(|_| root_key, |section_auth| section_auth.section_key());

        RelocateDetails {
            pub_id: peer.name(),
            dst,
            dst_key,
            age,
        }
    }
}

pub(crate) trait RelocatePayloadUtils {
    fn new(
        details: SystemMsg,
        section_auth: AuthorityProof<SectionAuth>,
        new_name: &XorName,
        old_keypair: &Keypair,
    ) -> Self;

    fn verify_identity(&self, new_name: &XorName) -> bool;

    fn relocate_details(&self) -> Result<&RelocateDetails, Error>;
}

impl RelocatePayloadUtils for RelocatePayload {
    fn new(
        details: SystemMsg,
        section_auth: AuthorityProof<SectionAuth>,
        new_name: &XorName,
        old_keypair: &Keypair,
    ) -> Self {
        let signature_of_new_name_with_old_key = ed25519::sign(&new_name.0, old_keypair);

        Self {
            details,
            section_signed: section_auth.into_inner(),
            signature_of_new_name_with_old_key,
        }
    }

    fn verify_identity(&self, new_name: &XorName) -> bool {
        let details = if let Ok(details) = self.relocate_details() {
            details
        } else {
            return false;
        };

        let pub_key = if let Ok(pub_key) = ed25519::pub_key(&details.pub_id) {
            pub_key
        } else {
            return false;
        };

        pub_key
            .verify(&new_name.0, &self.signature_of_new_name_with_old_key)
            .is_ok()
    }

    fn relocate_details(&self) -> Result<&RelocateDetails, Error> {
        if let SystemMsg::Relocate(relocate_details) = &self.details {
            Ok(relocate_details)
        } else {
            error!("RelocateDetails does not contain a NodeMsg::Relocate");
            Err(Error::InvalidMessage)
        }
    }
}
#[allow(clippy::large_enum_variant)]
pub(crate) enum RelocateState {
    // Node is undergoing delayed relocation. This happens when the node is selected for relocation
    // while being an elder. It must keep fulfilling its duties as elder until its demoted, then it
    // can send the bytes (which are serialized `RelocatePromise` message) back to the elders who
    // will exchange it for an actual `Relocate` message.
    Delayed(SystemMsg),
    // Relocation in progress.
    InProgress(Box<JoiningAsRelocated>),
}

/// Action to relocate a node.
#[derive(Debug)]
pub(crate) enum RelocateAction {
    /// Relocate the node instantly.
    Instant(RelocateDetails),
    /// Relocate the node after they are no longer our elder.
    Delayed(RelocatePromise),
}

impl RelocateAction {
    pub(crate) async fn new(
        network_knowledge: &NetworkKnowledge,
        peer: &Peer,
        churn_name: &XorName,
    ) -> Self {
        let dst = dst(&peer.name(), churn_name);

        if network_knowledge.is_elder(&peer.name()).await {
            RelocateAction::Delayed(RelocatePromise {
                name: peer.name(),
                dst,
            })
        } else {
            RelocateAction::Instant(RelocateDetails::new(network_knowledge, peer, dst).await)
        }
    }

    pub(crate) fn dst(&self) -> &XorName {
        match self {
            Self::Instant(details) => &details.dst,
            Self::Delayed(promise) => &promise.dst,
        }
    }

    #[cfg(test)]
    pub(crate) fn name(&self) -> &XorName {
        match self {
            Self::Instant(details) => &details.pub_id,
            Self::Delayed(promise) => &promise.name,
        }
    }
}

// Relocation check - returns whether a member with the given age is a candidate for relocation on
// a churn event with the given signature.
pub(crate) fn check(age: u8, churn_signature: &bls::Signature) -> bool {
    // Evaluate the formula: `signature % 2^age == 0` Which is the same as checking the signature
    // has at least `age` trailing zero bits.
    trailing_zeros(&churn_signature.to_bytes()[..]) >= age as u32
}

// Compute the destination for the node with `relocating_name` to be relocated to. `churn_name` is
// the name of the joined/left node that triggered the relocation.
fn dst(relocating_name: &XorName, churn_name: &XorName) -> XorName {
    XorName::from_content_parts(&[&relocating_name.0, &churn_name.0])
}

// Returns the number of trailing zero bits of the byte slice.
fn trailing_zeros(bytes: &[u8]) -> u32 {
    let mut output = 0;

    for &byte in bytes.iter().rev() {
        if byte == 0 {
            output += 8;
        } else {
            output += byte.trailing_zeros();
            break;
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elder_count;
    use crate::node::routing::{
        api::tests::SecretKeySet, dkg::test_utils::section_signed,
        network_knowledge::SectionAuthorityProvider, MIN_ADULT_AGE,
    };
    use crate::peer::test_utils::arbitrary_unique_peers;
    use assert_matches::assert_matches;
    use eyre::Result;
    use itertools::Itertools;
    use proptest::prelude::*;
    use rand::{rngs::SmallRng, Rng, SeedableRng};
    use secured_linked_list::SecuredLinkedList;
    use xor_name::Prefix;

    #[test]
    fn byte_slice_trailing_zeros() {
        assert_eq!(trailing_zeros(&[0]), 8);
        assert_eq!(trailing_zeros(&[1]), 0);
        assert_eq!(trailing_zeros(&[2]), 1);
        assert_eq!(trailing_zeros(&[4]), 2);
        assert_eq!(trailing_zeros(&[8]), 3);
        assert_eq!(trailing_zeros(&[0, 0]), 16);
        assert_eq!(trailing_zeros(&[1, 0]), 8);
        assert_eq!(trailing_zeros(&[2, 0]), 9);
    }

    const MAX_AGE: u8 = MIN_ADULT_AGE + 3;

    proptest! {
        #[test]
        fn proptest_actions(
            peers in arbitrary_unique_peers(2..(recommended_section_size() + elder_count()), MIN_ADULT_AGE..MAX_AGE),
            signature_trailing_zeros in 0..MAX_AGE,
            seed in any::<u64>().no_shrink())
        {
            proptest_actions_impl(peers, signature_trailing_zeros, seed).unwrap()
        }
    }

    fn proptest_actions_impl(
        peers: Vec<Peer>,
        signature_trailing_zeros: u8,
        seed: u64,
    ) -> Result<()> {
        let mut rng = SmallRng::seed_from_u64(seed);

        let sk_set = SecretKeySet::random();
        let sk = sk_set.secret_key();
        let genesis_pk = sk.public_key();

        // Create `Section` with `peers` as its members and set the `elder_count()` oldest peers as
        // the elders.
        let section_auth = SectionAuthorityProvider::new(
            peers
                .iter()
                .sorted_by_key(|peer| peer.age())
                .rev()
                .take(elder_count())
                .cloned(),
            Prefix::default(),
            sk_set.public_keys(),
        );
        let section_auth = section_signed(sk, section_auth)?;

        let network_knowledge = NetworkKnowledge::new(
            genesis_pk,
            SecuredLinkedList::new(genesis_pk),
            section_auth,
            None,
        )?;

        for peer in &peers {
            let info = NodeState::joined(peer.clone(), None);
            let info = section_signed(sk, info)?;

            let res = futures::executor::block_on(network_knowledge.update_member(info));
            assert!(res);
        }

        // Simulate a churn event whose signature has the given number of trailing zeros.
        let churn_name = rng.gen();
        let churn_signature = signature_with_trailing_zeros(signature_trailing_zeros as u32);

        let actions = actions(&network_knowledge, &churn_name, &churn_signature);
        let actions = futures::executor::block_on(actions);

        let actions: Vec<_> = actions
            .into_iter()
            .map(|(_, action)| action)
            .sorted_by_key(|action| *action.name())
            .collect();

        let allowed_relocations = if peers.len() > recommended_section_size() {
            min(elder_count() / 2, peers.len() - recommended_section_size())
        } else {
            0
        };

        // Only the oldest matching peers should be relocated.
        let expected_relocated_age = peers
            .iter()
            .map(Peer::age)
            .filter(|age| *age <= signature_trailing_zeros)
            .max();

        let expected_relocated_peers: Vec<_> = peers
            .iter()
            .filter(|peer| Some(peer.age()) == expected_relocated_age)
            .sorted_by_key(|peer| peer.name())
            .take(allowed_relocations)
            .collect();

        assert_eq!(expected_relocated_peers.len(), actions.len());

        // Verify the relocate action is correct depending on whether the peer is elder or not.
        // NOTE: `zip` works here, because both collections are sorted by name.
        for (peer, action) in expected_relocated_peers.into_iter().zip(actions) {
            assert_eq!(&peer.name(), action.name());

            let is_elder = futures::executor::block_on(network_knowledge.is_elder(&peer.name()));

            if is_elder {
                assert_matches!(action, RelocateAction::Delayed(_));
            } else {
                assert_matches!(action, RelocateAction::Instant(_));
            }
        }

        Ok(())
    }

    // Fetch a `bls::Signature` with the given number of trailing zeros. The signature is generated
    // from an unspecified random data using an unspecified random `SecretKey`. That is OK because
    // the relocation algorithm doesn't care about whether the signature is valid. It only
    // cares about its number of trailing zeros.
    fn signature_with_trailing_zeros(trailing_zeros_count: u32) -> bls::Signature {
        use std::{cell::RefCell, collections::HashMap};

        // Cache the signatures to avoid expensive re-computation.
        thread_local! {
            static CACHE: RefCell<HashMap<u32, bls::Signature>> = RefCell::new(HashMap::new());
        }

        CACHE.with(|cache| {
            cache
                .borrow_mut()
                .entry(trailing_zeros_count)
                .or_insert_with(|| gen_signature_with_trailing_zeros(trailing_zeros_count))
                .clone()
        })
    }

    fn gen_signature_with_trailing_zeros(trailing_zeros_count: u32) -> bls::Signature {
        let mut rng = SmallRng::seed_from_u64(0);
        let sk: bls::SecretKey = rng.gen();

        loop {
            let data: u64 = rng.gen();
            let signature = sk.sign(&data.to_be_bytes());

            if trailing_zeros(&signature.to_bytes()) == trailing_zeros_count {
                return signature;
            }
        }
    }
}
