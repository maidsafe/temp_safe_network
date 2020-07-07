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
    utils,
};
use log::trace;
use routing::{Node as Routing, SrcLocation};
use safe_nd::{
    Error, GatewayRequest, MessageId, NodePublicId, NodeRequest, PublicId, Request, Response,
    Result,
};
use std::{
    cell::RefCell,
    fmt::{self, Display, Formatter},
    rc::Rc,
};
use threshold_crypto::SignatureShare;

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

    fn section_account_id(&self) -> Result<safe_nd::PublicKey> {
        Ok(safe_nd::PublicKey::Bls(
            self.replica.borrow().replicas_pk_set()?.public_key(),
        ))
    }

    pub fn handle_write(
        &mut self,
        src: SrcLocation,
        requester: PublicId,
        request: GatewayRequest,
        message_id: MessageId,
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
        match &request {
            Write {
                write,
                debit_agreement,
            } => {
                // Make sure we are actually at the correct replicas,
                // before executing the debit.
                // (We could also add a method that executes both
                // debit + credit atomically, but this is much simpler).
                match self.section_account_id() {
                    Ok(section) => {
                        if debit_agreement.to() != section {
                            return self.error_response(
                                Error::NoSuchRecipient,
                                requester,
                                message_id,
                            );
                        }
                    }
                    _ => return self.error_response(Error::NoSuchRecipient, requester, message_id),
                }
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
                let signature = self.sign_with_signature_share(&utils::serialise(&request));
                wrap(PaymentCmd::SendToSection(Message::Request {
                    request: Request::Node(NodeRequest::Write(write.clone())),
                    requester,
                    message_id,
                    signature,
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

    fn sign_with_signature_share(&self, data: &[u8]) -> Option<(usize, SignatureShare)> {
        let signature = self
            .routing
            .borrow()
            .secret_key_share()
            .map_or(None, |key| Some(key.sign(data)));
        signature.map(|sig| (self.routing.borrow().our_index().unwrap_or(0), sig))
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
