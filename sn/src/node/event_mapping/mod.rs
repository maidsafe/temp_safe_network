// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::SrcLocation;
use crate::node::{network::Network, node_ops::NodeDuty};
use crate::routing::{Event as RoutingEvent, MessageReceived, MIN_AGE};
use tracing::info;

#[derive(Debug)]
pub(super) struct Mapping {
    pub(super) op: NodeDuty,
    pub(super) ctx: Option<MsgContext>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(super) enum MsgContext {
    Node {
        msg: MessageReceived,
        src: SrcLocation,
    },
}

// Process any routing event
pub(super) async fn map_routing_event(event: RoutingEvent, network_api: &Network) -> Mapping {
    info!("Handling RoutingEvent: {:?}", event);
    match event {
        RoutingEvent::Relocated { .. } => {
            // Check our current status
            let age = network_api.age().await;
            if age > MIN_AGE {
                info!("Relocated, our Age: {:?}", age);
            }
            Mapping {
                op: NodeDuty::NoOp,
                ctx: None,
            }
        }
        // Ignore all other events
        _ => Mapping {
            op: NodeDuty::NoOp,
            ctx: None,
        },
    }
}
