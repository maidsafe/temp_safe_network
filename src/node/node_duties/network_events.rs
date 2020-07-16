// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use safe_nd::{
    MsgEnvelope, XorName, PublicKey,
};
use super::msg_analysis::NetworkMsgAnalysis;

pub struct NetworkEvents {
    analysis: NetworkMsgAnalysis,
}

impl NetworkEvents {

    pub fn new(analysis: NetworkMsgAnalysis) -> Self {
        Self {
            analysis
        }
    }

    pub fn process(&mut self, event: RoutingEvent) -> Option<NodeOperation> {
        use NodeOperation::*;
        use GatewayDuty::*;
        use RewardDuty::*;
        use ElderDuty::*;
        match event {
            RoutingEvent::Consensus(custom_event) => {
                match bincode::deserialize::<GroupDecision>(&custom_event) {
                    Ok(group_decision) => Some(RunAsGateway(ProcessGroupDecision(group_decision))),
                    Err(e) => {
                        error!("Invalid GroupDecision passed from Routing: {:?}", e);
                        None
                    }
                }
            }
            RoutingEvent::Promoted => {
                info!("Node promoted to Elder");
                Some(NodeDuty::BecomeElder)
            }
            RoutingEvent::MemberLeft { name, age } => {
                trace!("A node has left the section. Node: {:?}", name);
                Some(RunAsElder(ProcessLostMember {
                    name: XorName(name.0), 
                    age, 
                })
            }
            RoutingEvent::MemberJoined { name, previous_name, .. } => {
                trace!("New member has joined the section");
                info!("No. of Elders: {}", self.routing.borrow().our_elders().count());
                info!("No. of Adults: {}", self.routing.borrow().our_adults().count());
                Some(RunAsRewards(AddRelocatedAccount {
                    old_node_id: XorName(name.0), 
                    new_node_id: XorName(previous_name.0),
                }))
            }
            RoutingEvent::Connected(_) => {
                info!("Node promoted to Adult");
                Some(NodeDuty::BecomeAdult)
            }
            RoutingEvent::MessageReceived { content, src, dst } => {
                info!(
                    "Received network message: {:8?}\n Sent from {:?} to {:?}",
                    HexFmt(&content),
                    src,
                    dst
                );
                self.evaluate_msg(content)
            }
            RoutingEvent::EldersChanged { prefix, key, elders } => {
                Some(RunAsElder(ProcessElderChange {
                    prefix, 
                    key: PublicKey::Bls(key), 
                    elders,
                })
            },
            // Ignore all other events
            _ => None,
        }
    }

    fn evaluate_msg(&mut self, content: Vec<u8>) -> Option<NodeOperation> {
        match bincode::deserialize::<MsgEnvelope>(&content) {
            Ok(msg) => self.analysis.evaluate(&msg),
            Err(e) => {
                error!(
                    "Error deserializing received network message into MsgEnvelope type: {:?}",
                    e
                );
                None
            }
        }
    }
}