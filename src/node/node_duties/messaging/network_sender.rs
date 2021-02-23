// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{
    node::node_ops::{GatewayDuty, Msg, NodeOperation},
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

    pub async fn send_to_client(&mut self, msg: Msg, _as_node: bool) -> Result<NodeOperation> {
        Ok(GatewayDuty::FindClientFor(msg).into())
        // let dst = match msg.destination()? {
        //     Address::Client(xorname) => xorname,
        //     Address::Node(_) => return Ok(NodeMessagingDuty::SendToNode(msg).into()),
        //     Address::Section(_) => {
        //         return Ok(NodeMessagingDuty::SendToSection { msg, as_node }.into())
        //     }
        // };
        // if self.network.matches_our_prefix(dst).await {
        //     Ok(GatewayDuty::FindClientFor(msg).into())
        // } else {
        //     Ok(NodeMessagingDuty::SendToSection { msg, as_node }.into())
        // }
    }

    pub async fn send_to_node(&mut self, msg: Msg, _as_node: bool) -> Result<NodeOperation> {
        let name = self.network.our_name().await;
        let dst = msg.dst; // DstLocation::Node(msg.dst.name());

        let result = self
            .network
            .send_message(SrcLocation::Node(name), dst, msg.msg.serialize()?)
            .await;

        result.map_or_else(
            |err| {
                error!("Unable to send Message to Peer: {:?}", err);
                Err(Error::Logic(format!(
                    "{:?}: Unable to send Msg to Peer",
                    msg.id()
                )))
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

    pub async fn send_to_network(
        &mut self,
        msg: Msg,
        // msg: Message,
        // location: XorName,
        as_node: bool,
    ) -> Result<NodeOperation> {
        let dst = msg.dst; //DstLocation::Section(location);
        let src = if as_node {
            SrcLocation::Node(self.network.our_name().await)
        } else {
            SrcLocation::Section(self.network.our_prefix().await)
        };
        let result = self
            .network
            .send_message(src, dst, msg.msg.serialize()?)
            .await;

        result.map_or_else(
            |err| {
                error!("Unable to send to section: {:?}", err);
                Err(Error::Logic(format!(
                    "{:?}: Unable to send to section",
                    msg.id()
                )))
            },
            |()| Ok(NodeOperation::NoOp),
        )
    }
}
