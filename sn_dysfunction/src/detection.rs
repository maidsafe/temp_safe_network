// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{error::Result, get_mean_of, std_deviation, DysfunctionDetection, OperationId};

use std::collections::{BTreeMap, BTreeSet};
use xor_name::XorName;

use std::time::Duration;
static RECENT_ISSUE_DURATION: Duration = Duration::from_secs(60 * 10); // 10 minutes

static CONN_WEIGHTING: f32 = 2.0;
static OP_WEIGHTING: f32 = 1.0;
static KNOWLEDGE_WEIGHTING: f32 = 3.0;
static DKG_WEIGHTING: f32 = 4.0;

/// Z-score value above which a node is dysfunctional
static DYSFUNCTIONAL_DEVIATION: f32 = 2.6;

#[derive(Clone, Debug)]
/// Represents the different type of issues that can be recorded by the Dysfunction Detection
/// system.
/// Issues have a xorname so they can be reliable assignd to the same nodes
pub enum IssueType {
    /// Represents a Dkg issue to be tracked by Dysfunction Detection.
    Dkg,
    /// Represents a communication issue to be tracked by Dysfunction Detection.
    Communication,
    /// Represents a knowledge issue to be tracked by Dysfunction Detection.
    Knowledge,
    /// Represents a pending request operation issue to be tracked by Dysfunction Detection.
    PendingRequestOperation(Option<OperationId>),
}

#[derive(Debug)]
pub struct ScoreResults {
    pub communication_scores: BTreeMap<XorName, f32>,
    pub dkg_scores: BTreeMap<XorName, f32>,
    pub knowledge_scores: BTreeMap<XorName, f32>,
    pub op_scores: BTreeMap<XorName, f32>,
}

/// Severity of dysfunction... Is it not yet fully dysfunctional? But out of line with neighbours?
/// Then it's Suspicious, or, if it's gone too far we can check if it's Dysfunctional.
/// These can be passed in to the `check_for_maliciousness` function.
#[derive(Debug)]
pub enum DysfunctionSeverity {
    /// A node is deemed suspicous (more dysfunction than neighbours, but not yet fully dysfunctional)
    Suspicious,
    /// A node is deemed dysfunctional when it is clearly having more issues than it's beighbours
    Dysfunctional,
}

impl DysfunctionDetection {
    /// Calculate the scores of all nodes being tracked and return them in a node -> score map.
    /// There is a map for each type of issue.
    ///
    /// The 'score' for a node is the number of issues logged against that node, minus the average
    /// of the number of issues at all the other nodes.
    ///
    /// These scores can then be used to highlight nodes that have a higher score than some
    /// particular ratio.
    pub async fn calculate_scores(&self) -> ScoreResults {
        let mut communication_scores = BTreeMap::new();
        let mut knowledge_scores = BTreeMap::new();
        let mut op_scores = BTreeMap::new();
        let mut dkg_scores = BTreeMap::new();

        let adults = self
            .adults
            .read()
            .await
            .iter()
            .copied()
            .collect::<Vec<XorName>>();
        for node in adults.iter() {
            let _ = dkg_scores.insert(
                *node,
                self.calculate_node_score(node, adults.clone(), &IssueType::Dkg)
                    .await,
            );
            let _ = communication_scores.insert(
                *node,
                self.calculate_node_score(node, adults.clone(), &IssueType::Communication)
                    .await,
            );
            let _ = knowledge_scores.insert(
                *node,
                self.calculate_node_score(node, adults.clone(), &IssueType::Knowledge)
                    .await,
            );
            let _ = op_scores.insert(
                *node,
                self.calculate_node_score(
                    node,
                    adults.clone(),
                    &IssueType::PendingRequestOperation(None),
                )
                .await,
            );
        }
        ScoreResults {
            communication_scores,
            dkg_scores,
            knowledge_scores,
            op_scores,
        }
    }

    async fn calculate_node_score(
        &self,
        node: &XorName,
        adults: Vec<XorName>,
        issue_type: &IssueType,
    ) -> f32 {
        let node_count = self.get_node_issue_count(node, issue_type).await;
        let mut other_node_counts = Vec::new();
        for adult in adults.clone() {
            if adult == *node {
                continue;
            }
            other_node_counts.push(self.get_node_issue_count(&adult, issue_type).await as f32);
        }
        let average = get_mean_of(&other_node_counts).unwrap_or(1.0);
        let score = node_count.checked_sub(average as usize).unwrap_or(1) as f32;
        if score < 1.0 {
            1.0
        } else {
            score
        }
    }

    async fn get_node_issue_count(&self, node: &XorName, issue_type: &IssueType) -> usize {
        match issue_type {
            IssueType::Communication => {
                let count = if let Some(entry) = self.communication_issues.get(node) {
                    entry.value().read().await.len()
                } else {
                    1
                };
                count
            }
            IssueType::Dkg => {
                let count = if let Some(entry) = self.dkg_issues.get(node) {
                    entry.value().read().await.len()
                } else {
                    1
                };
                count
            }
            IssueType::Knowledge => {
                let count = if let Some(entry) = self.knowledge_issues.get(node) {
                    entry.value().read().await.len()
                } else {
                    1
                };
                count
            }
            IssueType::PendingRequestOperation(_) => {
                let count = if let Some(entry) = self.unfulfilled_ops.get(node) {
                    entry.value().read().await.len()
                } else {
                    1
                };
                count
            }
        }
    }

    /// get scores mapped by name, to score and z-score, which is std dev's from the mean
    async fn get_weighted_scores(&self) -> BTreeMap<XorName, Option<f32>> {
        trace!("Getting weighted scores");
        let scores = self.calculate_scores().await;
        let ops_scores = scores.op_scores;
        let conn_scores = scores.communication_scores;
        let dkg_scores = scores.dkg_scores;
        let knowledge_scores = scores.knowledge_scores;

        let mut pre_z_scores = BTreeMap::default();
        let mut scores_only = vec![];
        // now we loop to get the scores per xorname, so we can then avg etc
        for (name, score) in ops_scores {
            trace!("Ops sore: {name}, {score}");

            let ops_score = score * OP_WEIGHTING;

            let node_conn_score = *conn_scores.get(&name).unwrap_or(&1.0);
            let node_conn_score = node_conn_score * CONN_WEIGHTING;

            let node_dkg_score = *dkg_scores.get(&name).unwrap_or(&1.0);
            let node_dkg_score = node_dkg_score * DKG_WEIGHTING;

            let node_knowledge_score = *knowledge_scores.get(&name).unwrap_or(&1.0);
            let node_knowledge_score = node_knowledge_score * KNOWLEDGE_WEIGHTING;

            trace!("Conns score: {name}, {node_conn_score}");
            trace!("Knowledge score: {name}, {node_knowledge_score}");
            trace!("Dkg score: {name}, {node_dkg_score}");
            let final_score = ops_score + node_conn_score + node_knowledge_score + node_dkg_score;

            scores_only.push(final_score);
            let _prev = pre_z_scores.insert(name, final_score);
        }

        let mean = get_mean_of(&scores_only);
        let std_dev = std_deviation(&scores_only);

        trace!("avg weighted score across all nodes: {mean:?}");
        trace!("std dev: {std_dev:?}");

        // now we store the z-score
        let mut final_scores = BTreeMap::default();

        for (name, score) in pre_z_scores {
            let zscore = match (mean, std_dev) {
                (Some(mean), Some(std_deviation)) => {
                    let diff = score - mean;

                    if diff < 1.0 {
                        // small or negative diff mean nodes are doing well
                        None
                    } else {
                        Some(diff / std_deviation)
                    }
                }
                _ => None,
            };

            trace!("Final Z-score for {name} is {zscore:?}");

            let _existed = final_scores.insert(name, zscore);
        }

        final_scores
    }

    async fn cleanup_time_sensistive_checks(&self) -> Result<()> {
        for node in self.communication_issues.iter() {
            let mut issues = node.value().write().await;
            issues.retain(|time| time.elapsed() < RECENT_ISSUE_DURATION);
        }

        for node in self.knowledge_issues.iter() {
            let mut issues = node.value().write().await;
            issues.retain(|time| time.elapsed() < RECENT_ISSUE_DURATION);
        }

        for node in self.dkg_issues.iter() {
            let mut issues = node.value().write().await;
            issues.retain(|time| time.elapsed() < RECENT_ISSUE_DURATION);
        }

        Ok(())
    }

    /// Get a list of nodes whose score is above mean * DysfunctionalSeverity weighting
    pub async fn get_nodes_beyond_severity(
        &self,
        severity: DysfunctionSeverity,
    ) -> Result<BTreeSet<XorName>> {
        self.cleanup_time_sensistive_checks().await?;

        let mut dysfunctional_nodes = BTreeSet::new();

        let final_scores = self.get_weighted_scores().await;

        for (name, node_zscore) in final_scores {
            if let Some(z) = node_zscore {
                if z > DYSFUNCTIONAL_DEVIATION {
                    info!("DysfunctionDetection: Adding {name} as {severity:?} node");
                    let _existed = dysfunctional_nodes.insert(name);
                }
            }
        }

        Ok(dysfunctional_nodes)
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::{
        detection::IssueType, tests::init_test_logger, DysfunctionDetection, DysfunctionSeverity,
    };
    use sn_interface::messaging::data::OperationId;

    use eyre::bail;
    use proptest::prelude::*;
    use rand::Rng;
    use tokio::runtime::Runtime;
    use xor_name::{rand::random as random_xorname, XorName};

    fn issue_type_strategy() -> impl Strategy<Value = IssueType> {
        // Weighted issues to be (vaguely) representative of a real network
        // (TODO: take stats from section)
        // higher numbers here are more frequent
        prop_oneof![
            30 => Just(IssueType::Communication),
            5 => Just(IssueType::Dkg),
            10 => Just(IssueType::Knowledge),
            100 => (any::<[u8; 32]>())
                .prop_map(|x| IssueType::PendingRequestOperation(Some(OperationId(x))))
        ]
    }

    /// Generate proptest issues, in a range from 1000 to...max_quantity
    fn generate_issues(min: usize, max: usize) -> impl Strategy<Value = Vec<(IssueType, XorName)>> {
        let issue_name_for_direction = generate_xorname();
        prop::collection::vec(
            (issue_type_strategy(), issue_name_for_direction),
            min..max + 1,
        )
    }

    fn generate_xorname() -> impl Strategy<Value = XorName> {
        // get a random string
        let str_val = "[a-z]{1,4}\\p{Cyrillic}{1,4}\\p{Greek}{1,32}";
        str_val.prop_map(|s| XorName::from_content(s.as_bytes()))
    }

    /// Generate proptest nodes, each a Xorname
    fn generate_nodes(min: usize, max: usize) -> impl Strategy<Value = Vec<XorName>> {
        prop::collection::vec(generate_xorname(), min..max)
    }

    #[derive(Debug, Clone)]
    enum NodeQuality {
        Bad,
        Good,
    }

    proptest! {
        #[test]
        fn pt_calculate_scores_should_include_all_nodes_in_score_map(
            adult_count in 4..50usize, issue_type in issue_type_strategy())
        {
            Runtime::new().unwrap().block_on(async {
                let adults = (0..adult_count).map(|_| random_xorname()).collect::<Vec<XorName>>();
                let dysfunctional_detection = DysfunctionDetection::new(adults.clone());
                for _ in 0..5 {
                    let _ = dysfunctional_detection.track_issue(
                        adults[0], issue_type.clone()).await;
                }

                let score_results = dysfunctional_detection
                    .calculate_scores()
                    .await;
                match issue_type {
                    IssueType::Dkg => {
                        assert_eq!(score_results.dkg_scores.len(), adult_count);
                    },
                    IssueType::Communication => {
                        assert_eq!(score_results.communication_scores.len(), adult_count);
                    },
                    IssueType::Knowledge => {
                        assert_eq!(score_results.knowledge_scores.len(), adult_count);
                    },
                    IssueType::PendingRequestOperation(_) => {
                        assert_eq!(score_results.op_scores.len(), adult_count);
                    },
                }
            })
        }

        #[test]
        fn pt_calculate_scores_one_node_with_issues_should_have_higher_score_and_others_should_have_one(
            adult_count in 4..50usize, issue_count in 0..50, issue_type in issue_type_strategy())
        {
            Runtime::new().unwrap().block_on(async {
                let adults = (0..adult_count).map(|_| random_xorname()).collect::<Vec<XorName>>();
                let dysfunctional_detection = DysfunctionDetection::new(adults.clone());
                for _ in 0..issue_count {
                    let _ = dysfunctional_detection.track_issue(
                        adults[0], issue_type.clone()).await;
                }

                let score_results = dysfunctional_detection
                    .calculate_scores()
                    .await;
                let scores = match issue_type {
                    IssueType::Dkg => {
                        score_results.dkg_scores
                    },
                    IssueType::Communication => {
                        score_results.communication_scores
                    },
                    IssueType::Knowledge => {
                        score_results.knowledge_scores
                    },
                    IssueType::PendingRequestOperation(_) => {
                        score_results.op_scores
                    },
                };
                let expected_score = if issue_count > 1 {
                    issue_count - 1
                } else {
                    1
                };
                assert_eq!(*scores.get(&adults[0]).unwrap(), expected_score as f32);
                for adult in adults.iter().take(adult_count).skip(1) {
                    assert_eq!(*scores.get(adult).unwrap(), 1.0);
                }
            })
        }


        #[test]
        /// Test that gives a range of nodes and a few bad nodes,
        /// we then check that we can reliably detect those nodes
        ///
        /// We do not want false positives, we do want -- over longer timeframes -- to find all bad nodes... there's a tough balance to strike here.
        ///
        /// "Nodes" are just random xornames,
        /// each issue has a random xorname attached to it to, and is sent to 4 nodes... each of which will fail a % of the time, depending on the
        /// NodeQuality (Good or Bad)
        fn pt_correct_or_less_amount_of_dysf_nodes_should_be_detected(
            good_nodes in generate_nodes(5,23), bad_nodes in generate_nodes(1,7), issues in generate_issues(1000,3000))
            {

                let good_len =good_nodes.len();
                let bad_len = bad_nodes.len();
            // finish early as we're over byzantine levels
            if good_len / 3 <= bad_len {
                return Ok(());
            }

            init_test_logger();
            let _outer_span = tracing::info_span!("pt_correct_amount_of_dysf_nodes_should_be_detected").entered();

            // tolerances...
            // So here a dysf node fails 65% of the time
            const DYSF_SUCCESS_RATIO : f32 = 0.35;
            const NORMAL_SUCCESS_RATIO: f32 = 0.95;

            let _res = Runtime::new().unwrap().block_on(async {
                // add dysf to our all_nodes
                let mut all_node_names= good_nodes.clone();
                all_node_names.extend(bad_nodes.clone());

                let all_nodes = good_nodes.clone().iter().map(|name| (*name, NodeQuality::Good)).collect::<Vec<(XorName, NodeQuality)>>();
                let bad_nodes = bad_nodes.iter().map(|name| (*name, NodeQuality::Bad)).collect::<Vec<(XorName, NodeQuality)>>();
                all_nodes.clone().extend(bad_nodes);

                let dysfunctional_detection = DysfunctionDetection::new(all_node_names);
                let mut rng = rand::thread_rng();

                // Now we loop through each issue/msg
                for (issue, issue_location ) in issues {

                    // we use the "issue location" to determine which four nodes to send to
                    // this should therefore be reproducible amongst proptest retries/shrinking etc
                    let target_nodes = all_nodes
                    .iter()
                    .sorted_by(|lhs, rhs| issue_location.cmp_distance(&lhs.0, &rhs.0))
                    // and we simul-send it to 4 nodes
                    .take(4)
                    .collect::<Vec<_>>();

                    // now we track our issue, but only if that node fails to passes muster...
                    for (node, quality) in target_nodes {
                        // if our random fail test is larger than the quality success rate.
                        // it's a fail
                        let fail_test: f32 = rng.gen_range(0.0..1.0);
                        let q_value = match quality {
                            NodeQuality::Good => NORMAL_SUCCESS_RATIO,
                            NodeQuality::Bad => DYSF_SUCCESS_RATIO,
                        };
                        if fail_test > q_value {
                            let _ = dysfunctional_detection.track_issue(
                                *node, issue.clone()).await;
                        }

                    }
                }
                // now we can see what we have...
                let dysfunctional_nodes_found = match dysfunctional_detection
                    .get_nodes_beyond_severity( DysfunctionSeverity::Dysfunctional)
                    .await {
                        Ok(nodes) => nodes,
                        Err(error) => bail!("Failed getting dysfunctional nodes from DysfunctionDetector: {error}")
                    };

                info!("======================");
                info!("dysf found len {:?}: {:?}", dysfunctional_nodes_found.len(), dysfunctional_nodes_found);
                info!("======================");

                // over a long enough time span, we should catch those bad nodes...
                // So long as dysfunction isn't returning _more_ than the bad node count, this can pass
                assert!(dysfunctional_nodes_found.len() <= bad_len, "checking {} dysf nodes found is equal or less than the {} actual bad nodes in test", dysfunctional_nodes_found.len(), bad_len);
                Ok(())
            });
        }

        #[test]
        fn pt_calculate_scores_when_all_nodes_have_the_same_number_of_issues_scores_should_all_be_one(
            adult_count in 4..50, issue_count in 0..50, issue_type in issue_type_strategy())
        {
            Runtime::new().unwrap().block_on(async {
                let adults = (0..adult_count).map(|_| random_xorname()).collect::<Vec<XorName>>();
                let dysfunctional_detection = DysfunctionDetection::new(adults.clone());
                for adult in adults.iter() {
                    for _ in 0..issue_count {
                        let _ = dysfunctional_detection.track_issue(
                            *adult, issue_type.clone()).await;
                    }
                }

                let score_results = dysfunctional_detection
                    .calculate_scores()
                    .await;
                let scores = match issue_type {
                    IssueType::Communication => {
                        score_results.communication_scores
                    },
                    IssueType::Dkg => {
                        score_results.dkg_scores
                    },
                    IssueType::Knowledge => {
                        score_results.knowledge_scores
                    },
                    IssueType::PendingRequestOperation(_) => {
                        score_results.op_scores
                    },
                };
                for adult in adults.iter() {
                    assert_eq!(*scores.get(adult).unwrap(), 1.0);
                }
            })
        }
    }
}

#[cfg(test)]
mod ops_tests {
    use crate::{error::Result, DysfunctionDetection, DysfunctionSeverity, IssueType, OperationId};
    use rand::Rng;
    use xor_name::{rand::random as random_xorname, XorName};

    // some example numbers as guidance
    // we can see 500 pending issues under load
    pub(crate) const NORMAL_OPERATIONS_ISSUES: usize = 500;

    fn get_random_operation_id() -> Option<OperationId> {
        let mut rng = rand::thread_rng();
        Some(OperationId(rng.gen()))
    }

    #[tokio::test]
    async fn op_dysfunction_no_variance_is_okay() -> Result<()> {
        let adults = (0..10).map(|_| random_xorname()).collect::<Vec<XorName>>();
        let dysfunctional_detection = DysfunctionDetection::new(adults.clone());
        for adult in &adults {
            for _ in 0..NORMAL_OPERATIONS_ISSUES {
                let _ = dysfunctional_detection
                    .track_issue(
                        *adult,
                        IssueType::PendingRequestOperation(get_random_operation_id()),
                    )
                    .await;
            }
        }

        assert_eq!(
            dysfunctional_detection
                .get_nodes_beyond_severity(DysfunctionSeverity::Dysfunctional)
                .await?
                .len(),
            0
        );
        assert_eq!(
            dysfunctional_detection
                .get_nodes_beyond_severity(DysfunctionSeverity::Suspicious)
                .await?
                .len(),
            0
        );

        Ok(())
    }
}

#[cfg(test)]
mod comm_tests {
    use crate::{DysfunctionDetection, DysfunctionSeverity, IssueType};

    use eyre::Error;
    use xor_name::{rand::random as random_xorname, XorName};

    type Result<T, E = Error> = std::result::Result<T, E>;

    // Above this, nodes should be sus
    // this is only counting last RECENT minutes atm
    pub(crate) const NORMAL_CONNECTION_PROBLEM_COUNT: usize = 50;

    #[tokio::test]
    async fn conn_dys_is_tolerant_of_norms() -> Result<()> {
        let adults = (0..10).map(|_| random_xorname()).collect::<Vec<XorName>>();

        let dysfunctional_detection = DysfunctionDetection::new(adults.clone());

        for adult in &adults {
            for _ in 0..NORMAL_CONNECTION_PROBLEM_COUNT {
                dysfunctional_detection
                    .track_issue(*adult, IssueType::Communication)
                    .await?;
            }
        }

        assert_eq!(
            dysfunctional_detection
                .get_nodes_beyond_severity(DysfunctionSeverity::Dysfunctional)
                .await?
                .len(),
            0,
            "no nodes are dysfunctional"
        );
        assert_eq!(
            dysfunctional_detection
                .get_nodes_beyond_severity(DysfunctionSeverity::Suspicious)
                .await?
                .len(),
            0,
            "no nodes are suspect"
        );

        Ok(())
    }
}

#[cfg(test)]
mod knowledge_tests {
    use crate::tests::init_test_logger;
    use crate::{DysfunctionDetection, DysfunctionSeverity, IssueType};

    use eyre::Error;
    use xor_name::{rand::random as random_xorname, XorName};

    type Result<T, E = Error> = std::result::Result<T, E>;

    // some example numbers as guidance
    // 5 here means we have some tolerance for AE rounds while nodes are getting up to speed on churn/split
    pub(crate) const NORMAL_KNOWLEDGE_ISSUES: usize = 70;

    #[tokio::test]
    async fn knowledge_dys_is_tolerant_of_norms() -> Result<()> {
        let adults = (0..10).map(|_| random_xorname()).collect::<Vec<XorName>>();

        let dysfunctional_detection = DysfunctionDetection::new(adults.clone());

        // Write data NORMAL_KNOWLEDGE_ISSUES times to the 10 adults
        for adult in &adults {
            for _ in 0..NORMAL_KNOWLEDGE_ISSUES {
                dysfunctional_detection
                    .track_issue(*adult, IssueType::Knowledge)
                    .await?;
            }
        }

        // Assert there are not any dysfuncitonal nodes
        // This is because all of them are within the tolerance ratio of each other
        assert_eq!(
            dysfunctional_detection
                .get_nodes_beyond_severity(DysfunctionSeverity::Dysfunctional)
                .await?
                .len(),
            0,
            "no nodes are dysfunctional"
        );
        assert_eq!(
            dysfunctional_detection
                .get_nodes_beyond_severity(DysfunctionSeverity::Suspicious)
                .await?
                .len(),
            0,
            "no nodes are suspect"
        );

        Ok(())
    }

    #[tokio::test]
    async fn knowledge_dys_is_not_too_sharp() -> Result<()> {
        init_test_logger();
        let _outer_span = tracing::info_span!("knowledge_dys_is_not_too_sharp").entered();

        let adults = (0..10).map(|_| random_xorname()).collect::<Vec<XorName>>();

        let dysfunctional_detection = DysfunctionDetection::new(adults.clone());

        // Add a new adults
        let new_adult = random_xorname();
        let _ = dysfunctional_detection.add_new_node(new_adult).await;

        // Add just one knowledge issue...
        for _ in 0..1 {
            dysfunctional_detection
                .track_issue(new_adult, IssueType::Knowledge)
                .await?;
        }

        let sus = dysfunctional_detection
            .get_nodes_beyond_severity(DysfunctionSeverity::Suspicious)
            .await?;

        // Assert that the new adult is not detected as suspect.
        assert!(!sus.contains(&new_adult), "our adult should not be sus");
        assert_eq!(sus.len(), 0, "no node is sus");

        let dysfunctional_nodes = dysfunctional_detection
            .get_nodes_beyond_severity(DysfunctionSeverity::Dysfunctional)
            .await?;

        // Assert that the new adult is not dysfuncitonal
        assert!(
            !dysfunctional_nodes.contains(&new_adult),
            "our adult should not be dysfunctional"
        );
        assert_eq!(
            dysfunctional_nodes.len(),
            0,
            "no node is dysfunctional node"
        );

        Ok(())
    }
}
