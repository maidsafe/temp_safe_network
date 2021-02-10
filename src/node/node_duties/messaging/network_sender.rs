// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{
    node::node_ops::{NodeOperation, OutgoingMsg},
    Error, Network, Result,
};
use log::error;
use sn_messaging::{client::Message, DstLocation, SrcLocation};
use std::collections::BTreeSet;
use xor_name::XorName;

/// Sending of msgs to other nodes in the network.
pub(super) struct NetworkSender {
    network: Network,
}

impl NetworkSender {
    pub fn new(network: Network) -> Self {
        Self { network }
    }

    pub async fn send(&mut self, msg: OutgoingMsg) -> Result<NodeOperation> {
        let src = if msg.to_be_aggregated {
            SrcLocation::Section(self.network.our_prefix().await)
        } else {
            SrcLocation::Node(self.network.our_name().await)
        };
        let result = self
            .network
            .send_message(src, msg.dst, msg.msg.serialize()?)
            .await;

        result.map_or_else(
            |err| {
                error!("Unable to send msg: {:?}", err);
                Err(Error::Logic(format!("Unable to send msg: {:?}", msg.id())))
            },
            |()| Ok(NodeOperation::NoOp),
        )
    }

    pub async fn send_to_nodes(
        &mut self,
        targets: BTreeSet<XorName>,
        msg: &Message,
    ) -> Result<NodeOperation> {
        let name = self.network.our_name().await;
        let bytes = &msg.serialize()?;
        for target in targets {
            self.network
                .send_message(
                    SrcLocation::Node(name),
                    DstLocation::Node(XorName(target.0)),
                    bytes.clone(),
                )
                .await
                .map_or_else(
                    |err| {
                        error!("Unable to send Message to Peer: {:?}", err);
                    },
                    |()| {},
                );
        }
        Ok(NodeOperation::NoOp)
    }
}
