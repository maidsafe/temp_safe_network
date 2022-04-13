// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! Dysfunctional "Node" detection for the SAFE Network.
//! All dysfunction detection should route through this for proper weighting in
//! relation to other possible sources of dysfunction

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
mod error;
mod network_knowledge;
mod operations;

use xor_name::XorName;

use crate::error::Result;
use dashmap::DashMap;
use std::collections::{BTreeSet, VecDeque};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

pub use crate::error::Error;
pub use detection::{DysfunctionSeverity, IssueType};

/// Some reproducible xorname derived from the operation. This is a permanent reference needed for logging all dysfunction.
type NodeIdentifier = XorName;

// re declaration here to not create circular dep w/ sn atm.
// TODO: depend on types once that's extracted
type OperationId = [u8; 32];

pub(crate) type TimedTracker = Arc<DashMap<NodeIdentifier, Arc<RwLock<VecDeque<Instant>>>>>;

#[derive(Clone, Debug)]
/// Dysfunctional node tracking. Allows various potential issues to be tracked and weighted,
/// with unresposive or suspect nodes being noted on request, against which action can then be taken.
pub struct DysfunctionDetection {
    /// The communication issues logged against a node, along with a timestamp.
    pub communication_issues: TimedTracker,
    /// The knowledge issues logged against a node, along with a timestamp.
    pub knowledge_issues: TimedTracker,
    /// The unfulfilled pending request operation issues logged against a node, along with an
    /// operation ID.
    pub unfulfilled_ops: Arc<DashMap<NodeIdentifier, Arc<RwLock<Vec<OperationId>>>>>, // OperationId = [u8; 32]
    adults: Arc<RwLock<Vec<XorName>>>,
}

impl DysfunctionDetection {
    /// Set up a new Dysfunctional Node Tracker.
    pub fn new(our_adults: Vec<NodeIdentifier>) -> Self {
        Self {
            communication_issues: Arc::new(DashMap::new()),
            knowledge_issues: Arc::new(DashMap::new()),
            unfulfilled_ops: Arc::new(DashMap::new()),
            adults: Arc::new(RwLock::new(our_adults)),
        }
    }

    /// Adds an issue to the dysfunction tracker.
    ///
    /// The `op_id` only applies when adding an operational issue.
    pub async fn track_issue(
        &self,
        node_id: NodeIdentifier,
        issue_type: IssueType,
        op_id: Option<[u8; 32]>,
    ) -> Result<()> {
        debug!("Adding a new issue to the dysfunction tracker: {issue_type:?}");
        match issue_type {
            IssueType::Communication => {
                if op_id.is_some() {
                    return Err(Error::UnusedOpIdSupplied(
                        "An operation ID only applies to pending unfulfilled requests.".to_string(),
                    ));
                }
                let mut entry = self.communication_issues.entry(node_id).or_default();
                let mut queue = entry.value_mut().write().await;
                queue.push_back(Instant::now());
            }
            IssueType::Knowledge => {
                if op_id.is_some() {
                    return Err(Error::UnusedOpIdSupplied(
                        "An operation ID only applies to pending unfulfilled requests.".to_string(),
                    ));
                }
                let mut entry = self.knowledge_issues.entry(node_id).or_default();
                let mut queue = entry.value_mut().write().await;
                queue.push_back(Instant::now());
            }
            IssueType::PendingRequestOperation => {
                let entry = self.unfulfilled_ops.entry(node_id).or_default();
                let v = entry.value();
                let op_id = op_id.ok_or_else(|| {
                    Error::OpIdNotSupplied(
                        "An operation ID must be supplied for a pending request operation."
                            .to_string(),
                    )
                })?;
                debug!("New issue has associated operation ID: {op_id:#?}");
                v.write().await.push(op_id);
            }
        }
        Ok(())
    }

    /// List all current tracked nodes
    pub async fn current_nodes(&self) -> Vec<XorName> {
        self.adults
            .read()
            .await
            .iter()
            .copied()
            .collect::<Vec<XorName>>()
    }

    /// Add a new node to the tracker and recompute closest nodes.
    pub async fn add_new_node(&self, adult: XorName) {
        debug!("Adding new adult:{adult} to DysfunctionDetection tracker");
        self.adults.write().await.push(adult);
    }

    /// Removes tracked nodes not present in `current_members`.
    ///
    /// Tracked issues related to nodes that were removed will also be removed.
    pub async fn retain_members_only(&self, current_members: BTreeSet<XorName>) {
        let mut nodes = self.adults.write().await;
        let nodes_being_removed = nodes
            .iter()
            .filter(|x| !current_members.contains(x))
            .copied()
            .collect::<Vec<XorName>>();
        nodes.retain(|x| current_members.contains(x));
        for node in nodes_being_removed.iter() {
            let _ = self.communication_issues.remove(node);
            let _ = self.knowledge_issues.remove(node);
            let _ = self.unfulfilled_ops.remove(node);
        }
    }
}

/// Calculates the avg value in a data set
/// https://rust-lang-nursery.github.io/rust-cookbook/science/mathematics/statistics.html
pub(crate) fn get_mean_of(data: &[f32]) -> Option<f32> {
    let sum: f32 = data.iter().sum();
    let count = data.len();
    if count > 0 {
        Some(sum / count as f32)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{DysfunctionDetection, IssueType};

    use eyre::Error;
    use std::collections::BTreeSet;
    use std::sync::Once;
    use xor_name::{rand::random as random_xorname, XorName};

    type Result<T, E = Error> = std::result::Result<T, E>;

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
    async fn retain_members_should_remove_other_adults() -> Result<()> {
        let adults = (0..10).map(|_| random_xorname()).collect::<Vec<XorName>>();
        let dysfunctional_detection = DysfunctionDetection::new(adults.clone());
        let nodes_to_retain = adults[5..10].iter().cloned().collect::<BTreeSet<XorName>>();

        let _ = dysfunctional_detection
            .retain_members_only(nodes_to_retain.clone())
            .await;

        let current_nodes = dysfunctional_detection.current_nodes().await;
        assert_eq!(current_nodes.len(), 5);
        for member in current_nodes {
            assert!(nodes_to_retain.contains(&member));
        }

        Ok(())
    }

    #[tokio::test]
    async fn retain_members_should_remove_issues_relating_to_nodes_not_retained() -> Result<()> {
        let adults = (0..10).map(|_| random_xorname()).collect::<Vec<XorName>>();
        let dysfunctional_detection = DysfunctionDetection::new(adults.clone());

        // Track some issues for nodes that are going to be removed.
        for adult in adults.iter().take(3) {
            let _ = dysfunctional_detection
                .track_issue(*adult, IssueType::Communication, None)
                .await;
            let _ = dysfunctional_detection
                .track_issue(*adult, IssueType::Knowledge, None)
                .await;
            let _ = dysfunctional_detection
                .track_issue(*adult, IssueType::PendingRequestOperation, None)
                .await;
        }

        // Track some issues for nodes that will be retained.
        let _ = dysfunctional_detection
            .track_issue(adults[5], IssueType::Communication, None)
            .await;
        let _ = dysfunctional_detection
            .track_issue(adults[6], IssueType::Knowledge, None)
            .await;
        let _ = dysfunctional_detection
            .track_issue(adults[7], IssueType::PendingRequestOperation, None)
            .await;

        let nodes_to_retain = adults[5..10].iter().cloned().collect::<BTreeSet<XorName>>();

        let _ = dysfunctional_detection
            .retain_members_only(nodes_to_retain.clone())
            .await;

        assert_eq!(dysfunctional_detection.communication_issues.len(), 1);
        assert_eq!(dysfunctional_detection.knowledge_issues.len(), 1);
        assert_eq!(dysfunctional_detection.unfulfilled_ops.len(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn track_issue_should_add_a_comm_issue() -> Result<()> {
        let adults = (0..10).map(|_| random_xorname()).collect::<Vec<XorName>>();
        let dysfunctional_detection = DysfunctionDetection::new(adults.clone());

        dysfunctional_detection
            .track_issue(adults[0], IssueType::Communication, None)
            .await?;

        assert_eq!(dysfunctional_detection.communication_issues.len(), 1);
        assert_eq!(dysfunctional_detection.knowledge_issues.len(), 0);
        assert_eq!(dysfunctional_detection.unfulfilled_ops.len(), 0);
        Ok(())
    }

    #[tokio::test]
    async fn track_issue_should_return_error_when_adding_a_comm_issue_with_op_id() -> Result<()> {
        let adults = (0..10).map(|_| random_xorname()).collect::<Vec<XorName>>();
        let dysfunctional_detection = DysfunctionDetection::new(adults.clone());

        let result = dysfunctional_detection
            .track_issue(adults[0], IssueType::Communication, Some([1; 32]))
            .await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "An operation ID only applies to pending unfulfilled requests."
        );
        Ok(())
    }

    #[tokio::test]
    async fn track_issue_should_add_a_knowledge_issue() -> Result<()> {
        let adults = (0..10).map(|_| random_xorname()).collect::<Vec<XorName>>();
        let dysfunctional_detection = DysfunctionDetection::new(adults.clone());

        dysfunctional_detection
            .track_issue(adults[0], IssueType::Knowledge, None)
            .await?;

        assert_eq!(dysfunctional_detection.knowledge_issues.len(), 1);
        assert_eq!(dysfunctional_detection.communication_issues.len(), 0);
        assert_eq!(dysfunctional_detection.unfulfilled_ops.len(), 0);
        Ok(())
    }

    #[tokio::test]
    async fn track_issue_should_return_error_when_adding_a_knowledge_issue_with_op_id() -> Result<()>
    {
        let adults = (0..10).map(|_| random_xorname()).collect::<Vec<XorName>>();
        let dysfunctional_detection = DysfunctionDetection::new(adults.clone());

        let result = dysfunctional_detection
            .track_issue(adults[0], IssueType::Knowledge, Some([1; 32]))
            .await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "An operation ID only applies to pending unfulfilled requests."
        );
        Ok(())
    }

    #[tokio::test]
    async fn track_issue_should_add_a_pending_op_issue() -> Result<()> {
        let adults = (0..10).map(|_| random_xorname()).collect::<Vec<XorName>>();
        let dysfunctional_detection = DysfunctionDetection::new(adults.clone());

        dysfunctional_detection
            .track_issue(adults[0], IssueType::PendingRequestOperation, Some([1; 32]))
            .await?;

        assert_eq!(dysfunctional_detection.unfulfilled_ops.len(), 1);
        assert_eq!(dysfunctional_detection.knowledge_issues.len(), 0);
        assert_eq!(dysfunctional_detection.communication_issues.len(), 0);
        Ok(())
    }

    #[tokio::test]
    async fn track_issue_should_return_error_when_adding_pending_op_issue_without_op_id(
    ) -> Result<()> {
        let adults = (0..10).map(|_| random_xorname()).collect::<Vec<XorName>>();
        let dysfunctional_detection = DysfunctionDetection::new(adults.clone());

        let result = dysfunctional_detection
            .track_issue(adults[0], IssueType::PendingRequestOperation, None)
            .await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "An operation ID must be supplied for a pending request operation."
        );
        Ok(())
    }

    #[tokio::test]
    async fn add_new_node_should_track_new_node() -> Result<()> {
        let adults = (0..10).map(|_| random_xorname()).collect::<Vec<XorName>>();
        let dysfunctional_detection = DysfunctionDetection::new(adults.clone());

        let new_adult = random_xorname();
        let _ = dysfunctional_detection.add_new_node(new_adult).await;

        let current_nodes = dysfunctional_detection.current_nodes().await;

        assert_eq!(current_nodes.len(), 11);
        Ok(())
    }
}
