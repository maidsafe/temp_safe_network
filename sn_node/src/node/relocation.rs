// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! Relocation related types and utilities.

use sn_interface::{
    elder_count,
    network_knowledge::{recommended_section_size, NetworkKnowledge, NodeState},
};
use std::{
    cmp::min,
    collections::BTreeSet,
    fmt::{self, Display, Formatter},
};
use xor_name::XorName;

// Unique identifier for a churn event, which is used to select nodes to relocate.
pub(crate) struct ChurnId(pub(crate) [u8; bls::SIG_SIZE]);

impl Display for ChurnId {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(
            fmt,
            "Churn-{:02x}{:02x}{:02x}..",
            self.0[0], self.0[1], self.0[2]
        )
    }
}

/// Find all nodes to relocate after a churn event and generate the relocation details for them.
pub(super) fn find_nodes_to_relocate(
    network_knowledge: &NetworkKnowledge,
    churn_id: &ChurnId,
    excluded: BTreeSet<XorName>,
) -> Vec<(NodeState, XorName)> {
    // Find the nodes that pass the relocation check and take only the oldest ones to avoid
    // relocating too many nodes at the same time.
    // Capped by criteria that cannot relocate too many node at once.
    let section_members = network_knowledge.section_members();
    let section_size = section_members.len();
    let recommended_section_size = recommended_section_size();
    info!(
        "Finding relocation candidates, having {section_size} members, \
        recommended section_size {recommended_section_size}"
    );

    // no relocation if total section size is too small
    if section_size <= recommended_section_size {
        return vec![];
    }

    let max_reloctions = elder_count() / 2;
    let allowed_relocations = min(section_size - recommended_section_size, max_reloctions);

    // Find the nodes that pass the relocation check
    let mut candidates: Vec<_> = section_members
        .into_iter()
        .filter(|state| network_knowledge.is_adult(&state.name()))
        .filter(|info| check(info.age(), churn_id))
        // the newly joined node shall not be relocated immediately
        .filter(|info| !excluded.contains(&info.name()))
        .collect();
    // To avoid a node to manipulate its name to gain priority of always being first in XorName,
    // here we sort the nodes by its distance to the churn_id.
    let target_name = XorName::from_content(&churn_id.0);
    candidates.sort_by(|lhs, rhs| target_name.cmp_distance(&lhs.name(), &rhs.name()));

    info!("Finding relocation candidates {candidates:?}");
    let max_age = if let Some(age) = candidates.iter().map(|info| info.age()).max() {
        age
    } else {
        return vec![];
    };

    candidates
        .into_iter()
        .filter(|node| node.age() == max_age)
        .map(|node| {
            let dst_section = XorName::from_content_parts(&[&node.name().0, &churn_id.0]);
            (node, dst_section)
        })
        .take(allowed_relocations)
        .collect()
}

// Relocation check - returns whether a member with the given age is a candidate for relocation on
// a churn event with the given churn id.
pub(crate) fn check(age: u8, churn_id: &ChurnId) -> bool {
    // Evaluate the formula: `signature % 2^age == 0` Which is the same as checking the signature
    // has at least `age` trailing zero bits.
    trailing_zeros(&churn_id.0) >= age as u32
}

// Returns the number of trailing zero bits of the bytes slice.
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

    use sn_interface::{
        elder_count,
        network_knowledge::{NodeState, SectionAuthorityProvider, SectionTree, MIN_ADULT_AGE},
        test_utils::TestKeys,
        types::{NodeId, RewardNodeId},
    };

    use eyre::Result;
    use itertools::Itertools;
    use proptest::{collection::SizeRange, prelude::*};
    use rand::{rngs::SmallRng, thread_rng, Rng, SeedableRng};
    use std::net::SocketAddr;
    use xor_name::{Prefix, XOR_NAME_LEN};

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
        #[allow(clippy::unwrap_used)]
        fn proptest_actions(
            nodes in arbitrary_unique_nodes(2..(recommended_section_size() + elder_count()), MIN_ADULT_AGE..MAX_AGE),
            signature_trailing_zeros in 0..MAX_AGE)
        {
            proptest_actions_impl(nodes, signature_trailing_zeros).unwrap()
        }
    }

    fn proptest_actions_impl(nodes: Vec<RewardNodeId>, signature_trailing_zeros: u8) -> Result<()> {
        let sk_set = bls::SecretKeySet::random(0, &mut thread_rng());
        let sk = sk_set.secret_key();

        // Create `Section` with `nodes` as its members and set the `elder_count()` oldest nodes as
        // the elders.
        let sap = SectionAuthorityProvider::new(
            nodes
                .iter()
                .sorted_by_key(|node_id| node_id.age())
                .rev()
                .take(elder_count())
                .cloned(),
            Prefix::default(),
            nodes.iter().map(|p| NodeState::joined(*p, None)),
            sk_set.public_keys(),
            0,
        );
        let sap = TestKeys::get_section_signed(&sk, sap)?;
        let tree = SectionTree::new(sap)?;
        let mut network_knowledge = NetworkKnowledge::new(Prefix::default(), tree)?;

        for node_id in &nodes {
            let info = NodeState::joined(*node_id, None);
            let info = TestKeys::get_section_signed(&sk, info)?;
            assert!(network_knowledge.update_member(info));
        }

        // Simulate a churn event whose signature has the given number of trailing zeros.
        let churn_id =
            ChurnId(signature_with_trailing_zeros(signature_trailing_zeros as u32).to_bytes());

        let relocations =
            find_nodes_to_relocate(&network_knowledge, &churn_id, BTreeSet::default());

        let allowed_relocations = if nodes.len() > recommended_section_size() {
            min(elder_count() / 2, nodes.len() - recommended_section_size())
        } else {
            0
        };

        // Only the oldest matching nodes should be relocated.
        let expected_relocated_age = nodes
            .iter()
            .filter(|node_id| network_knowledge.is_adult(&node_id.name()))
            .map(|p| p.age())
            .filter(|age| *age <= signature_trailing_zeros)
            .max();

        let mut expected_relocated_nodes: Vec<_> = nodes
            .iter()
            .filter(|node_id| network_knowledge.is_adult(&node_id.name()))
            .filter(|node_id| Some(node_id.age()) == expected_relocated_age)
            .collect();

        let churn_id_name = XorName::from_content(&churn_id.0);
        expected_relocated_nodes
            .sort_by(|lhs, rhs| churn_id_name.cmp_distance(&lhs.name(), &rhs.name()));
        let expected_relocated_nodes: Vec<_> = expected_relocated_nodes
            .iter()
            .take(allowed_relocations)
            .collect();

        assert_eq!(expected_relocated_nodes.len(), relocations.len());

        // NOTE: `zip` works here, as both collections are sorted by the same criteria.
        for (node_id, (state, dst)) in expected_relocated_nodes.into_iter().zip(relocations) {
            assert_eq!(node_id.name(), state.node_id().name()); // Note: shouldn't addr also be compared?
            let dst_section = XorName::from_content_parts(&[&node_id.name().0, &churn_id.0]);
            assert_eq!(dst_section, dst);
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
            let signature = sk.sign(data.to_be_bytes());

            if trailing_zeros(&signature.to_bytes()) == trailing_zeros_count {
                return signature;
            }
        }
    }

    // Generate Vec<NodeId> where no two nodes have the same name.
    fn arbitrary_unique_nodes(
        count: impl Into<SizeRange>,
        age: impl Strategy<Value = u8>,
    ) -> impl Strategy<Value = Vec<RewardNodeId>> {
        proptest::collection::btree_map(arbitrary_bytes(), (any::<SocketAddr>(), age), count)
            .prop_map(|nodes| {
                nodes
                    .into_iter()
                    .map(|(mut bytes, (addr, age))| {
                        bytes[XOR_NAME_LEN - 1] = age;
                        let name = XorName(bytes);
                        RewardNodeId::random_w_node_id(NodeId::new(name, addr))
                    })
                    .collect()
            })
    }

    fn arbitrary_bytes() -> impl Strategy<Value = [u8; XOR_NAME_LEN]> {
        any::<[u8; XOR_NAME_LEN]>()
    }
}
