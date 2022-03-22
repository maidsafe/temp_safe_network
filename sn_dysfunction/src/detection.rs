// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{error::Result, get_mean_of, DysfunctionDetection, TimedTracker};
use std::collections::{BTreeMap, BTreeSet};
use xor_name::XorName;

use std::time::{Duration, SystemTime};
static RECENT_ISSUE_DURATION: Duration = Duration::from_secs(60 * 15);

static CONN_WEIGHTING: f32 = 25.0;
static OP_WEIGHTING: f32 = 0.02;
static KNOWLEDGE_WEIGHTING: f32 = 35.0;

// Ratio to mean scores should be over to be considered dys/sus
static DYSFUNCTION_MEAN_RATIO: f32 = 8.0;
static SUSPECT_MEAN_RATIO: f32 = 3.0;

#[derive(Clone)]
enum ScoreType {
    Timed(TimedTracker),
    Op,
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
    /// Helper func to get vec of a node and their neighbours for comparison
    pub fn get_node_and_neighbours_vec(&self) -> Vec<(XorName, Vec<XorName>)> {
        let mut node_neighbours = vec![];
        for entry in self.closest_nodes_to.iter() {
            let (node, neighbours) = entry.pair();
            node_neighbours.push((*node, neighbours.clone()));
        }

        node_neighbours
    }

    /// Calculate a score of this node, as compared to its closest neighbours...
    async fn calculate_scores(&self, score_type: ScoreType) -> BTreeMap<XorName, f32> {
        let mut score_map = BTreeMap::default();

        // loop over all node/neighbour comparisons
        for (node, neighbours) in self.get_node_and_neighbours_vec() {
            let score_type = score_type.clone();

            let (count_at_node, all_neighbourhood_counts) = match score_type {
                ScoreType::Timed(tracker) => {
                    let count = if let Some(entry) = tracker.get(&node) {
                        entry.value().read().await.len()
                    } else {
                        1
                    };

                    let mut all_neighbourhood_counts = vec![];
                    for neighbour in neighbours {
                        if let Some(entry) = tracker.get(&neighbour) {
                            if neighbour == node {
                                continue;
                            }

                            let val = entry.value().read().await.len();

                            all_neighbourhood_counts.push(val as f32);
                        }
                    }

                    (count, all_neighbourhood_counts)
                }
                ScoreType::Op => {
                    let count = if let Some(entry) = self.unfulfilled_ops.get(&node) {
                        entry.value().read().await.len()
                    } else {
                        1
                    };

                    let mut all_neighbourhood_counts = vec![];
                    for neighbour in neighbours {
                        if let Some(entry) = self.unfulfilled_ops.get(&neighbour) {
                            let val = entry.value().read().await.len();

                            all_neighbourhood_counts.push(val as f32);
                        }
                    }

                    (count, all_neighbourhood_counts)
                }
            };

            let avg_in_neighbourhood = get_mean_of(&all_neighbourhood_counts).unwrap_or(1.0);

            trace!(
                "node's ops {count_at_node:?} mean ops: {:?}",
                avg_in_neighbourhood
            );

            let final_score = count_at_node
                .checked_sub(avg_in_neighbourhood as usize)
                .unwrap_or(1) as f32;

            let _prev = score_map.insert(node, final_score);
        }

        score_map
    }

    async fn get_weighted_scores(&self) -> (BTreeMap<XorName, f32>, f32) {
        trace!("Getting weighted scores");
        let ops_scores = self.calculate_scores(ScoreType::Op).await;

        let conn_scores = self
            .calculate_scores(ScoreType::Timed(self.communication_issues.clone()))
            .await;
        let knowledge_scores = self
            .calculate_scores(ScoreType::Timed(self.knowledge_issues.clone()))
            .await;

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
        // first remove anything older than RECENT_ISSUE_DURATION from the timed trackers
        if let Some(time_check) = SystemTime::now().checked_sub(RECENT_ISSUE_DURATION) {
            let time_check = time_check.duration_since(SystemTime::UNIX_EPOCH)?.as_secs();

            // remove old comms issues
            for node in self.communication_issues.iter() {
                let mut issues = node.value().write().await;
                issues.retain(|time| time > &time_check);
            }

            // remove old knowledge issues
            for node in self.knowledge_issues.iter() {
                let mut issues = node.value().write().await;
                issues.retain(|time| time > &time_check);
            }
        }

        Ok(())
    }

    /// Get a list of all nodes who'se score is above mean * DysfunctionalSeverity weighting
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
