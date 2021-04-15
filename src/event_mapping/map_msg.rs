// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{Mapping, MsgContext};
use crate::error::convert_to_error_message;
use crate::node_ops::{MsgType, OutgoingMsg};
use crate::{event_mapping::Msg, node_ops::NodeDuty, Error};
use log::{debug, warn};
use sn_messaging::client::{ClientMsg, ProcessingError};
use sn_messaging::node::{
    NodeCmd, NodeEvent, NodeMsg, NodeQuery, NodeRewardQuery, NodeSystemCmd, NodeSystemQuery,
    NodeTransferCmd, NodeTransferQuery,
};
use sn_messaging::{
    client::{
        Cmd, Message, NodeCmd, NodeEvent, NodeQuery, NodeQueryResponse, NodeRewardQuery,
        NodeSystemCmd, NodeSystemQuery, NodeTransferCmd, NodeTransferQuery, ProcessMsg, Query,
        QueryResponse, TransferCmd, TransferQuery,
    },
    Aggregation, DstLocation, EndUser, MessageId, Msg, SrcLocation,
};

pub fn match_user_sent_msg(msg: ProcessMsg, origin: EndUser) -> Mapping {
    match msg.to_owned() {
        ProcessMsg::Query {
            query: Query::Data(query),
            id,
            ..
        } => Mapping::Ok {
            op: NodeDuty::ProcessRead { query, id, origin },
            ctx: Some(super::MsgContext::Msg {
                msg: Msg::Client(ClientMsg::Process(msg)),
                src: SrcLocation::EndUser(origin),
            }),
        },
        ProcessMsg::QueryResponse {
            response,
            correlation_id,
            ..
        } => Mapping::Ok {
            op: NodeDuty::ProcessBlobReadResult {
                response,
                original_msg_id: correlation_id,
                src: origin.name(),
            },
            ctx: Some(super::MsgContext::Msg {
                msg: Msg::Client(ClientMsg::Process(msg)),
                src: SrcLocation::EndUser(origin),
            }),
        },
        ProcessMsg::Cmd {
            cmd: Cmd::Data { .. },
            ..
        } => Mapping::Ok {
            op: NodeDuty::ProcessDataPayment {
                msg: msg.clone(),
                origin,
            },
            ctx: Some(MsgContext::Msg {
                msg: Msg::Client(ClientMsg::Process(msg)),
                src: SrcLocation::EndUser(origin),
            }),
        },
        ProcessMsg::Cmd {
            cmd: Cmd::Transfer(TransferCmd::ValidateTransfer(signed_transfer)),
            id,
            ..
        } => Mapping::Ok {
            op: NodeDuty::ValidateClientTransfer {
                signed_transfer,
                origin: SrcLocation::EndUser(origin),
                msg_id: id,
            },
            ctx: Some(MsgContext::Msg {
                msg: Msg::Client(ClientMsg::Process(msg)),
                src: SrcLocation::EndUser(origin),
            }),
        },
        // TODO: Map more transfer cmds
        ProcessMsg::Cmd {
            cmd: Cmd::Transfer(TransferCmd::SimulatePayout(transfer)),
            id,
            ..
        } => Mapping::Ok {
            op: NodeDuty::SimulatePayout {
                transfer,
                origin: SrcLocation::EndUser(origin),
                msg_id: id,
            },
            ctx: Some(MsgContext::Msg {
                msg: Msg::Client(ClientMsg::Process(msg)),
                src: SrcLocation::EndUser(origin),
            }),
        },
        ProcessMsg::Cmd {
            cmd: Cmd::Transfer(TransferCmd::RegisterTransfer(proof)),
            id,
            ..
        } => Mapping::Ok {
            op: NodeDuty::RegisterTransfer { proof, msg_id: id },
            ctx: Some(MsgContext::Msg {
                msg: Msg::Client(ClientMsg::Process(msg)),
                src: SrcLocation::EndUser(origin),
            }),
        },
        // TODO: Map more transfer queries
        ProcessMsg::Query {
            query: Query::Transfer(TransferQuery::GetHistory { at, since_version }),
            id,
            ..
        } => Mapping::Ok {
            op: NodeDuty::GetTransfersHistory {
                at,
                since_version,
                origin: SrcLocation::EndUser(origin),
                msg_id: id,
            },
            ctx: Some(MsgContext::Msg {
                msg: Msg::Client(ClientMsg::Process(msg)),
                src: SrcLocation::EndUser(origin),
            }),
        },
        ProcessMsg::Query {
            query: Query::Transfer(TransferQuery::GetBalance(at)),
            id,
            ..
        } => Mapping::Ok {
            op: NodeDuty::GetBalance {
                at,
                origin: SrcLocation::EndUser(origin),
                msg_id: id,
            },
            ctx: Some(MsgContext::Msg {
                msg: Msg::Client(ClientMsg::Process(msg)),
                src: SrcLocation::EndUser(origin),
            }),
        },
        ProcessMsg::Query {
            query: Query::Transfer(TransferQuery::GetStoreCost { bytes, .. }),
            id,
            ..
        } => Mapping::Ok {
            op: NodeDuty::GetStoreCost {
                bytes,
                origin: SrcLocation::EndUser(origin),
                msg_id: id,
            },
            ctx: Some(MsgContext::Msg {
                msg: Msg::Client(ClientMsg::Process(msg)),
                src: SrcLocation::EndUser(origin),
            }),
        },
        _ => Mapping::Error {
            error: Error::InvalidMessage(msg.id(), format!("Unknown user msg: {:?}", msg)),
            msg: MsgContext::Msg {
                msg: Msg::Client(ClientMsg::Process(msg)),
                src: SrcLocation::EndUser(origin),
            },
        },
    }
}

pub fn map_node_msg(msg: NodeMsg, src: SrcLocation, dst: DstLocation) -> Mapping {
    match &dst {
        DstLocation::Section(_name) | DstLocation::Node(_name) => Mapping {
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
            ))?;
            if let SrcLocation::EndUser(_) = src {
                log::error!("Logic error! EndUser cannot send NodeMsgs. ({:?})", msg);
                return Mapping {
                    op: NodeDuty::NoOp,
                    ctx: None,
                };
            }
            Mapping {
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

/// Map a process error to relevant node duties
pub fn map_node_process_err_msg(
    msg: ProcessingError,
    src: SrcLocation,
    dst: DstLocation,
) -> Mapping {
    // debug!(" Handling received process err msg. {:?}.", msg);
    match &dst {
        DstLocation::Section(_name) | DstLocation::Node(_name) => match_process_err(msg, src),
        _ => Mapping::Error {
            error: Error::InvalidMessage(msg.id(), format!("Invalid dst: {:?}", msg)),
            msg: MsgContext::Msg {
                msg: Msg::Client(ClientMsg::ProcessingError(msg)),
                src,
            },
        },
    }
}

fn match_process_err(msg: ProcessingError, src: SrcLocation) -> Mapping {
    if let Some(reason) = msg.clone().reason() {
        // debug!("ProcessingError with reason")
        return match reason {
            sn_messaging::client::Error::NoSectionFunds => {
                debug!("error NO FUNDS BEING HANDLED");
                Mapping::Ok {
                    op: NodeDuty::ProvideSectionWalletSupportingInfo,
                    ctx: Some(MsgContext::Msg {
                        msg: Msg::Client(ClientMsg::ProcessingError(msg)),
                        src,
                    }),
                }
            }
            _ => {
                warn!(
                    "TODO: We do not handle this process error reason yet. {:?}",
                    reason
                );
                // do nothing
                Mapping::Ok {
                    op: NodeDuty::NoOp,
                    ctx: None,
                }
            }
        };
    }

    Mapping::Error {
        error: Error::CannotUpdateProcessErrorNode,
        msg: MsgContext::Msg {
            msg: Msg::Client(ClientMsg::ProcessingError(msg)),
            src,
        },
    }
}

fn match_or_err(msg: NodeMsg, src: SrcLocation) -> Mapping {
    match match_node_msg(msg.clone(), src) {
        NodeDuty::NoOp => Mapping::Error {
            error: Error::InvalidMessage(msg.id(), format!("Unknown msg: {:?}", msg)),
            msg: MsgContext::Msg {
                msg: Msg::Node(msg),
                src,
            },
        },
        op => Mapping::Ok {
            op,
            ctx: Some(MsgContext::Msg {
                msg: Msg::Node(msg),
                src,
            }),
        },
    }
}

fn match_node_msg(msg: NodeMsg, origin: SrcLocation) -> NodeDuty {
    match &msg {
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
        ProcessMsg::NodeEvent {
            event: NodeEvent::SectionWalletCreated(wallet_history),
            id,
            ..
        } => NodeDuty::ReceiveSectionWalletHistory {
            wallet_history: wallet_history.clone(),
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
        // ------ adult ------
        NodeMsg::NodeQuery {
            query: NodeQuery::Chunks { query, origin },
            id,
            ..
        } => NodeDuty::ReadChunk {
            read: query.clone(),
            msg_id: *id,
            origin: *origin,
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
        // --- Adult ---
        NodeMsg::NodeQuery {
            query: NodeQuery::Chunks { query, origin },
            id,
            ..
        } => NodeDuty::ReadChunk {
            read: query.clone(),
            msg_id: *id,
            origin: *origin,
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
        NodeMsg::QueryResponse {
            response,
            correlation_id,
            ..
        } if matches!(response, QueryResponse::GetBlob(_)) => NodeDuty::RecordAdultReadLiveness {
            response: response.clone(),
            correlation_id: *correlation_id,
            src: origin.name(),
        },
        _ => {
            warn!("Node ProcessMsg from not handled: {:?}", msg);
            NodeDuty::NoOp
        }
    }
}
