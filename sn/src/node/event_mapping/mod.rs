// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::SrcLocation;
use crate::node::{network::Network, node_ops::NodeDuty};
use crate::routing::{Event as RoutingEvent, MessageReceived, NodeElderChange, MIN_AGE};
use crate::types::PublicKey;
use tracing::{debug, error, info, trace};

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
        RoutingEvent::SectionSplit {
            elders,
            self_status_change,
        } => {
            let newbie = match self_status_change {
                NodeElderChange::None => false,
                NodeElderChange::Promoted => true,
                NodeElderChange::Demoted => {
                    error!("This should be unreachable, as there would be no demotions of Elders during a split.");
                    return Mapping {
                        op: NodeDuty::NoOp,
                        ctx: None,
                    };
                }
            };
            Mapping {
                op: NodeDuty::SectionSplit {
                    our_prefix: elders.prefix,
                    our_key: PublicKey::from(elders.key),
                    newbie,
                },
                ctx: None,
            }
        }

        RoutingEvent::MemberJoined { previous_name, .. } => {
            log_network_stats(network_api).await;
            let op = if previous_name.is_some() {
                trace!("A relocated node has joined the section.");
                // Switch joins_allowed off a new adult joining.
                NodeDuty::SetNodeJoinsAllowed(false)
            } else if network_api.our_prefix().await.is_empty() {
                NodeDuty::NoOp
            } else {
                NodeDuty::SetNodeJoinsAllowed(false)
            };
            Mapping { op, ctx: None }
        }
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

pub(super) async fn log_network_stats(network_api: &Network) {
    let adults = network_api.our_adults().await.len();
    let elders = network_api.our_elder_names().await.len();
    let prefix = network_api.our_prefix().await;
    debug!("{:?}: {:?} Elders, {:?} Adults.", prefix, elders, adults);
}
