use safe_nd::{
    DebitAgreementProof, Money, PublicKey, ReplicaEvent, Request, Response, SignatureShare,
    SignedTransfer, Transfer, TransferPropagated, Transfers as MoneyRequest,
};
use safe_transfers::{ActorEvent, TransfersSynched};

use crate::client::{Client, TransferActor};
use crate::errors::CoreError;

use std::str::FromStr;

use log::{debug, info, trace};

#[cfg(feature = "simulated-payouts")]
use threshold_crypto::SecretKeySet;

#[cfg(feature = "simulated-payouts")]
use rand::thread_rng;
#[cfg(feature = "simulated-payouts")]

/// Handle all Money transfers and Write API requests for a given ClientId.
impl TransferActor {
    #[cfg(not(feature = "simulated-payouts"))]
    /// Simulate a farming payout
    pub async fn trigger_simulated_farming_payout(
        &mut self,
        _to: PublicKey,
        _amount: Money,
    ) -> Result<Response, CoreError> {
        Err(CoreError::from(
            "Simulated payouts not available without 'simulated-payouts' feature flag",
        ))
    }

    #[cfg(feature = "simulated-payouts")]
    /// Simulate a farming payout
    pub async fn trigger_simulated_farming_payout(
        &mut self,
        to: PublicKey,
        amount: Money,
    ) -> Result<Response, CoreError> {
        info!("Triggering a test farming payout to: {:?}", &to);
        let mut cm = self.connection_manager();
        let safe_key = self.safe_key.clone();
        self.simulated_farming_payout_dot.apply_inc();

        let simulated_transfer = Transfer {
            to,
            amount,
            id: self.simulated_farming_payout_dot,
        };

        let simluated_farming_request = Self::wrap_money_request(MoneyRequest::SimulatePayout {
            transfer: simulated_transfer.clone(),
        });

        let (message, _message_id) =
            TransferActor::create_network_message(safe_key.clone(), simluated_farming_request)?;

        let pub_id = safe_key.public_id();

        let _bootstrapped = cm.bootstrap(safe_key.clone()).await;
        let res = cm.send(&pub_id, &message).await?;

        match res.clone() {
            Response::TransferRegistration(result) => {
                match result {
                    Ok(transfer_response) => {
                        // If we're getting the payout for our own actor, update it here
                        if to == self.safe_key.public_key() {
                            // get full history from network and apply locally
                            self.get_history().await;
                        }
                        Ok(res)
                    }
                    Err(error) => Err(CoreError::from(error)),
                }
            }
            _ => Err(CoreError::from(
                "Unexpected Reponse received to 'send money' request in ClientTransferActor",
            )),
        }
    }
}

// --------------------------------
// Tests
// ---------------------------------

// TODO: Do we need "new" to actually instantiate with a transfer?...
#[cfg(all(test, feature = "simulated-payouts"))]
mod tests {

    use super::*;

    use crate::client::transfer_actor::test_utils::get_keys_and_connection_manager;

    #[tokio::test]
    #[cfg(feature = "simulated-payouts")]
    async fn transfer_actor_can_receive_simulated_farming_payout() -> Result<(), CoreError> {
        let (safe_key, cm) = get_keys_and_connection_manager().await;
        let mut initial_actor =
            TransferActor::new_no_initial_balance(safe_key.clone(), cm.clone()).await?;

        let _ = initial_actor
            .trigger_simulated_farming_payout(safe_key.public_key(), Money::from_str("100")?)
            .await?;

        // 100 sent
        assert_eq!(
            initial_actor.get_local_balance().await,
            Money::from_str("100")?
        );

        assert_eq!(
            initial_actor.get_balance_from_network(None).await?,
            Money::from_str("100")?
        );

        Ok(())
    }
}
