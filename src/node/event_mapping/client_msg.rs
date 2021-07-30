// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{Mapping, MsgContext};
use crate::messaging::{
    data::{ServiceError, ServiceMsg},
    Authority, DstLocation, EndUser, MessageId, ServiceOpSig, SrcLocation,
};
use crate::node::{
    error::convert_to_error_message,
    node_ops::{MsgType, NodeDuty, OutgoingMsg},
    Error,
};

pub(super) fn map_client_msg(
    msg_id: MessageId,
    msg: ServiceMsg,
    auth: Authority<ServiceOpSig>,
    user: EndUser,
) -> Mapping {
    // Signature has already been validated by the routing layer
    let op = map_client_service_msg(msg_id, msg.clone(), user, auth);

    let ctx = Some(MsgContext::Client {
        msg,
        src: SrcLocation::EndUser(user),
    });

    Mapping { op, ctx }
}

fn map_client_service_msg(
    msg_id: MessageId,
    service_msg: ServiceMsg,
    origin: EndUser,
    auth: Authority<ServiceOpSig>,
) -> NodeDuty {
    match service_msg {
        ServiceMsg::Query(query) => NodeDuty::ProcessRead {
            query,
            msg_id,
            auth,
            origin,
        },
        ServiceMsg::Cmd(cmd) => NodeDuty::ProcessWrite {
            cmd,
            msg_id,
            auth,
            origin,
        },
        ServiceMsg::QueryResponse {
            response,
            correlation_id,
        } => {
            let outgoing_msg = OutgoingMsg {
                id: MessageId::in_response_to(&correlation_id),
                msg: MsgType::Client(ServiceMsg::QueryResponse {
                    response,
                    correlation_id,
                }),
                dst: DstLocation::EndUser(origin),
                aggregation: false,
            };
            NodeDuty::Send(outgoing_msg)
        }
        _ => {
            let error_data = convert_to_error_message(Error::InvalidMessage(
                msg_id,
                format!("Unknown user msg: {:?}", service_msg),
            ));
            let src = SrcLocation::EndUser(origin);
            let id = MessageId::in_response_to(&msg_id);

            NodeDuty::Send(OutgoingMsg {
                id,
                msg: MsgType::Client(ServiceMsg::ServiceError(ServiceError {
                    reason: Some(error_data),
                    source_message: Some(Box::new(service_msg)),
                })),
                dst: src.to_dst(),
                aggregation: false,
            })
        }
    }
}
