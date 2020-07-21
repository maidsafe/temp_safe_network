// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::node_duties::accumulation::Accumulation;
use crate::node::node_ops::{
    AdultDuty, ChunkDuty, GatewayDuty, MessagingDuty, MetadataDuty, NodeDuty, NodeOperation,
    PaymentDuty, RewardDuty, TransferDuty,
};
use crate::node::state_db::AgeGroup;

use log::error;
use routing::Node as Routing;
use safe_nd::{
    Address, Cmd, DataCmd, DataQuery, Duty, ElderDuties, Message, MsgEnvelope, MsgSender, NodeCmd,
    NodeEvent, NodeRewardCmd, Query, XorName,
};
use std::{cell::RefCell, rc::Rc};

/// Currently, this is only evaluating
/// remote msgs from the network, i.e.
/// it is not evaluating msgs sent
/// directly from the client.
pub struct NetworkMsgAnalysis {
    accumulation: Accumulation,
    routing: Rc<RefCell<Routing>>,
}

impl NetworkMsgAnalysis {
    pub fn new(routing: Rc<RefCell<Routing>>) -> Self {
        Self {
            accumulation: Accumulation::new(routing.clone()),
            routing,
        }
    }

    pub fn is_dst_for(&self, msg: &MsgEnvelope) -> bool {
        self.self_is_handler_for(&msg.destination().xorname())
    }

    /// Currently, this is only evaluating
    /// remote msgs from the network, i.e.
    /// it is not evaluating msgs sent
    /// directly from the client.
    pub fn evaluate(&mut self, msg: &MsgEnvelope) -> Option<NodeOperation> {
        use NodeDuty::*;
        use NodeOperation::*;
        let result = if self.should_accumulate(msg) {
            let msg = self.accumulation.process(msg)?;
            self.evaluate(&msg)?
        } else if let Some(duty) = self.try_messaging(msg) {
            // Identified as an outbound msg, to be sent on the wire.
            RunAsNode(ProcessMessaging(duty))
        } else if let Some(duty) = self.try_gateway(msg) {
            // Client auth cmd finalisation (Temporarily handled here, will be at app layer (Authenticator)).
            // The auth cmd has been agreed by the Gateway section.
            // (All other client msgs are handled when received from client).
            RunAsGateway(duty)
        } else if let Some(duty) = self.try_data_payment(msg) {
            // Incoming msg from `Gateway`!
            RunAsPayment(duty) // Payment Elders should just execute and send onwards.
        } else if let Some(duty) = self.try_metadata(msg) {
            // Accumulated msg from `Payment`!
            RunAsMetadata(duty)
        } else if let Some(duty) = self.try_adult(msg) {
            // Accumulated msg from `Metadata`!
            RunAsAdult(duty)
        } else if let Some(duty) = self.try_rewards(msg) {
            // Identified as a Rewards msg
            RunAsRewards(duty)
        } else if let Some(duty) = self.try_transfers(msg) {
            // Identified as a Transfers msg
            RunAsTransfers(duty)
        } else {
            error!("Unknown message destination: {:?}", msg.id());
            Unknown
        };
        Some(result)
    }

    fn try_messaging(&self, msg: &MsgEnvelope) -> Option<MessagingDuty> {
        use Address::*;
        let destined_for_network = || match msg.destination() {
            Client(address) => !self.self_is_handler_for(&address),
            Node(address) => routing::XorName(address.0) != *self.routing.borrow().id().name(),
            Section(address) => !self.self_is_handler_for(&address),
        };
        let from_client = || match msg.most_recent_sender() {
            MsgSender::Client { .. } => true,
            _ => false,
        };
        let is_auth_cmd = || match msg.message {
            Message::Cmd {
                cmd: Cmd::Auth { .. },
                ..
            } => true,
            _ => false,
        };

        if destined_for_network() || (from_client() && !is_auth_cmd()) {
            Some(MessagingDuty::SendToSection(msg.clone()))
        } else {
            None
        }
    }

    // ----  Accumulation ----

    fn should_accumulate(&self, msg: &MsgEnvelope) -> bool {
        // Incoming msg from `Payment`!
        self.should_accumulate_for_metadata_write(msg) // Metadata Elders accumulate the msgs from Payment Elders.
        // Incoming msg from `Metadata`!
        || self.should_accumulate_for_adult(msg) // Adults accumulate the msgs from Metadata Elders.
        || self.should_accumulate_for_rewards(msg) // Rewards Elders accumulate the claim counter cmd from other Rewards Elders
    }

    /// The individual Payment Elder nodes send their msgs
    /// to Metadata section, where it is accumulated.
    fn should_accumulate_for_metadata_write(&self, msg: &MsgEnvelope) -> bool {
        let from_single_payment_elder = || match msg.most_recent_sender() {
            MsgSender::Node {
                duty: Duty::Elder(ElderDuties::Payment),
                ..
            } => true,
            _ => false,
        };
        let is_data_cmd = || match msg.message {
            Message::Cmd {
                cmd: Cmd::Data { .. },
                ..
            } => true,
            _ => false,
        };

        is_data_cmd() && from_single_payment_elder() && self.is_dst_for(msg) && self.is_elder()
    }

    fn should_accumulate_for_rewards(&self, msg: &MsgEnvelope) -> bool {
        let from_single_rewards_elder = || match msg.most_recent_sender() {
            MsgSender::Node {
                duty: Duty::Elder(ElderDuties::Rewards),
                ..
            } => true,
            _ => false,
        };
        let is_accumulating_reward_cmd = || match msg.message {
            Message::NodeCmd {
                cmd: NodeCmd::Rewards(NodeRewardCmd::ClaimRewardCounter { .. }),
                ..
            } => true,
            _ => false,
        };

        is_accumulating_reward_cmd()
            && from_single_rewards_elder()
            && self.is_dst_for(msg)
            && self.is_elder()
    }

    /// Adults accumulate the write requests from Elders.
    fn should_accumulate_for_adult(&self, msg: &MsgEnvelope) -> bool {
        let from_single_metadata_elder = || match msg.most_recent_sender() {
            MsgSender::Node {
                duty: Duty::Elder(ElderDuties::Metadata),
                ..
            } => true,
            _ => false,
        };
        let is_chunk_msg = || match msg.message {
            Message::Cmd {
                cmd:
                    Cmd::Data {
                        cmd: DataCmd::Blob(_),
                        ..
                    },
                ..
            }
            | Message::Query {
                query: Query::Data(DataQuery::Blob(_)),
                ..
            } => true,
            _ => false,
        };

        is_chunk_msg() && from_single_metadata_elder() && self.is_dst_for(msg) && self.is_adult()
    }

    // ---- .... -----

    // todo: eval all msg types!
    fn try_gateway(&self, msg: &MsgEnvelope) -> Option<GatewayDuty> {
        let from_client = || match msg.origin {
            MsgSender::Client { .. } => true,
            _ => false,
        };
        let agreed_by_gateway_section = || match msg.most_recent_sender() {
            MsgSender::Section {
                duty: Duty::Elder(ElderDuties::Gateway),
                ..
            } => true,
            _ => false,
        };
        let is_auth_cmd = || match msg.message {
            Message::Cmd {
                cmd: Cmd::Auth { .. },
                ..
            } => true,
            _ => false,
        };

        let from_network_to_client = || match msg.destination() {
            Address::Client(xorname) => {
                let from_gateway = match msg.most_recent_sender() {
                    MsgSender::Node {
                        duty: Duty::Elder(ElderDuties::Gateway),
                        ..
                    }
                    | MsgSender::Section {
                        duty: Duty::Elder(ElderDuties::Gateway),
                        ..
                    } => true,
                    _ => false,
                };
                !from_gateway && self.self_is_handler_for(&xorname)
            }
            _ => false,
        };

        let is_gateway_msg = from_network_to_client()
            || (from_client()
                && agreed_by_gateway_section()
                && is_auth_cmd()
                && self.is_dst_for(msg)
                && self.is_elder());

        if is_gateway_msg {
            Some(GatewayDuty::ProcessMsg(msg.clone()))
        } else {
            None
        }
    }

    /// We do not accumulate these request, they are executed
    /// at once (i.e. payment carried out) and sent on to
    /// Metadata section. (They however, will accumulate those msgs.)
    /// The reason for this is that the payment request is already signed
    /// by the client and validated by its replicas,
    /// so there is no reason to accumulate it here.
    fn try_data_payment(&self, msg: &MsgEnvelope) -> Option<PaymentDuty> {
        let from_gateway_single_elder = || match msg.most_recent_sender() {
            MsgSender::Node {
                duty: Duty::Elder(ElderDuties::Gateway),
                ..
            } => true,
            _ => false,
        };

        let is_data_write = || match msg.message {
            Message::Cmd {
                cmd: Cmd::Data { .. },
                ..
            } => true,
            _ => false,
        };

        if is_data_write() && from_gateway_single_elder() && self.is_dst_for(msg) && self.is_elder()
        {
            Some(PaymentDuty::ProcessPayment(msg.clone()))
        } else {
            None
        }
    }

    /// After the data write sent from Payment Elders has been
    /// accumulated (can be seen since the sender is `Section`),
    /// it is time to actually carry out the write operation.
    fn try_metadata(&self, msg: &MsgEnvelope) -> Option<MetadataDuty> {
        // Is it a data cmd?
        let is_data_cmd = || match msg.message {
            Message::Cmd {
                cmd: Cmd::Data { .. },
                ..
            } => true,
            _ => false,
        };
        let from_payment_section = || match msg.most_recent_sender() {
            MsgSender::Section {
                duty: Duty::Elder(ElderDuties::Payment),
                ..
            } => true,
            _ => false,
        };

        // Is it a data query?
        let is_data_query = || match msg.message {
            Message::Query {
                query: Query::Data(_),
                ..
            } => true,
            _ => false,
        };
        let from_single_gateway_elder = || match msg.most_recent_sender() {
            MsgSender::Node {
                duty: Duty::Elder(ElderDuties::Gateway),
                ..
            } => true,
            _ => false,
        };

        let is_correct_dst = |msg| self.is_dst_for(msg) && self.is_elder();

        let duty = if is_data_query() && from_single_gateway_elder() && is_correct_dst(msg) {
            MetadataDuty::ProcessRead(msg.clone())
        } else if is_data_cmd() && from_payment_section() && is_correct_dst(msg) {
            MetadataDuty::ProcessWrite(msg.clone())
        } else {
            return None;
        };
        Some(duty)
    }

    /// When the write requests from Elders has been accumulated
    /// at an Adult, it is time to carry out the write operation.
    fn try_adult(&self, msg: &MsgEnvelope) -> Option<AdultDuty> {
        let from_metadata_section = || match msg.most_recent_sender() {
            MsgSender::Section {
                duty: Duty::Elder(ElderDuties::Metadata),
                ..
            } => true,
            _ => false,
        };

        let is_chunk_query = || match msg.message {
            Message::Query {
                query: Query::Data(DataQuery::Blob(_)),
                ..
            } => true,
            _ => false,
        };

        let is_chunk_cmd = || match msg.message {
            Message::Cmd {
                cmd:
                    Cmd::Data {
                        cmd: DataCmd::Blob(_),
                        ..
                    },
                ..
            } => true,
            _ => false,
        };

        if from_metadata_section() && self.is_dst_for(msg) && self.is_adult() {
            let duty = if is_chunk_cmd() {
                AdultDuty::RunAsChunks(ChunkDuty::WriteChunk(msg.clone()))
            } else if is_chunk_query() {
                AdultDuty::RunAsChunks(ChunkDuty::ReadChunk(msg.clone()))
            } else {
                return None;
            };
            return Some(duty);
        }
        None
    }

    fn try_rewards(&self, msg: &MsgEnvelope) -> Option<RewardDuty> {
        let from_rewards_section = || match msg.most_recent_sender() {
            MsgSender::Section {
                duty: Duty::Elder(ElderDuties::Rewards),
                ..
            } => true,
            _ => false,
        };

        if from_rewards_section() && self.is_dst_for(msg) && self.is_elder() {
            use NodeRewardCmd::*;
            let duty = match &msg.message {
                Message::NodeCmd {
                    cmd:
                        NodeCmd::Rewards(ClaimRewardCounter {
                            old_node_id,
                            new_node_id,
                        }),
                    id,
                } => RewardDuty::ClaimRewardCounter {
                    old_node_id: *old_node_id,
                    new_node_id: *new_node_id,
                    msg_id: *id,
                    origin: msg.origin.address(),
                },
                Message::NodeEvent {
                    event:
                        NodeEvent::RewardCounterClaimed {
                            account_id,
                            new_node_id,
                            counter,
                        },
                    ..
                } => RewardDuty::ReceiveClaimedRewards {
                    id: *account_id,
                    node_id: *new_node_id,
                    counter: counter.clone(),
                },
                Message::NodeEvent {
                    event: NodeEvent::RewardPayoutValidated(validation),
                    ..
                } => RewardDuty::ReceiveRewardValidation(validation.clone()),
                _ => return None,
            };
            return Some(duty);
        }
        None
    }

    fn try_transfers(&self, msg: &MsgEnvelope) -> Option<TransferDuty> {
        let from_single_gateway_elder = || match msg.most_recent_sender() {
            MsgSender::Node {
                duty: Duty::Elder(ElderDuties::Gateway),
                ..
            } => true,
            _ => false,
        };

        let shall_process =
            |msg| from_single_gateway_elder() && self.is_dst_for(msg) && self.is_elder();

        let duty = match &msg.message {
            Message::Cmd {
                cmd: Cmd::Transfer(cmd),
                ..
            } => {
                if !shall_process(msg) {
                    return None;
                }
                TransferDuty::ProcessCmd {
                    cmd: cmd.clone().into(),
                    msg_id: msg.id(),
                    origin: msg.origin.address(),
                }
            }
            Message::Query {
                query: Query::Transfer(query),
                ..
            } => {
                if !shall_process(msg) {
                    return None;
                }
                TransferDuty::ProcessQuery {
                    query: query.clone().into(),
                    msg_id: msg.id(),
                    origin: msg.origin.address(),
                }
            }
            _ => return None,
        };
        Some(duty)
    }

    fn self_is_handler_for(&self, address: &XorName) -> bool {
        let xorname = routing::XorName(address.0);
        match self.routing.borrow().matches_our_prefix(&xorname) {
            Ok(result) => result,
            _ => false,
        }
    }

    fn is_elder(&self) -> bool {
        if let AgeGroup::Elder = self.our_duties() {
            true
        } else {
            false
        }
    }

    fn is_adult(&self) -> bool {
        if let AgeGroup::Adult = self.our_duties() {
            true
        } else {
            false
        }
    }

    fn our_duties(&self) -> AgeGroup {
        if self.routing.borrow().is_elder() {
            AgeGroup::Elder
        } else if self
            .routing
            .borrow()
            .our_adults()
            .map(|c| c.name())
            .any(|x| x == self.routing.borrow().name())
        {
            AgeGroup::Adult
        } else {
            AgeGroup::Infant
        }
    }
}
