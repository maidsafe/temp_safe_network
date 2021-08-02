// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::routing::XorName;

use dashmap::DashMap;
use dashmap::DashSet;
use itertools::Itertools;
use std::collections::BTreeSet;
use std::sync::Arc;

const NEIGHBOUR_COUNT: usize = 2;
const MIN_PENDING_OPS: usize = 10;
const PENDING_OP_TOLERANCE_RATIO: f64 = 0.1;

/// Some reproducible xorname derived from the operation. Which can be re-derived from the appropriate response when received (to remove from tracking)
type OperationIdentifier = XorName;
type NodeIdentifier = XorName;

/// Something the node in question is yet to do
// https://en.wikipedia.org/wiki/Colonel_Cathcart
type BlackEye = OperationIdentifier;

#[derive(Clone, Debug)]
pub(crate) struct Liveness {
    // ops: Arc<DashMap<MessageId, ReadOperation>>,
    black_eyes: Arc<DashMap<NodeIdentifier, DashSet<BlackEye>>>,
    closest_nodes_to: Arc<DashMap<XorName, Vec<XorName>>>,
}

impl Liveness {
    pub(crate) fn new() -> Self {
        Self {
            // ops: Arc::new(DashMap::new()),
            black_eyes: Arc::new(DashMap::new()),
            closest_nodes_to: Arc::new(DashMap::new()),
        }
    }

    // Inserts a black eye, and is deemed as such until we get the appropriate response from the node
    // Returns false if the operation already existed.
    pub(crate) fn is_fresh_black_eye(
        &self,
        node_id: NodeIdentifier,
        operation_id: &OperationIdentifier,
    ) -> bool {
        let new_operation = if let Some(black_eyes) = self.black_eyes.get(&node_id) {
            let black_eyes_for_node = black_eyes.value();
            if black_eyes_for_node.contains(operation_id) {
                false
            } else {
                let _ = black_eyes_for_node.insert(*operation_id);
                debug!("BLACK EYE ADDED: {:?}", operation_id);

                true
            }
        } else {
            let black_eyes = DashSet::new();
            let _ = black_eyes.insert(operation_id.clone());
            let _ = self.black_eyes.insert(node_id, black_eyes);

            debug!("BLACK EYE ADDED: {:?}", operation_id);

            true
        };

        new_operation
    }

    pub(crate) fn retain_members_only(&self, current_members: BTreeSet<XorName>) {
        let all_keys: Vec<_> = self
            .closest_nodes_to
            .iter()
            .map(|entry| *entry.key())
            .collect();

        let _removed_members = all_keys
            .iter()
            .filter_map(|key| {
                if !current_members.contains(key) {
                    let _ = self.black_eyes.remove(key);
                    let _ = self.closest_nodes_to.remove(key);
                    Some(*key)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        self.recompute_closest_nodes();
    }

    /// Removes a black eye from the node liveness records
    pub(crate) fn remove_black_eye(
        &self,
        node_id: &NodeIdentifier,
        operation_id: &OperationIdentifier,
    ) {
        if let Some(black_eyes) = self.black_eyes.get_mut(node_id) {
            let _ = black_eyes.remove(operation_id);
            debug!(
                "BLACK EYE REMOVED for node: {:?} op: {:?}",
                node_id, operation_id
            );
        }
    }

    pub(crate) fn recompute_closest_nodes(&self) {
        let all_known_nodes: Vec<_> = self
            .closest_nodes_to
            .iter()
            .map(|entry| *entry.key())
            .collect();

        self.closest_nodes_to.alter_all(|name, _| {
            all_known_nodes
                .iter()
                .filter(|&key| key != name)
                .sorted_by(|lhs, rhs| name.cmp_distance(lhs, rhs))
                .take(NEIGHBOUR_COUNT)
                .copied()
                .collect::<Vec<_>>()
        });
    }

    // this is not an exact definition, thus has tolerance for variance due to concurrency
    pub(crate) fn find_unresponsive_nodes(&self) -> Vec<(XorName, usize)> {
        let mut unresponsive_nodes = Vec::new();
        for entry in self.closest_nodes_to.iter() {
            let (node, neighbours) = entry.pair();
            if let Some(max_pending_by_neighbours) = neighbours
                .iter()
                .map(|neighbour| {
                    self.black_eyes
                        .get(neighbour)
                        .map(|entry| entry.value().len())
                        .unwrap_or(0)
                })
                .max()
            {
                let black_eyes_count = self
                    .black_eyes
                    .get(&node)
                    .map(|entry| entry.value().len())
                    .unwrap_or(0);

                if black_eyes_count > MIN_PENDING_OPS
                    && max_pending_by_neighbours > MIN_PENDING_OPS
                    && black_eyes_count as f64 * PENDING_OP_TOLERANCE_RATIO
                        > max_pending_by_neighbours as f64
                {
                    tracing::info!(
                        "Pending ops for {}: {} Neighbour max: {}",
                        node,
                        black_eyes_count,
                        max_pending_by_neighbours
                    );
                    unresponsive_nodes.push((node.clone(), black_eyes_count));
                }
            }
        }
        unresponsive_nodes
    }
}
