// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{
    error::convert_to_error_message,
    event_mapping::MsgContext,
    node_ops::{MsgType, NodeDuty, OutgoingMsg},
    Error,
};
use sn_messaging::{
    node::{
        NodeCmd, NodeDataQueryResponse, NodeEvent, NodeMsg, NodeQuery, NodeQueryResponse,
        NodeRewardQuery, NodeSystemCmd, NodeSystemQuery, NodeTransferCmd, NodeTransferQuery,
    },
    Aggregation, DstLocation, MessageId, Msg, SrcLocation,
};

use super::Mapping;

pub fn map_node_msg(msg: &NodeMsg, src: SrcLocation, dst: DstLocation) -> Mapping {
    match &dst {
        DstLocation::Section(_name) | DstLocation::Node(_name) => Mapping::Ok {
            op: match_node_msg(msg, src),
            ctx: Some(MsgContext::Msg {
                msg: Msg::Node(msg.clone()),
                src,
            }),
        },
        _ => {
            let msg_id = msg.id();
            let error = convert_to_error_message(Error::InvalidMessage(
                msg_id,
                format!("Invalid dst: {:?}", msg),
            ));
            if let SrcLocation::EndUser(_) = src {
                log::error!("Logic error! EndUser cannot send NodeMsgs. ({:?})", msg);
                return Mapping::Ok {
                    op: NodeDuty::NoOp,
                    ctx: None,
                };
            }
            Mapping::Ok {
                op: NodeDuty::Send(OutgoingMsg {
                    msg: MsgType::Node(NodeMsg::NodeMsgError {
                        error,
                        id: MessageId::in_response_to(&msg_id),
                        correlation_id: msg_id,
                    }),
                    section_source: false, // strictly this is not correct, but we don't expect responses to an error..
                    dst: src.to_dst(),
                    aggregation: Aggregation::AtDestination,
                }),
                ctx: Some(MsgContext::Msg {
                    msg: Msg::Node(msg.clone()),
                    src,
                }),
            }
        }
    }
}

fn match_node_msg(msg: &NodeMsg, origin: SrcLocation) -> NodeDuty {
    match msg {
        // ------ wallet register ------
        NodeMsg::NodeCmd {
            cmd: NodeCmd::System(NodeSystemCmd::RegisterWallet(wallet)),
            ..
        } => NodeDuty::SetNodeWallet {
            wallet_id: *wallet,
            node_id: origin.name(),
        },
        // Churn synch
        NodeMsg::NodeCmd {
            cmd:
                NodeCmd::System(NodeSystemCmd::ReceiveExistingData {
                    node_rewards,
                    user_wallets,
                    metadata,
                }),
            ..
        } => NodeDuty::SynchState {
            node_rewards: node_rewards.to_owned(),
            user_wallets: user_wallets.to_owned(),
            metadata: metadata.to_owned(),
        },
        NodeMsg::NodeCmd {
            cmd: NodeCmd::System(NodeSystemCmd::ProposeRewardPayout(proposal)),
            ..
        } => NodeDuty::ReceiveRewardProposal(proposal.clone()),
        NodeMsg::NodeCmd {
            cmd: NodeCmd::System(NodeSystemCmd::AccumulateRewardPayout(accumulation)),
            ..
        } => NodeDuty::ReceiveRewardAccumulation(accumulation.clone()),
        // ------ section funds -----
        NodeMsg::NodeQuery {
            query: NodeQuery::Rewards(NodeRewardQuery::GetNodeWalletKey(node_name)),
            id,
            ..
        } => NodeDuty::GetNodeWalletKey {
            node_name: *node_name,
            msg_id: *id,
            origin,
        },
        //
        // ------ transfers --------
        NodeMsg::NodeCmd {
            cmd: NodeCmd::Transfers(NodeTransferCmd::PropagateTransfer(proof)),
            id,
            ..
        } => NodeDuty::PropagateTransfer {
            proof: proof.to_owned(),
            msg_id: *id,
            origin,
        },
        // ------ metadata ------
        NodeMsg::NodeQuery {
            query: NodeQuery::Metadata { query, origin },
            id,
            ..
        } => NodeDuty::ProcessRead {
            query: query.clone(),
            id: *id,
            origin: *origin,
        },
        NodeMsg::NodeCmd {
            cmd: NodeCmd::Metadata { cmd, origin },
            id,
            ..
        } => NodeDuty::ProcessWrite {
            cmd: cmd.clone(),
            id: *id,
            origin: *origin,
        },
        //
        // ------ Adult ------
        NodeMsg::NodeQuery {
            query: NodeQuery::Chunks { query, .. },
            id,
            ..
        } => NodeDuty::ReadChunk {
            read: query.clone(),
            msg_id: *id,
        },
        NodeMsg::NodeCmd {
            cmd: NodeCmd::Chunks { cmd, origin },
            id,
            ..
        } => NodeDuty::WriteChunk {
            write: cmd.clone(),
            msg_id: *id,
            origin: *origin,
        },
        // this cmd is accumulated, thus has authority
        NodeMsg::NodeCmd {
            cmd: NodeCmd::System(NodeSystemCmd::ReplicateChunk(data)),
            id,
        } => NodeDuty::ReplicateChunk {
            data: data.clone(),
            id: *id,
        },
        NodeMsg::NodeCmd {
            cmd: NodeCmd::System(NodeSystemCmd::RepublishChunk(data)),
            id,
        } => NodeDuty::ProcessRepublish {
            chunk: data.clone(),
            msg_id: *id,
        },
        // Aggregated by us, for security
        NodeMsg::NodeQuery {
            query: NodeQuery::System(NodeSystemQuery::GetSectionElders),
            id,
            ..
        } => NodeDuty::GetSectionElders {
            msg_id: *id,
            origin,
        },
        //
        // ------ system cmd ------
        NodeMsg::NodeCmd {
            cmd: NodeCmd::System(NodeSystemCmd::StorageFull { node_id, .. }),
            ..
        } => NodeDuty::IncrementFullNodeCount { node_id: *node_id },
        //
        // ------ transfers ------
        NodeMsg::NodeQuery {
            query: NodeQuery::Transfers(NodeTransferQuery::GetReplicaEvents),
            id,
            ..
        } => NodeDuty::GetTransferReplicaEvents {
            msg_id: *id,
            origin,
        },
        // --- Adult Operation response ---
        NodeMsg::NodeEvent {
            event: NodeEvent::ChunkWriteHandled(result),
            correlation_id,
            ..
        } => NodeDuty::RecordAdultWriteLiveness {
            result: result.clone(),
            correlation_id: *correlation_id,
            src: origin.name(),
        },
        NodeMsg::NodeQueryResponse {
            response: NodeQueryResponse::Data(NodeDataQueryResponse::GetChunk(res)),
            correlation_id,
            ..
        } => NodeDuty::RecordAdultReadLiveness {
            response: sn_messaging::client::QueryResponse::GetBlob(res.clone()),
            correlation_id: *correlation_id,
            src: origin.name(),
        },
        _ => {
            let msg_id = msg.id();
            let error = convert_to_error_message(Error::InvalidMessage(
                msg_id,
                format!("Invalid dst: {:?}", msg),
            ));

            NodeDuty::Send(OutgoingMsg {
                msg: MsgType::Node(NodeMsg::NodeMsgError {
                    error,
                    id: MessageId::in_response_to(&msg_id),
                    correlation_id: msg_id,
                }),
                section_source: false, // strictly this is not correct, but we don't expect responses to an error..
                dst: origin.to_dst(),
                aggregation: Aggregation::AtDestination,
            })
        }
    }
}
