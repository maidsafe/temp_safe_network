// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::client::Client;
use crate::crypto::shared_secretbox;
use crate::self_encryption_storage::{
    SEStorageError, SelfEncryptionStorage, SelfEncryptionStorageDryRun,
};
use crate::utils;
use crate::CoreError;

use bincode::{deserialize, serialize};

use log::trace;

use safe_nd::{Blob, BlobAddress, PrivateBlob, PublicBlob};
use self_encryption::{DataMap, SelfEncryptor, Storage};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
enum DataTypeEncoding {
    Serialised(Vec<u8>),
    DataMap(DataMap),
}

/// Create and obtain immutable data out of the given raw bytes. This will encrypt the right content
/// if the keys are provided and will ensure the maximum immutable data chunk size is respected.
pub async fn create(
    client: &(impl Client + 'static),
    value: &[u8],
    published: bool,
    encryption_key: Option<shared_secretbox::Key>,
) -> Result<Blob, CoreError> {
    trace!("Creating conformant Blob.");
    let se_storage = SelfEncryptionStorage::new(client.clone(), published);
    write_with_self_encryptor(se_storage, client, value, published, encryption_key).await
}

/// Create and obtain immutable data out of the given raw bytes. This will encrypt the right content
/// if the keys are provided and will ensure the maximum immutable data chunk size is respected.
/// The DataMap is generated but the chunks are not uploaded to the network.
pub async fn gen_data_map(
    client: &(impl Client + 'static),
    value: &[u8],
    published: bool,
    encryption_key: Option<shared_secretbox::Key>,
) -> Result<Blob, CoreError> {
    trace!("Creating conformant Blob data map.");
    let se_storage = SelfEncryptionStorageDryRun::new(client.clone(), published);
    write_with_self_encryptor(se_storage, client, value, published, encryption_key).await
}

/// Get the raw bytes from `Blob` created via the `create` function in this module.
pub async fn extract_value(
    client: &(impl Client + 'static),
    data: Blob,
    position: Option<u64>,
    len: Option<u64>,
    decryption_key: Option<shared_secretbox::Key>,
) -> Result<Vec<u8>, CoreError> {
    let published = data.is_pub();
    let se_storage = SelfEncryptionStorage::new(client.clone(), published);
    let value = unpack(se_storage.clone(), data).await?;

    let data_map = if let Some(key) = decryption_key {
        let plain_text = utils::symmetric_decrypt(&value, &key)?;
        deserialize(&plain_text)?
    } else {
        deserialize(&value)?
    };

    let self_encryptor = SelfEncryptor::new(se_storage, data_map)?;

    let length = match len {
        None => self_encryptor.len().await,
        Some(request_length) => request_length,
    };

    let read_position = match position {
        None => 0,
        Some(pos) => pos,
    };

    match self_encryptor.read(read_position, length).await {
        Ok(data) => Ok(data),
        Err(error) => Err(CoreError::from(error)),
    }
}

/// Get immutable data from the network and extract its value, decrypting it in the process (if keys
/// provided). This combines `get_blob` in `Client` and `extract_value` in this module into one
/// function.
pub async fn get_value(
    client: &(impl Client + 'static),
    address: BlobAddress,
    position: Option<u64>,
    len: Option<u64>,
    decryption_key: Option<shared_secretbox::Key>,
) -> Result<Vec<u8>, CoreError> {
    let client2 = client.clone();
    let data = client.get_blob(address).await?;
    extract_value(&client2, data, position, len, decryption_key).await
}

async fn write_with_self_encryptor<S>(
    se_storage: S,
    client: &(impl Client + 'static),
    value: &[u8],
    published: bool,
    encryption_key: Option<shared_secretbox::Key>,
) -> Result<Blob, CoreError>
where
    S: Storage<Error = SEStorageError> + Clone + Send + Sync + 'static,
{
    let self_encryptor = SelfEncryptor::new(se_storage.clone(), DataMap::None)?;
    self_encryptor.write(value, 0).await?;

    let (data_map, _) = self_encryptor.close().await?;

    let serialised_data_map = serialize(&data_map)?;

    let value = if let Some(key) = encryption_key {
        let cipher_text = utils::symmetric_encrypt(&serialised_data_map, &key, None)?;
        serialize(&DataTypeEncoding::Serialised(cipher_text))?
    } else {
        serialize(&DataTypeEncoding::Serialised(serialised_data_map))?
    };

    pack(se_storage, client, value, published).await
}

async fn pack<S>(
    se_storage: S,
    client: &(impl Client + 'static),
    mut value: Vec<u8>,
    published: bool,
) -> Result<Blob, CoreError>
where
    S: Storage<Error = SEStorageError> + Clone + 'static + Sync + Send,
{
    loop {
        let data: Blob = if published {
            PublicBlob::new(value).into()
        } else {
            PrivateBlob::new(value, client.public_key().await).into()
        };

        let serialised_data = serialize(&data)?;

        if data.validate_size() {
            return Ok(data);
        }

        let self_encryptor = SelfEncryptor::new(se_storage.clone(), DataMap::None)?;

        // TODO make read/write properly x-thread compatible in self_encrypt
        self_encryptor.write(&serialised_data, 0).await?;

        let (data_map, _) = self_encryptor.close().await?;

        value = serialize(&DataTypeEncoding::DataMap(data_map))?;
    }
}

async fn unpack<S>(se_storage: S, mut data: Blob) -> Result<Vec<u8>, CoreError>
where
    S: Storage<Error = SEStorageError> + Clone + 'static + Send + Sync,
{
    loop {
        match deserialize(data.value())? {
            DataTypeEncoding::Serialised(value) => return Ok(value),
            DataTypeEncoding::DataMap(data_map) => {
                let self_encryptor = SelfEncryptor::new(se_storage.clone(), data_map)?;
                let length = self_encryptor.len().await;

                let serialised_data = self_encryptor.read(0, length).await?;

                data = deserialize(&serialised_data)?;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::CoreError;
    use safe_nd::Error as SndError;
    use utils::{self, test_utils::random_client};

    // Test creating and retrieving a 1kb blob.
    #[tokio::test]
    async fn create_and_retrieve_1kb_pub_unencrypted() -> Result<(), CoreError> {
        let size = 1024;

        gen_data_then_map_create_and_retrieve(size, true, None).await?;

        Ok(())
    }

    #[tokio::test]
    async fn create_and_retrieve_1kb_unpub_unencrypted() -> Result<(), CoreError> {
        let size = 1024;

        gen_data_then_map_create_and_retrieve(size, false, None).await?;
        Ok(())
    }

    #[tokio::test]
    async fn create_and_retrieve_1kb_unpub_encrypted() -> Result<(), CoreError> {
        let size = 1024;

        let key = shared_secretbox::gen_key();
        gen_data_then_map_create_and_retrieve(size, false, Some(key)).await?;

        Ok(())
    }

    #[tokio::test]
    async fn create_and_retrieve_1kb_pub_encrypted() -> Result<(), CoreError> {
        let size = 1024;
        let key = shared_secretbox::gen_key();
        gen_data_then_map_create_and_retrieve(size, true, Some(key)).await?;
        Ok(())
    }

    #[tokio::test]
    async fn create_and_retrieve_1kb_unencrypted_put_retrieval_of_encrypted(
    ) -> Result<(), CoreError> {
        let size = 1024;
        let value = utils::generate_random_vector(size);

        let value = value.clone();
        let key = shared_secretbox::gen_key();

        let client = random_client()?;
        let client2 = client.clone();
        let client3 = client.clone();

        let data = create(&client, &value, true, None).await?;
        let address = *data.address();
        client2.put_blob(data).await?;

        let res = get_value(&client3, address, None, None, Some(key)).await;
        assert!(res.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn create_and_retrieve_1kb_encrypted_put_retrieval_of_unencrypted(
    ) -> Result<(), CoreError> {
        let size = 1024;
        let value = utils::generate_random_vector(size);

        let value = value.clone();
        let key = shared_secretbox::gen_key();

        let client = random_client()?;
        let client2 = client.clone();
        let client3 = client.clone();

        let data = create(&client, &value, true, Some(key)).await?;
        let address = *data.address();
        client2.put_blob(data).await?;

        let res = get_value(&client3, address, None, None, None).await;
        assert!(res.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn create_and_retrieve_1kb_encrypted_put_pub_retrieval_of_unpub() -> Result<(), CoreError>
    {
        let size = 1024;
        let value = utils::generate_random_vector(size);

        let client = random_client()?;
        let client2 = client.clone();
        let client3 = client.clone();

        let data = create(&client, &value, true, None).await?;
        let data_name = *data.name();
        client2.put_blob(data).await?;

        let address = BlobAddress::Private(data_name);
        let res = get_value(&client3, address, None, None, None).await;
        assert!(res.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn create_and_retrieve_1kb_encrypted_put_unpub_retrieval_of_pub() -> Result<(), CoreError>
    {
        let size = 1024;

        let value = utils::generate_random_vector(size);

        let client = random_client()?;
        let client2 = client.clone();
        let client3 = client.clone();

        let data = create(&client, &value, false, None).await?;
        let data_name = *data.name();
        client2.put_blob(data).await?;

        let address = BlobAddress::Public(data_name);
        let res = get_value(&client3, address, None, None, None).await;
        assert!(res.is_err());

        Ok(())
    }

    // ----------------------------------------------------------------
    // 10mb (ie. more than 1 chunk)
    // ----------------------------------------------------------------

    // Test creating and retrieving a 1kb blob.
    #[tokio::test]
    async fn create_and_retrieve_10mb_pub_unencrypted() -> Result<(), CoreError> {
        let size = 1024 * 1024 * 10;

        gen_data_then_map_create_and_retrieve(size, true, None).await?;

        Ok(())
    }

    #[tokio::test]
    async fn create_and_retrieve_10mb_unpub_unencrypted() -> Result<(), CoreError> {
        let size = 1024 * 1024 * 10;

        gen_data_then_map_create_and_retrieve(size, false, None).await?;
        Ok(())
    }

    #[tokio::test]
    async fn create_and_retrieve_10mb_unpub_encrypted() -> Result<(), CoreError> {
        let size = 1024 * 1024 * 10;

        let key = shared_secretbox::gen_key();
        gen_data_then_map_create_and_retrieve(size, false, Some(key)).await?;

        Ok(())
    }

    #[tokio::test]
    async fn create_and_retrieve_10mb_pub_encrypted() -> Result<(), CoreError> {
        let size = 1024 * 1024 * 10;
        let key = shared_secretbox::gen_key();
        gen_data_then_map_create_and_retrieve(size, true, Some(key)).await?;
        Ok(())
    }

    #[tokio::test]
    async fn create_and_retrieve_10mb_unencrypted_put_retrieval_of_encrypted(
    ) -> Result<(), CoreError> {
        let size = 1024 * 1024 * 10;
        let value = utils::generate_random_vector(size);

        let value = value.clone();
        let key = shared_secretbox::gen_key();

        let client = random_client()?;
        let client2 = client.clone();
        let client3 = client.clone();

        let data = create(&client, &value, true, None).await?;
        let address = *data.address();
        client2.put_blob(data).await?;

        let res = get_value(&client3, address, None, None, Some(key)).await;
        assert!(res.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn create_and_retrieve_10mb_encrypted_put_retrieval_of_unencrypted(
    ) -> Result<(), CoreError> {
        let size = 1024 * 1024 * 10;
        let value = utils::generate_random_vector(size);

        let value = value.clone();
        let key = shared_secretbox::gen_key();

        let client = random_client()?;
        let client2 = client.clone();
        let client3 = client.clone();

        let data = create(&client, &value, true, Some(key)).await?;
        let address = *data.address();
        client2.put_blob(data).await?;

        let res = get_value(&client3, address, None, None, None).await;
        assert!(res.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn create_and_retrieve_10mb_encrypted_put_pub_retrieval_of_unpub() -> Result<(), CoreError>
    {
        let size = 1024 * 1024 * 10;
        let value = utils::generate_random_vector(size);

        let client = random_client()?;
        let client2 = client.clone();
        let client3 = client.clone();

        let data = create(&client, &value, true, None).await?;
        let data_name = *data.name();
        client2.put_blob(data).await?;

        let address = BlobAddress::Private(data_name);
        let res = get_value(&client3, address, None, None, None).await;
        assert!(res.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn create_and_retrieve_10mb_encrypted_put_unpub_retrieval_of_pub() -> Result<(), CoreError>
    {
        let size = 1024 * 1024 * 10;

        let value = utils::generate_random_vector(size);

        let client = random_client()?;
        let client2 = client.clone();
        let client3 = client.clone();

        let data = create(&client, &value, false, None).await?;
        let data_name = *data.name();
        client2.put_blob(data).await?;

        let address = BlobAddress::Public(data_name);
        let res = get_value(&client3, address, None, None, None).await;
        assert!(res.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn create_and_retrieve_index_based() -> Result<(), CoreError> {
        create_and_index_based_retrieve(1024).await
    }
    async fn create_and_index_based_retrieve(size: usize) -> Result<(), CoreError> {
        let value = utils::generate_random_vector(size);
        {
            // Read first half
            let client = random_client()?;
            let client2 = client.clone();
            let client3 = client.clone();

            let data = create(&client, &value, true, None).await?;
            let address = *data.address();
            client2.put_blob(data).await?;

            let fetched_value =
                get_value(&client3, address, None, Some(size as u64 / 2), None).await?;
            assert_eq!(fetched_value, value[0..size / 2].to_vec());
        }

        let value2 = utils::generate_random_vector(size);
        {
            // Read Second half
            let client = random_client()?;

            let client2 = client.clone();
            let client3 = client.clone();

            let data = create(&client, &value2, true, None).await?;
            let address = *data.address();
            client2.put_blob(data).await?;

            let fetched_value = get_value(
                &client3,
                address,
                Some(size as u64 / 2),
                Some(size as u64 / 2),
                None,
            )
            .await?;
            assert_eq!(fetched_value, value2[size / 2..size].to_vec());
        }

        Ok(())
    }

    #[allow(clippy::match_wild_err_arm)]
    async fn gen_data_then_map_create_and_retrieve(
        size: usize,
        published: bool,
        key: Option<shared_secretbox::Key>,
    ) -> Result<(), CoreError> {
        let value = utils::generate_random_vector(size);
        let value_before = value.clone();
        let value_before2 = value.clone();

        let client = random_client()?;
        let client2 = client.clone();
        let client3 = client.clone();
        let client4 = client.clone();
        let client5 = client.clone();

        let key2 = key.clone();
        let key3 = key.clone();

        // gen address without putting to the network (published and unencrypted)
        let data = gen_data_map(&client, &value.clone(), published, key2.clone()).await?;
        let address_before = *data.address();

        // attempt to retrieve it with generated address (it should error)
        let res = get_value(&client2, address_before, None, None, key2.clone()).await;
        let data_map_before = match res {
            Err(CoreError::DataError(SndError::NoSuchData)) => {
                // let's put it to the network (published and unencrypted)
                create(&client3, &value_before2.clone(), published, key3).await?
            }
            Ok(_) => panic!("Blob unexpectedly retrieved using address generated by gen_data_map"),
            Err(_) => panic!(
                "Unexpected error when Blob retrieved using address generated by gen_data_map"
            ),
        };

        let address_after = *data_map_before.address();
        if key2.is_none() {
            // the addresses generated without/with putting to the network should match
            assert_eq!(address_after, address_before);
        } else {
            // in this case the addresses generated without/with putting to the network
            // don't match since the encryption uses a random nonce
            // which changes the address of the chunks
            assert_ne!(address_after, address_before);
        }
        client4.put_blob(data_map_before).await?;

        let address = address_after;

        // now that it was put to the network we should be able to retrieve it
        let value_after = get_value(&client5, address, None, None, key).await?;

        // then the content should be what we put
        assert_eq!(value_after, value_before);

        // sleep
        std::thread::sleep(std::time::Duration::from_millis(5500));

        Ok(())
    }
}
