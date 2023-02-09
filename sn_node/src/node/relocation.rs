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
    network_knowledge::{
        node_state::RelocationTrigger, recommended_section_size, relocation_check,
        NetworkKnowledge, NodeState,
    },
};
use std::{cmp::min, collections::BTreeSet};
use xor_name::XorName;

/// Find all nodes to relocate after a churn event and generate the relocation details for them.
pub(super) fn find_nodes_to_relocate(
    network_knowledge: &NetworkKnowledge,
    relocation_trigger: &RelocationTrigger,
    excluded: BTreeSet<XorName>,
) -> Vec<NodeState> {
    // Find the peers that pass the relocation check and take only the oldest ones to avoid
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

    let max_relocations = elder_count() / 2;
    let allowed_relocations = min(section_size - recommended_section_size, max_relocations);
    let churn_id = relocation_trigger.churn_id();

    // Find the nodes that pass the relocation check
    let mut candidates: Vec<_> = section_members
        .into_iter()
        .filter(|state| network_knowledge.is_adult(&state.name()))
        .filter(|info| relocation_check(info.age(), &churn_id))
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
        .take(allowed_relocations)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    use sn_interface::{
        elder_count,
        network_knowledge::{NodeState, SectionAuthorityProvider, SectionTree, MIN_ADULT_AGE},
        test_utils::{create_relocation_trigger, TestKeys},
        types::NodeId,
    };

    use eyre::Result;
    use itertools::Itertools;
    use proptest::{collection::SizeRange, prelude::*};
    use rand::thread_rng;
    use std::net::SocketAddr;
    use xor_name::{Prefix, XOR_NAME_LEN};

    const MAX_AGE: u8 = MIN_ADULT_AGE + 3;

    proptest! {
        #[test]
        #[allow(clippy::unwrap_used)]
        fn proptest_actions(
            nodes in arbitrary_unique_peers(2..(recommended_section_size() + elder_count()), MIN_ADULT_AGE..MAX_AGE),
            signature_trailing_zeros in 0..MAX_AGE)
        {
            proptest_actions_impl(nodes, signature_trailing_zeros).unwrap()
        }
    }

    fn proptest_actions_impl(nodes: Vec<NodeId>, signature_trailing_zeros: u8) -> Result<()> {
        let age = signature_trailing_zeros;
        let sk_set = bls::SecretKeySet::random(0, &mut thread_rng());
        let sk = sk_set.secret_key();
        let (relocation_trigger, _) = create_relocation_trigger(&sk_set, age)?;
        let churn_id = relocation_trigger.churn_id();

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

        let relocations =
            find_nodes_to_relocate(&network_knowledge, &relocation_trigger, BTreeSet::default());

        let allowed_relocations = if nodes.len() > recommended_section_size() {
            min(elder_count() / 2, nodes.len() - recommended_section_size())
        } else {
            0
        };

        // Only the oldest matching nodes should be relocated.
        let expected_relocated_age = nodes
            .iter()
            .filter(|node| network_knowledge.is_adult(&node.name()))
            .map(NodeId::age)
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
        for (node_id, state) in expected_relocated_nodes.into_iter().zip(relocations) {
            assert_eq!(node_id.name(), state.node_id().name());
        }

        Ok(())
    }

    // Generate Vec<Peer> where no two peers have the same name.
    fn arbitrary_unique_peers(
        count: impl Into<SizeRange>,
        age: impl Strategy<Value = u8>,
    ) -> impl Strategy<Value = Vec<NodeId>> {
        proptest::collection::btree_map(arbitrary_bytes(), (any::<SocketAddr>(), age), count)
            .prop_map(|nodes| {
                nodes
                    .into_iter()
                    .map(|(mut bytes, (addr, age))| {
                        bytes[XOR_NAME_LEN - 1] = age;
                        let name = XorName(bytes);
                        NodeId::new(name, addr)
                    })
                    .collect()
            })
    }

    fn arbitrary_bytes() -> impl Strategy<Value = [u8; XOR_NAME_LEN]> {
        any::<[u8; XOR_NAME_LEN]>()
    }
}
