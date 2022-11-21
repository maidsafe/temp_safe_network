use std::{collections::BTreeSet, sync::Arc};

// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{core::MyNode, flow_ctrl::FlowCtrl};
use sn_dysfunction::{DysfunctionDetection, IssueType};

use tokio::sync::{
    mpsc::{self, Receiver},
    RwLock,
};
use xor_name::XorName;

/// Set of Cmds to interact with the `DysfunctionDetection` module
pub(crate) enum DysCmds {
    AddNode(XorName),
    RetainNodes(BTreeSet<XorName>),
    TrackIssue(XorName, IssueType),
    UntrackIssue(XorName, IssueType),
    GetDysfunctionalNodes,
}

impl FlowCtrl {
    /// Spawns a tokio task that listens for the `DysCmds` and processes them
    pub(crate) fn start_dysfunction_detection(
        node: Arc<RwLock<MyNode>>,
        mut dys_cmds_from_node: Receiver<DysCmds>,
    ) -> Receiver<BTreeSet<XorName>> {
        let (dys_nodes_sender, dys_nodes_receiver) = mpsc::channel(20);

        let _ = tokio::task::spawn(async move {
            debug!("[NODE READ]: flowctrl start dysfunction detection");
            let mut dysfunction = DysfunctionDetection::new(
                node.read()
                    .await
                    .network_knowledge
                    .members()
                    .iter()
                    .map(|peer| peer.name())
                    .collect::<Vec<XorName>>(),
            );
            while let Some(cmd) = dys_cmds_from_node.recv().await {
                match cmd {
                    DysCmds::AddNode(node) => dysfunction.add_new_node(node),
                    DysCmds::RetainNodes(nodes) => dysfunction.retain_members_only(nodes),
                    DysCmds::TrackIssue(node, issue) => dysfunction.track_issue(node, issue),
                    DysCmds::UntrackIssue(node, issue) => {
                        debug!("Attempting to remove {issue:?} from {node:?}");
                        match issue {
                            IssueType::AwaitingProbeResponse => {
                                dysfunction.ae_update_msg_received(&node)
                            }
                            IssueType::Dkg => dysfunction.dkg_ack_fulfilled(&node),
                            IssueType::PendingRequestOperation(op_id) => {
                                let _ = dysfunction.request_operation_fulfilled(&node, op_id);
                            }
                            _ => {}
                        };
                    }
                    DysCmds::GetDysfunctionalNodes => {
                        if let Err(error) = dys_nodes_sender
                            .send(dysfunction.get_dysfunctional_nodes())
                            .await
                        {
                            error!("Could not send dysfunctional nodes through the mpsc channel: {error:?}");
                        }
                    }
                }
            }
        });

        dys_nodes_receiver
    }

    /// returns names that are relatively dysfunctional
    pub(crate) async fn get_dysfunctional_node_names(&mut self) -> BTreeSet<XorName> {
        // send a DysCmd asking for the dysfunctional nodes
        debug!("[NODE READ]: flowctrl send DysCmds to read dysfunctional nodes");
        if let Err(error) = self
            .node
            .read()
            .await
            .dysfunction_cmds_sender
            .send(DysCmds::GetDysfunctionalNodes)
            .await
        {
            error!("Could not send DysCmds through dysfunctional_cmds_tx: {error}");
            BTreeSet::new()
        } else {
            // read the rx channel to get the dysfunctional nodes
            if let Some(dysfunctional_nodes) = self.dysfunctional_nodes_receiver.recv().await {
                dysfunctional_nodes
            } else {
                error!("dysfunctional_nodes_rx channel closed?");
                BTreeSet::new()
            }
        }
    }
}
