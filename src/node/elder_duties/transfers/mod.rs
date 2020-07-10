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
use crate::{
    cmd::{NodeCmd, TransferCmd as TCmd},
    msg::Message,
};
//use log::{error, trace};
use safe_nd::{
    DebitAgreementProof, Error as NdError, MessageId, NodePublicId, PublicId,
    PublicKey, SignedTransfer, Event,
    MsgEnvelope, Message, TransferCmd, TransferQuery, Cmd, CmdError,
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
    id: NodePublicId,
    replica: Rc<RefCell<ReplicaManager>>,
}

impl Transfers {
    pub fn new(id: NodePublicId, replica: Rc<RefCell<ReplicaManager>>) -> Self {
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
            .borrow_mut()
            .churn(sec_key_share, index, pub_key_set, proof_chain)
            .ok()
    }

    /// When handled by Elders in the dst
    /// section, the actual business logic is executed.
    pub(super) fn handle_request(
        &mut self,
        requester: PublicId,
        msg: MsgEnvelope,
    ) -> Option<NodeCmd> {
        match msg.message {
            Message::Cmd(Cmd::Transfer(cmd)) => self.handle_client_cmd(cmd),
            Message::NetworkCmd(cmd) => self.handle_network_cmd(cmd),
            _ => None
        }
    }

    fn handle_network_cmd(&mut self, cmd: NetworkCmd) {
        match cmd {
            NetworkCmd::InitiateRewardPayout {
                signed_transfer,
                ..
            } => ,
            NetworkCmd::FinaliseRewardPayout{
                debit_agreement,
                ..
            } => {
                match self.replica.borrow_mut().register(proof) {
                    Ok(Some(event)) => {
                        // the transfer is then propagated, and will reach the recipient section
                        self.send(Message::NetworkCmd { cmd: NetworkCmd::PropagateTransfer(proof), })
                    }
                    Ok(None) => None,
                    Err(err) => self.error(CmdError::TransferRegistration(err)),
                }
            },
        }
    }

    fn handle_client_cmd(&mut self, cmd: TransferCmd) -> Option<NodeCmd> {
        match cmd {
            TransferCmd::ValidateTransfer { signed_transfer } => {
                self.validate(signed_transfer, &requester, message_id)
            }
            TransferCmd::RegisterTransfer { proof } => {
                self.register(&proof, requester, message_id)
            }
            TransferCmd::PropagateTransfer { proof } => {
                self.receive_propagated(&proof, &requester, message_id)
            }
            #[cfg(feature = "simulated-payouts")]
            TransferCmd::SimulatePayout { transfer } => self
                .replica
                .borrow_mut()
                .credit_without_proof(requester, transfer, message_id, *self.id.name()),
        }
    }

    fn handle_client_query(&mut self, query: TransferQuery) -> Option<NodeCmd> {
        match query {
            TransferQuery::GetBalance(public_key) => self.balance(public_key, requester, message_id),
            TransferQuery::GetReplicaKeys(public_key) => {
                self.get_replica_pks(public_key, requester, message_id)
            }
            TransferQuery::GetHistory { at, since_version } => {
                self.history(at, since_version, requester, message_id)
            }
        }
    }

    /// Get the PublicKeySet of our replicas
    fn get_replica_pks(
        &self,
        _public_key: PublicKey,
        requester: PublicId,
        // signature: Signature,
        message_id: MessageId,
    ) -> Option<NodeCmd> {
        // validate signature

        let result = self.replica.borrow().replicas_pk_set();
        NodeCmd::RespondToGateway {
            sender: *self.id.name(),
            msg: Message::Response {
                response: Response::GetReplicaKeys(result),
                requester: requester.clone(),
                message_id,
                proof: None,
            },
        }
    }

    fn balance(
        &self,
        public_key: PublicKey,
        requester: PublicId,
        // signature: Signature,
        message_id: MessageId,
    ) -> Option<NodeCmd> {
        // validate signature

        let result = self
            .replica
            .borrow()
            .balance(&public_key)
            .ok_or(NdError::NoSuchBalance);

        NodeCmd::SendToSection {
            sender: *self.id.name(),
            msg: Message::Response {
                response: Response::GetBalance(result),
                requester: requester.clone(),
                message_id,
                proof: None,
            },
        }
    }

    fn history(
        &self,
        public_key: PublicKey,
        _since_version: usize,
        requester: PublicId,
        // signature: Signature,
        message_id: MessageId,
    ) -> Option<NodeCmd> {
        // validate signature

        let result = match self
            .replica
            .borrow()
            .history(&public_key) // since_version
        {
            None => Ok(vec![]),
            Some(history) => Ok(history.clone()),
        };
        NodeCmd::SendToSection {
            sender: *self.id.name(),
            msg: Message::Response {
                response: Response::GetHistory(result),
                requester: requester.clone(),
                message_id,
                proof: None,
            },
        }
    }

    /// This validation will render a signature over the
    /// original request (ValidateTransfer), giving a partial
    /// proof by this individual Elder, that the transfer is valid.
    fn validate(
        &mut self,
        transfer: SignedTransfer,
        requester: &PublicId,
        message_id: MessageId,
    ) -> Option<NodeCmd> {
        match self.replica.borrow_mut().validate(transfer) {
            Ok(Some(event)) => {
                self.send(Message::Event { 
                    event: Event::TransferValidated(event), 
                    id: MessageId::new(), 
                    correlaction_id: message_id,
                })
            },
            Ok(None) => None,
            Err(err) => self.error(CmdError::TransferValidation(err)),
        }
    }

    /// Registration of a transfer is requested,
    /// with a proof of enough Elders having validated it.
    fn register(
        &mut self,
        proof: &DebitAgreementProof,
        requester: PublicId,
        message_id: MessageId,
    ) -> Option<NodeCmd> {
        match self.replica.borrow_mut().register(proof) {
            Ok(Some(event)) => {
                self.send(Message::Cmd { 
                    cmd: Cmd::Transfer(TransferCmd::PropagateTransfer { proof: event.proof }), // this should be a network cmd
                    id: MessageId::new(), 
                    correlaction_id: message_id,
                })
            },
            Ok(None) => None,
            Err(err) => self.error(CmdError::TransferRegistration(err)),
        }
    }

    /// The only step that is triggered by a Replica.
    /// (See fn register_transfer).
    /// After a successful registration of a transfer at
    /// the source, the transfer is propagated to the destination.
    pub(crate) fn receive_propagated(
        &mut self,
        proof: &DebitAgreementProof,
        requester: &PublicId,
        message_id: MessageId,
    ) -> Option<NodeCmd> {
        // We will just validate the proofs and then apply the event.
        match self.replica.borrow_mut().receive_propagated(proof) {
            Ok(Some(event)) => None, 
            // self.send(Message {
            //     event: Event::TransferReceived
            // }),
            Err(err) => self.error(CmdError::Transfer(TransferError::TransferPropagation(err))),
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
    ) -> Option<NodeCmd> {
        None
    }

    #[cfg(feature = "simulated-payouts")]
    pub fn pay(&mut self, transfer: Transfer) {
        self.replica.borrow_mut().debit_without_proof(transfer)
    }

    fn send(&self, message: Message) -> Option<NodeCmd> {
        Some(NodeCmd::SendToSection(MsgEnvelope {
            message,
            origin: self.origin_for(&message),
            proxies: Default::default(),
        }))
    }
    
    fn origin_for(&self, message: &mut Message) -> MsgSender {
        // origin signs the message, while proxies sign the envelope
        let signature = &utils::sign(self.routing.borrow(), &utils::serialise(&message));
        MsgSender {
            id: self.id.into(),
            duty: Duty::Elder(ElderDuty::Transfers),
            signature,
        }
    }
    
    fn set_proxy(&self, msg: &mut MsgEnvelope) {
        // origin signs the message, while proxies sign the envelope
        let signature = &utils::sign(self.routing.borrow(), &utils::serialise(&msg));
        msg.add_proxy(MsgSender {
            id: self.id.into(),
            duty: Duty::Elder(ElderDuty::Transfers),
            signature,
        })
    }
}

fn wrap(cmd: TCmd) -> Option<NodeCmd> {
    Some(NodeCmd::Transfer(cmd))
}

impl Display for Transfers {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.id)
    }
}
