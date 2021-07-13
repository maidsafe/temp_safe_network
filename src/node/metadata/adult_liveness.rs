// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::{EndUser, MessageId};
use crate::routing::XorName;
use crate::types::ChunkAddress;
use dashmap::mapref::entry::Entry;
use dashmap::DashMap;
use itertools::Itertools;
use std::collections::BTreeSet;

use crate::node::capacity::CHUNK_COPY_COUNT;

const NEIGHBOUR_COUNT: usize = 2;
const MIN_PENDING_OPS: usize = 10;
const PENDING_OP_TOLERANCE_RATIO: f64 = 0.1;

#[derive(Clone, Debug)]
struct ReadOperation {
    head_address: ChunkAddress,
    origin: EndUser,
    origin_msg_id: MessageId,
    targets: BTreeSet<XorName>,
    responded_with_success: bool,
}

pub(crate) struct AdultLiveness {
    ops: DashMap<MessageId, ReadOperation>,
    pending_ops: DashMap<XorName, usize>,
    closest_adults: DashMap<XorName, Vec<XorName>>,
}

impl AdultLiveness {
    pub(crate) fn new() -> Self {
        Self {
            ops: DashMap::new(),
            pending_ops: DashMap::new(),
            closest_adults: DashMap::new(),
        }
    }

    // Inserts a new read operation
    // Returns false if the operation already existed.
    pub(crate) fn new_read(
        &self,
        msg_id: MessageId,
        head_address: ChunkAddress,
        origin: EndUser,
        origin_msg_id: MessageId,
        targets: BTreeSet<XorName>,
    ) -> bool {
        let new_operation = if let Entry::Vacant(entry) = self.ops.entry(msg_id) {
            let _ = entry.insert(ReadOperation {
                head_address,
                origin,
                origin_msg_id,
                targets: targets.clone(),
                responded_with_success: false,
            });
            true
        } else {
            false
        };
        if new_operation {
            self.increment_pending_op(&targets);
        }
        new_operation
    }

    pub(crate) fn retain_members_only(&self, current_members: BTreeSet<XorName>) {
        let all_keys: Vec<_> = self
            .closest_adults
            .iter()
            .map(|entry| *entry.key())
            .collect();

        let removed_members = all_keys
            .iter()
            .filter_map(|key| {
                if !current_members.contains(key) {
                    let _ = self.pending_ops.remove(key);
                    let _ = self.closest_adults.remove(key);
                    Some(*key)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let message_ids = self
            .ops
            .iter()
            .map(|entry| *entry.key())
            .collect::<Vec<_>>();

        for name in removed_members {
            // TODO(after T4): For write operations perhaps we need to write it to a different Adult
            for msg_id in &message_ids {
                self.remove_target(msg_id, &name);
            }
        }

        self.recompute_closest_adults();
    }

    pub(crate) fn remove_target(&self, msg_id: &MessageId, name: &XorName) {
        if let Some(mut count) = self.pending_ops.get_mut(name) {
            let counter = *count;
            if counter > 0 {
                let count = count.value_mut();
                *count -= 1;
            }
        }
        let complete = if let Some(mut op) = self.ops.get_mut(&msg_id) {
            let ReadOperation { targets, .. } = op.value_mut();
            let _ = targets.remove(name);
            targets.is_empty()
        } else {
            true
        };
        if complete {
            let _ = self.ops.remove(&msg_id);
        }
    }

    pub(crate) fn record_adult_read_liveness(
        &self,
        correlation_id: &MessageId,
        src: &XorName,
        success: bool,
    ) -> Option<(ChunkAddress, EndUser, MessageId)> {
        self.remove_target(correlation_id, src);
        let op = self.ops.get_mut(&correlation_id);
        op.and_then(|mut op| {
            let ReadOperation {
                head_address,
                origin,
                origin_msg_id,
                targets,
                responded_with_success,
            } = op.value_mut();

            if targets.len() < CHUNK_COPY_COUNT && *responded_with_success {
                None
            } else {
                *responded_with_success = success;
                Some((*head_address, *origin, *origin_msg_id))
            }
        })
    }

    fn increment_pending_op(&self, targets: &BTreeSet<XorName>) {
        let mut closest_changed = false;

        for node in targets {
            if !self.pending_ops.contains_key(node) {
                let _ = self.pending_ops.insert(*node, 1);
            }
            if let Some(mut pair) = self.pending_ops.get_mut(node) {
                *pair.value_mut() += 1;
            }
            if !self.closest_adults.contains_key(node) {
                closest_changed = true;
                let _ = self.closest_adults.insert(*node, Vec::new());
            }
        }

        if closest_changed {
            self.recompute_closest_adults();
        }
    }

    pub(crate) fn recompute_closest_adults(&self) {
        let all_keys: Vec<_> = self
            .closest_adults
            .iter()
            .map(|entry| *entry.key())
            .collect();
        self.closest_adults.alter_all(|name, _| {
            all_keys
                .iter()
                .filter(|&key| key != name)
                .sorted_by(|lhs, rhs| name.cmp_distance(lhs, rhs))
                .take(NEIGHBOUR_COUNT)
                .copied()
                .collect::<Vec<_>>()
        });
    }

    // this is not an exact definition, thus has tolerance for variance due to concurrency
    pub(crate) fn find_unresponsive_adults(&self) -> Vec<(XorName, usize)> {
        let mut unresponsive_adults = Vec::new();
        for entry in &self.closest_adults {
            let (adult, neighbours) = entry.pair();
            if let Some(max_pending_by_neighbours) = neighbours
                .iter()
                .map(|neighbour| {
                    self.pending_ops
                        .get(neighbour)
                        .map(|entry| *entry.value())
                        .unwrap_or(0)
                })
                .max()
            {
                let adult_pending_ops = self
                    .pending_ops
                    .get(adult)
                    .map(|entry| *entry.value())
                    .unwrap_or(0);
                if adult_pending_ops > MIN_PENDING_OPS
                    && max_pending_by_neighbours > MIN_PENDING_OPS
                    && adult_pending_ops as f64 * PENDING_OP_TOLERANCE_RATIO
                        > max_pending_by_neighbours as f64
                {
                    tracing::info!(
                        "Pending ops for {}: {} Neighbour max: {}",
                        adult,
                        adult_pending_ops,
                        max_pending_by_neighbours
                    );
                    unresponsive_adults.push((*adult, adult_pending_ops));
                }
            }
        }
        unresponsive_adults
    }
}
