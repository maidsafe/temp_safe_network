use safe_nd::{Cmd, Money, PublicKey, Transfer, TransferCmd};

use crate::client::{create_cmd_message, TransferActor};
use crate::errors::CoreError;

/// Handle all Money transfers and Write API requests for a given ClientId.
impl TransferActor {
    #[cfg(not(feature = "simulated-payouts"))]
    /// Simulate a farming payout
    pub async fn trigger_simulated_farming_payout(
        &mut self,
        _to: PublicKey,
        _amount: Money,
    ) -> Result<(), CoreError> {
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
    ) -> Result<(), CoreError> {
        dbg!("Triggering a test farming payout to: {:?}", &to);
        let mut cm = self.connection_manager();
        let safe_key = self.safe_key.clone();
        self.simulated_farming_payout_dot.apply_inc();

        let simulated_transfer = Transfer {
            to,
            amount,
            id: self.simulated_farming_payout_dot,
        };

        let simluated_farming_cmd =
            Cmd::Transfer(TransferCmd::SimulatePayout(simulated_transfer.clone()));

        let message = create_cmd_message(simluated_farming_cmd);

        let pub_id = safe_key.public_id();

        let _bootstrapped = cm.bootstrap(safe_key.clone()).await;
        let _ = cm.send_cmd(&pub_id, &message).await?;

        // If we're getting the payout for our own actor, update it here
        if to == self.safe_key.public_key() {
            // get full history from network and apply locally
            self.get_history().await?;
        }
        Ok(())
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
    use std::str::FromStr;

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
