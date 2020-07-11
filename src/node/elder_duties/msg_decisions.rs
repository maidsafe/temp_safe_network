// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//use crate::{cmd::ConsensusAction, utils};
use safe_nd::{AdultDuty, DataCmd, Duty, ElderDuty, Read, Write};

#[derive(Clone)]
pub(super) struct ElderMsgDecisions {
    util: MsgDecisions,
}

#[derive(Clone)]
pub(super) struct AdultMsgDecisions {
    util: MsgDecisions,
}

#[derive(Clone)]
struct MsgDecisions {
    keys: NodeKeys,
    duty: Duty,
}

impl AdultMsgDecisions {}

impl ElderMsgDecisions {}

impl MsgDecisions {
    pub fn new(keys: NodeKeys, duty: Duty) Self {
        Self {
            keys,
            duty,
        }
    }

    pub fn vote(&self, msg: MsgEnvelope) -> Option<NodeCmd> {
        let msg = self.set_proxy(msg);
        Some(NodeCmd::VoteFor(ConsensusAction::Forward(msg)))
    }

    pub fn forward(&self, msg: MsgEnvelope) -> Option<NodeCmd> {
        let msg = self.set_proxy(msg);
        Some(NodeCmd::SendToSection(msg))
    }

    pub fn send(&self, message: Message) -> Option<NodeCmd> {
        let msg = MsgEnvelope {
            message,
            origin: self.sign(message),
            proxies: Default::default(),
        };
        Some(NodeCmd::SendToSection(msg))
    }

    pub fn error(&self,
        error: CmdError,
        msg_id: MessageId,
        origin: MsgSender,
    ) -> Option<NodeCmd> {
        self.wrap(Message::CmdError {
            id: MessageId::new(),
            error,
            correlation_id: msg_id,
            cmd_origin: origin,
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
