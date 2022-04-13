// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{error::Result, get_mean_of, DysfunctionDetection};
use std::collections::{BTreeMap, BTreeSet};
use xor_name::XorName;

use std::time::Duration;
static RECENT_ISSUE_DURATION: Duration = Duration::from_secs(60 * 15);

static CONN_WEIGHTING: f32 = 20.0;
static OP_WEIGHTING: f32 = 1.5;
static KNOWLEDGE_WEIGHTING: f32 = 60.0;

// Ratio to mean scores should be over to be considered dys/sus
static DYSFUNCTION_MEAN_RATIO: f32 = 4.5;
static SUSPECT_MEAN_RATIO: f32 = 1.5;

#[derive(Clone, Debug)]
/// Represents the different type of issues that can be recorded by the Dysfunction Detection
/// system.
pub enum IssueType {
    /// Represents a communication issue to be tracked by Dysfunction Detection.
    Communication,
    /// Represents a knowledge issue to be tracked by Dysfunction Detection.
    Knowledge,
    /// Represents a pending request operation issue to be tracked by Dysfunction Detection.
    PendingRequestOperation,
}

#[derive(Debug)]
pub struct ScoreResults {
    pub communication_scores: BTreeMap<XorName, f32>,
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

        let adults = self
            .adults
            .read()
            .await
            .iter()
            .copied()
            .collect::<Vec<XorName>>();
        for node in adults.iter() {
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
                    &IssueType::PendingRequestOperation,
                )
                .await,
            );
        }
        ScoreResults {
            communication_scores,
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
            IssueType::Knowledge => {
                let count = if let Some(entry) = self.knowledge_issues.get(node) {
                    entry.value().read().await.len()
                } else {
                    1
                };
                count
            }
            IssueType::PendingRequestOperation => {
                let count = if let Some(entry) = self.unfulfilled_ops.get(node) {
                    entry.value().read().await.len()
                } else {
                    1
                };
                count
            }
        }
    }

    async fn get_weighted_scores(&self) -> (BTreeMap<XorName, f32>, f32) {
        trace!("Getting weighted scores");
        let scores = self.calculate_scores().await;
        let ops_scores = scores.op_scores;
        let conn_scores = scores.communication_scores;
        let knowledge_scores = scores.knowledge_scores;

        let mut final_scores = BTreeMap::default();
        let mut scores_only = vec![];
        // now we loop to get final scores
        for (name, score) in ops_scores {
            trace!("Ops sore: {name}, {score}");

            let ops_score = score * OP_WEIGHTING;

            let node_conn_score = *conn_scores.get(&name).unwrap_or(&1.0);
            let node_conn_score = node_conn_score * CONN_WEIGHTING;

            let node_knowledge_score = *knowledge_scores.get(&name).unwrap_or(&1.0);
            let node_knowledge_score = node_knowledge_score * KNOWLEDGE_WEIGHTING;

            trace!("Conns score: {name}, {node_conn_score}");
            trace!("Knowledge score: {name}, {node_knowledge_score}");
            let final_score = ops_score + node_conn_score + node_knowledge_score;

            scores_only.push(final_score);
            let _prev = final_scores.insert(name, final_score);
        }

        let mean = get_mean_of(&scores_only).unwrap_or(1.0);
        let mean = f32::max(mean, 1.0);

        debug!("avg weighted score: {mean}");
        (final_scores, mean)
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

        Ok(())
    }

    /// Get a list of nodes whose score is above mean * DysfunctionalSeverity weighting
    pub async fn get_nodes_beyond_severity(
        &self,
        severity: DysfunctionSeverity,
    ) -> Result<BTreeSet<XorName>> {
        self.cleanup_time_sensistive_checks().await?;

        let mut dysfunctional_nodes = BTreeSet::new();

        let (final_scores, mean) = self.get_weighted_scores().await;

        let to_beat = match severity {
            DysfunctionSeverity::Dysfunctional => mean * DYSFUNCTION_MEAN_RATIO,
            DysfunctionSeverity::Suspicious => mean * SUSPECT_MEAN_RATIO,
        };

        for (name, nodes_score) in final_scores {
            trace!(
                "Final {severity:?} score for {name} is {nodes_score} (mean is {mean}), needs to beat {:?}",
                to_beat
            );

            if nodes_score >= to_beat {
                debug!("DysfunctionDetection: Adding {name} as {severity:?} node");
                let _existed = dysfunctional_nodes.insert(name);
            }
        }

        Ok(dysfunctional_nodes)
    }
}

#[cfg(test)]
mod tests {
    use crate::{detection::IssueType, DysfunctionDetection};
    use proptest::prelude::*;
    use tokio::runtime::Runtime;
    use xor_name::{rand::random as random_xorname, XorName};

    fn issue_type_strategy() -> impl Strategy<Value = IssueType> {
        prop_oneof![
            Just(IssueType::Communication),
            Just(IssueType::Knowledge),
            Just(IssueType::PendingRequestOperation),
        ]
    }

    proptest! {
        #[test]
        fn pt_calculate_scores_should_include_all_nodes_in_score_map(
            adult_count in 4..50usize, issue_type in issue_type_strategy())
        {
            Runtime::new().unwrap().block_on(async {
                let adults = (0..adult_count).map(|_| random_xorname()).collect::<Vec<XorName>>();
                let dysfunctional_detection = DysfunctionDetection::new(adults.clone());
                let op_id = match issue_type {
                    IssueType::Communication | IssueType::Knowledge => {
                        None
                    },
                    IssueType::PendingRequestOperation => {
                        Some([1; 32])
                    }
                };
                for _ in 0..5 {
                    let _ = dysfunctional_detection.track_issue(
                        adults[0], issue_type.clone(), op_id).await;
                }

                let score_results = dysfunctional_detection
                    .calculate_scores()
                    .await;
                match issue_type {
                    IssueType::Communication => {
                        assert_eq!(score_results.communication_scores.len(), adult_count);
                    },
                    IssueType::Knowledge => {
                        assert_eq!(score_results.knowledge_scores.len(), adult_count);
                    },
                    IssueType::PendingRequestOperation => {
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
                let op_id = match issue_type {
                    IssueType::Communication | IssueType::Knowledge => {
                        None
                    },
                    IssueType::PendingRequestOperation => {
                        Some([1; 32])
                    }
                };
                for _ in 0..issue_count {
                    let _ = dysfunctional_detection.track_issue(
                        adults[0], issue_type.clone(), op_id).await;
                }

                let score_results = dysfunctional_detection
                    .calculate_scores()
                    .await;
                let scores = match issue_type {
                    IssueType::Communication => {
                        score_results.communication_scores
                    },
                    IssueType::Knowledge => {
                        score_results.knowledge_scores
                    },
                    IssueType::PendingRequestOperation => {
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
        fn pt_calculate_scores_when_all_nodes_have_the_same_number_of_issues_scores_should_all_be_one(
            adult_count in 4..50, issue_count in 0..50, issue_type in issue_type_strategy())
        {
            Runtime::new().unwrap().block_on(async {
                let adults = (0..adult_count).map(|_| random_xorname()).collect::<Vec<XorName>>();
                let dysfunctional_detection = DysfunctionDetection::new(adults.clone());
                let op_id = match issue_type {
                    IssueType::Communication | IssueType::Knowledge => {
                        None
                    },
                    IssueType::PendingRequestOperation => {
                        Some([1; 32])
                    }
                };
                for adult in adults.iter() {
                    for _ in 0..issue_count {
                        let _ = dysfunctional_detection.track_issue(
                            *adult, issue_type.clone(), op_id).await;
                    }
                }

                let score_results = dysfunctional_detection
                    .calculate_scores()
                    .await;
                let scores = match issue_type {
                    IssueType::Communication => {
                        score_results.communication_scores
                    },
                    IssueType::Knowledge => {
                        score_results.knowledge_scores
                    },
                    IssueType::PendingRequestOperation => {
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
