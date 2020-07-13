// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::transfers::replica_manager::ReplicaManager;
use crate::{cmd::OutboundMsg, node::keys::NodeKeys, node::msg_decisions::ElderMsgDecisions};
use safe_nd::{
    Cmd, CmdError, ElderDuty, Error, Message, MsgEnvelope, PublicKey, Result, TransferError,
};
use std::{
    cell::{RefCell, RefMut},
    fmt::{self, Display, Formatter},
    rc::Rc,
};

pub(crate) struct DataPayment {
    keys: NodeKeys,
    replica: Rc<RefCell<ReplicaManager>>,
    decisions: ElderMsgDecisions,
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
    pub fn new(keys: NodeKeys, replica: Rc<RefCell<ReplicaManager>>) -> Self {
        let decisions = ElderMsgDecisions::new(keys.clone(), ElderDuty::Payment);
        Self {
            keys,
            replica,
            decisions,
        }
    }

    // The code in this method is very messy, needs to be cleand up.
    pub fn pay_for_data(&mut self, msg: &MsgEnvelope) -> Option<OutboundMsg> {
        use TransferError::*;
        let payment = match &msg.message {
            Message::Cmd {
                cmd: Cmd::Data { payment, .. },
                ..
            } => payment,
            _ => return None,
        };
        // Make sure we are actually at the correct replicas,
        // before executing the debit.
        // (We could also add a method that executes both
        // debit + credit atomically, but this is much simpler).
        let recipient_is_not_section = match self.section_account_id() {
            Ok(section) => payment.to() != section,
            _ => true, // this would be strange, is it even possible?
        };

        if recipient_is_not_section {
            let error = CmdError::Transfer(TransferRegistration(Error::NoSuchRecipient));
            return self.decisions.error(error, msg.id(), &msg.origin);
        }
        let registration = self.replica_mut().register(&payment);
        let result = match registration {
            Ok(_) => match self.replica_mut().receive_propagated(&payment) {
                Ok(_) => Ok(()),
                Err(error) => Err(error),
            },
            Err(error) => Err(error), // not using TransferPropagation error, since that is for NetworkCmds, so wouldn't be returned to client.
        };
        match result {
            Ok(_) => self.decisions.forward(msg),
            Err(error) => self.decisions.error(
                CmdError::Transfer(TransferRegistration(error)),
                msg.id(),
                &msg.origin,
            ),
        }
    }

    fn section_account_id(&self) -> Result<PublicKey> {
        match self.replica.borrow().replicas_pk_set() {
            Some(keys) => Ok(PublicKey::Bls(keys.public_key())),
            None => Err(Error::NoSuchKey),
        }
    }

    fn replica_mut(&mut self) -> RefMut<ReplicaManager> {
        self.replica.borrow_mut()
    }
}

impl Display for DataPayment {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.keys.public_key())
    }
}
