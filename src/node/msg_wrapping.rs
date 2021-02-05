// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::Result;
use crate::{node::node_ops::NodeMessagingDuty, AdultState, ElderState, NodeState};
use log::info;
use sn_messaging::client::{
    Address, AdultDuties, CmdError, Duty, ElderDuties, Message, MessageId, NodeDuties,
};
use sn_routing::DstLocation;
use std::collections::BTreeSet;
use xor_name::XorName;

use super::node_ops::Msg;

/// Wrapping of msgs sent by Elders.
#[derive(Clone)]
pub struct ElderMsgWrapping {
    inner: MsgWrapping,
}

/// Wrapping of msgs sent by Adults.
#[derive(Clone)]
pub struct AdultMsgWrapping {
    inner: MsgWrapping,
}

/// Wrapping of msgs sent by any Node.
#[derive(Clone)]
pub struct NodeMsgWrapping {
    inner: MsgWrapping,
}

/// Msg wrapping simplifies
/// the signing and stamping of
/// a sender duty onto remote msgs
/// to be sent on the wire.
#[derive(Clone)]
struct MsgWrapping {
    node_state: NodeState,
    duty: Duty,
}

impl NodeMsgWrapping {
    pub fn new(node_state: NodeState, duty: NodeDuties) -> Self {
        let inner = MsgWrapping::new(node_state, Duty::Node(duty));
        Self { inner }
    }

    pub async fn send_to_section(&self, msg: Msg, as_node: bool) -> Result<NodeMessagingDuty> {
        self.inner.send_to_section(msg, as_node).await
    }

    // pub async fn send_to_node(&self, message: Message) -> Result<NodeMessagingDuty> {
    //     self.inner.send_to_node(message).await
    // }
}

impl AdultMsgWrapping {
    pub fn new(adult_state: AdultState, duty: AdultDuties) -> Self {
        let inner = MsgWrapping::new(NodeState::Adult(adult_state), Duty::Adult(duty));
        Self { inner }
    }

    pub fn name(&self) -> XorName {
        self.inner.name()
    }

    pub async fn send_to_section(&self, msg: Msg, as_node: bool) -> Result<NodeMessagingDuty> {
        self.inner.send_to_section(msg, as_node).await
    }

    pub async fn send_to_node(&self, msg: Msg) -> Result<NodeMessagingDuty> {
        self.inner.send_to_node(msg).await
    }

    pub async fn send_to_adults(
        &self,
        msg: Message,
        targets: BTreeSet<XorName>,
        duty: AdultDuties,
    ) -> Result<NodeMessagingDuty> {
        // let (key, sig) = self.inner.ed_key_sig(&message.serialize()?).await?;
        // let origin = MsgSender::adult(key, duty, sig)?;
        self.inner.send_to_adults(targets, msg, false).await
    }

    pub async fn error(
        &self,
        error: CmdError,
        msg_id: MessageId,
        cmd_origin: XorName,
    ) -> Result<NodeMessagingDuty> {
        self.inner.error(error, msg_id, cmd_origin).await
    }
}

impl ElderMsgWrapping {
    pub fn new(elder_state: ElderState, duty: ElderDuties) -> Self {
        let inner = MsgWrapping::new(NodeState::Elder(elder_state), Duty::Elder(duty));
        Self { inner }
    }

    pub async fn forward(&self, msg: Msg) -> Result<NodeMessagingDuty> {
        //let msg = self.inner.set_proxy(&msg, true).await?;
        Ok(NodeMessagingDuty::SendToSection {
            msg: msg.clone(),
            as_node: false,
        })
    }

    pub async fn send_to_client(&self, msg: Message, client: XorName) -> Result<NodeMessagingDuty> {
        self.inner
            .send_to_client(Msg {
                msg,
                dst: DstLocation::Client(client),
            })
            .await
    }

    pub async fn send_to_section(
        &self,
        msg: Message,
        location: XorName,
        as_node: bool,
    ) -> Result<NodeMessagingDuty> {
        self.inner
            .send_to_section(
                Msg {
                    msg,
                    dst: DstLocation::Section(location),
                },
                as_node,
            )
            .await
    }

    pub async fn send_to_node(&self, msg: Message, node: XorName) -> Result<NodeMessagingDuty> {
        self.inner
            .send_to_node(Msg {
                msg,
                dst: DstLocation::Node(node),
            })
            .await
    }

    pub async fn send_to_adults(
        &self,
        targets: BTreeSet<XorName>,
        msg: Message,
        as_section: bool,
        //origin: MsgSender,
        //proxies: Vec<MsgSender>,
    ) -> Result<NodeMessagingDuty> {
        self.inner.send_to_adults(targets, msg, as_section).await
    }

    pub async fn error(
        &self,
        error: CmdError,
        msg_id: MessageId,
        cmd_origin: XorName,
    ) -> Result<NodeMessagingDuty> {
        self.inner.error(error, msg_id, cmd_origin).await
    }
}

impl MsgWrapping {
    pub fn new(node_state: NodeState, duty: Duty) -> Self {
        Self { node_state, duty }
    }

    pub fn name(&self) -> XorName {
        match &self.node_state {
            NodeState::Adult(state) => state.node_name(),
            NodeState::Elder(state) => state.node_name(),
        }
    }

    pub async fn send_to_client(&self, msg: Msg) -> Result<NodeMessagingDuty> {
        // let origin = self.sign(&message, true).await?;
        // let msg = Message {
        //     message,
        //     origin,
        //     proxies: Default::default(),
        // };
        Ok(NodeMessagingDuty::SendToClient(msg))
    }

    pub async fn send_to_node(&self, msg: Msg) -> Result<NodeMessagingDuty> {
        // let origin = self.sign(&message, false).await?;
        // let msg = Message {
        //     message,
        //     origin,
        //     proxies: Default::default(),
        // };
        Ok(NodeMessagingDuty::SendToNode(msg))
    }

    pub async fn send_to_section(&self, msg: Msg, as_node: bool) -> Result<NodeMessagingDuty> {
        // let origin = self.sign(&message, !as_node).await?;
        // let msg = Message {
        //     message,
        //     origin,
        //     proxies: Default::default(),
        // };
        Ok(NodeMessagingDuty::SendToSection { msg, as_node })
    }

    pub async fn send_to_adults(
        &self,
        targets: BTreeSet<XorName>,
        msg: Message,
        as_section: bool,
        //origin: MsgSender,
        //proxies: Vec<MsgSender>,
    ) -> Result<NodeMessagingDuty> {
        // let msg_envelope = Message {
        //     message,
        //     origin,
        //     proxies,
        // };
        // let msg = self.set_proxy(&msg_envelope, as_section).await?;
        Ok(NodeMessagingDuty::SendToAdults { targets, msg })
    }

    pub async fn error(
        &self,
        error: CmdError,
        msg_id: MessageId,
        cmd_origin: XorName,
    ) -> Result<NodeMessagingDuty> {
        info!("Error {:?}", error);
        self.send_to_section(
            Msg {
                msg: Message::CmdError {
                    id: MessageId::new(),
                    error,
                    correlation_id: msg_id,
                    cmd_origin: Address::Client(cmd_origin.into()),
                },
                dst: DstLocation::Section(cmd_origin.into()),
            },
            true,
        )
        .await
    }
}
