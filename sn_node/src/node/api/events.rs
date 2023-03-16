// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{NodeContext, Prefix, RejoinReason};

use strum_macros::Display as StrumDisplay;
use tokio::sync::broadcast;

/// Channel where users of the public API can listen to events broadcasted by the node.
#[derive(Clone, Debug)]
pub struct NodeEventsChannel(broadcast::Sender<NodeEvent>);

impl Default for NodeEventsChannel {
    fn default() -> Self {
        Self(broadcast::channel(100).0)
    }
}

impl NodeEventsChannel {
    /// Returns a new receiver to listen to the channel.
    /// Multiple receivers can be actively listening.
    pub fn subscribe(&self) -> broadcast::Receiver<NodeEvent> {
        self.0.subscribe()
    }

    // Broadcast a new event, meant to be a helper only used by the sn_node's internals.
    pub(crate) fn broadcast(&self, event: NodeEvent) {
        debug!("Node event: {event}");
        if let Err(err) = self.0.send(event.clone()) {
            trace!("Error occurred when trying to broadcast a node event ({event}): {err}");
        }
    }
}

/// Type of events broadcasted by the sn_node to the public API.
#[derive(Clone, StrumDisplay, Debug)]
pub enum NodeEvent {
    /// Node needs to rejoin the network. The user application is expected to perform this duty.
    RejoinRequired(RejoinReason),
    /// Node's internal state was updated.
    ContextUpdated(NodeContext),
    /// The node has been promoted to be an Elder in its section.
    PromotedToElder,
    /// The node has been demoted from being an Elder in its section.
    DemotedFromElder,
    /// The node's section successfully split, and it now has a new prefix.
    SplitSuccess(Prefix),
    /// The node started a relocation process
    RelocateStart,
    /// The node finished a relocation process
    RelocateEnd,
}
