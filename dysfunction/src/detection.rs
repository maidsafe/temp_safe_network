// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{get_mean_of, BasicCountTracker, DysfunctionDetection};
use std::collections::{BTreeMap, BTreeSet};
use xor_name::XorName;

static CONN_WEIGHTING: f32 = 1.2;
static OP_WEIGHTING: f32 = 1.0;
static KNOWLEDGE_WEIGHTING: f32 = 1.3;

// Number of mean scores to be over to be considered dys/sus
static DYSFUNCTION_MEAN_RATIO: f32 = 2.75;
static SUSPECT_MEAN_RATIO: f32 = 1.2;

#[derive(Clone)]
enum ScoreType {
    Basic(BasicCountTracker),
    Op,
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
                ScoreType::Basic(tracker) => {
                    let count = tracker.get(&node).map(|entry| *entry.value()).unwrap_or(0);

                    let all_neighbourhood_counts = Vec::from_iter(
                        neighbours
                            .iter()
                            .filter_map(|n| tracker.get(n))
                            .map(|entry| *entry.value() as f32),
                    );

                    (count, all_neighbourhood_counts)
                }
                ScoreType::Op => {
                    let count = if let Some(entry) = self.unfulfilled_ops.get(&node) {
                        entry.value().read().await.len()
                    } else {
                        0
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

            // let final_score = nodes_count.checked_sub(avg_in_neighbourhood).unwrap_or(0);
            let final_score = if count_at_node as f32 > avg_in_neighbourhood {
                count_at_node as f32 / avg_in_neighbourhood
            } else {
                0.0
            };

            let _prev = score_map.insert(node, final_score);
        }

        score_map
    }

    async fn get_weighted_scores(&self) -> (BTreeMap<XorName, f32>, f32) {
        trace!("Getting weighted scores");
        let ops_scores = self.calculate_scores(ScoreType::Op).await;

        let conn_scores = self
            .calculate_scores(ScoreType::Basic(self.communication_issues.clone()))
            .await;
        let knowledge_scores = self
            .calculate_scores(ScoreType::Basic(self.knowledge_issues.clone()))
            .await;

        let mut final_scores = BTreeMap::default();
        let mut scores_only = vec![];
        // now we loop to get final scores
        for (name, score) in ops_scores {
            trace!("Ops sore: {name}, {score}");

            let ops_score = score * OP_WEIGHTING;

            let node_conn_score = *conn_scores.get(&name).unwrap_or(&0.0);
            let node_conn_score = node_conn_score * CONN_WEIGHTING;

            let node_knowledge_score = *knowledge_scores.get(&name).unwrap_or(&0.0);
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

    /// Get a list of all nodes deemed dysfunctional
    pub async fn get_dysfunctional_node_names(&self) -> BTreeSet<XorName> {
        let mut dysfunctional_nodes = BTreeSet::new();

        let (final_scores, mean) = self.get_weighted_scores().await;

        let to_beat = mean * DYSFUNCTION_MEAN_RATIO;

        for (name, nodes_score) in final_scores {
            trace!(
                "Final dys score for {name} is {nodes_score} (mean is {mean}), needs to beat {:?}",
                to_beat
            );

            if nodes_score >= to_beat {
                debug!("Adding {name} as dsyf node");
                let _existed = dysfunctional_nodes.insert(name);
            }
        }
        dysfunctional_nodes
    }

    /// Get a list of all nodes deemed suspect
    pub async fn get_suspicious_node_names(&self) -> BTreeSet<XorName> {
        let mut sus_nodes = BTreeSet::new();

        let (final_scores, mean) = self.get_weighted_scores().await;

        let to_beat = mean * SUSPECT_MEAN_RATIO;
        for (name, nodes_score) in final_scores {
            trace!(
                "Final sus score for {name} is {nodes_score} (mean is {mean}), needs to beat {:?}",
                to_beat
            );

            if nodes_score >= to_beat {
                debug!("Adding {name} as sus node");
                let _existed = sus_nodes.insert(name);
            }
        }
        sus_nodes
    }
}
