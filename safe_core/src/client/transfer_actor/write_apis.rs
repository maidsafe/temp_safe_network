use safe_nd::{
    Account, AccountRead, AccountWrite, BlobRead, BlobWrite, ClientRequest, DebitAgreementProof,
    IData, IDataAddress, MData, MDataAddress, MDataEntryActions, MDataPermissionSet, MapRead,
    MapWrite, PublicKey, Read, Request, Response, SData, SDataAddress, SDataOwner,
    SDataPrivPermissions, SDataPubPermissions, SDataWriteOp, SequenceRead, SequenceWrite, Write,
};
use safe_transfers::ActorEvent;

use crate::client::TransferActor;
use crate::errors::CoreError;
use log::info;

/// Handle Write API requests for a given TransferActor.
impl TransferActor {
    /// Delete mutable data user permission
    pub async fn delete_blob(&mut self, address: IDataAddress) -> Result<Response, CoreError> {
        let mut cm = self.connection_manager();

        // --------------------------
        // Payment for PUT
        // --------------------------
        let payment_proof = self.create_write_payment_proof().await?;

        //---------------------------------
        // The _actual_ request
        //---------------------------------
        let req = wrap_blob_write(BlobWrite::DeletePrivate(address), payment_proof.clone());
        let (message, _msg_id) = self.create_network_message(req)?;
        let response = cm.send(&self.safe_key.public_id(), &message).await?;

        self.apply_success_write_locally(response, payment_proof)
            .await
    }

    /// Delete sequence
    pub async fn delete_sequence(&mut self, address: SDataAddress) -> Result<Response, CoreError> {
        let mut cm = self.connection_manager();

        // --------------------------
        // Payment for PUT
        // --------------------------
        let payment_proof = self.create_write_payment_proof().await?;

        //---------------------------------
        // The _actual_ request
        //---------------------------------
        let req = wrap_seq_write(SequenceWrite::Delete(address), payment_proof.clone());
        let (message, _msg_id) = self.create_network_message(req)?;
        let response = cm.send(&self.safe_key.public_id(), &message).await?;

        self.apply_success_write_locally(response, payment_proof)
            .await
    }

    /// Delete sequence
    pub async fn delete_map(&mut self, address: MDataAddress) -> Result<Response, CoreError> {
        let mut cm = self.connection_manager();

        // --------------------------
        // Payment for PUT
        // --------------------------
        let payment_proof = self.create_write_payment_proof().await?;

        //---------------------------------
        // The _actual_ request
        //---------------------------------
        let req = wrap_map_write(MapWrite::Delete(address), payment_proof.clone());
        let (message, _msg_id) = self.create_network_message(req)?;
        let response = cm.send(&self.safe_key.public_id(), &message).await?;

        self.apply_success_write_locally(response, payment_proof)
            .await
    }

    /// Delete mutable data user permission
    pub async fn delete_map_user_perms(
        &mut self,
        address: MDataAddress,
        user: PublicKey,
        version: u64,
    ) -> Result<Response, CoreError> {
        let mut cm = self.connection_manager();

        // --------------------------
        // Payment for PUT
        // --------------------------
        let payment_proof = self.create_write_payment_proof().await?;

        //---------------------------------
        // The _actual_ request
        //---------------------------------

        let req = wrap_map_write(
            MapWrite::DelUserPermissions {
                address,
                user,
                version,
            },
            payment_proof.clone(),
        );

        let (message, _msg_id) = self.create_network_message(req)?;

        let response = cm.send(&self.safe_key.public_id(), &message).await?;

        self.apply_success_write_locally(response, payment_proof)
            .await
    }

    /// Set mutable data user permissions
    pub async fn set_map_user_perms(
        &mut self,
        address: MDataAddress,
        user: PublicKey,
        permissions: MDataPermissionSet,
        version: u64,
    ) -> Result<Response, CoreError> {
        let mut cm = self.connection_manager();

        // --------------------------
        // Payment for PUT
        // --------------------------
        let payment_proof = self.create_write_payment_proof().await?;

        //---------------------------------
        // The _actual_ request
        //---------------------------------

        let req = wrap_map_write(
            MapWrite::SetUserPermissions {
                address,
                user,
                permissions,
                version,
            },
            payment_proof.clone(),
        );

        let (message, _msg_id) = self.create_network_message(req)?;

        // TODO what will be the correct reponse here?... We have it validated, so registered?
        let response = cm.send(&self.safe_key.public_id(), &message).await?;

        self.apply_success_write_locally(response, payment_proof)
            .await
    }

    /// Mutate mutable data user entries
    pub async fn edit_map_entries(
        &mut self,
        address: MDataAddress,
        changes: MDataEntryActions,
    ) -> Result<Response, CoreError> {
        let mut cm = self.connection_manager();

        // --------------------------
        // Payment for PUT
        // --------------------------
        let payment_proof = self.create_write_payment_proof().await?;

        //---------------------------------
        // The _actual_ request
        //---------------------------------

        let req = wrap_map_write(MapWrite::Edit { address, changes }, payment_proof.clone());

        let (message, _msg_id) = self.create_network_message(req)?;
        let response = cm.send(&self.safe_key.public_id(), &message).await?;

        self.apply_success_write_locally(response, payment_proof)
            .await
    }

    /// Mutate sequence data owners
    pub async fn set_sequence_owner(
        &mut self,
        op: SDataWriteOp<SDataOwner>,
    ) -> Result<Response, CoreError> {
        let mut cm = self.connection_manager();

        // --------------------------
        // Payment for PUT
        // --------------------------
        let payment_proof = self.create_write_payment_proof().await?;

        //---------------------------------
        // The _actual_ request
        //---------------------------------
        let req = wrap_seq_write(SequenceWrite::SetOwner(op), payment_proof.clone());
        let (message, _msg_id) = self.create_network_message(req)?;
        let response = cm.send(&self.safe_key.public_id(), &message).await?;

        self.apply_success_write_locally(response, payment_proof)
            .await
    }

    /// Mutate sequenced data private permissions
    /// Wraps requests for payment validation and mutation
    pub async fn edit_sequence_private_perms(
        &mut self,
        op: SDataWriteOp<SDataPrivPermissions>,
    ) -> Result<Response, CoreError> {
        let mut cm = self.connection_manager();

        // --------------------------
        // Payment for PUT
        // --------------------------
        let payment_proof = self.create_write_payment_proof().await?;

        //---------------------------------
        // The _actual_ request
        //---------------------------------
        let req = wrap_seq_write(SequenceWrite::SetPrivPermissions(op), payment_proof.clone());
        let (message, _msg_id) = self.create_network_message(req)?;
        let response = cm.send(&self.safe_key.public_id(), &message).await?;

        self.apply_success_write_locally(response, payment_proof)
            .await
    }

    /// Mutate sequenced data public permissions
    /// Wraps requests for payment validation and mutation
    pub async fn edit_sequence_public_perms(
        &mut self,
        op: SDataWriteOp<SDataPubPermissions>,
    ) -> Result<Response, CoreError> {
        let mut cm = self.connection_manager();

        // --------------------------
        // Payment for PUT
        // --------------------------
        let payment_proof = self.create_write_payment_proof().await?;

        //---------------------------------
        // The _actual_ request
        //---------------------------------
        let req = wrap_seq_write(SequenceWrite::SetPubPermissions(op), payment_proof.clone());
        let (message, _msg_id) = self.create_network_message(req)?;
        let response = cm.send(&self.safe_key.public_id(), &message).await?;

        self.apply_success_write_locally(response, payment_proof)
            .await
    }

    /// Append data to a sequenced data object
    /// Wraps requests for payment validation and mutation
    pub async fn append_to_sequence(
        &mut self,
        op: SDataWriteOp<Vec<u8>>,
    ) -> Result<Response, CoreError> {
        let mut cm = self.connection_manager();

        // --------------------------
        // Payment for PUT
        // --------------------------
        let payment_proof = self.create_write_payment_proof().await?;

        //---------------------------------
        // The _actual_ request
        //---------------------------------
        let req = wrap_seq_write(SequenceWrite::Edit(op), payment_proof.clone());
        let (message, _msg_id) = self.create_network_message(req)?;
        let response = cm.send(&self.safe_key.public_id(), &message).await?;

        self.apply_success_write_locally(response, payment_proof)
            .await
    }

    /// Store a new public sequenced data object
    /// Wraps requests for payment validation and mutation
    pub async fn new_sequence(&mut self, data: SData) -> Result<Response, CoreError> {
        let mut cm = self.connection_manager();

        // --------------------------
        // Payment for PUT
        // --------------------------
        let payment_proof = self.create_write_payment_proof().await?;

        //---------------------------------
        // The _actual_ request
        //---------------------------------
        let req = wrap_seq_write(SequenceWrite::New(data), payment_proof.clone());
        let (message, _msg_id) = self.create_network_message(req)?;
        let response = cm.send(&self.safe_key.public_id(), &message).await?;

        self.apply_success_write_locally(response, payment_proof)
            .await
    }

    /// Store a new public mutable data object
    /// Wraps requests for payment validation and mutation
    pub async fn new_map(&mut self, data: MData) -> Result<Response, CoreError> {
        let mut cm = self.connection_manager();

        // --------------------------
        // Payment for PUT
        // --------------------------
        let payment_proof = self.create_write_payment_proof().await?;

        //---------------------------------
        // The _actual_ request
        //---------------------------------
        let req = wrap_map_write(MapWrite::New(data), payment_proof.clone());
        let (message, _msg_id) = self.create_network_message(req)?;
        let response = cm.send(&self.safe_key.public_id(), &message).await?;
        println!("STORE MD COME IN..... response: {:?}", response);
        self.apply_success_write_locally(response, payment_proof)
            .await
    }

    /// Store a new immutabledata object
    /// Wraps requests for payment validation and mutation
    pub async fn new_blob(&mut self, data: IData) -> Result<Response, CoreError> {
        let mut cm = self.connection_manager();

        // --------------------------
        // Payment for PUT
        // --------------------------
        let payment_proof = self.create_write_payment_proof().await?;

        //---------------------------------
        // The _actual_ request
        //---------------------------------
        let req = wrap_blob_write(BlobWrite::New(data), payment_proof.clone());
        let (message, _msg_id) = self.create_network_message(req)?;
        let response = cm.send(&self.safe_key.public_id(), &message).await?;

        self.apply_success_write_locally(response, payment_proof)
            .await
    }

    /// Store a new login packet
    /// Wraps requests for payment validation and mutation
    pub async fn new_account(&mut self, account: Account) -> Result<Response, CoreError> {
        info!("Store login packet");
        let mut cm = self.connection_manager();

        // --------------------------
        // Payment for PUT
        // --------------------------
        let payment_proof = self.create_write_payment_proof().await?;

        //---------------------------------
        // The _actual_ request
        //---------------------------------
        let req = wrap_account_write(AccountWrite::New(account), payment_proof.clone());
        let (message, _msg_id) = self.create_network_message(req)?;
        let response = cm.send(&self.safe_key.public_id(), &message).await?;

        self.apply_success_write_locally(response, payment_proof)
            .await
    }

    async fn apply_success_write_locally(
        &mut self,
        response: Response,
        debit_proof: DebitAgreementProof,
    ) -> Result<Response, CoreError> {
        match response.clone() {
            Response::Write(result) => {
                let mut actor = self.transfer_actor.lock().await;
                // First register with local actor, then reply.
                let register_event = actor.register(debit_proof.clone())?;

                actor.apply(ActorEvent::TransferRegistrationSent(register_event));

                Ok(response)
            }
            _ => Err(CoreError::from(format!(
                "Unexpected response received for write request: {:?}",
                response
            ))),
        }
    }
}

// TODO: Do we need "new" to actually instantiate with a transfer?...
#[cfg(all(test, feature = "simulated-payouts"))]
mod tests {

    use super::*;
    use crate::client::transfer_actor::test_utils::get_keys_and_connection_manager;
    use safe_nd::{Error as SndError, XorName};

    #[tokio::test]
    #[cfg(feature = "simulated-payouts")]
    async fn transfer_actor_with_no_balance_cannot_store_data() -> Result<(), CoreError> {
        let (safe_key, cm) = get_keys_and_connection_manager().await;

        let data = SData::new_pub(safe_key.public_key(), XorName::default(), 33323);

        let mut initial_actor =
            TransferActor::new_no_initial_balance(safe_key.clone(), cm.clone()).await?;

        match initial_actor.new_sequence(data).await {
            Err(CoreError::DataError(e)) => {
                assert_eq!(e.to_string(), "Not enough money to complete this operation");
            }
            res => panic!(
                "Unexpected response from mutation request from 0 balance key: {:?}",
                res
            ),
        }

        Ok(())
    }
}

fn wrap_blob_read(read: BlobRead) -> Request {
    Request::Client(ClientRequest::Read(Read::Blob(read)))
}

fn wrap_blob_write(write: BlobWrite, debit_agreement: DebitAgreementProof) -> Request {
    Request::Client(ClientRequest::Write {
        write: Write::Blob(write),
        debit_agreement,
    })
}

fn wrap_map_read(read: MapRead) -> Request {
    Request::Client(ClientRequest::Read(Read::Map(read)))
}

fn wrap_map_write(write: MapWrite, debit_agreement: DebitAgreementProof) -> Request {
    Request::Client(ClientRequest::Write {
        write: Write::Map(write),
        debit_agreement,
    })
}

fn wrap_seq_read(read: SequenceRead) -> Request {
    Request::Client(ClientRequest::Read(Read::Sequence(read)))
}

fn wrap_seq_write(write: SequenceWrite, debit_agreement: DebitAgreementProof) -> Request {
    Request::Client(ClientRequest::Write {
        write: Write::Sequence(write),
        debit_agreement,
    })
}

fn wrap_account_read(read: AccountRead) -> Request {
    Request::Client(ClientRequest::Read(Read::Account(read)))
}

fn wrap_account_write(write: AccountWrite, debit_agreement: DebitAgreementProof) -> Request {
    Request::Client(ClientRequest::Write {
        write: Write::Account(write),
        debit_agreement,
    })
}
