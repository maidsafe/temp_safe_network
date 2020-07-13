// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{
    cmd::{GroupDecision, OutboundMsg},
    node::keys::NodeKeys,
    utils,
};
use safe_nd::{
    AdultDuty, CmdError, DataCmd, Duty, ElderDuty, Message, MessageId, MsgEnvelope, MsgSender,
    XorName,
};
use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Clone)]
pub struct ElderMsgDecisions {
    inner: MsgDecisions,
}

#[derive(Clone)]
pub struct AdultMsgDecisions {
    inner: MsgDecisions,
}

#[derive(Clone)]
struct MsgDecisions {
    keys: NodeKeys,
    duty: Duty,
}

impl AdultMsgDecisions {
    pub fn new(keys: NodeKeys, duty: AdultDuty) -> Self {
        let inner = MsgDecisions::new(keys, Duty::Adult(duty));
        Self { inner }
    }

    pub fn send(&self, message: Message) -> Option<OutboundMsg> {
        self.inner.send(message)
    }

    // pub fn send_to_adults(
    //     &self,
    //     targets: BTreeSet<XorName>,
    //     msg: &MsgEnvelope,
    // ) -> Option<OutboundMsg> {
    //     self.inner.send_to_adults(targets, msg)
    // }

    pub fn error(
        &self,
        error: CmdError,
        msg_id: MessageId,
        origin: &MsgSender,
    ) -> Option<OutboundMsg> {
        self.inner.error(error, msg_id, &origin)
    }
}

impl ElderMsgDecisions {
    pub fn new(keys: NodeKeys, duty: ElderDuty) -> Self {
        let inner = MsgDecisions::new(keys, Duty::Elder(duty));
        Self { inner }
    }

    pub fn vote(&self, msg: &MsgEnvelope) -> Option<OutboundMsg> {
        let msg = self.inner.set_proxy(msg);
        Some(OutboundMsg::VoteFor(GroupDecision::Forward(msg)))
    }

    pub fn forward(&self, msg: &MsgEnvelope) -> Option<OutboundMsg> {
        let msg = self.inner.set_proxy(&msg);
        Some(OutboundMsg::SendToSection(msg))
    }

    pub fn send(&self, message: Message) -> Option<OutboundMsg> {
        self.inner.send(message)
    }

    pub fn send_to_adults(
        &self,
        targets: BTreeSet<XorName>,
        msg: &MsgEnvelope,
    ) -> Option<OutboundMsg> {
        self.inner.send_to_adults(targets, msg)
    }

    pub fn error(
        &self,
        error: CmdError,
        msg_id: MessageId,
        origin: &MsgSender,
    ) -> Option<OutboundMsg> {
        self.inner.error(error, msg_id, &origin)
    }
}

impl MsgDecisions {
    pub fn new(keys: NodeKeys, duty: Duty) -> Self {
        Self { keys, duty }
    }

    pub fn send(&self, message: Message) -> Option<OutboundMsg> {
        let origin = self.sign(&message);
        let msg = MsgEnvelope {
            message,
            origin,
            proxies: Default::default(),
        };
        Some(OutboundMsg::SendToSection(msg))
    }

    pub fn send_to_adults(
        &self,
        targets: BTreeSet<XorName>,
        msg: &MsgEnvelope,
    ) -> Option<OutboundMsg> {
        let msg = self.set_proxy(&msg);
        Some(OutboundMsg::SendToAdults { targets, msg })
    }

    pub fn error(
        &self,
        error: CmdError,
        msg_id: MessageId,
        origin: &MsgSender,
    ) -> Option<OutboundMsg> {
        self.send(Message::CmdError {
            id: MessageId::new(),
            error,
            correlation_id: msg_id,
            cmd_origin: origin.address(),
        })
    }

    fn sign<T: Serialize>(&self, data: &T) -> MsgSender {
        let signature = self.keys.sign(&utils::serialise(data));
        MsgSender::Node {
            id: self.keys.public_key(),
            duty: self.duty,
            signature,
        }
    }

    fn set_proxy(&self, msg: &MsgEnvelope) -> MsgEnvelope {
        // origin signs the message, while proxies sign the envelope
        msg.with_proxy(self.sign(msg))
    }
}
