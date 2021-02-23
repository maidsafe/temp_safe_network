// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::node_ops::{MetadataDuty, NodeOperation, TransferCmd, TransferDuty};
use crate::ElderState;
use crate::{Error, Result};
use log::info;
use sn_messaging::{
    client::{Cmd, Message, Query},
    location::User,
    SrcLocation,
};

// NB: Just as with the msg_analysis.rs,
// this approach is not entirely good, so will need to be improved.

/// Evaluates msgs sent directly from a client,
/// i.e. not remote msgs from the network.
pub struct ClientMsgAnalysis {
    _elder_state: ElderState,
}

impl ClientMsgAnalysis {
    pub fn new(_elder_state: ElderState) -> Self {
        Self { _elder_state }
    }

    pub async fn evaluate(&self, msg: Message, origin: User) -> Result<NodeOperation> {
        info!("Evaluation of client msg envelope: {:?}", msg);
        let msg_id = msg.id();
        match msg {
            Message::Query {
                query: Query::Data { .. },
                ..
            } => Ok(MetadataDuty::ProcessRead { msg, origin }.into()), // TODO: Fix these for type safety
            Message::Cmd {
                cmd: Cmd::Data { .. },
                ..
            } => Ok(TransferDuty::ProcessCmd {
                cmd: TransferCmd::ProcessPayment(msg.clone()),
                msg_id: msg.id(),
                origin: SrcLocation::User(origin),
            }
            .into()),
            Message::Cmd {
                cmd: Cmd::Transfer(cmd),
                ..
            } => Ok(TransferDuty::ProcessCmd {
                cmd: cmd.into(),
                msg_id,
                origin: SrcLocation::User(origin),
            }
            .into()),
            Message::Query {
                query: Query::Transfer(query),
                ..
            } => Ok(TransferDuty::ProcessQuery {
                query: query.into(),
                msg_id,
                origin: SrcLocation::User(origin),
            }
            .into()),
            _ => Err(Error::Logic(format!(
                "Could not evaluate Client Msg w/id {:?}",
                msg.id()
            ))),
        }
    }
}
