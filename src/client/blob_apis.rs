use crate::errors::CoreError;
use crate::Client;
use log::trace;

use safe_nd::{
    AppPermissions, AuthQuery, Blob, BlobAddress, BlobRead, BlobWrite, ClientFullId, Cmd, DataCmd,
    DataQuery, DebitAgreementProof, Map, MapAddress, MapEntries, MapEntryActions, MapPermissionSet,
    MapRead, MapSeqEntries, MapSeqEntryActions, MapSeqValue, MapUnseqEntryActions, MapValue,
    MapValues, Message, MessageId, Money, PublicId, PublicKey, Query, QueryResponse, SeqMap,
    Sequence, SequenceAction, SequenceAddress, SequenceEntries, SequenceEntry, SequenceIndex,
    SequenceOwner, SequencePrivUserPermissions, SequencePrivatePermissions,
    SequencePubUserPermissions, SequencePublicPermissions, SequenceRead, SequenceUser,
    SequenceUserPermissions, UnseqMap,
};

fn wrap_blob_write(write: BlobWrite, payment: DebitAgreementProof) -> Cmd {
    Cmd::Data {
        cmd: DataCmd::Blob(write),
        payment,
    }
}

impl Client {
    /// Get immutable data from the network. If the data exists locally in the cache then it will be
    /// immediately returned without making an actual network request.
    async fn get_blob(&mut self, address: BlobAddress) -> Result<Blob, CoreError>
    where
        Self: Sized,
    {
        trace!("Fetch Blob");

        if let Some(data) = self.blob_cache.get_mut(&address) {
            trace!("Blob found in cache.");
            return Ok(data.clone());
        }

        let res = self
            .send_query(Query::Data(DataQuery::Blob(BlobRead::Get(address))))
            .await?;
        let data = match res {
            QueryResponse::GetBlob(res) => res.map_err(CoreError::from),
            _ => return Err(CoreError::ReceivedUnexpectedEvent),
        }?;

        // Put to cache
        self.blob_cache.put(*data.address(), data.clone());
        Ok(data)
    }

    /// Store a new immutabledata object
    /// Wraps msg_contents for payment validation and mutation
    pub async fn store_blob(&mut self, data: Blob) -> Result<(), CoreError> {
        // --------------------------
        // Payment for PUT
        // --------------------------
        let payment_proof = self.create_write_payment_proof().await?;

        //---------------------------------
        // The _actual_ message
        //---------------------------------
        let msg_contents = wrap_blob_write(BlobWrite::New(data), payment_proof.clone());
        let message = Self::create_cmd_message(msg_contents);
        let _ = self.connection_manager.send_cmd(&message).await?;

        self.apply_write_locally(payment_proof).await
    }

    /// Delete blob
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
        let message = Self::create_cmd_message(msg_contents);
        let _ = self.connection_manager.send_cmd(&message).await?;

        self.apply_write_locally(payment_proof).await
    }
}

#[cfg(all(test, feature = "simulated-payouts"))]
mod tests {

    use super::*;
    use crate::utils::{
        generate_random_vector,
        test_utils::{calculate_new_balance, gen_bls_keypair},
    };
    use safe_nd::{
        Error as SndError, MapAction, MapKind, Money, PrivateBlob, PublicBlob,
        SequencePrivUserPermissions,
    };
    use std::str::FromStr;
    use unwrap::unwrap;
    use xor_name::XorName;

    // Test putting and getting pub blob.
    #[tokio::test]
    async fn pub_blob_test() -> Result<(), CoreError> {
        let mut client = Client::new(None).await?;
        // The `Client::new(None)` initializes the client with 10 money.
        let start_bal = unwrap!(Money::from_str("10"));

        let value = generate_random_vector::<u8>(10);
        let data = Blob::Public(PublicBlob::new(value.clone()));
        let address = *data.address();
        let pk = gen_bls_keypair().public_key();

        let test_data = Blob::Private(PrivateBlob::new(value, pk));
        let res = client
            // Get inexistent blob
            .get_blob(address)
            .await;
        match res {
            Ok(data) => panic!("Pub blob should not exist yet: {:?}", data),
            Err(CoreError::DataError(SndError::NoSuchData)) => (),
            Err(e) => panic!("Unexpected: {:?}", e),
        }
        // Put blob
        client.store_blob(data.clone()).await?;
        let res = client.store_blob(test_data.clone()).await;
        match res {
            Ok(_) => panic!("Unexpected Success: Validating owners should fail"),
            Err(CoreError::DataError(SndError::InvalidOwners)) => (),
            Err(e) => panic!("Unexpected: {:?}", e),
        }

        let balance = client.get_balance(None).await?;
        let expected_bal = calculate_new_balance(start_bal, Some(2), None);
        assert_eq!(balance, expected_bal);
        // Fetch blob
        let fetched_data = client.get_blob(address).await?;
        assert_eq!(*fetched_data.address(), address);
        Ok(())
    }

    // Test putting, getting, and deleting unpub blob.
    #[tokio::test]
    async fn unpub_blob_test() -> Result<(), CoreError> {
        println!("blob_Test________");
        crate::utils::test_utils::init_log();
        // The `Client::new(None)` initializes the client with 10 money.
        let start_bal = unwrap!(Money::from_str("10"));
        println!("blob_Test_______pre client_");

        let mut client = Client::new(None).await?;
        println!("blob_Test_______post client_");

        // let client = client.clone();

        let value = generate_random_vector::<u8>(10);
        let data = Blob::Private(PrivateBlob::new(value.clone(), client.public_key().await));
        let data2 = data.clone();
        let data3 = data.clone();
        let address = *data.address();
        assert_eq!(address, *data2.address());

        let pub_data = Blob::Public(PublicBlob::new(value));

        let res = client
            // Get inexistent blob
            .get_blob(address)
            .await;
        match res {
            Ok(_) => panic!("Private blob should not exist yet"),
            Err(CoreError::DataError(SndError::NoSuchData)) => (),
            Err(e) => panic!("Unexpected: {:?}", e),
        }

        // Put blob
        client.store_blob(data.clone()).await?;
        // Test putting unpub blob with the same value.
        // Should conflict because duplication does .await?;not apply to unpublished data.
        let res = client.store_blob(data2.clone()).await;
        match res {
            Err(CoreError::DataError(SndError::DataExists)) => (),
            res => panic!("Unexpected: {:?}", res),
        }
        let balance = client.get_balance(None).await?;
        // mutation_count of 3 as even our failed op counts as a mutation
        let expected_bal = calculate_new_balance(start_bal, Some(3), None);
        assert_eq!(balance, expected_bal);

        // Test putting published blob with the same value. Should not conflict.
        client.store_blob(pub_data).await?;
        // Fetch blob
        let fetched_data = client.get_blob(address).await?;

        assert_eq!(*fetched_data.address(), address);

        // Delete blob
        client.delete_blob(address).await?;
        // Make sure blob was deleted
        let res = client.get_blob(address).await;
        match res {
            Ok(_) => panic!("Private blob still exists after deletion"),
            Err(error) => assert!(error.to_string().contains("Chunk not found")),
        }

        // Test putting unpub blob with the same value again. Should not conflict.
        client.store_blob(data3.clone()).await?;
        Ok(())
    }

    #[tokio::test]
    pub async fn blob_deletions_should_cost_put_price() -> Result<(), CoreError> {
        let mut client = Client::new(None).await?;

        let blob = Blob::Private(PrivateBlob::new(generate_random_vector::<u8>(10), client.public_key().await));
        let blob_address = *blob.address();
        client.store_blob(blob).await?;

        let balance_before_delete = client.get_balance(None).await?;
        client.delete_blob(blob_address).await?;
        let new_balance = client.get_balance(None).await?;

        // make sure we have _some_ balance
        assert_ne!(balance_before_delete, Money::from_str("0")?);
        assert_ne!(balance_before_delete, new_balance);

        Ok(())
    }
}
