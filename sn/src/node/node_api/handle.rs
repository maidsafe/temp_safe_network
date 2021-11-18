// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{
    messaging::{send, send_error, send_to_nodes},
    Node,
};
use crate::node::{
    event_mapping::MsgContext,
    node_ops::{NodeDuties, NodeDuty},
    Result,
};
use tokio::task::JoinHandle;
use tracing::{debug, info};

#[derive(Debug)]
pub(super) enum NodeTask {
    None,
    Result(Box<(NodeDuties, Option<MsgContext>)>),
    Thread(JoinHandle<Result<NodeTask>>),
}

impl From<NodeDuties> for NodeTask {
    fn from(duties: NodeDuties) -> Self {
        Self::Result(Box::new((duties, None)))
    }
}

impl Node {
    ///
    pub(super) async fn handle(&self, duty: NodeDuty) -> Result<NodeTask> {
        if !matches!(duty, NodeDuty::NoOp) {
            debug!("Handling NodeDuty: {:?}", duty);
        }

        match duty {
            NodeDuty::SectionSplit {
                our_key, newbie, ..
            } => {
                if newbie {
                    info!("Beginning split as Newbie");
                    self.begin_split_as_newbie(our_key).await?;
                }

                Ok(NodeTask::None)
            }
            //
            // ------- Misc ------------
            NodeDuty::Send(msg) => {
                let network_api = self.network_api.clone();
                let handle = tokio::spawn(async move {
                    send(msg, &network_api).await?;
                    Ok(NodeTask::None)
                });
                Ok(NodeTask::Thread(handle))
            }
            NodeDuty::SendError(msg) => {
                let network_api = self.network_api.clone();
                let handle = tokio::spawn(async move {
                    send_error(msg, &network_api).await?;
                    Ok(NodeTask::None)
                });
                Ok(NodeTask::Thread(handle))
            }
            NodeDuty::SendToNodes {
                msg_id,
                msg,
                targets,
                aggregation,
            } => {
                let network_api = self.network_api.clone();
                let handle = tokio::spawn(async move {
                    send_to_nodes(msg_id, msg, targets, aggregation, &network_api).await?;
                    Ok(NodeTask::None)
                });
                Ok(NodeTask::Thread(handle))
            }
            NodeDuty::SetNodeJoinsAllowed(joins_allowed) => {
                let mut network_api = self.network_api.clone();
                let handle = tokio::spawn(async move {
                    network_api
                        .set_joins_allowed(cfg!(feature = "always-joinable") || joins_allowed)
                        .await?;
                    Ok(NodeTask::None)
                });
                Ok(NodeTask::Thread(handle))
            }
            NodeDuty::NoOp => Ok(NodeTask::None),
        }
    }
}
