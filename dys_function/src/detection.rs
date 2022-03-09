// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{
    DysfunctionDetection, EXCESSIVE_OPS_TOLERANCE, MIN_PENDING_OPS,
    PREEMPTIVE_REPLICATION_TOLERANCE,
};
use std::collections::BTreeSet;
use xor_name::XorName;

impl DysfunctionDetection {
    /// This is not an exact definition, thus has tolerance for variance due to concurrency
    /// returns Unresponsive Nodes, to be voted off and Deviants... which are more likely dysfunctional
    pub async fn find_unresponsive_and_deviant_nodes(
        &self,
    ) -> (BTreeSet<XorName>, BTreeSet<XorName>) {
        debug!("Checking for unresponsive and deviant nodes");
        let mut unresponsive_nodes = BTreeSet::new();
        let mut deviants = Vec::new();

        for entry in self.closest_nodes_to.iter() {
            let (node, neighbours) = entry.pair();

            let node = *node;
            let mut max_pending_by_neighbours = 0;
            for neighbour in neighbours.iter() {
                if let Some(entry) = self.unfulfilled_requests.get(neighbour) {
                    let val = entry.value().read().await.len();

                    if val > max_pending_by_neighbours {
                        max_pending_by_neighbours = val
                    }
                }
            }

            let pending_operations_count = if let Some(entry) = self.unfulfilled_requests.get(&node)
            {
                entry.value().read().await.len()
            } else {
                0
            };

            let crossed_min_threshold = pending_operations_count > MIN_PENDING_OPS;

            let excessive_stagnating_ops = pending_operations_count as f64
                > max_pending_by_neighbours as f64 * EXCESSIVE_OPS_TOLERANCE;

            let need_for_preemptive_replication = pending_operations_count as f64
                > max_pending_by_neighbours as f64 * PREEMPTIVE_REPLICATION_TOLERANCE;

            // Replicate preemptively at 2.5x of neighbours max pending ops and kick at 5x.
            // Checks begin only if op count crosses MIN_PENDING_OPS
            if crossed_min_threshold && excessive_stagnating_ops {
                tracing::info!(
                    "Pending ops for {}: {} Neighbour max: {}",
                    node,
                    pending_operations_count,
                    max_pending_by_neighbours
                );
                let _existed = unresponsive_nodes.insert(node);
            }

            if crossed_min_threshold && need_for_preemptive_replication {
                info!(
                    "Probable deviant {node} crossed PREEMPTIVE_REPLICATION_TOLERANCE: \
                {pending_operations_count}: Neighbour max: {max_pending_by_neighbours}",
                );
                deviants.push(node);
            }
        }
        (
            unresponsive_nodes,
            deviants.into_iter().collect::<BTreeSet<XorName>>(),
        )
    }
}

#[cfg(test)]
mod tests {
    // use super::{DysfunctionDetection, EXCESSIVE_OPS_TOLERANCE};

    // use crate::messaging::data::chunk_operation_id;
    // use crate::node::Error;
    // use crate::types::ChunkAddress;

    // use crate::node::core::data::records::liveness_tracking::MIN_PENDING_OPS;
    // use itertools::Itertools;
    // use std::collections::BTreeSet;
    // use xor_name::XorName;
}
