// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{
    error::convert_to_error_message,
    node_ops::{MsgType, NodeDuty, OutgoingMsg},
    Error,
};
use sn_messaging::{
    client::{ClientMsg, Cmd, ProcessMsg, ProcessingError, Query, TransferCmd, TransferQuery},
    Aggregation, EndUser, MessageId, SrcLocation,
};

pub fn map_client_msg(msg: ProcessMsg, origin: EndUser) -> NodeDuty {
    match msg {
        ProcessMsg::Query {
            query: Query::Data(query),
            id,
            ..
        } => NodeDuty::ProcessRead { query, id, origin },
        ProcessMsg::Cmd {
            cmd: Cmd::Data { .. },
            ..
        } => NodeDuty::ProcessDataPayment {
            msg: msg.clone(),
            origin,
        },
        ProcessMsg::Cmd {
            cmd: Cmd::Transfer(TransferCmd::ValidateTransfer(signed_transfer)),
            id,
            ..
        } => NodeDuty::ValidateClientTransfer {
            signed_transfer,
            origin: SrcLocation::EndUser(origin),
            msg_id: id,
        },
        // TODO: Map more transfer cmds
        ProcessMsg::Cmd {
            cmd: Cmd::Transfer(TransferCmd::SimulatePayout(transfer)),
            id,
            ..
        } => NodeDuty::SimulatePayout {
            transfer,
            origin: SrcLocation::EndUser(origin),
            msg_id: id,
        },
        ProcessMsg::Cmd {
            cmd: Cmd::Transfer(TransferCmd::RegisterTransfer(proof)),
            id,
            ..
        } => NodeDuty::RegisterTransfer { proof, msg_id: id },
        // TODO: Map more transfer queries
        ProcessMsg::Query {
            query: Query::Transfer(TransferQuery::GetHistory { at, since_version }),
            id,
            ..
        } => NodeDuty::GetTransfersHistory {
            at,
            since_version,
            origin: SrcLocation::EndUser(origin),
            msg_id: id,
        },
        ProcessMsg::Query {
            query: Query::Transfer(TransferQuery::GetBalance(at)),
            id,
            ..
        } => NodeDuty::GetBalance {
            at,
            origin: SrcLocation::EndUser(origin),
            msg_id: id,
        },
        ProcessMsg::Query {
            query: Query::Transfer(TransferQuery::GetStoreCost { bytes, .. }),
            id,
            ..
        } => NodeDuty::GetStoreCost {
            bytes,
            origin: SrcLocation::EndUser(origin),
            msg_id: id,
        },
        _ => {
            let msg_id = msg.id();
            let error_data = convert_to_error_message(Error::InvalidMessage(
                msg_id,
                format!("Unknown user msg: {:?}", msg),
            ));
            let src = SrcLocation::EndUser(origin);

            NodeDuty::Send(OutgoingMsg {
                msg: MsgType::Client(ClientMsg::ProcessingError(ProcessingError::new(
                    Some(error_data),
                    Some(msg),
                    MessageId::in_response_to(&msg_id),
                ))),
                section_source: false, // strictly this is not correct, but we don't expect responses to an error..
                dst: src.to_dst(),
                aggregation: Aggregation::None,
            })
        }
    }
}
