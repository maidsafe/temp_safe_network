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

    // 5 here means we have some tolerance for AE rounds while nodes are getting up to speed on churn/split
    pub(crate) const NORMAL_KNOWLEDGE_ISSUES: usize = 7;
    pub(crate) const SUSPECT_KNOWLEDGE_ISSUES: usize = 15;
    pub(crate) const DYSFUNCTIONAL_KNOWLEDGE_ISSUES: usize = 28;

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

    #[tokio::test]
    async fn knowledge_dysfunction_basics_sus_comes_first() -> Result<()> {
        init_test_logger();
        let _outer_span =
            tracing::info_span!("knowledge_dysfunction_basics_sus_comes_first").entered();

        let adults = (0..10).map(|_| random_xorname()).collect::<Vec<XorName>>();

        let dysfunctional_detection = DysfunctionDetection::new(adults.clone());

        for adult in &adults {
            for _ in 0..NORMAL_KNOWLEDGE_ISSUES {
                dysfunctional_detection
                    .track_issue(*adult, IssueType::Knowledge)
                    .await?;
            }
        }

        let new_adult = random_xorname();
        let _ = dysfunctional_detection.add_new_node(new_adult).await;
        for _ in 0..SUSPECT_KNOWLEDGE_ISSUES {
            dysfunctional_detection
                .track_issue(new_adult, IssueType::Knowledge)
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
        println!("len = {}", dysfunctional_nodes.len());

        assert!(
            !dysfunctional_nodes.contains(&new_adult),
            "our added node is dysfunctional when it should not be"
        );

        assert_eq!(
            dysfunctional_nodes.len(),
            0,
            "more nodes are dysfunctional than they should be"
        );

        for _ in 0..DYSFUNCTIONAL_KNOWLEDGE_ISSUES - SUSPECT_KNOWLEDGE_ISSUES {
            dysfunctional_detection
                .track_issue(new_adult, IssueType::Knowledge)
                .await?;
        }

        let sus = dysfunctional_detection
            .get_nodes_beyond_severity(DysfunctionSeverity::Suspicious)
            .await?;
        assert!(sus.contains(&new_adult), "our adult is still sus");
        assert_eq!(sus.len(), 1, "only one adult is sus");

        let dysfunctional_nodes = dysfunctional_detection
            .get_nodes_beyond_severity(DysfunctionSeverity::Dysfunctional)
            .await?;

        assert!(
            dysfunctional_nodes.contains(&new_adult),
            "our adult should now be dysfunctional, but is not"
        );
        assert_eq!(
            dysfunctional_nodes.len(),
            1,
            "our adult is the only dysfunctional node"
        );

        Ok(())
    }
}
