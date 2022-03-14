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

mod connections;
mod detection;
mod network_knowledge;
mod operations;

use xor_name::XorName;

use dashmap::DashMap;
use itertools::Itertools;
use std::collections::BTreeSet;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Some reproducible xorname derived from the operation. This is a permanent reference needed for logging all dysfunction.
type NodeIdentifier = XorName;

// re declaration here to not create circular dep w/ sn atm.
// TODO: depend on types once that's extracted
type OperationId = [u8; 32];

#[derive(Clone, Debug)]
/// Dysfunctional node tracking. Allows various potential issues to be tracked and weighted,
/// with unresposive or suspect nodes being noted on request, against which action can then be taken.
pub struct DysfunctionDetection {
    neighbour_count: usize,
    communication_issues: Arc<DashMap<NodeIdentifier, usize>>, // count of comm issues
    knowledge_issues: Arc<DashMap<NodeIdentifier, usize>>,     // count of comm issues
    unfulfilled_ops: Arc<DashMap<NodeIdentifier, Arc<RwLock<Vec<OperationId>>>>>, // OperationId = [u8; 32]
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
            knowledge_issues: Arc::new(DashMap::new()),
            unfulfilled_ops: Arc::new(DashMap::new()),
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
        debug!("Adding new adult:{adult} to DysfunctionDetection tracker");

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

        trace!("Closest nodes to {adult}:{closest_nodes:?}");

        if let Some(_old_entry) = self.closest_nodes_to.insert(adult, closest_nodes) {
            warn!("Throwing old dysfunction tracker for Adult {adult}:{_old_entry:?}");
        }

        self.recompute_closest_nodes();
    }

    /// Removes any tracked nodes not present in the passed `current_members`
    pub fn retain_members_only(&self, current_members: BTreeSet<XorName>) {
        let all_keys: Vec<_> = self.current_nodes();

        for key in &all_keys {
            if !current_members.contains(key) {
                let _prev = self.communication_issues.remove(key);
                let _prev = self.unfulfilled_ops.remove(key);
                let _prev = self.closest_nodes_to.remove(key);
            }
        }

        self.recompute_closest_nodes();
    }

    /// Recalculates the closest nodes
    fn recompute_closest_nodes(&self) {
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

/// Calculates the avg value in a data set
/// https://rust-lang-nursery.github.io/rust-cookbook/science/mathematics/statistics.html
pub(crate) fn get_mean_of(data: &[f32]) -> Option<f32> {
    let sum = data.iter().sum::<f32>();
    let count = data.len();
    match count {
        positive if positive > 0 => Some(sum / count as f32),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::DysfunctionDetection;

    use eyre::Error;
    use std::collections::BTreeSet;
    use std::sync::Once;
    use xor_name::XorName;

    type Result<T, E = Error> = std::result::Result<T, E>;

    pub(crate) static ELDER_COUNT: usize = 7;

    static INIT: Once = Once::new();

    /// Initialise logger for tests, this is run only once, even if called multiple times.
    pub(crate) fn init_test_logger() {
        INIT.call_once(|| {
            tracing_subscriber::fmt::fmt()
                // NOTE: uncomment this line for pretty printed log output.
                .with_thread_names(true)
                .with_ansi(false)
                .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
                .with_target(false)
                // .event_format(LogFormatter::default())
                .try_init().unwrap_or_else(|_| println!("Error initializing logger"));
        });
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
