// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::{
    node::{InfrastructureMsg, NodeCmd},
    MessageId,
};
use crate::node::{node_ops::NodeDuty, Result};
use crate::routing::{Prefix, XorName};
use std::collections::BTreeSet;

use super::role::ElderRole;

/// Push our state to the given dst
pub(crate) async fn push_state(
    elder: &ElderRole,
    prefix: Prefix,
    msg_id: MessageId,
    peers: BTreeSet<XorName>,
) -> Result<NodeDuty> {
    // Create an aggregated map of all the metadata of the provided prefix
    let metadata = elder
        .meta_data
        .read()
        .await
        .get_data_exchange_packet(prefix)
        .await?;

    Ok(NodeDuty::SendToNodes {
        msg_id,
        msg: InfrastructureMsg::NodeCmd(NodeCmd::ReceiveExistingData { metadata }),
        targets: peers,
        aggregation: false,
    })
}
