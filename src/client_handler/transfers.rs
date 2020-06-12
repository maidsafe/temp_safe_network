// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{auth::ClientInfo, messaging::Messaging, replica_manager::ReplicaManager};
use crate::{action::Action, rpc::Rpc};
//use log::{error, trace};
use safe_nd::{
    DebitAgreementProof, Error as NdError, MessageId, Money, MoneyRequest, NodePublicId, PublicId,
    PublicKey, Request, Response, SignedTransfer, XorName,
};
use std::fmt::{self, Display, Formatter};

/*
Transfers is the layer that manages
interaction with an AT2 Replica.

Flow overview:
1. Client-to-Elders: Request::ValidateTransfer
2. Elders-to-Client: Response::TransferValidation
3. Client-to-Elders: Request::RegisterTransfer
4. Elders-to-Client: Response::TransferRegistration
5. Elders-to-Elders: Request::PropagateTransfer

The Replica is the part of an AT2 system
that forms validating groups, and signs individual
Actors' transfers.
They validate incoming requests for transfer, and
apply operations that has a valid proof of agreement from the group.
Replicas don't initiate transfers or drive the algo - only Actors do.
*/

/// Transfers is the layer that manages
/// interaction with an AT2 Replica.
pub(super) struct Transfers {
    id: NodePublicId,
    replica: ReplicaManager,
}

impl Transfers {
    pub fn new(id: NodePublicId, replica: ReplicaManager) -> Self {
        Self { id, replica }
    }

    /// Elders that aren't in the dst
    /// section, will forward the request.
    pub(super) fn process_client_request(
        &mut self,
        client: &ClientInfo,
        request: MoneyRequest,
        message_id: MessageId,
    ) -> Option<Action> {
        Some(Action::ForwardClientRequest(Rpc::Request {
            request: Request::Money(request),
            requester: client.public_id.clone(),
            message_id,
        }))
    }

    /// When handled by Elders in the dst
    /// section, the actual business logic is executed.
    pub(super) fn finalise_client_request(
        &mut self,
        requester: PublicId,
        request: MoneyRequest,
        message_id: MessageId,
        messaging: &mut Messaging,
    ) -> Option<Action> {
        match request {
            MoneyRequest::ValidateTransfer { signed_transfer } => {
                self.validate(signed_transfer, &requester, message_id)
            }
            MoneyRequest::RegisterTransfer { proof } => {
                self.register(&proof, &requester, message_id, messaging)
            }
            MoneyRequest::PropagateTransfer { proof } => {
                self.receive_propagated(&proof, &requester, message_id, messaging)
            }
            MoneyRequest::GetBalance(xorname) => {
                self.balance(XorName::from(xorname), requester, message_id, messaging)
            }
            MoneyRequest::GetHistory { at, since_version } => self.history(
                XorName::from(at),
                since_version,
                requester,
                message_id,
                messaging,
            ),
            #[cfg(features = "testing")]
            MoneyRequest::SimulatePayout { transfer } => self
                .replica
                .register_without_proof(transfer, requester, message_id),
        }
    }

    fn balance(
        &mut self,
        xorname: XorName,
        requester: PublicId,
        message_id: MessageId,
        messaging: &mut Messaging,
    ) -> Option<Action> {
        let authorized = xorname == requester.public_key().into();
        let result = if !authorized {
            Err(NdError::NoSuchBalance)
        } else {
            self.replica
                .balance(&requester.public_key())
                .ok_or(NdError::NoSuchBalance)
        };
        let response = Response::GetBalance(result);
        messaging.respond_to_client(message_id, response);
        None
    }

    fn history(
        &mut self,
        at: XorName,
        since_version: usize,
        requester: PublicId,
        message_id: MessageId,
        messaging: &mut Messaging,
    ) -> Option<Action> {
        let authorized = at == requester.public_key().into();
        let result = if !authorized {
            Err(NdError::NoSuchBalance)
        } else {
            match self
                    .replica
                    .history(&requester.public_key()) // since_version
                {
                    None => Ok(vec![]),
                    Some(history) => Ok(history.clone()),
                }
        };
        let response = Response::GetHistory(result);
        messaging.respond_to_client(message_id, response);
        None
    }

    /// This validation will render a signature over the
    /// original request (ValidateTransfer), giving a partial
    /// proof by this individual Elder, that the transfer is valid.
    fn validate(
        &mut self,
        transfer: SignedTransfer,
        requester: &PublicId,
        message_id: MessageId,
    ) -> Option<Action> {
        let result = self.replica.validate(transfer);
        Some(Action::RespondToClientHandlers {
            sender: *self.id.name(),
            rpc: Rpc::Response {
                response: Response::TransferValidation(result),
                requester: requester.clone(),
                message_id,
            },
        })
    }

    /// Registration of a transfer is requested,
    /// with a proof of enough Elders having validated it.
    fn register(
        &mut self,
        proof: &DebitAgreementProof,
        requester: &PublicId,
        message_id: MessageId,
        messaging: &mut Messaging,
    ) -> Option<Action> {
        match self.replica.register(proof) {
            Ok(event) => {
                let transfer = &proof.signed_transfer.transfer;
                // sender is notified with a push msg (only delivered if recipient is online)
                messaging.notify_client(&XorName::from(transfer.id.actor), proof);

                // the transfer is then propagated, and will reach the recipient section
                Some(Action::ForwardClientRequest(Rpc::Request {
                    request: Request::Money(MoneyRequest::PropagateTransfer {
                        proof: proof.clone(),
                    }),
                    requester: requester.clone(),
                    message_id,
                }))
            }
            Err(err) => Some(Action::RespondToClientHandlers {
                sender: *self.id.name(),
                rpc: Rpc::Response {
                    response: Response::TransferRegistration(Err(err)),
                    requester: requester.clone(),
                    message_id,
                },
            }),
        }
    }

    /// The only step that is triggered by a Replica.
    /// (See fn register_transfer).
    /// After a successful registration of a transfer at
    /// the source, the transfer is propagated to the destionation.
    fn receive_propagated(
        &mut self,
        proof: &DebitAgreementProof,
        requester: &PublicId,
        message_id: MessageId,
        messaging: &mut Messaging,
    ) -> Option<Action> {
        // We will just validate the proofs and then apply the event.
        match self.replica.receive_propagated(proof) {
            Ok(event) => {
                let transfer = &proof.signed_transfer.transfer;
                // notify recipient, with a push msg (only delivered if recipient is online)
                messaging.notify_client(&XorName::from(transfer.to), proof);
                None
            }
            Err(err) => Some(Action::RespondToClientHandlers {
                sender: *self.id.name(),
                rpc: Rpc::Response {
                    response: Response::TransferPropagation(Err(err)),
                    requester: requester.clone(),
                    message_id,
                },
            }),
        }
    }

    pub fn pay_section(
        &mut self,
        amount: Money,
        from: PublicKey,
        request: &Request,
        message_id: MessageId,
    ) -> Option<Action> {
        None
    }
}

impl Display for Transfers {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.id)
    }
}
