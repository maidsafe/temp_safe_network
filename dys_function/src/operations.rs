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
        let entry = self.unfulfilled_requests.entry(node_id).or_default();

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

        if let Some(entry) = self.unfulfilled_requests.get(node_id) {
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
    // use super::{DysfunctionDetection, EXCESSIVE_OPS_TOLERANCE};

    // use crate::messaging::data::chunk_operation_id;
    // use crate::node::Error;
    // use crate::types::ChunkAddress;

    // use crate::node::core::data::records::liveness_tracking::MIN_PENDING_OPS;
    // use itertools::Itertools;
    // use std::collections::BTreeSet;
    // use xor_name::XorName;
}
