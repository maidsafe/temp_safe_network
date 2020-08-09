use safe_nd::{
    Account, AccountWrite, AppPermissions, AuthCmd, Blob, BlobAddress, BlobWrite, Cmd, DataCmd,
    DebitAgreementProof, Map, MapAddress, MapEntryActions, MapPermissionSet, MapWrite, PublicKey,
    Sequence, SequenceAddress, SequenceOwner, SequencePrivatePermissions,
    SequencePublicPermissions, SequenceWrite, SequenceWriteOp,
};
use safe_transfers::ActorEvent;

use crate::client::Client;
use crate::errors::CoreError;


/// Handle Write API msg_contents for a given Client.
impl Client {
    pub(crate) async fn apply_write_locally(
        &mut self,
        debit_proof: DebitAgreementProof,
    ) -> Result<(), CoreError> {
        let mut actor = self.transfer_actor.lock().await;
        // First register with local actor, then reply.
        let register_event = actor
            .register(debit_proof.clone())?
            .ok_or_else(|| CoreError::from("No events to register for proof."))?;

        actor.apply(ActorEvent::TransferRegistrationSent(register_event))?;

        Ok(())
    }
}

// TODO: Do we need "new" to actually instantiate with a transfer?...
#[cfg(all(test, feature = "simulated-payouts"))]
mod tests {

    use super::*;
    use crate::crypto::shared_box;
    use xor_name::XorName;

    #[tokio::test]
    #[cfg(feature = "simulated-payouts")]
    async fn transfer_actor_with_no_balance_cannot_store_data() -> Result<(), CoreError> {
        let (sk, pk) = shared_box::gen_bls_keypair();
        let pk = PublicKey::Bls(pk);

        let data = Sequence::new_pub(pk, XorName::random(), 33323);

        let mut initial_actor =
            Client::new_no_initial_balance(Some(sk.clone())).await?;

        match initial_actor.new_sequence(data).await {
            Err(CoreError::DataError(e)) => {
                assert_eq!(e.to_string(), "Not enough money to complete this operation");
            }
            res => panic!(
                "Unexpected response from mutation msg_contentsuest from 0 balance key: {:?}",
                res
            ),
        }

        Ok(())
    }
}
