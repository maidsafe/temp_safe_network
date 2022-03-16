// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{DysfunctionDetection, NodeIdentifier, OperationId};

impl DysfunctionDetection {
    /// Inserts a pending_operation, and is deemed as such until we get the appropriate response from the node
    /// Returns false if the operation already existed.
    pub async fn add_a_pending_request_operation(
        &self,
        node_id: NodeIdentifier,
        operation_id: OperationId,
    ) {
        let entry = self.unfulfilled_ops.entry(node_id).or_default();

        trace!(
            "Adding pending operation against node: {:?}: for op: {:?}",
            node_id,
            &operation_id
        );

        let v = entry.value();
        v.write().await.push(operation_id);
    }

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
    use crate::tests::{init_test_logger, ELDER_COUNT};
    use crate::{error::Result, DysfunctionDetection, DysfunctionSeverity, OperationId};
    use rand::Rng;
    use xor_name::{rand::random as random_xorname, XorName};

    // we can see 500 pending issues under load
    pub(crate) const NORMAL_OPERATIONS_ISSUES: usize = 500;
    // Above this, nodes should be sus
    pub(crate) const SUSPECT_OPERATIONS_ISSUES: usize = 800;
    pub(crate) const DYSFUNCTIONAL_OPERATIONS_ISSUES: usize = 1100;

    fn get_random_operation_id() -> OperationId {
        let mut rng = rand::thread_rng();
        rng.gen()
    }

    #[tokio::test]
    async fn op_dysfunction_no_variance_is_okay() -> Result<()> {
        let adults = (0..10).map(|_| random_xorname()).collect::<Vec<XorName>>();

        let dysfunctional_detection = DysfunctionDetection::new(adults.clone(), ELDER_COUNT);

        // Write data PENDING_OPS_TOLERANCE times to the 10 adults
        for adult in &adults {
            for _ in 0..NORMAL_OPERATIONS_ISSUES {
                let op_id = get_random_operation_id();
                dysfunctional_detection
                    .add_a_pending_request_operation(*adult, op_id)
                    .await;
            }
        }

        // Assert there are not any dysfunctional nodes
        // This is because all of them are within the tolerance ratio of each other
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

        let dysfunctional_detection = DysfunctionDetection::new(adults.clone(), ELDER_COUNT);

        // Write data NORMAL_OPERATIONS_ISSUES times to the 10 adults
        for adult in &adults {
            for _ in 0..NORMAL_OPERATIONS_ISSUES {
                let op_id = get_random_operation_id();
                dysfunctional_detection
                    .add_a_pending_request_operation(*adult, op_id)
                    .await;
            }
        }

        // Add a new adults
        let new_adult = random_xorname();
        dysfunctional_detection.add_new_node(new_adult);

        // Assert total adult count
        assert_eq!(dysfunctional_detection.closest_nodes_to.len(), 11);

        // Write data a suspect amount above PENDING_OPS_TOLERANCE times to the new adult to check for preemptive replication
        for _ in 0..SUSPECT_OPERATIONS_ISSUES {
            let op_id = get_random_operation_id();
            dysfunctional_detection
                .add_a_pending_request_operation(new_adult, op_id)
                .await;
        }

        // Assert that the new adult is detected as suspect.
        assert!(
            dysfunctional_detection
                .get_nodes_beyond_severity(DysfunctionSeverity::Suspicious)
                .await?
                .contains(&new_adult),
            "node is not showing as sus"
        );

        // but it should not yet be dysfunctional
        assert!(
            !dysfunctional_detection
                .get_nodes_beyond_severity(DysfunctionSeverity::Dysfunctional)
                .await?
                .contains(&new_adult),
            "node is dysfunctional when it should only be sus"
        );

        // Write data another set of excessive ops.
        for _ in 0..DYSFUNCTIONAL_OPERATIONS_ISSUES - SUSPECT_OPERATIONS_ISSUES {
            let op_id = get_random_operation_id();
            dysfunctional_detection
                .add_a_pending_request_operation(new_adult, op_id)
                .await;
        }

        // Assert that the new adult is detected as suspect.
        assert!(dysfunctional_detection
            .get_nodes_beyond_severity(DysfunctionSeverity::Suspicious)
            .await?
            .contains(&new_adult));

        // Assert that the new adult is detected dysfunctional.
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
