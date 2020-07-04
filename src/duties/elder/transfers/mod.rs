// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

pub mod replica_manager;

//use super::{messaging::Messaging, replica_manager::ReplicaManager};
use self::replica_manager::ReplicaManager;
use crate::{action::Action, rpc::Rpc};
//use log::{error, trace};
use safe_nd::{
    DebitAgreementProof, Error as NdError, MessageId, NodePublicId, NodeRequest, PublicId, Request,
    Response, SignedTransfer, SystemOp, Transfers as MoneyRequest, XorName,
};
use std::fmt::{self, Display, Formatter};

#[cfg(not(feature = "simulated-payouts"))]
use safe_nd::PublicKey;

use routing::SectionProofChain;
#[cfg(feature = "simulated-payouts")]
use safe_nd::Transfer;
use threshold_crypto::{PublicKeySet, SecretKeyShare};
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
pub(crate) struct Transfers {
    id: NodePublicId,
    replica: ReplicaManager,
}

impl Transfers {
    pub fn new(id: NodePublicId, replica: ReplicaManager) -> Self {
        Self { id, replica }
    }

    pub fn update_replica_on_churn(
        &mut self,
        pub_key_set: PublicKeySet,
        sec_key_share: SecretKeyShare,
        index: usize,
        proof_chain: SectionProofChain,
    ) -> Option<()> {
        self.replica
            .churn(sec_key_share, index, pub_key_set, proof_chain)
            .ok()
    }

    // /// Elders that aren't in the dst
    // /// section, will forward the request.
    // pub(super) fn initiate(
    //     &mut self,
    //     requester: PublicId,
    //     request: MoneyRequest,
    //     message_id: MessageId,
    // ) -> Option<Action> {
    //     Some(Action::ForwardClientRequest(Rpc::Request {
    //         request: Request::Node(NodeRequest::System(SystemOp::Transfers(request))),
    //         requester,
    //         message_id,
    //         signature: None,
    //     }))
    // }

    /// When handled by Elders in the dst
    /// section, the actual business logic is executed.
    pub(super) fn handle_request(
        &mut self,
        requester: PublicId,
        request: MoneyRequest,
        message_id: MessageId,
        //messaging: &mut Messaging,
    ) -> Option<Action> {
        let mut messaging = Messaging {};
        match request {
            MoneyRequest::ValidateTransfer { signed_transfer } => {
                self.validate(signed_transfer, &requester, message_id)
            }
            MoneyRequest::RegisterTransfer { proof } => {
                self.register(&proof, requester, message_id, &mut messaging)
            }
            MoneyRequest::PropagateTransfer { proof } => {
                self.receive_propagated(&proof, &requester, message_id, &mut messaging)
            }
            MoneyRequest::GetBalance(public_key) => self.balance(
                XorName::from(public_key),
                requester,
                message_id,
                &mut messaging,
            ),
            MoneyRequest::GetReplicaKeys(_public_key) => {
                // Here we assume we're at the right section.
                // TODO: verify this, or move transfers out of client handler in general
                self.get_replica_pks(message_id, &mut messaging)
            }
            MoneyRequest::GetHistory { at, since_version } => self.history(
                XorName::from(at),
                since_version,
                requester,
                message_id,
                &mut messaging,
            ),
            #[cfg(feature = "simulated-payouts")]
            MoneyRequest::SimulatePayout { transfer } => {
                self.replica
                    .credit_without_proof(requester, transfer, message_id, *self.id.name())
            }
        }
    }

    /// Get the PublicKeySet of our replicas
    fn get_replica_pks(&self, message_id: MessageId, messaging: &mut Messaging) -> Option<Action> {
        let result = self.replica.replicas_pk_set();
        let response = Response::GetReplicaKeys(result);
        messaging.respond_to_client(message_id, response);
        None
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
        _since_version: usize,
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
                refund: None,
                proof: None,
            },
        })
    }

    /// Registration of a transfer is requested,
    /// with a proof of enough Elders having validated it.
    fn register(
        &mut self,
        proof: &DebitAgreementProof,
        requester: PublicId,
        message_id: MessageId,
        messaging: &mut Messaging,
    ) -> Option<Action> {
        match self.replica.register(proof) {
            Ok(event) => {
                // sender is notified with a push msg (only delivered if recipient is online)
                messaging.respond_to_client(message_id, Response::TransferRegistration(Ok(event)));

                // the transfer is then propagated, and will reach the recipient section
                Some(Action::ForwardClientRequest(Rpc::Request {
                    request: Request::Node(NodeRequest::System(SystemOp::Transfers(
                        MoneyRequest::PropagateTransfer {
                            proof: proof.clone(),
                        },
                    ))),
                    requester,
                    message_id,
                    signature: None,
                }))
            }
            Err(err) => Some(Action::RespondToClientHandlers {
                sender: *self.id.name(),
                rpc: Rpc::Response {
                    response: Response::TransferRegistration(Err(err)),
                    requester: requester.clone(),
                    message_id,
                    refund: None,
                    proof: None,
                },
            }),
        }
    }

    /// The only step that is triggered by a Replica.
    /// (See fn register_transfer).
    /// After a successful registration of a transfer at
    /// the source, the transfer is propagated to the destionation.
    pub(crate) fn receive_propagated(
        &mut self,
        proof: &DebitAgreementProof,
        requester: &PublicId,
        message_id: MessageId,
        messaging: &mut Messaging,
    ) -> Option<Action> {
        // We will just validate the proofs and then apply the event.
        match self.replica.receive_propagated(proof) {
            Ok(_event) => {
                // notify recipient, with a push msg (only delivered if recipient is online)
                messaging.notify_client(&XorName::from((&proof).to()), proof);
                None
            }
            Err(err) => Some(Action::RespondToClientHandlers {
                sender: *self.id.name(),
                rpc: Rpc::Response {
                    response: Response::TransferPropagation(Err(err)),
                    requester: requester.clone(),
                    message_id,
                    refund: None,
                    proof: None,
                },
            }),
        }
    }

    #[allow(unused)]
    #[cfg(not(feature = "simulated-payouts"))]
    pub fn pay_section(
        &mut self,
        _transfer: DebitAgreementProof,
        _from: PublicKey,
        _request: &Request,
        _message_id: MessageId,
    ) -> Option<Action> {
        None
    }

    #[cfg(feature = "simulated-payouts")]
    pub fn pay(&mut self, transfer: Transfer) {
        self.replica.debit_without_proof(transfer)
    }
}

impl Display for Transfers {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.id)
    }
}

pub struct Messaging {}

impl Messaging {
    pub fn notify_client(&mut self, _xorname: &XorName, _proof: &DebitAgreementProof) {}
    pub fn respond_to_client(&mut self, _message_id: MessageId, _response: Response) {}
}
