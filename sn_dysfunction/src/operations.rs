// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{DysfunctionDetection, NodeIdentifier, OperationId};

impl DysfunctionDetection {
    /// Removes a pending_operation from the node liveness records
    pub async fn request_operation_fulfilled(
        &self,
        node_id: &NodeIdentifier,
        operation_id: OperationId,
    ) -> bool {
        trace!(
            "Attempting to remove pending_operation {:?} op: {:?}",
            node_id,
            operation_id
        );
        let mut has_removed = false;

        if let Some(entry) = self.unfulfilled_ops.get(node_id) {
            let v = entry.value();

            // only remove the first instance from the vec
            v.write().await.retain(|x| {
                if has_removed || x != &operation_id {
                    true
                } else {
                    has_removed = true;
                    false
                }
            });
            if has_removed {
                trace!(
                    "Pending operation removed for node: {:?} op: {:?}",
                    node_id,
                    operation_id
                );
            } else {
                trace!(
                    "No Pending operation found for node: {:?} op: {:?}",
                    node_id,
                    operation_id
                );
            }
        }
        has_removed
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::init_test_logger;
    use crate::{error::Result, DysfunctionDetection, DysfunctionSeverity, IssueType, OperationId};
    use rand::Rng;
    use xor_name::{rand::random as random_xorname, XorName};

    // we can see 500 pending issues under load
    pub(crate) const NORMAL_OPERATIONS_ISSUES: usize = 500;
    // Above this, nodes should be sus
    pub(crate) const SUSPECT_OPERATIONS_ISSUES: usize = 800;
    pub(crate) const DYSFUNCTIONAL_OPERATIONS_ISSUES: usize = 1500;

    fn get_random_operation_id() -> Option<OperationId> {
        let mut rng = rand::thread_rng();
        Some(rng.gen())
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

    #[tokio::test]
    async fn op_dysfunction_basics() -> Result<()> {
        init_test_logger();
        let _outer_span = tracing::info_span!("op_dysfunction_basics").entered();

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

        let new_adult = random_xorname();
        let _ = dysfunctional_detection.add_new_node(new_adult).await;
        for _ in 0..SUSPECT_OPERATIONS_ISSUES {
            let _ = dysfunctional_detection
                .track_issue(
                    new_adult,
                    IssueType::PendingRequestOperation(get_random_operation_id()),
                )
                .await;
        }

        assert!(
            dysfunctional_detection
                .get_nodes_beyond_severity(DysfunctionSeverity::Suspicious)
                .await?
                .contains(&new_adult),
            "node is not showing as sus"
        );

        assert!(
            !dysfunctional_detection
                .get_nodes_beyond_severity(DysfunctionSeverity::Dysfunctional)
                .await?
                .contains(&new_adult),
            "node is dysfunctional when it should only be sus"
        );

        for _ in 0..DYSFUNCTIONAL_OPERATIONS_ISSUES - SUSPECT_OPERATIONS_ISSUES {
            let _ = dysfunctional_detection
                .track_issue(
                    new_adult,
                    IssueType::PendingRequestOperation(get_random_operation_id()),
                )
                .await;
        }

        assert!(dysfunctional_detection
            .get_nodes_beyond_severity(DysfunctionSeverity::Suspicious)
            .await?
            .contains(&new_adult));

        assert!(
            dysfunctional_detection
                .get_nodes_beyond_severity(DysfunctionSeverity::Dysfunctional)
                .await?
                .contains(&new_adult),
            "our node has not been listed as dysfunctional"
        );

        Ok(())
    }
}
