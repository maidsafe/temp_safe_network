use super::*;
use crate::client::{attempt_bootstrap, ConnectionManager, TransferActor};
use crate::config_handler::Config;
use futures::channel::mpsc;
use rand::thread_rng;
use safe_nd::ClientFullId;
use threshold_crypto::SecretKey;

#[cfg(feature = "simulated-payouts")]
impl TransferActor {
    pub async fn new_no_initial_balance(
        full_id: ClientFullId,
        connection_manager: ConnectionManager,
    ) -> Result<Self, CoreError> {
        info!(
            "Initiating Safe Transfer Actor for PK {:?}",
            full_id.public_key()
        );
        let simulated_farming_payout_dot =
            Dot::new(PublicKey::from(SecretKey::random().public_key()), 0);

        let replicas_pk_set =
            TransferActor::get_replica_keys(full_id.clone(), connection_manager.clone()).await?;

        let validator = ClientTransferValidator {};

        let transfer_actor = Arc::new(Mutex::new(SafeTransferActor::new(
            full_id.keypair(),
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

        Ok(actor)
    }
}

pub async fn get_keys_and_connection_manager() -> (ClientFullId, ConnectionManager) {
    let mut rng = thread_rng();
    let client_full_id = ClientFullId::new_ed25519(&mut rng);

    let (net_sender, _net_receiver) = mpsc::unbounded();

    // Create the connection manager
    let connection_manager =
        attempt_bootstrap(&Config::new().quic_p2p, &net_sender, client_full_id.clone())
            .await
            .unwrap();

    (client_full_id, connection_manager)
}
