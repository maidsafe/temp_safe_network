// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod replica_manager;

pub use self::replica_manager::ReplicaManager;
use crate::{cmd::OutboundMsg, keys::NodeKeys, node::elder_duties::msg_decisions::ElderMsgDecisions};
use safe_nd::{
    Cmd, CmdError, DebitAgreementProof, Error, Event, Message, MessageId, MsgEnvelope, MsgSender,
    NetworkCmd, NetworkCmdError, PublicKey, Query, QueryResponse, SignedTransfer,
    TransferCmd, TransferQuery, ElderDuty, TransferError,
};
use std::{
    cell::RefCell,
    fmt::{self, Display, Formatter},
    rc::Rc,
};

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
    keys: NodeKeys,
    replica: Rc<RefCell<ReplicaManager>>,
    decisions: ElderMsgDecisions,
}

impl Transfers {
    pub fn new(keys: NodeKeys, replica: Rc<RefCell<ReplicaManager>>) -> Self {
        let decisions = ElderMsgDecisions::new(keys.clone(), ElderDuty::Transfer);
        Self { keys, replica, decisions }
    }

    pub fn update_replica_on_churn(
        &mut self,
        pub_key_set: PublicKeySet,
        sec_key_share: SecretKeyShare,
        index: usize,
        proof_chain: SectionProofChain,
    ) -> Option<()> {
        self.replica
            .borrow_mut()
            .churn(sec_key_share, index, pub_key_set, proof_chain)
            .ok()
    }

    /// When handled by Elders in the dst
    /// section, the actual business logic is executed.
    pub(super) fn handle_request(&mut self, msg: MsgEnvelope) -> Option<OutboundMsg> {
        match msg.message {
            Message::Cmd {
                cmd: Cmd::Transfer(cmd),
                ..
            } => self.handle_client_cmd(cmd, msg),
            Message::NetworkCmd { cmd, .. } => self.handle_network_cmd(cmd, msg.id(), msg.origin),
            Message::Query {
                query: Query::Transfer(query), ..
            } => self.handle_client_query(query, msg),
            _ => None,
        }
    }

    fn handle_network_cmd(&mut self, cmd: NetworkCmd, msg_id: MessageId, origin: MsgSender) -> Option<OutboundMsg> {
        match cmd {
            NetworkCmd::PropagateTransfer(proof) => self.receive_propagated(&proof, msg_id, origin),
            NetworkCmd::InitiateRewardPayout {
                signed_transfer,
                ..
            } => ,
            NetworkCmd::FinaliseRewardPayout{
                debit_agreement,
                ..
            } => match self.replica.borrow_mut().register(proof) {
                    Ok(None) => None,
                    Ok(Some(event)) => {
                        // the transfer is then propagated, and will reach the recipient section
                        self.decisions.send(Message::NetworkCmd { 
                            cmd: NetworkCmd::PropagateTransfer(proof),
                            id: MessageId::new(),
                        })
                    }
                    Err(err) => self.decisions.error(CmdError::Transfer(error), msg_id, origin),
                }
        }
    }

    fn handle_client_cmd(&mut self, cmd: TransferCmd, msg: MsgEnvelope) -> Option<OutboundMsg> {
        match cmd {
            TransferCmd::ValidateTransfer(signed_transfer) => {
                self.validate(signed_transfer, msg.id(), msg.origin)
            }
            TransferCmd::RegisterTransfer(proof) => self.register(&proof, msg.id(), msg.origin),
            #[cfg(feature = "simulated-payouts")]
            TransferCmd::SimulatePayout(transfer) => self
                .replica
                .borrow_mut()
                .credit_without_proof(transfer, msg.id(), *self.id.name()),
        }
    }

    fn handle_client_query(&mut self, query: TransferQuery, msg: MsgEnvelope) -> Option<OutboundMsg> {
        match query {
            TransferQuery::GetBalance(public_key) => self.balance(public_key, msg),
            TransferQuery::GetReplicaKeys(public_key) => self.get_replica_pks(public_key, msg),
            TransferQuery::GetHistory { at, since_version } => self.history(at, since_version, msg),
        }
    }

    /// Get the PublicKeySet of our replicas
    fn get_replica_pks(&self, _public_key: PublicKey, msg: MsgEnvelope) -> Option<OutboundMsg> {
        // validate signature
        let result = match self.replica.borrow().replicas_pk_set() {
            None => Err(Error::NoSuchKey),
            Some(keys) => Ok(keys),
        };
        self.decisions.send(Message::QueryResponse {
            response: QueryResponse::GetReplicaKeys(result),
            id: MessageId::new(),
            correlation_id: msg.id(),
            query_origin: msg.origin.address(),
        })
    }

    fn balance(&self, public_key: PublicKey, msg: MsgEnvelope) -> Option<OutboundMsg> {
        // validate signature
        let result = self
            .replica
            .borrow()
            .balance(&public_key)
            .ok_or(Error::NoSuchBalance);
        self.decisions.send(Message::QueryResponse {
            response: QueryResponse::GetBalance(result),
            id: MessageId::new(),
            correlation_id: msg.id(),
            query_origin: msg.origin.address(),
        })
    }

    fn history(
        &self,
        public_key: PublicKey,
        _since_version: usize,
        msg: MsgEnvelope,
    ) -> Option<OutboundMsg> {
        // validate signature
        let result = match self
            .replica
            .borrow()
            .history(&public_key) // since_version
        {
            None => Ok(vec![]),
            Some(history) => Ok(history.clone()),
        };
        self.decisions.send(Message::QueryResponse {
            response: QueryResponse::GetHistory(result),
            id: MessageId::new(),
            correlation_id: msg.id(),
            query_origin: msg.origin.address(),
        })
    }

    /// This validation will render a signature over the
    /// original request (ValidateTransfer), giving a partial
    /// proof by this individual Elder, that the transfer is valid.
    fn validate(
        &mut self,
        transfer: SignedTransfer,
        msg_id: MessageId,
        origin: MsgSender,
    ) -> Option<OutboundMsg> {
        let message = match self.replica.borrow_mut().validate(transfer) {
            Ok(None) => return None,
            Ok(Some(event)) => Message::Event {
                event: Event::TransferValidated {
                    client: (*origin.id()).into(),
                    event,
                },
                id: MessageId::new(),
                correlation_id: msg_id,
            },
            Err(error) => Message::CmdError {
                id: MessageId::new(),
                error: CmdError::Transfer(TransferError::TransferValidation(error)),
                correlation_id: msg_id,
                cmd_origin: origin.address(),
            },
        };
        self.decisions.send(message)
    }

    /// Registration of a transfer is requested,
    /// with a proof of enough Elders having validated it.
    fn register(
        &mut self,
        proof: &DebitAgreementProof,
        msg_id: MessageId,
        origin: MsgSender,
    ) -> Option<OutboundMsg> {
        let message = match self.replica.borrow_mut().register(proof) {
            Ok(None) => return None,
            Ok(Some(event)) => Message::NetworkCmd {
                cmd: NetworkCmd::PropagateTransfer(event.debit_proof),
                id: MessageId::new(),
            },
            Err(error) => Message::CmdError {
                id: MessageId::new(),
                error: CmdError::Transfer(TransferError::TransferRegistration(error)),
                correlation_id: msg_id,
                cmd_origin: origin.address(),
            },
        };
        self.decisions.send(message)
    }

    /// The only step that is triggered by a Replica.
    /// (See fn register_transfer).
    /// After a successful registration of a transfer at
    /// the source, the transfer is propagated to the destination.
    pub(crate) fn receive_propagated(
        &mut self,
        proof: &DebitAgreementProof,
        msg_id: MessageId,
        origin: MsgSender,
    ) -> Option<OutboundMsg> {
        // We will just validate the proofs and then apply the event.
        let message = match self.replica.borrow_mut().receive_propagated(proof) {
            Ok(Some(event)) => return None,
            // self.send(Message {
            //     event: Event::TransferReceived
            // }),
            Err(err) => Message::NetworkCmdError {
                error: NetworkCmdError::TransferPropagation(err),
                id: MessageId::new(),
                correlation_id: msg_id,
                cmd_origin: origin.address(),
            },
        };
        self.decisions.send(message)
    }

    #[cfg(feature = "simulated-payouts")]
    pub fn pay(&mut self, transfer: Transfer) {
        self.replica.borrow_mut().debit_without_proof(transfer)
    }
}

impl Display for Transfers {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.keys.public_key())
    }
}
