// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! Dysfunctional "Node" detection for the SAFE Network.

// For quick_error
#![recursion_limit = "256"]
#![doc(
    html_logo_url = "https://github.com/maidsafe/QA/raw/master/Images/maidsafe_logo.png",
    html_favicon_url = "https://maidsafe.net/img/favicon.ico",
    test(attr(deny(warnings)))
)]
// Forbid some very bad patterns. Forbid is stronger than `deny`, preventing us from suppressing the
// lint with `#[allow(...)]` et-all.
#![forbid(
    arithmetic_overflow,
    mutable_transmutes,
    no_mangle_const_items,
    unknown_crate_types,
    unsafe_code
)]
// Turn on some additional warnings to encourage good style.
#![warn(
    missing_debug_implementations,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results,
    clippy::unicode_not_nfc
)]

#[macro_use]
extern crate tracing;

mod detection;
mod operations;

use xor_name::XorName;

use dashmap::DashMap;
use itertools::Itertools;
use std::collections::BTreeSet;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Minimum number of pending operations that are allowed to stagnate before dysfunction checks kick in
// ~400 ops was the maximum number stagnating ops seen with client_api tests(multi-threaded).
// Therefore 500 is better higher cap with some buffer. Setting it to 400 or lower might start replication on CI unnecessarily.
const MIN_PENDING_OPS: usize = 500;

// If the pending ops count of a node is 5 times higher than it's neighbours, it will be kicked.
const EXCESSIVE_OPS_TOLERANCE: f64 = 5.0; // increasing this number increases tolerance

// If the pending ops count of a node is EXCESSIVE_OPS_TOLERANCE / 2 times higher than it's neighbours, preemptive replication starts.
const PREEMPTIVE_REPLICATION_TOLERANCE: f64 = EXCESSIVE_OPS_TOLERANCE / 2.0; // increasing this number increases tolerance

/// Some reproducible xorname derived from the operation. This is a permanent reference needed for logging all dysfunction.
type NodeIdentifier = XorName;

// re declaration here to not create circular dep w/ sn atm.
// TODO: depend on types once that's extracted
type OperationId = [u8; 32];

#[derive(Clone, Debug)]
/// Dysfunctional node tracking. Allows various potential issues to be tracked and weighted,
/// with unresposive or deviant nodes being noted on request, against which action can then be taken.
pub struct DysfunctionDetection {
    neighbour_count: usize,
    communication_issues: Arc<DashMap<NodeIdentifier, usize>>, // count of comm issues
    /// One of (potentially many) different ways of assessing unresponsiveness of nodes.
    unfulfilled_requests: Arc<DashMap<NodeIdentifier, Arc<RwLock<Vec<OperationId>>>>>, // OperationId = [u8; 32]
    closest_nodes_to: Arc<DashMap<XorName, Vec<XorName>>>,
}

impl DysfunctionDetection {
    /// Set up a new Dysfunctional Node Tracker, averaging over a given neighbour count
    pub fn new(our_adults: Vec<NodeIdentifier>, neighbour_count: usize) -> Self {
        let closest_nodes_to = DashMap::new();

        for adult in our_adults.iter() {
            let closest_nodes = our_adults
                .iter()
                .filter(|&key| key != adult)
                .sorted_by(|lhs, rhs| adult.cmp_distance(lhs, rhs))
                .take(neighbour_count)
                .cloned()
                .collect::<Vec<_>>();
            let _old_entry = closest_nodes_to.insert(*adult, closest_nodes);
        }
        Self {
            neighbour_count,
            communication_issues: Arc::new(DashMap::new()),
            unfulfilled_requests: Arc::new(DashMap::new()),
            closest_nodes_to: Arc::new(closest_nodes_to),
        }
    }

    /// List all current tracked nodes
    pub fn current_nodes(&self) -> Vec<XorName> {
        self.closest_nodes_to
            .iter()
            .map(|entry| *entry.key())
            .collect::<Vec<XorName>>()
    }

    /// Add a new node to the tracker and recompute closest nodes.
    pub fn add_new_node(&self, adult: XorName) {
        info!("Adding new adult:{adult} to DysfunctionDetection tracker");

        let our_adults: Vec<_> = self
            .closest_nodes_to
            .iter()
            .map(|entry| *entry.key())
            .collect();

        let closest_nodes = our_adults
            .iter()
            .filter(|&key| key != &adult)
            .sorted_by(|lhs, rhs| adult.cmp_distance(lhs, rhs))
            .take(self.neighbour_count)
            .cloned()
            .collect::<Vec<_>>();

        info!("Closest nodes to {adult}:{closest_nodes:?}");

        if let Some(_old_entry) = self.closest_nodes_to.insert(adult, closest_nodes) {
            warn!("Throwing old dysfunction tracker for Adult {adult}:{_old_entry:?}");
        }

        self.recompute_closest_nodes();
    }

    /// Track a communication issue for a given node
    pub fn track_comm_issue(&self, node_id: NodeIdentifier) {
        // iniital entry setup if non existent
        let _entry = self.communication_issues.entry(node_id).or_default();

        trace!("Noting comms issue against node: {:?}", node_id,);

        if let Some(mut v) = self.communication_issues.get_mut(&node_id) {
            *v += 1;
        }
    }

    /// Removes any tracked nodes not present in the passed `current_members`
    pub fn retain_members_only(&self, current_members: BTreeSet<XorName>) {
        let all_keys: Vec<_> = self.current_nodes();

        for key in &all_keys {
            if !current_members.contains(key) {
                let _prev = self.unfulfilled_requests.remove(key);
                let _prev = self.closest_nodes_to.remove(key);
            }
        }

        self.recompute_closest_nodes();
    }

    /// Recalculats the closest nodes
    pub fn recompute_closest_nodes(&self) {
        let all_known_nodes: Vec<_> = self.current_nodes();

        self.closest_nodes_to.alter_all(|name, _| {
            all_known_nodes
                .iter()
                .filter(|&key| key != name)
                .sorted_by(|lhs, rhs| name.cmp_distance(lhs, rhs))
                .take(self.neighbour_count)
                .copied()
                .collect::<Vec<_>>()
        });
    }
}

#[cfg(test)]
mod tests {
    use super::{DysfunctionDetection, OperationId, EXCESSIVE_OPS_TOLERANCE};

    use crate::MIN_PENDING_OPS;
    use eyre::Error;
    use itertools::Itertools;
    use rand::Rng;
    use std::collections::BTreeSet;
    use xor_name::XorName;

    type Result<T, E = Error> = std::result::Result<T, E>;

    static ELDER_COUNT: usize = 7;

    fn get_random_operation_id() -> OperationId {
        let mut rng = rand::thread_rng();
        rng.gen()
    }

    #[tokio::test]
    async fn dysfunction_basics() -> Result<()> {
        let adults = (0..10).map(|_| XorName::random()).collect::<Vec<XorName>>();

        let dysfunctional_detection = DysfunctionDetection::new(adults.clone(), ELDER_COUNT);

        // Write data MIN_PENDING_OPS times to the 10 adults
        for adult in &adults {
            for _ in 0..MIN_PENDING_OPS {
                let op_id = get_random_operation_id();
                dysfunctional_detection
                    .add_a_pending_request_operation(*adult, op_id)
                    .await;
            }
        }

        // Assert there are not any unresponsive nodes
        // This is because all of them are within the tolerance ratio of each other
        assert_eq!(
            dysfunctional_detection
                .find_unresponsive_and_deviant_nodes()
                .await
                .0
                .len(),
            0
        );

        // Add a new adults
        let new_adult = XorName::random();
        dysfunctional_detection.add_new_node(new_adult);

        // Assert total adult count
        assert_eq!(dysfunctional_detection.closest_nodes_to.len(), 11);

        // Write data (EXCESSIVE_OPS_TOLERANCE/2) + 1 x MIN_PENDING_OPS times to the new adult to check for preemptive replication
        for _ in 0..MIN_PENDING_OPS * ((EXCESSIVE_OPS_TOLERANCE as usize / 2) + 1) {
            let op_id = get_random_operation_id();
            dysfunctional_detection
                .add_a_pending_request_operation(new_adult, op_id)
                .await;
        }

        // Assert that the new adult is detected as deviant.
        assert!(dysfunctional_detection
            .find_unresponsive_and_deviant_nodes()
            .await
            .1
            .iter()
            .contains(&new_adult));

        // Write data another EXCESSIVE_OPS_TOLERANCE x 50 times to the new adult to check for unresponsiveness.
        for _ in 0..MIN_PENDING_OPS * EXCESSIVE_OPS_TOLERANCE as usize {
            let op_id = get_random_operation_id();
            dysfunctional_detection
                .add_a_pending_request_operation(new_adult, op_id)
                .await;
        }

        let (unresponsive_nodes, _deviants) = dysfunctional_detection
            .find_unresponsive_and_deviant_nodes()
            .await;

        // Assert that the new adult is detected unresponsive.
        assert!(unresponsive_nodes.contains(&new_adult));

        Ok(())
    }

    #[tokio::test]
    async fn dysfunction_retain_members() -> Result<()> {
        let adults = (0..10).map(|_| XorName::random()).collect::<Vec<XorName>>();
        let dysfunctional_detection = DysfunctionDetection::new(adults.clone(), ELDER_COUNT);

        let live_adults = adults[5..10].iter().cloned().collect::<BTreeSet<XorName>>();

        dysfunctional_detection.retain_members_only(live_adults.clone());

        let all_known_nodes: Vec<_> = dysfunctional_detection.current_nodes();

        for member in all_known_nodes {
            assert!(live_adults.contains(&member));
        }

        Ok(())
    }

    #[tokio::test]
    async fn dysfunction_compute_closest() -> Result<()> {
        // Adults with prefix 0
        let mut adults0 = (0..10)
            .map(|_| xor_name::rand::random::<XorName>().with_bit(0, false))
            .collect::<Vec<XorName>>();

        // Adults with prefix 1
        let mut adults1 = (0..10)
            .map(|_| xor_name::rand::random::<XorName>().with_bit(0, true))
            .collect::<Vec<XorName>>();

        // Whole set of Adults
        let mut all_adults = vec![];
        all_adults.extend(adults0.clone());
        all_adults.extend(adults1.clone());

        let dysfunctional_detection = DysfunctionDetection::new(all_adults, ELDER_COUNT);

        for entry in dysfunctional_detection.closest_nodes_to.iter() {
            let (node, neighbours) = entry.pair();

            // Assert all the neigbours fall under the correct set
            if node.bit(0) {
                for neigbour in neighbours {
                    assert!(adults1.contains(neigbour))
                }
            } else {
                for neigbour in neighbours {
                    assert!(adults0.contains(neigbour))
                }
            }
        }

        // Add 5 new adults for each 0 and 1 prefix
        let new_adults0 = (0..5)
            .map(|_| xor_name::rand::random::<XorName>().with_bit(0, false))
            .collect::<Vec<XorName>>();

        let new_adults1 = (0..5)
            .map(|_| xor_name::rand::random::<XorName>().with_bit(0, true))
            .collect::<Vec<XorName>>();

        let mut new_adults = vec![];
        new_adults.extend(new_adults0.clone());
        new_adults.extend(new_adults1.clone());

        // Add them to the larger set
        adults0.extend(new_adults0);
        adults1.extend(new_adults1);

        // Add all of them to the dysfunction tracker
        for new_adult in new_adults {
            dysfunctional_detection.add_new_node(new_adult);
        }

        // Check if the recomputed nodes all fall within their respective sets
        for entry in dysfunctional_detection.closest_nodes_to.iter() {
            let (node, neighbours) = entry.pair();

            // Assert all the neighbours fall under the correct set
            if node.bit(0) {
                for neigbour in neighbours {
                    assert!(adults1.contains(neigbour))
                }
            } else {
                for neigbour in neighbours {
                    assert!(adults0.contains(neigbour))
                }
            }
        }

        Ok(())
    }
}
