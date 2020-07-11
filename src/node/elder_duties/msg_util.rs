// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{cmd::ConsensusAction, utils};
use log::trace;
use safe_nd::{
    Account, AccountRead, AccountWrite, BlobRead, BlobWrite, Cmd, DataCmd, DebitAgreementProof,
    Duty, Duty, ElderDuty, ElderDuty, Error as NdError, IData, IDataAddress, IDataKind, MData,
    MapRead, MapWrite, Message, MessageId, MsgEnvelope, MsgSender, NodeCmd, NodePublicId, PublicId,
    Read, SData, SDataAddress, SequenceRead, SequenceWrite, Write,
};
use std::fmt::{self, Display, Formatter};

#[derive(Clone)]
pub(crate) struct ElderMsgUtil {
    id: NodeFullId,
    duty: ElderDuty,
    routing: Rc<RefCell<Routing>>,
}

impl ElderMsgUtil {
    pub fn new(id: NodeFullId, duty: ElderDuty, routing: Rc<RefCell<Routing>>) Self {
        Self {
            id,
            duty,
            routing,
        }
    }

    pub fn ok_or_error(
        &self,
        result: Result<()>,
        msg_id: MessageId,
        origin: MsgSender,
    ) -> Option<NodeCmd> {
        let error = match result {
            Ok(()) => return None,
            Err(error) => error,
        };
        self.error(error, msg_id, origin)
    }

    pub fn error(&self,
        error: NdError,
        msg_id: MessageId,
        origin: MsgSender,
    ) -> Option<NodeCmd> {
        self.wrap(Message::CmdError {
            id: MessageId::new(),
            error: CmdError::Data(error),
            correlation_id: msg_id,
            cmd_origin: origin,
        })
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

    fn sign<T: Serialize>(&self, data: &T) -> MsgSender {
        let signature = &utils::sign(self.routing.borrow(), &utils::serialise(data));
        MsgSender::Node {
            id: self.public_key(),
            duty: Duty::Elder(self.duty),
            signature,
        }
    }

    fn set_proxy(&self, msg: &MsgEnvelope) -> MsgEnvelope {
    // origin signs the message, while proxies sign the envelope
        msg.with_proxy(self.sign(msg))
    }

    fn public_key(&self) -> PublicKey {
        PublicKey::Bls(self.id.public_id().bls_public_key())
    }
}
