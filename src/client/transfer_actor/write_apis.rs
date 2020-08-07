use safe_nd::{
    Account, AccountWrite, AppPermissions, AuthCmd, Blob, BlobAddress, BlobWrite, Cmd, DataCmd,
    DebitAgreementProof, Map, MapAddress, MapEntryActions, MapPermissionSet, MapWrite, PublicKey,
    Sequence, SequenceAddress, SequenceOwner, SequencePrivatePermissions,
    SequencePublicPermissions, SequenceWrite, SequenceWriteOp,
};
use safe_transfers::ActorEvent;

use crate::client::{create_cmd_message, TransferActor};
use crate::errors::CoreError;
use log::info;

/// Handle Write API msg_contents for a given TransferActor.
impl TransferActor {
    /// Delete mutable data user permission
    pub async fn delete_blob(&mut self, address: BlobAddress) -> Result<(), CoreError> {
        // --------------------------
        // Payment for PUT
        // --------------------------
        let payment_proof = self.create_write_payment_proof().await?;

        //---------------------------------
        // The _actual_ message
        //---------------------------------
        let msg_contents =
            wrap_blob_write(BlobWrite::DeletePrivate(address), payment_proof.clone());
        let message = create_cmd_message(msg_contents);
        let _ = self.connection_manager.send_cmd(&message).await?;

        self.apply_write_locally(payment_proof).await
    }

    /// Delete sequence
    pub async fn delete_sequence(&mut self, address: SequenceAddress) -> Result<(), CoreError> {
        // --------------------------
        // Payment for PUT
        // --------------------------
        let payment_proof = self.create_write_payment_proof().await?;

        //---------------------------------
        // The _actual_ message
        //---------------------------------
        let msg_contents = wrap_seq_write(SequenceWrite::Delete(address), payment_proof.clone());
        let message = create_cmd_message(msg_contents);
        let _ = self.connection_manager.send_cmd(&message).await?;

        self.apply_write_locally(payment_proof).await
    }

    /// Delete sequence
    pub async fn delete_map(&mut self, address: MapAddress) -> Result<(), CoreError> {
        // --------------------------
        // Payment for PUT
        // --------------------------
        let payment_proof = self.create_write_payment_proof().await?;

        //---------------------------------
        // The _actual_ message
        //---------------------------------
        let msg_contents = wrap_map_write(MapWrite::Delete(address), payment_proof.clone());
        let message = create_cmd_message(msg_contents);
        let _ = self.connection_manager.send_cmd(&message).await?;

        self.apply_write_locally(payment_proof).await
    }

    /// Delete mutable data user permission
    pub async fn delete_map_user_perms(
        &mut self,
        address: MapAddress,
        user: PublicKey,
        version: u64,
    ) -> Result<(), CoreError> {
        // --------------------------
        // Payment for PUT
        // --------------------------
        let payment_proof = self.create_write_payment_proof().await?;

        //---------------------------------
        // The _actual_ message
        //---------------------------------

        let msg_contents = wrap_map_write(
            MapWrite::DelUserPermissions {
                address,
                user,
                version,
            },
            payment_proof.clone(),
        );

        let message = create_cmd_message(msg_contents);

        let _ = self.connection_manager.send_cmd(&message).await?;

        self.apply_write_locally(payment_proof).await
    }

    /// Set mutable data user permissions
    pub async fn set_map_user_perms(
        &mut self,
        address: MapAddress,
        user: PublicKey,
        permissions: MapPermissionSet,
        version: u64,
    ) -> Result<(), CoreError> {
        // --------------------------
        // Payment for PUT
        // --------------------------
        let payment_proof = self.create_write_payment_proof().await?;

        //---------------------------------
        // The _actual_ message
        //---------------------------------

        let msg_contents = wrap_map_write(
            MapWrite::SetUserPermissions {
                address,
                user,
                permissions,
                version,
            },
            payment_proof.clone(),
        );

        let message = create_cmd_message(msg_contents);

        // TODO what will be the correct reponse here?... We have it validated, so registered?
        let _ = self.connection_manager.send_cmd(&message).await?;

        self.apply_write_locally(payment_proof).await
    }

    /// Mutate mutable data user entries
    pub async fn edit_map_entries(
        &mut self,
        address: MapAddress,
        changes: MapEntryActions,
    ) -> Result<(), CoreError> {
        // --------------------------
        // Payment for PUT
        // --------------------------
        let payment_proof = self.create_write_payment_proof().await?;

        //---------------------------------
        // The _actual_ message
        //---------------------------------

        let msg_contents =
            wrap_map_write(MapWrite::Edit { address, changes }, payment_proof.clone());

        let message = create_cmd_message(msg_contents);
        let _ = self.connection_manager.send_cmd(&message).await?;

        self.apply_write_locally(payment_proof).await
    }

    /// Mutate sequence data owners
    pub async fn set_sequence_owner(
        &mut self,
        op: SequenceWriteOp<SequenceOwner>,
    ) -> Result<(), CoreError> {
        // --------------------------
        // Payment for PUT
        // --------------------------
        let payment_proof = self.create_write_payment_proof().await?;

        //---------------------------------
        // The _actual_ message
        //---------------------------------
        let msg_contents = wrap_seq_write(SequenceWrite::SetOwner(op), payment_proof.clone());
        let message = create_cmd_message(msg_contents);
        let _ = self.connection_manager.send_cmd(&message).await?;

        self.apply_write_locally(payment_proof).await
    }

    /// Mutate sequenced data private permissions
    /// Wraps msg_contents for payment validation and mutation
    pub async fn edit_sequence_private_perms(
        &mut self,
        op: SequenceWriteOp<SequencePrivatePermissions>,
    ) -> Result<(), CoreError> {
        // --------------------------
        // Payment for PUT
        // --------------------------
        let payment_proof = self.create_write_payment_proof().await?;

        //---------------------------------
        // The _actual_ message
        //---------------------------------
        let msg_contents = wrap_seq_write(
            SequenceWrite::SetPrivatePermissions(op),
            payment_proof.clone(),
        );
        let message = create_cmd_message(msg_contents);
        let _ = self.connection_manager.send_cmd(&message).await?;

        self.apply_write_locally(payment_proof).await
    }

    /// Mutate sequenced data public permissions
    /// Wraps msg_contents for payment validation and mutation
    pub async fn edit_sequence_public_perms(
        &mut self,
        op: SequenceWriteOp<SequencePublicPermissions>,
    ) -> Result<(), CoreError> {
        // --------------------------
        // Payment for PUT
        // --------------------------
        let payment_proof = self.create_write_payment_proof().await?;

        //---------------------------------
        // The _actual_ message
        //---------------------------------
        let msg_contents = wrap_seq_write(
            SequenceWrite::SetPublicPermissions(op),
            payment_proof.clone(),
        );
        let message = create_cmd_message(msg_contents);
        let _ = self.connection_manager.send_cmd(&message).await?;

        self.apply_write_locally(payment_proof).await
    }

    /// Append data to a sequenced data object
    /// Wraps msg_contents for payment validation and mutation
    pub async fn append_to_sequence(
        &mut self,
        op: SequenceWriteOp<Vec<u8>>,
    ) -> Result<(), CoreError> {
        // --------------------------
        // Payment for PUT
        // --------------------------
        let payment_proof = self.create_write_payment_proof().await?;

        //---------------------------------
        // The _actual_ message
        //---------------------------------
        let msg_contents = wrap_seq_write(SequenceWrite::Edit(op), payment_proof.clone());
        let message = create_cmd_message(msg_contents);
        let _ = self.connection_manager.send_cmd(&message).await?;

        self.apply_write_locally(payment_proof).await
    }

    /// Store a new public sequenced data object
    /// Wraps msg_contents for payment validation and mutation
    pub async fn new_sequence(&mut self, data: Sequence) -> Result<(), CoreError> {
        // --------------------------
        // Payment for PUT
        // --------------------------
        let payment_proof = self.create_write_payment_proof().await?;

        //---------------------------------
        // The _actual_ message
        //---------------------------------
        let msg_contents = wrap_seq_write(SequenceWrite::New(data), payment_proof.clone());
        let message = create_cmd_message(msg_contents);
        let _ = self.connection_manager.send_cmd(&message).await?;

        self.apply_write_locally(payment_proof).await
    }

    /// Store a new public mutable data object
    /// Wraps msg_contents for payment validation and mutation
    pub async fn new_map(&mut self, data: Map) -> Result<(), CoreError> {
        // --------------------------
        // Payment for PUT
        // --------------------------
        let payment_proof = self.create_write_payment_proof().await?;

        //---------------------------------
        // The _actual_ message
        //---------------------------------
        let msg_contents = wrap_map_write(MapWrite::New(data), payment_proof.clone());
        let message = create_cmd_message(msg_contents);
        let _ = self.connection_manager.send_cmd(&message).await?;

        self.apply_write_locally(payment_proof).await
    }

    /// Store a new immutabledata object
    /// Wraps msg_contents for payment validation and mutation
    pub async fn new_blob(&mut self, data: Blob) -> Result<(), CoreError> {
        // --------------------------
        // Payment for PUT
        // --------------------------
        let payment_proof = self.create_write_payment_proof().await?;

        //---------------------------------
        // The _actual_ message
        //---------------------------------
        let msg_contents = wrap_blob_write(BlobWrite::New(data), payment_proof.clone());
        let message = create_cmd_message(msg_contents);
        let _ = self.connection_manager.send_cmd(&message).await?;

        self.apply_write_locally(payment_proof).await
    }

    /// Store a new login packet
    /// Wraps msg_contents for payment validation and mutation
    pub async fn new_account(&mut self, account: Account) -> Result<(), CoreError> {
        info!("Store login packet");
        // --------------------------
        // Payment for PUT
        // --------------------------
        let payment_proof = self.create_write_payment_proof().await?;

        //---------------------------------
        // The _actual_ message
        //---------------------------------
        let msg_contents = wrap_account_write(AccountWrite::New(account), payment_proof.clone());
        let message = create_cmd_message(msg_contents);
        let _ = self.connection_manager.send_cmd(&message).await?;

        self.apply_write_locally(payment_proof).await
    }

    /// Delete a key at the ClientHandler/Authenticator data structs
    /// Wraps msg_contents for payment validation and mutation
    pub async fn delete_auth_key(&mut self, key: PublicKey, version: u64) -> Result<(), CoreError> {
        info!("Store login packet");
        // --------------------------
        // Payment for PUT
        // --------------------------
        let payment_proof = self.create_write_payment_proof().await?;

        //---------------------------------
        // The _actual_ message
        //---------------------------------
        let msg_contents = wrap_auth_cmd(AuthCmd::DelAuthKey {
            client: *self.full_id.public_key(),
            key,
            version,
        });
        let message = create_cmd_message(msg_contents);
        let _ = self.connection_manager.send_cmd(&message).await?;

        self.apply_write_locally(payment_proof).await
    }

    /// Insert a key at the ClientHandler/Authenticator data structs
    /// Wraps msg_contents for payment validation and mutation
    pub async fn insert_auth_key(
        &mut self,
        key: PublicKey,
        permissions: AppPermissions,
        version: u64,
    ) -> Result<(), CoreError> {
        info!("Store login packet");
        // --------------------------
        // Payment for PUT
        // --------------------------
        let payment_proof = self.create_write_payment_proof().await?;

        //---------------------------------
        // The _actual_ message
        //---------------------------------
        let msg_contents = wrap_auth_cmd(AuthCmd::InsAuthKey {
            client: *self.full_id.public_key(),
            permissions,
            key,
            version,
        });
        let message = create_cmd_message(msg_contents);
        let _ = self.connection_manager.send_cmd(&message).await?;

        self.apply_write_locally(payment_proof).await
    }

    async fn apply_write_locally(
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
    use crate::client::transfer_actor::test_utils::get_keys_and_connection_manager;
    use xor_name::XorName;

    #[tokio::test]
    #[cfg(feature = "simulated-payouts")]
    async fn transfer_actor_with_no_balance_cannot_store_data() -> Result<(), CoreError> {
        let (safe_key, cm) = get_keys_and_connection_manager().await;

        let data = Sequence::new_pub(safe_key.public_key(), XorName::random(), 33323);

        let mut initial_actor =
            TransferActor::new_no_initial_balance(safe_key.clone(), cm.clone()).await?;

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

fn wrap_blob_write(write: BlobWrite, payment: DebitAgreementProof) -> Cmd {
    Cmd::Data {
        cmd: DataCmd::Blob(write),
        payment,
    }
}

fn wrap_map_write(write: MapWrite, payment: DebitAgreementProof) -> Cmd {
    Cmd::Data {
        cmd: DataCmd::Map(write),
        payment,
    }
}

fn wrap_seq_write(write: SequenceWrite, payment: DebitAgreementProof) -> Cmd {
    Cmd::Data {
        cmd: DataCmd::Sequence(write),
        payment,
    }
}

fn wrap_account_write(write: AccountWrite, payment: DebitAgreementProof) -> Cmd {
    Cmd::Data {
        cmd: DataCmd::Account(write),
        payment,
    }
}

fn wrap_auth_cmd(cmd: AuthCmd) -> Cmd {
    Cmd::Auth(cmd)
}
