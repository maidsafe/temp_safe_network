// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{Mapping, MsgContext};
use crate::messaging::{
    client::QueryResponse,
    node::{
        NodeCmd, NodeDataQueryResponse, NodeMsg, NodeQuery, NodeQueryResponse, NodeSystemCmd,
        NodeSystemQuery,
    },
    Aggregation, DstLocation, MessageId, SrcLocation,
};
use crate::node::{
    error::convert_to_error_message,
    node_ops::{MsgType, NodeDuty, OutgoingMsg},
    Error,
};
use crate::routing::MessageReceived;
use tracing::debug;

pub fn map_node_msg(
    msg_id: MessageId,
    msg: MessageReceived, /*, src: SrcLocation, dst: DstLocation*/
) -> Mapping {
    debug!(
        "Handling Node message received event with id {}: {:?}",
        msg_id, msg
    );
    Mapping {
        op: match_node_msg(msg, src),
        ctx: Some(MsgContext {
            msg: MsgType::Node(msg),
            src,
        }),
    }

    /*
    match &dst {
        DstLocation::Section(_) | DstLocation::Node(_) => Mapping {
            op: match_node_msg(msg.clone(), src),
            ctx: Some(MsgContext {
                msg: MsgType::Node(msg),
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
                tracing::error!("Logic error! EndUser cannot send NodeMsgs. ({:?})", msg);
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
                ctx: Some(MsgContext {
                    msg: MsgType::Node(msg),
                    src,
                }),
            }
        }
    }
    */
}

fn match_node_msg(msg: MessageReceived, origin: SrcLocation) -> NodeDuty {
    match msg {
        // Churn synch
        NodeMsg::NodeCmd(NodeCmd::System(NodeSystemCmd::ReceiveExistingData { metadata })) => NodeDuty::SynchState { metadata },
        // ------ metadata ------
        MessageReceived::NodeQuery {
            query:
                NodeQuery::Metadata {
                    query,
                    client_sig,
                    origin,
                },
            id,
            ..
        } => {
            // FIXME: ******** validate client signature!!!! *********
            NodeDuty::ProcessRead {
                query,
                msg_id: id,
                client_sig,
                origin,
            }
        }
        MessageReceived::NodeCmd {
            cmd:
                NodeCmd::Metadata {
                    cmd,
                    client_sig,
                    origin,
                },
            id,
            ..
        } => {
            // FIXME: ******** validate client signature!!!! *********
            NodeDuty::ProcessWrite {
                cmd,
                msg_id: id,
                client_sig,
                origin,
            }
        }
        //
        // ------ Adult ------
        MessageReceived::NodeQuery {
            query: NodeQuery::Chunks { query, .. },
            id,
            ..
        } => NodeDuty::ReadChunk {
            read: query,
            msg_id: id,
        },
        MessageReceived::NodeCmd {
            cmd: NodeCmd::Chunks {
                cmd, client_sig, ..
            },
            id,
            ..
        } => NodeDuty::WriteChunk {
            write: cmd,
            msg_id: id,
            client_sig,
        },
        // this cmd is accumulated, thus has authority
        MessageReceived::NodeCmd {
            cmd: NodeCmd::System(NodeSystemCmd::ReplicateChunk(chunk)),
            id,
        } => NodeDuty::ReplicateChunk { chunk, msg_id: id },
        MessageReceived::NodeCmd {
            cmd: NodeCmd::System(NodeSystemCmd::RepublishChunk(chunk)),
            id,
        } => NodeDuty::ProcessRepublish { chunk, msg_id: id },
        // Aggregated by us, for security
        MessageReceived::NodeQuery {
            query: NodeQuery::System(NodeSystemQuery::GetSectionElders),
            id,
            ..
        } => NodeDuty::GetSectionElders { msg_id: id, origin },
        //
        // ------ system cmd ------
        MessageReceived::NodeCmd {
            cmd: NodeCmd::System(NodeSystemCmd::StorageFull { node_id, .. }),
            ..
        } => NodeDuty::IncrementFullNodeCount { node_id },
        // --- Adult Operation response ---
        MessageReceived::NodeQueryResponse {
            response: NodeQueryResponse::Data(NodeDataQueryResponse::GetChunk(res)),
            correlation_id,
            ..
        } => NodeDuty::RecordAdultReadLiveness {
            response: QueryResponse::GetChunk(res),
            correlation_id,
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
