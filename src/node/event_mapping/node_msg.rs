// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{Mapping, MsgContext};
use crate::messaging::{
    data::{DataCmd, QueryResponse, ServiceMsg},
    node::{NodeCmd, NodeMsg, NodeQuery, NodeQueryResponse},
    Authority, DstLocation, MessageId, ServiceOpSig, SrcLocation, WireMsg,
};
use crate::node::{
    error::convert_to_error_message,
    node_ops::{MsgType, NodeDuty, OutgoingMsg},
    Error,
};
use crate::routing::MessageReceived;
use tracing::debug;

pub(super) fn map_node_msg(
    msg_id: MessageId,
    src: SrcLocation,
    dst: DstLocation,
    msg: MessageReceived,
) -> Mapping {
    debug!(
        "Handling Node message received event with id {}: {:?}",
        msg_id, msg
    );

    match dst {
        DstLocation::Section { .. } | DstLocation::Node { .. } => Mapping {
            op: match_node_msg(msg_id, msg.clone(), src),
            ctx: Some(MsgContext::Node { msg, src }),
        },
        _ => {
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
                    id: MessageId::in_response_to(&msg_id),
                    msg: MsgType::Node(NodeMsg::NodeMsgError {
                        error,
                        correlation_id: msg_id,
                    }),
                    dst: src.to_dst(),
                    aggregation: true,
                }),
                ctx: Some(MsgContext::Node { msg, src }),
            }
        }
    }
}

fn match_node_msg(msg_id: MessageId, msg: MessageReceived, origin: SrcLocation) -> NodeDuty {
    match msg {
        // Churn synch
        MessageReceived::NodeCmd(NodeCmd::ReceiveExistingData { metadata }) => {
            NodeDuty::SynchState { metadata }
        }
        // ------ metadata ------
        MessageReceived::NodeQuery(NodeQuery::Metadata {
            query,
            data_signed,
            origin: query_origin,
        }) => {
            match verify_authority(
                msg_id,
                origin,
                data_signed,
                ServiceMsg::Query(query.clone()),
            ) {
                Ok(auth) => NodeDuty::ProcessRead {
                    query,
                    msg_id,
                    auth,
                    origin: query_origin,
                },
                Err(duty) => duty,
            }
        }
        MessageReceived::NodeCmd(NodeCmd::Metadata {
            cmd,
            data_signed,
            origin: cmd_origin,
        }) => match verify_authority(msg_id, origin, data_signed, ServiceMsg::Cmd(cmd.clone())) {
            Ok(auth) => NodeDuty::ProcessWrite {
                cmd,
                msg_id,
                auth,
                origin: cmd_origin,
            },
            Err(duty) => duty,
        },
        //
        // ------ Adult ------
        MessageReceived::NodeQuery(NodeQuery::Chunks { query, .. }) => NodeDuty::ReadChunk {
            read: query,
            msg_id,
        },
        MessageReceived::NodeCmd(NodeCmd::Chunks {
            cmd, data_signed, ..
        }) => {
            match verify_authority(
                msg_id,
                origin,
                data_signed,
                ServiceMsg::Cmd(DataCmd::Chunk(cmd.clone())),
            ) {
                Ok(auth) => NodeDuty::WriteChunk {
                    write: cmd,
                    msg_id,
                    auth,
                },
                Err(duty) => duty,
            }
        }
        // this cmd is accumulated, thus has authority
        MessageReceived::NodeCmd(NodeCmd::ReplicateChunk(chunk)) => {
            NodeDuty::ReplicateChunk { chunk, msg_id }
        }
        // Send a message to a section telling them to initiate replication of this chunk
        MessageReceived::NodeCmd(NodeCmd::RepublishChunk(chunk)) => {
            NodeDuty::ProcessRepublish { chunk, msg_id }
        }
        //
        // ------ system cmd ------
        MessageReceived::NodeCmd(NodeCmd::StorageFull { node_id, .. }) => {
            NodeDuty::IncrementFullNodeCount { node_id }
        }
        // --- Adult Operation response ---
        MessageReceived::NodeQueryResponse {
            response: NodeQueryResponse::GetChunk(res),
            correlation_id,
        } => NodeDuty::RecordAdultReadLiveness {
            response: QueryResponse::GetChunk(res),
            correlation_id,
            src: origin.name(),
        },
        _ => send_error(
            msg_id,
            origin,
            Error::InvalidMessage(msg_id, format!("Invalid dst: {:?}", msg)),
            true,
        ),
    }
}

fn verify_authority(
    msg_id: MessageId,
    origin: SrcLocation,
    data_signed: ServiceOpSig,
    msg: ServiceMsg,
) -> Result<Authority<ServiceOpSig>, NodeDuty> {
    WireMsg::serialize_msg_payload(&msg)
        .and_then(|payload| Authority::verify(data_signed, &payload))
        .map_err(|error| send_error(msg_id, origin, Error::Message(error), false))
}

fn send_error(msg_id: MessageId, origin: SrcLocation, error: Error, aggregation: bool) -> NodeDuty {
    NodeDuty::Send(OutgoingMsg {
        id: MessageId::in_response_to(&msg_id),
        msg: MsgType::Node(NodeMsg::NodeMsgError {
            error: convert_to_error_message(error),
            correlation_id: msg_id,
        }),
        dst: origin.to_dst(),
        aggregation,
    })
}
