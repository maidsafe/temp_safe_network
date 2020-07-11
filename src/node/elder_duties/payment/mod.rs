// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::transfers::replica_manager::ReplicaManager;
use crate::{cmd::NodeCmd, utils};
use routing::Node as Routing;
use safe_nd::{
    Cmd, CmdError, Duty, ElderDuty, Error, Message, MessageId, MsgEnvelope, MsgSender,
    NodePublicId, PublicKey, Result, TransferError,
};
use serde::Serialize;
use std::{
    cell::RefCell,
    fmt::{self, Display, Formatter},
    rc::Rc,
};

pub(crate) struct DataPayment {
    id: NodePublicId,
    routing: Rc<RefCell<Routing>>,
    replica: Rc<RefCell<ReplicaManager>>,
}

/// An Elder in S(R) is responsible for
/// data payment, and will receive write
/// requests from S(G) (Gateway nodes).
/// These will simply be forwarded requests
/// from clients.
/// At DataPayment, a local request to Transfers module
/// will clear the payment, and thereafter the node forwards
/// the actual write request (without payment info) to data section (S(D), i.e. elders with Metadata duties).
impl DataPayment {
    pub fn new(
        id: NodePublicId,
        routing: Rc<RefCell<Routing>>,
        replica: Rc<RefCell<ReplicaManager>>,
    ) -> Self {
        Self {
            id,
            routing,
            replica,
        }
    }

    pub fn pay_for_data(&mut self, msg: MsgEnvelope) -> Option<NodeCmd> {
        let (cmd, payment) = match msg.message {
            Message::Cmd {
                cmd: Cmd::Data { cmd, payment },
            } => (cmd, payment),
            _ => return None,
        };
        use TransferError::*;
        // Make sure we are actually at the correct replicas,
        // before executing the debit.
        // (We could also add a method that executes both
        // debit + credit atomically, but this is much simpler).
        match self.section_account_id() {
            Ok(section) => {
                if payment.to() != section {
                    return self.error_response(
                        TransferRegistration(Error::NoSuchRecipient),
                        msg.id(),
                        msg.origin,
                    );
                }
            }
            _ => {
                return self.error_response(
                    TransferRegistration(Error::NoSuchRecipient),
                    msg.id(),
                    msg.origin,
                )
            }
        };
        if let Err(err) = self.replica_mut().register(&payment) {
            return self.error_response(TransferRegistration(err), msg.id(), msg.origin);
        }
        if let Err(err) = self.replica_mut().receive_propagated(&payment) {
            return self.error_response(
                TransferRegistration(err), // CAnnot use TransferPropagation, since it's is not a client error... To be solved.
                msg.id(),
                msg.origin,
            );
        }
        self.set_proxy(&msg);
        NodeCmd::SendToSection(msg)
    }

    fn error_response(
        &self,
        error: TransferError,
        correlation_id: MessageId,
        cmd_origin: MsgSender,
    ) -> Option<NodeCmd> {
        self.wrap(Message::CmdError {
            error: CmdError::Transfer(error),
            id: MessageId::new(),
            correlation_id,
            cmd_origin,
        })
    }

    fn section_account_id(&self) -> Result<PublicKey> {
        Ok(PublicKey::Bls(
            self.replica.borrow().replicas_pk_set()?.public_key(),
        ))
    }

    fn replica_mut(&mut self) -> &mut ReplicaManager {
        self.replica.borrow_mut()
    }

    fn set_proxy(&self, msg: &mut MsgEnvelope) {
        // origin signs the message, while proxies sign the envelope
        msg.add_proxy(self.sign(msg))
    }

    fn ok_or_error(
        &self,
        result: Result<()>,
        msg_id: MessageId,
        origin: MsgSender,
    ) -> Option<NodeCmd> {
        let error = match result {
            Ok(()) => return None,
            Err(error) => error,
        };
        let message = Message::CmdError {
            id: MessageId::new(),
            error: CmdError::Data(error),
            correlation_id: msg_id,
            cmd_origin: origin,
        };
        self.wrap(message)
    }

    fn wrap(&self, message: Message) -> Option<NodeCmd> {
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
            duty: Duty::Elder(ElderDuty::Payment),
            signature,
        }
    }

    fn public_key(&self) -> PublicKey {
        PublicKey::Bls(self.id.public_id().bls_public_key())
    }
}

impl Display for DataPayment {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.id)
    }
}
