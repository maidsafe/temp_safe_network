use safe_nd::{
    ClientRequest, DebitAgreementProof, Message, MessageId, PublicId, PublicKey, Request, Response,
    SystemOp, Transfers as MoneyRequest, Write,
};

use safe_transfers::{ActorEvent, ReplicaValidator, TransferActor as SafeTransferActor};

use crate::client::ConnectionManager;
use crate::client::{Client, SafeKey, COST_OF_PUT};
use crate::errors::CoreError;
use crdts::Dot;
use futures::lock::Mutex;

use log::{debug, info, trace, warn};

#[cfg(feature = "simulated-payouts")]
use std::sync::Arc;
use threshold_crypto::PublicKeySet;

pub mod balance_management;
pub mod setup;
pub mod simulated_payouts;
pub mod write_apis;

#[cfg(test)]
pub mod test_utils;

/// Handle Money Transfers, requests and locally stores a balance
#[derive(Clone, Debug)]
pub struct TransferActor {
    transfer_actor: Arc<Mutex<SafeTransferActor<ClientTransferValidator>>>,
    safe_key: SafeKey,
    replicas_pk_set: PublicKeySet,
    simulated_farming_payout_dot: Dot<PublicKey>,
    connection_manager: ConnectionManager,
}

/// Simple client side validations
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientTransferValidator {}

impl ReplicaValidator for ClientTransferValidator {
    fn is_valid(&self, _replica_group: PublicKey) -> bool {
        true
    }
}

impl TransferActor {
    fn wrap_money_request(req: MoneyRequest) -> ClientRequest {
        ClientRequest::System(SystemOp::Transfers(req))
    }

    /// Get a payment proof
    pub async fn get_payment_proof(&mut self) -> Result<DebitAgreementProof, CoreError> {
        let mut cm = self.connection_manager();

        // --------------------------
        // Payment for PUT
        // --------------------------
        self.create_write_payment_proof().await
    }

    pub fn connection_manager(&self) -> ConnectionManager {
        self.connection_manager.clone()
    }

    /// Retrieve the history of the acocunt from the network and apply to our local actor
    pub async fn get_history(&mut self) -> Result<(), CoreError> {
        let mut cm = self.connection_manager();
        let public_key = self.safe_key.public_key();
        info!("Getting SafeTransfers history for pk: {:?}", public_key);

        let request = Self::wrap_money_request(MoneyRequest::GetHistory {
            at: public_key,
            since_version: 0,
        });

        let (message, _messafe_id) =
            TransferActor::create_network_message(self.safe_key.clone(), request)?;

        let _bootstrapped = cm.bootstrap(self.safe_key.clone()).await;

        // This is a normal response manager request. We want quorum on this for now...
        let res = cm.send(&self.safe_key.public_id(), &message).await?;

        let history = match res {
            Response::GetHistory(history) => history.map_err(CoreError::from),
            _ => Err(CoreError::from(format!(
                "Unexpected response when retrieving account history {:?}",
                res
            ))),
        }?;

        let mut actor = self.transfer_actor.lock().await;
        match actor.synch(history) {
            Ok(synced_transfers) => {
                actor.apply(ActorEvent::TransfersSynched(synced_transfers));
            }
            Err(error) => {
                if !error
                    .clone()
                    .to_string()
                    .contains("No credits or debits found to sync to actor")
                {
                    return Err(CoreError::from(error));
                }

                warn!("No transfer history retrieved for pk: {:?}", public_key);
            }
        }

        Ok(())
    }

    // build, sign and send a validation type message, await appropriate response
    // TODO: remove old client sign req
    pub(crate) fn create_network_message(
        safe_key: SafeKey,
        request: ClientRequest,
    ) -> Result<(Message, MessageId), CoreError> {
        trace!("Creating signed network message");

        let message_id = MessageId::new();

        let signature = Some(safe_key.sign(&unwrap::unwrap!(bincode::serialize(&(
            &request, message_id
        )))));

        let request = Request::Client(request);

        let message = Message::Request {
            request,
            message_id: message_id.clone(),
            signature,
        };

        Ok((message, message_id))
    }

    /// Validates a tranction for paying store_cost
    async fn create_write_payment_proof(&mut self) -> Result<DebitAgreementProof, CoreError> {
        info!("Sending requests for payment for write operation");

        let mut cm = self.connection_manager();

        //set up message
        let safe_key = self.safe_key.clone();

        self.get_history().await?;

        let section_key = PublicKey::Bls(self.replicas_pk_set.public_key());

        let signed_transfer = self
            .transfer_actor
            .lock()
            .await
            .transfer(COST_OF_PUT, section_key)?
            .signed_transfer;

        let request = Self::wrap_money_request(MoneyRequest::ValidateTransfer {
            signed_transfer: signed_transfer.clone(),
        });

        debug!("Transfer to be sent: {:?}", &signed_transfer);

        let (transfer_message, message_id) =
            TransferActor::create_network_message(safe_key.clone(), request)?;

        // setup connection manager
        let _bootstrapped = cm.bootstrap(safe_key.clone()).await;

        let payment_proof: DebitAgreementProof = self
            .await_validation(message_id, &safe_key.public_id(), &transfer_message)
            .await?;

        Ok(payment_proof)
    }

    /// Send message and await validation and constructin of DebitAgreementProof
    async fn await_validation(
        &mut self,
        _message_id: MessageId,
        pub_id: &PublicId,
        message: &Message,
    ) -> Result<DebitAgreementProof, CoreError> {
        trace!("Awaiting transfer validation");
        let mut cm = self.connection_manager();

        let proof = cm.send_for_validation(&pub_id, &message, self).await?;

        Ok(proof)
    }
}

// TODO: Do we need "new" to actually instantiate with a transfer?...
#[cfg(all(test, feature = "simulated-payouts"))]
mod tests {

    use super::*;
    use test_utils::get_keys_and_connection_manager;

    #[tokio::test]
    async fn transfer_actor_creation__() {
        let (safe_key, cm) = get_keys_and_connection_manager().await;
        let _transfer_actor = TransferActor::new(safe_key, cm.clone()).await.unwrap();

        assert!(true);
    }
}
