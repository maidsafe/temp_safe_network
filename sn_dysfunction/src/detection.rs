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

static CONN_WEIGHTING: f32 = 3.0;
static OP_WEIGHTING: f32 = 1.0;
static KNOWLEDGE_WEIGHTING: f32 = 4.0;
static DKG_WEIGHTING: f32 = 1.0;

/// Z-score value above which a node is dysfunctional
static DYSFUNCTIONAL_DEVIATION: f32 = 2.0;

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
    pub fn calculate_scores(&self) -> ScoreResults {
        let mut communication_scores = BTreeMap::new();
        let mut knowledge_scores = BTreeMap::new();
        let mut op_scores = BTreeMap::new();
        let mut dkg_scores = BTreeMap::new();

        let adults = self.adults.to_vec();
        for node in adults.iter() {
            let _ = dkg_scores.insert(
                *node,
                self.calculate_node_score(node, adults.clone(), &IssueType::Dkg),
            );
            let _ = communication_scores.insert(
                *node,
                self.calculate_node_score(node, adults.clone(), &IssueType::Communication),
            );
            let _ = knowledge_scores.insert(
                *node,
                self.calculate_node_score(node, adults.clone(), &IssueType::Knowledge),
            );
            let _ = op_scores.insert(
                *node,
                self.calculate_node_score(
                    node,
                    adults.clone(),
                    &IssueType::PendingRequestOperation(None),
                ),
            );
        }
        ScoreResults {
            communication_scores,
            dkg_scores,
            knowledge_scores,
            op_scores,
        }
    }

    fn calculate_node_score(
        &self,
        node: &XorName,
        adults: Vec<XorName>,
        issue_type: &IssueType,
    ) -> f32 {
        let node_count = self.get_node_issue_count(node, issue_type);
        let mut other_node_counts = Vec::new();
        for adult in adults {
            if adult == *node {
                continue;
            }
            other_node_counts.push(self.get_node_issue_count(&adult, issue_type) as f32);
        }
        let average = get_mean_of(&other_node_counts).unwrap_or(1.0);
        let score = node_count.checked_sub(average as usize).unwrap_or(1) as f32;
        if score < 1.0 {
            1.0
        } else {
            score
        }
    }

    fn get_node_issue_count(&self, node: &XorName, issue_type: &IssueType) -> usize {
        match issue_type {
            IssueType::Communication => {
                let count = if let Some(issues) = self.communication_issues.get(node) {
                    issues.len()
                } else {
                    1
                };
                count
            }
            IssueType::Dkg => {
                let count = if let Some(issues) = self.dkg_issues.get(node) {
                    issues.len()
                } else {
                    1
                };
                count
            }
            IssueType::Knowledge => {
                let count = if let Some(issues) = self.knowledge_issues.get(node) {
                    issues.len()
                } else {
                    1
                };
                count
            }
            IssueType::PendingRequestOperation(_) => {
                let count = if let Some(issues) = self.unfulfilled_ops.get(node) {
                    issues.len()
                } else {
                    1
                };
                count
            }
        }
    }

    /// get scores mapped by name, to score and z-score, which is std dev's from the mean
    fn get_weighted_scores(&self) -> BTreeMap<XorName, Option<f32>> {
        trace!("Getting weighted scores");
        let scores = self.calculate_scores();
        let ops_scores = scores.op_scores;
        let conn_scores = scores.communication_scores;
        let dkg_scores = scores.dkg_scores;
        let knowledge_scores = scores.knowledge_scores;

        let mut pre_z_scores = BTreeMap::default();
        let mut scores_only = vec![];
        // now we loop to get the scores per xorname, so we can then avg etc
        for (name, score) in ops_scores {
            debug!("Ops sore: {name}, {score}");

            let ops_score = score * OP_WEIGHTING;

            let node_conn_score = *conn_scores.get(&name).unwrap_or(&1.0);
            let node_conn_score = node_conn_score * CONN_WEIGHTING;

            let node_dkg_score = *dkg_scores.get(&name).unwrap_or(&1.0);
            let node_dkg_score = node_dkg_score * DKG_WEIGHTING;

            let node_knowledge_score = *knowledge_scores.get(&name).unwrap_or(&1.0);
            let node_knowledge_score = node_knowledge_score * KNOWLEDGE_WEIGHTING;

            debug!("Conns score: {name}, {node_conn_score}");
            debug!("Knowledge score: {name}, {node_knowledge_score}");
            debug!("Dkg score: {name}, {node_dkg_score}");
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

            debug!("Final Z-score for {name} is {zscore:?}");

            let _existed = final_scores.insert(name, zscore);
        }

        final_scores
    }

    fn cleanup_time_sensistive_checks(&mut self) -> Result<()> {
        for (_name, issues) in self.communication_issues.iter_mut() {
            issues.retain(|time| time.elapsed() < RECENT_ISSUE_DURATION);
        }

        for (_name, issues) in self.knowledge_issues.iter_mut() {
            issues.retain(|time| time.elapsed() < RECENT_ISSUE_DURATION);
        }

        for (_name, issues) in self.dkg_issues.iter_mut() {
            issues.retain(|time| time.elapsed() < RECENT_ISSUE_DURATION);
        }

        Ok(())
    }

    /// Get a list of nodes whose score is above mean * `DysfunctionalSeverity` weighting
    /// TODO: order these to act upon _most_ dysfunctional first
    /// (the nodes must all `ProposeOffline` over a dysfunctional node and then _immediately_ vote it off. So any other membershipn changes in flight could block this.
    /// thus, we need to be callling this function often until nodes are removed.)
    pub fn get_nodes_beyond_severity(
        &mut self,
        severity: DysfunctionSeverity,
    ) -> Result<BTreeSet<XorName>> {
        self.cleanup_time_sensistive_checks()?;

        let mut dysfunctional_nodes = BTreeSet::new();

        let final_scores = self.get_weighted_scores();

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
    use tokio::runtime::Runtime;
    use xor_name::{rand::random as random_xorname, XorName};

    #[derive(Debug, Clone)]
    enum NodeQualityScored {
        Bad(f32),
        Good(f32),
    }

    impl NodeQualityScored {
        fn get_failure_rate(&self) -> &f32 {
            match self {
                NodeQualityScored::Good(r) => r,
                NodeQualityScored::Bad(r) => r,
            }
        }
    }

    /// In a standard network startup (as of 24/06/22)
    /// we see:
    /// 0 op requests
    /// 2407 `DkgBroadcastMsg` DKG (each are tracked as an eror until a respnose comes in...) this is total across all nodes...
    ///
    /// This includes:
    /// 510 "tracker: Dkg..." (the initial black mark)
    /// ~2394 "Logging Dkg session as responded to in dysfunction." (aka removing a black mark) < -- we're not simulating this,
    /// only the stains that stick... So in reality, over time we' see 0 DKG issues in a normal startup
    /// ~469 "tracker: Know"
    /// ~230 "tracker: Communication""
    /// 0 "tracker: `PendingOp`..." (equally a lot of these are being responded to...)
    fn generate_network_startup_msg_issues() -> impl Strategy<Value = IssueType> {
        // higher numbers here are more frequent
        prop_oneof![
        230 => Just(IssueType::Communication),
        500 => Just(IssueType::Dkg),
        450 => Just(IssueType::Knowledge),
        ]
    }

    /// In a standard network startup (as of 24/06/22)
    /// these values are on top of the above...
    /// after we then we run the client test suite (once),
    /// (and yes, some of them have not changed)
    /// 510 "tracker: Dkg..."
    /// ~2394 "Attempting to remove logged dkg"
    /// ~469 "tracker: Know"
    /// ~1588 "tracker: Communication""
    /// ~3376 "tracker: `PendingOp`..." (equally a lot of these are being responded to...)
    fn generate_no_churn_normal_use_msg_issues() -> impl Strategy<Value = IssueType> {
        // higher numbers here are more frequent
        prop_oneof![
            1200 => Just(IssueType::Communication),
            0 => Just(IssueType::Dkg),
            0 => Just(IssueType::Knowledge),
            3400 => (any::<[u8; 32]>())
                .prop_map(|x| IssueType::PendingRequestOperation(Some(OperationId(x))))
        ]
    }

    /// Generate proptest issues, in a range from 1000 `to...max_uantity`
    fn generate_msg_issues(
        min: usize,
        max: usize,
    ) -> impl Strategy<Value = Vec<(IssueType, XorName, f32)>> {
        let issue_name_for_direction = generate_xorname();
        prop::collection::vec(
            (
                generate_no_churn_normal_use_msg_issues(),
                issue_name_for_direction,
                0.0..1.0f32,
            ),
            min..max + 1,
        )
    }

    /// Generate proptest issues, in a range from 1000 `to...max_quantity`
    /// issues had a name for reliably routing
    /// issues come with a random f32 0-1 to use as our test against `NodeQuality`
    fn generate_startup_issues(
        min: usize,
        max: usize,
    ) -> impl Strategy<Value = Vec<(IssueType, XorName, f32)>> {
        let issue_name_for_direction = generate_xorname();
        prop::collection::vec(
            (
                generate_network_startup_msg_issues(),
                issue_name_for_direction,
                0.0..1.0f32,
            ),
            min..max + 1,
        )
    }

    fn generate_xorname() -> impl Strategy<Value = XorName> {
        // get a random string
        let str_val = "[1-9]{32}[a-zA-Z]{32}[1-9]{32}[a-zA-Z]{32}[1-9]{32}[a-zA-Z]{32}";

        str_val.prop_map(|s| XorName::from_content(s.as_bytes()))
    }

    /// Generate proptest nodes, each a Xorname, this will generate nodes with different `NodeQualities`
    fn generate_nodes_and_quality(
        min: usize,
        max: usize,
    ) -> impl Strategy<Value = Vec<(XorName, NodeQualityScored)>> {
        prop::collection::vec(
            (
                generate_xorname(),
                prop_oneof![
                    // 3 x as likely to have good nodes vs bad
                    // good nodes fail only 10% of the time
                    3 => Just(NodeQualityScored::Good(0.05)),
                    // bad nodes fail 75% of the time
                    1 => Just(NodeQualityScored::Bad(0.75)),

                ],
            ),
            min..max,
        )
        .prop_filter(
            "there should be at least two good and one bad node",
            |nodes| {
                let mut good_len: f32 = 0.0;
                let mut bad_len: f32 = 0.0;

                for (_name, quality) in nodes {
                    match quality {
                        NodeQualityScored::Good(_) => good_len += 1.0,
                        NodeQualityScored::Bad(_) => bad_len += 1.0,
                    }
                }

                let byzantine_level = good_len / 3.0;

                // we have at least one bad node
                bad_len >= 1.0 &&
                // at least two good
                good_len >=2.0 &&
                // we're not overly byzantine (ie no more than 30% bad)
                byzantine_level >= 1.0 &&
                // otherwise, 3 good and 2 bad nodes
                byzantine_level > bad_len
            },
        )
    }

    /// for a given issue and a "Root address" to base elder selection off, this returns
    /// the nodes we should target for this specific issue:
    /// eg if DKG, it's the closest to the `root_addr`
    /// if anything else, we base it off issue name closeness
    fn get_target_nodes_for_issue(
        issue: IssueType,
        issue_location: XorName,
        root: XorName,
        nodes: &[(XorName, NodeQualityScored)],
        elders_count: usize,
    ) -> Vec<(XorName, NodeQualityScored)> {
        if matches!(issue, IssueType::Dkg) {
            nodes
                .iter()
                .sorted_by(|lhs, rhs| root.clone().cmp_distance(&lhs.0, &rhs.0))
                .take(elders_count)
                .cloned()
                .collect::<Vec<_>>()
        } else {
            // we use the "issue location" to determine which four nodes to send to
            // this should therefore be reproducible amongst proptest retries/shrinking etc
            nodes
                .iter()
                .sorted_by(|lhs, rhs| issue_location.cmp_distance(&lhs.0, &rhs.0))
                // and we simul-send it to 4 nodes
                .take(4)
                .cloned()
                .collect::<Vec<_>>()
        }
    }

    proptest! {
        #[test]
        fn pt_calculate_scores_should_include_all_nodes_in_score_map(
            adult_count in 4..50usize, issue_type in generate_no_churn_normal_use_msg_issues())
        {
            Runtime::new().unwrap().block_on(async {
                let adults = (0..adult_count).map(|_| random_xorname()).collect::<Vec<XorName>>();
                let mut dysfunctional_detection = DysfunctionDetection::new(adults.clone());
                for _ in 0..5 {
                    let _ = dysfunctional_detection.track_issue(
                        adults[0], issue_type.clone());
                }

                let score_results = dysfunctional_detection
                    .calculate_scores();
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
            adult_count in 4..50usize, issue_count in 0..50, issue_type in generate_no_churn_normal_use_msg_issues())
        {
            Runtime::new().unwrap().block_on(async {
                let adults = (0..adult_count).map(|_| random_xorname()).collect::<Vec<XorName>>();
                let mut dysfunctional_detection = DysfunctionDetection::new(adults.clone());
                for _ in 0..issue_count {
                    let _ = dysfunctional_detection.track_issue(
                        adults[0], issue_type.clone());
                }

                let score_results = dysfunctional_detection
                    .calculate_scores();
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
        /// We do not want false positives, We do want -- over longer timeframes -- to find all bad nodes... there's a tough balance to strike here.
        /// Given that the tests _must_ terminate, there will be some instances where a bad node may not be found. But we can assume as long as we're
        /// getting _some_ that most will be caught over the long term. So we opt to check that every bad node we get from dysf is indeed bad,
        /// and that we don't exceed the count of bad_nodes per test
        ///
        /// "Nodes" are just random xornames,
        /// each issue has a random xorname attached to it to, and is sent to 4 nodes... each of which will fail a % of the time, depending on the
        /// NodeQuality (Good or Bad)
        fn pt_detect_correct_or_less_amount_of_dysf_nodes(
            elders_in_dkg in 2..7usize,
            nodes in generate_nodes_and_quality(3,30), issues in generate_msg_issues(500,1500))
            {
                init_test_logger();
                let _outer_span = tracing::info_span!("pt_correct_less").entered();

                let mut good_len = 0;
                let mut bad_len = 0;

                for (_node, quality) in &nodes {
                    match quality {
                        NodeQualityScored::Good(_) => good_len += 1,
                        NodeQualityScored::Bad(_) => bad_len += 1,
                    }
                }

                debug!("Good {good_len}");
                debug!("Bad {bad_len}");

                // random xorname to pick 7 nodes as "elders" for DKG
                let random_xorname_root = nodes[0].0;


                let _res = Runtime::new().unwrap().block_on(async {
                    // add dysf to our all_nodes
                    let all_node_names = nodes.clone().iter().map(|(name, _)| *name).collect::<Vec<XorName>>();

                    let mut dysfunctional_detection = DysfunctionDetection::new(all_node_names);

                    // Now we loop through each issue/msg
                    for (issue, issue_location, fail_test ) in issues {

                        let target_nodes = get_target_nodes_for_issue(issue.clone(), issue_location, random_xorname_root, &nodes, elders_in_dkg);

                        // now we track our issue, but only if that node fails to passes muster...
                        for (node, quality) in target_nodes {
                            // if our random fail test is less than the failure rate.
                            let failure_chance = quality.get_failure_rate();
                            let msg_failed = &fail_test < failure_chance;

                            if msg_failed {
                                let _ = dysfunctional_detection.track_issue(
                                    node, issue.clone());
                            }

                        }
                    }
                    // now we can see what we have...
                    let dysfunctional_nodes_found = match dysfunctional_detection
                        .get_nodes_beyond_severity( DysfunctionSeverity::Dysfunctional) {
                            Ok(nodes) => nodes,
                            Err(error) => bail!("Failed getting dysfunctional nodes from DysfunctionDetector: {error}")
                        };

                    info!("======================");
                    info!("dysf found len {:?}:, expected {:}", dysfunctional_nodes_found.len(), bad_len );
                    info!("======================");

                    // over a long enough time span, we should catch those bad nodes...
                    // So long as dysfunction isn't returning _more_ than the bad node count, this can pass
                    assert!(dysfunctional_nodes_found.len() <= bad_len, "checking {} dysf nodes found is equal or less than the {} actual bad nodes in test", dysfunctional_nodes_found.len(), bad_len);

                    // check that these were indeed bad nodes
                    for bad_node in dysfunctional_nodes_found {
                        if let Some((_, quality)) = nodes.iter().find(|(name, _)| {name == &bad_node }) {
                            match quality {
                                NodeQualityScored::Good(_) => bail!("identified a good node as bad"),
                                NodeQualityScored::Bad(_) => {
                                    // everything is fine
                                }
                            }
                        }
                        else {
                            bail!("bad node not found in our original node set!?")
                        }

                    }

                    Ok(())
                });
        }


        #[test]
        /// Test to check if we have more DKG messages, that bad nodes are found, within our expected issue count
        /// we then check that we can reliably detect those nodes
        ///
        /// We do not want false positives, We do want -- over longer timeframes -- to find all bad nodes... there's a tough balance to strike here.
        /// Given that the tests _must_ terminate, there will be some instances where a bad node may not be found. But we can assume as long as we're
        /// getting _some_ that most will be caught over the long term. So we opt to check that every bad node we get from dysf is indeed bad,
        /// and that we don't exceed the count of bad_nodes per test
        ///
        /// "Nodes" are just random xornames,
        /// each issue has a random xorname attached to it to, and is sent to 4 nodes... each of which will fail a % of the time, depending on the
        /// NodeQuality (Good or Bad)
        fn pt_detect_dkg_bad_nodes(
            elders_in_dkg in 2..7usize,
            // ~1500 msgs total should get us ~500 dkg which would be representative
            nodes in generate_nodes_and_quality(3,30), issues in generate_startup_issues(500,2500))
            {
                init_test_logger();
                let _outer_span = tracing::info_span!("pt_dkg").entered();
                let mut good_len = 0;
                let mut bad_len = 0;
                let random_xorname_root = nodes[0].0;

                for (_, quality) in &nodes {
                    match quality {
                        NodeQualityScored::Good(_) => good_len += 1,
                        NodeQualityScored::Bad(_) => bad_len += 1,
                    }
                }

                debug!("Good {good_len}");
                debug!("Bad {bad_len}");

                let _res = Runtime::new().unwrap().block_on(async {
                // add dysf to our all_nodes
                let all_node_names = nodes.clone().iter().map(|(name, _)| *name).collect::<Vec<XorName>>();

                let mut dysfunctional_detection = DysfunctionDetection::new(all_node_names);

                // Now we loop through each issue/msg
                for (issue, issue_location, fail_test ) in issues {

                    let target_nodes = get_target_nodes_for_issue(issue.clone(), issue_location, random_xorname_root, &nodes, elders_in_dkg);

                    // we send each message to all nodes in this situation where we're looking at elder comms alone over dkg
                    // now we track our issue, but only if that node fails to passes muster...
                    for (node, quality) in target_nodes.clone() {
                        // if our random fail test is less than the quality failure rate.
                        let failure_chance = quality.get_failure_rate();
                        let msg_failed = &fail_test < failure_chance;

                        if msg_failed {
                            let _ = dysfunctional_detection.track_issue(
                                node, issue.clone());
                        }

                    }
                }
                // now we can see what we have...
                let dysfunctional_nodes_found = match dysfunctional_detection
                    .get_nodes_beyond_severity( DysfunctionSeverity::Dysfunctional) {
                        Ok(nodes) => nodes,
                        Err(error) => bail!("Failed getting dysfunctional nodes from DysfunctionDetector: {error}")
                    };

                info!("======================");
                info!("dysf found len {:?}:, expected {:}?", dysfunctional_nodes_found.len(), bad_len );
                info!("======================");

                // over a long enough time span, we should catch those bad nodes...
                // So long as dysfunction isn't returning _more_ than the bad node count, this can pass
                assert!(dysfunctional_nodes_found.len() <= bad_len, "checking {} dysf nodes found is less or equal to the {} actual bad nodes in test", dysfunctional_nodes_found.len(), bad_len);

                // check that these were indeed bad nodes
                for bad_node in dysfunctional_nodes_found {
                    if let Some((_, quality)) = nodes.iter().find(|(name, _)| {name == &bad_node }) {
                        match quality {
                            NodeQualityScored::Good(_) => bail!("identified a good node as bad"),
                            NodeQualityScored::Bad(_) => {
                                // everything is fine
                            }
                        }
                    }
                    else {
                        bail!("bad node not found in our original node set!?")
                    }

                }
                Ok(())
            });
        }

        #[test]
        fn pt_calculate_scores_when_all_nodes_have_the_same_number_of_issues_scores_should_all_be_one(
            adult_count in 4..50, issue_count in 0..50, issue_type in generate_no_churn_normal_use_msg_issues())
        {
            Runtime::new().unwrap().block_on(async {
                let adults = (0..adult_count).map(|_| random_xorname()).collect::<Vec<XorName>>();
                let mut dysfunctional_detection = DysfunctionDetection::new(adults.clone());
                for adult in adults.iter() {
                    for _ in 0..issue_count {
                        let _ = dysfunctional_detection.track_issue(
                            *adult, issue_type.clone());
                    }
                }

                let score_results = dysfunctional_detection
                    .calculate_scores();
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
        let mut dysfunctional_detection = DysfunctionDetection::new(adults.clone());
        for adult in &adults {
            for _ in 0..NORMAL_OPERATIONS_ISSUES {
                let _ = dysfunctional_detection.track_issue(
                    *adult,
                    IssueType::PendingRequestOperation(get_random_operation_id()),
                );
            }
        }

        assert_eq!(
            dysfunctional_detection
                .get_nodes_beyond_severity(DysfunctionSeverity::Dysfunctional)?
                .len(),
            0
        );
        assert_eq!(
            dysfunctional_detection
                .get_nodes_beyond_severity(DysfunctionSeverity::Suspicious)?
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

        let mut dysfunctional_detection = DysfunctionDetection::new(adults.clone());

        for adult in &adults {
            for _ in 0..NORMAL_CONNECTION_PROBLEM_COUNT {
                dysfunctional_detection.track_issue(*adult, IssueType::Communication)?;
            }
        }

        assert_eq!(
            dysfunctional_detection
                .get_nodes_beyond_severity(DysfunctionSeverity::Dysfunctional)?
                .len(),
            0,
            "no nodes are dysfunctional"
        );
        assert_eq!(
            dysfunctional_detection
                .get_nodes_beyond_severity(DysfunctionSeverity::Suspicious)?
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

        let mut dysfunctional_detection = DysfunctionDetection::new(adults.clone());

        // Write data NORMAL_KNOWLEDGE_ISSUES times to the 10 adults
        for adult in &adults {
            for _ in 0..NORMAL_KNOWLEDGE_ISSUES {
                dysfunctional_detection.track_issue(*adult, IssueType::Knowledge)?;
            }
        }

        // Assert there are not any dysfuncitonal nodes
        // This is because all of them are within the tolerance ratio of each other
        assert_eq!(
            dysfunctional_detection
                .get_nodes_beyond_severity(DysfunctionSeverity::Dysfunctional)?
                .len(),
            0,
            "no nodes are dysfunctional"
        );
        assert_eq!(
            dysfunctional_detection
                .get_nodes_beyond_severity(DysfunctionSeverity::Suspicious)?
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

        let mut dysfunctional_detection = DysfunctionDetection::new(adults.clone());

        // Add a new adults
        let new_adult = random_xorname();
        dysfunctional_detection.add_new_node(new_adult).await;

        // Add just one knowledge issue...
        for _ in 0..1 {
            dysfunctional_detection.track_issue(new_adult, IssueType::Knowledge)?;
        }

        let sus =
            dysfunctional_detection.get_nodes_beyond_severity(DysfunctionSeverity::Suspicious)?;

        // Assert that the new adult is not detected as suspect.
        assert!(!sus.contains(&new_adult), "our adult should not be sus");
        assert_eq!(sus.len(), 0, "no node is sus");

        let dysfunctional_nodes = dysfunctional_detection
            .get_nodes_beyond_severity(DysfunctionSeverity::Dysfunctional)?;

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
