// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::transfers::replica_manager::ReplicaManager;
use crate::{
    cmd::{ElderCmd, PaymentCmd},
    msg::Message,
};
use log::trace;
use routing::SrcLocation;
use safe_nd::{
    Error, GatewayRequest, MessageId, NodePublicId, NodeRequest, PublicId, Request, Response,
};
use std::{
    cell::RefCell,
    fmt::{self, Display, Formatter},
    rc::Rc,
};
use threshold_crypto::Signature;

pub(crate) struct DataPayment {
    id: NodePublicId,
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
    pub fn new(id: NodePublicId, replica: Rc<RefCell<ReplicaManager>>) -> Self {
        Self { id, replica }
    }

    pub fn handle_write(
        &mut self,
        src: SrcLocation,
        requester: PublicId,
        request: GatewayRequest,
        message_id: MessageId,
        _accumulated_signature: Option<Signature>,
    ) -> Option<ElderCmd> {
        trace!(
            "{}: Received ({:?} {:?}) from src {:?} (client {:?})",
            self,
            request,
            message_id,
            src,
            requester
        );
        use GatewayRequest::*;
        match request {
            Write {
                write,
                debit_agreement,
            } => {
                if let Err(err) = self.replica.borrow_mut().register(&debit_agreement) {
                    return self.error_response(err, requester, message_id);
                }
                if let Err(err) = self
                    .replica
                    .borrow_mut()
                    .receive_propagated(&debit_agreement)
                {
                    return self.error_response(err, requester, message_id);
                }
                wrap(PaymentCmd::SendToSection(Message::Request {
                    request: Request::Node(NodeRequest::Write(write)),
                    requester,
                    message_id,
                    signature: None,
                }))
            }
            _ => None,
        }
    }

    fn error_response(
        &self,
        err: Error,
        requester: PublicId,
        message_id: MessageId,
    ) -> Option<ElderCmd> {
        wrap(PaymentCmd::RespondToGateway {
            sender: *self.id.name(),
            msg: Message::Response {
                response: Response::Write(Err(err)),
                requester,
                message_id,
                proof: None,
            },
        })
    }
}

fn wrap(cmd: PaymentCmd) -> Option<ElderCmd> {
    Some(ElderCmd::Payment(cmd))
}

impl Display for DataPayment {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.id)
    }
}
