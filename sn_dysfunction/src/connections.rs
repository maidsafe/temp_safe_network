// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

#[cfg(test)]
mod tests {
    use crate::tests::init_test_logger;
    use crate::{DysfunctionDetection, DysfunctionSeverity, IssueType};

    use eyre::Error;
    use xor_name::{rand::random as random_xorname, XorName};

    type Result<T, E = Error> = std::result::Result<T, E>;

    // Above this, nodes should be sus
    // this is only counting last RECENT minutes atm
    pub(crate) const NORMAL_CONNECTION_PROBLEM_COUNT: usize = 5;
    pub(crate) const SUSPECT_CONNECTION_PROBLEM_COUNT: usize = 20;
    pub(crate) const DYSFUNCTIONAL_CONNECTION_PROBLEM_COUNT: usize = 35;

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

    #[tokio::test]
    async fn conn_dysfunction_basics_sus_comes_first() -> Result<()> {
        init_test_logger();
        let _outer_span = tracing::info_span!("conn_dysfunction_basics_sus_comes_first").entered();

        let adults = (0..10).map(|_| random_xorname()).collect::<Vec<XorName>>();

        let dysfunctional_detection = DysfunctionDetection::new(adults.clone());

        for adult in &adults {
            for _ in 0..NORMAL_CONNECTION_PROBLEM_COUNT {
                dysfunctional_detection
                    .track_issue(*adult, IssueType::Communication)
                    .await?;
            }
        }

        let new_adult = random_xorname();
        let _ = dysfunctional_detection.add_new_node(new_adult).await;
        for _ in 0..SUSPECT_CONNECTION_PROBLEM_COUNT {
            dysfunctional_detection
                .track_issue(new_adult, IssueType::Communication)
                .await?;
        }

        let sus = dysfunctional_detection
            .get_nodes_beyond_severity(DysfunctionSeverity::Suspicious)
            .await?;
        assert_eq!(sus.len(), 1, "only one node is sus");
        assert!(sus.contains(&new_adult), "our adult is not sus");

        let dysfunctional_nodes = dysfunctional_detection
            .get_nodes_beyond_severity(DysfunctionSeverity::Dysfunctional)
            .await?;

        assert!(
            !dysfunctional_nodes.contains(&new_adult),
            "our added node is dysfunctional when it should not be"
        );

        assert_eq!(
            dysfunctional_nodes.len(),
            0,
            "more nodes are dysfunctional than they should be"
        );

        for _ in 0..DYSFUNCTIONAL_CONNECTION_PROBLEM_COUNT - SUSPECT_CONNECTION_PROBLEM_COUNT {
            dysfunctional_detection
                .track_issue(new_adult, IssueType::Communication)
                .await?;
        }

        let sus = dysfunctional_detection
            .get_nodes_beyond_severity(DysfunctionSeverity::Suspicious)
            .await?;
        assert!(sus.contains(&new_adult), "our adult is still sus");
        assert_eq!(sus.len(), 1, "only one adult is sus");

        debug!("=============================");
        let dysfunctional_nodes = dysfunctional_detection
            .get_nodes_beyond_severity(DysfunctionSeverity::Dysfunctional)
            .await?;

        assert!(
            dysfunctional_nodes.contains(&new_adult),
            "our adult should now be dysfunctional but is not"
        );
        assert_eq!(
            dysfunctional_nodes.len(),
            1,
            "our node should be the only dysfunctional nodes"
        );

        Ok(())
    }
}
