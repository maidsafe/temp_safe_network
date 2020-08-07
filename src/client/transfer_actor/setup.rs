use safe_nd::{ClientFullId, Money, PublicKey, Query, QueryResponse, TransferQuery};
use safe_transfers::TransferActor as SafeTransferActor;

use std::str::FromStr;

use crate::client::ConnectionManager;
use crate::client::{create_query_message, ClientTransferValidator, TransferActor};
use crate::errors::CoreError;
use crdts::Dot;

use futures::lock::Mutex;
use log::{info, trace};

use std::sync::Arc;
use threshold_crypto::{PublicKeySet, SecretKey};

/// Handle all Money transfers and Write API requests for a given ClientId.
impl TransferActor {
    /// Get our replica instance PK set
    pub async fn get_replica_keys(
        full_id: ClientFullId,
        cm: &mut ConnectionManager,
    ) -> Result<PublicKeySet, CoreError> {
        trace!("Getting replica keys for {:?}", full_id);

        let keys_query_msg = Query::Transfer(TransferQuery::GetReplicaKeys(*full_id.public_key()));

        let message = create_query_message(keys_query_msg);

        cm.bootstrap().await?;
        let res = cm.send_query(&message).await?;

        match res {
            QueryResponse::GetReplicaKeys(pk_set) => Ok(pk_set?),
            _ => Err(CoreError::from(format!(
                "Unexpected response when retrieving account replica keys for {:?}",
                full_id.public_key()
            ))),
        }
    }

    /// Create a new Transfer Actor for a previously unused public key
    pub async fn new(
        full_id: &ClientFullId,
        mut connection_manager: ConnectionManager,
    ) -> Result<Self, CoreError> {
        info!(
            "Initiating Safe Transfer Actor for PK {:?}",
            full_id.public_key()
        );
        let simulated_farming_payout_dot =
            Dot::new(PublicKey::from(SecretKey::random().public_key()), 0);

        let replicas_pk_set =
            TransferActor::get_replica_keys(full_id.clone(), &mut connection_manager).await?;

        let validator = ClientTransferValidator {};

        let transfer_actor = Arc::new(Mutex::new(SafeTransferActor::new(
            full_id.keypair().clone(),
            replicas_pk_set.clone(),
            validator,
        )));

        let actor = Self {
            full_id: full_id.clone(),
            transfer_actor,
            connection_manager,
            replicas_pk_set,
            simulated_farming_payout_dot, // replicas_sk_set
        };

        #[cfg(feature = "simulated-payouts")]
        {
            // we're testing, and currently a lot of tests expect 10 money to start
            let _ = actor
                .trigger_simulated_farming_payout(full_id.public_key(), Money::from_str("10")?)
                .await?;
        }
        Ok(actor)
    }

    /// Create a Transfer Actor from an existing public key with an account history
    pub async fn for_existing_account(
        full_id: ClientFullId,
        // history: History,
        mut connection_manager: ConnectionManager,
    ) -> Result<Self, CoreError> {
        info!(
            "Setting up SafeTransferActor for existing PK : {:?}",
            full_id.public_key()
        );
        let simulated_farming_payout_dot =
            Dot::new(PublicKey::from(SecretKey::random().public_key()), 0);

        let replicas_pk_set =
            TransferActor::get_replica_keys(full_id.clone(), &mut connection_manager).await?;

        let validator = ClientTransferValidator {};

        let transfer_actor = SafeTransferActor::new(
            full_id.keypair().clone(),
            replicas_pk_set.clone(),
            validator,
        );

        let mut full_actor = Self {
            full_id,
            transfer_actor: Arc::new(Mutex::new(transfer_actor)),
            connection_manager,
            replicas_pk_set,
            simulated_farming_payout_dot, // replicas_sk_set
        };

        full_actor.get_history().await?;

        Ok(full_actor)
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
    async fn transfer_actor_creation__() -> Result<(), CoreError> {
        let (full_id, cm) = get_keys_and_connection_manager().await;
        let _transfer_actor = TransferActor::new(full_id, cm.clone()).await?;

        assert!(true);

        Ok(())
    }

    #[tokio::test]
    async fn transfer_actor_creation_hydration_for_nonexistant_balance() -> Result<(), CoreError> {
        let (full_id, cm) = get_keys_and_connection_manager().await;

        match TransferActor::for_existing_account(full_id, cm.clone()).await {
            Ok(actor) => {
                assert_eq!(actor.get_local_balance().await, Money::from_str("0").unwrap() );
                Ok(())
            },
            Err(e) => panic!("Should not error for nonexistant keys, only create a new instance with no history, we got: {:?}" , e )
        }
    }

    // TODO: only do this for real vault until we a local replica bank
    #[tokio::test]
    #[cfg(not(feature = "mock-network"))]
    async fn transfer_actor_creation_hydration_for_existing_balance() -> Result<(), CoreError> {
        let (full_id, _cm) = get_keys_and_connection_manager().await;
        let (full_id_two, cm) = get_keys_and_connection_manager().await;

        let mut initial_actor = TransferActor::new(full_id.clone(), cm.clone()).await?;

        let _ = initial_actor
            .trigger_simulated_farming_payout(full_id_two.public_key(), Money::from_str("100")?)
            .await?;

        match TransferActor::for_existing_account(full_id_two, cm.clone()).await {
            Ok(actor) => {
                assert_eq!(
                    actor.get_balance_from_network(None).await?,
                    Money::from_str("100")?
                );
                assert_eq!(actor.get_local_balance().await, Money::from_str("100")?);

                Ok(())
            }
            Err(e) => panic!("Account should exist {:?}", e),
        }
    }
}
