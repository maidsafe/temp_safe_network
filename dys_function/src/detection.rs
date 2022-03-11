// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{get_mean_of, DysfunctionDetection};
use std::collections::{BTreeMap, BTreeSet};
use xor_name::XorName;

static COMM_WEIGHTING: f32 = 2.0;
static OP_WEIGHTING: f32 = 1.0;

// Number of mean scores to be over to be considered dys/sus
static DYSFUNCTION_MEAN_RATIO: f32 = 2.5;
static SUSPECT_MEAN_RATIO: f32 = 1.2;

impl DysfunctionDetection {
    /// Helper func to get vec of a node and their neighbours for comparison
    pub(crate) fn get_node_and_neighbours_vec(&self) -> Vec<(XorName, Vec<XorName>)> {
        let mut node_neighbours = vec![];
        for entry in self.closest_nodes_to.iter() {
            let (node, neighbours) = entry.pair();
            node_neighbours.push((*node, neighbours.clone()));
        }

        node_neighbours
    }

    async fn get_weighted_scores(&self) -> (BTreeMap<XorName, usize>, f32) {
        debug!("Getting weighted scores");
        let ops_scores = self.calculate_ops_score().await;

        let comm_scores = self.calculate_connections_score();

        let mut final_scores = BTreeMap::default();
        let mut scores_only = vec![];
        // now we loop to get final scores
        for (name, score) in ops_scores {
            trace!("Ops sore: {name}, {score}");
            let node_comm_score = *comm_scores.get(&name).unwrap_or(&0) as f32;
            let node_comm_score = node_comm_score * OP_WEIGHTING;

            trace!("Comms sore: {name}, {node_comm_score}");
            let final_score = score + (node_comm_score * &COMM_WEIGHTING) as usize;

            scores_only.push(final_score as f32);
            let _prev = final_scores.insert(name, final_score);
        }

        let mean = get_mean_of(&scores_only).unwrap_or(1.0);

        (final_scores, mean)
    }

    /// Get a list of all nodes deemed dysfunctional
    pub async fn get_dysfunctional_node_names(&self) -> BTreeSet<XorName> {
        debug!("Checking for dysfunctional nodes");
        let mut dysfunctional_nodes = BTreeSet::new();

        let (final_scores, mean) = self.get_weighted_scores().await;

        let to_beat = mean * DYSFUNCTION_MEAN_RATIO;
        for (name, nodes_score) in final_scores {
            debug!(
                "Final score for {name} is {nodes_score} (mean is {mean}), needs to beat {:?}",
                to_beat
            );

            if nodes_score as f32 > to_beat {
                debug!("!!!!!!!!!!!!!!! Adding dsyf node {name}");
                let _existed = dysfunctional_nodes.insert(name);
            }
        }
        dysfunctional_nodes
    }

    /// Get a list of all nodes deemed suspect
    pub async fn get_suspicious_node_names(&self) -> BTreeSet<XorName> {
        debug!("Checking for sus nodes");
        let mut sus_nodes = BTreeSet::new();

        let (final_scores, mean) = self.get_weighted_scores().await;

        for (name, nodes_score) in final_scores {
            if nodes_score as f32 > mean * SUSPECT_MEAN_RATIO {
                let _existed = sus_nodes.insert(name);
            }
        }
        sus_nodes
    }
}
