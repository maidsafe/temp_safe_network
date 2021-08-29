// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use std::{collections::BTreeMap, iter::FromIterator, path::PathBuf};

use super::{
    blob_storage::{ChunkUploader, Uploader},
    data::{get_data_chunks, Batch},
    Client,
};
use crate::messaging::data::{DataQuery, QueryResponse};
use crate::types::{Chunk, ChunkAddress, Encryption};
use crate::{
    client::{client_api::data::SecretKeyLevel, utils::encryption, Error, Result},
    url::Scope,
};

use async_trait::async_trait;
use bincode::deserialize;
use bytes::Bytes;
use futures::future::join_all;
use itertools::Itertools;
use self_encryption::{self, ChunkKey, EncryptedChunk, SecretKey};
use tokio::task;
use tracing::trace;

#[derive(Clone)]
pub(crate) struct UploaderImpl {}

#[async_trait]
impl Uploader for UploaderImpl {
    async fn upload(&self, _bytes: &[u8]) -> Result<()> {
        todo!()
    }
}

impl Client {
    /// Read the contents of a blob from the network. The contents might be spread across
    /// different chunks in the network. This function invokes the self-encryptor and returns
    /// the data that was initially stored.
    ///
    /// # Examples
    ///
    /// TODO: update once data types are crdt compliant
    ///
    pub async fn read_blob(&self, head_address: ChunkAddress) -> Result<Bytes>
    where
        Self: Sized,
    {
        trace!("Fetch head chunk of blob at: {:?}", &head_address,);

        let chunk = self.fetch_chunk_from_network(head_address, false).await?;
        let public = head_address.is_public();
        let secret_key = self.unpack(chunk).await?;

        let raw_data = self.read_all(secret_key, public).await?;

        Ok(raw_data)
    }

    /// Read the contents of a blob from the network. The contents might be spread across
    /// different chunks in the network. This function invokes the self-encryptor and returns
    /// the data that was initially stored.
    ///
    /// Takes `position` and `len` arguments which specify the start position
    /// and the length of bytes to be read. Passing `0` to position reads the data from the beginning.
    /// Passing `None` to length reads the full length of the data.
    ///
    /// # Examples
    ///
    /// TODO: update once data types are crdt compliant
    ///
    pub async fn read_blob_from(
        &self,
        head_address: ChunkAddress,
        position: usize,
        length: usize,
    ) -> Result<Bytes>
    where
        Self: Sized,
    {
        trace!(
            "Fetch head chunk of blob at: {:?} Position: {:?} Length: {:?}",
            &head_address,
            &position,
            &length
        );

        let chunk = self.fetch_chunk_from_network(head_address, false).await?;
        let public = head_address.is_public();
        let secret_key = self.unpack(chunk).await?;

        let raw_data = self.seek(secret_key, public, position, length).await?;

        Ok(raw_data)
    }

    pub(crate) async fn fetch_chunk_from_network(
        &self,
        head_address: ChunkAddress,
        allow_cache: bool,
    ) -> Result<Chunk> {
        if allow_cache {
            if let Some(chunk) = self.blob_cache.write().await.get(&head_address) {
                trace!("Blob chunk retrieved from cache: {:?}", head_address);
                return Ok(chunk.clone());
            }
        }

        let res = self.send_query(DataQuery::GetChunk(head_address)).await?;

        let operation_id = res.operation_id;
        let chunk: Chunk = match res.response {
            QueryResponse::GetChunk(result) => {
                result.map_err(|err| Error::from((err, operation_id)))
            }
            _ => return Err(Error::ReceivedUnexpectedEvent),
        }?;

        if allow_cache {
            let _ = self
                .blob_cache
                .write()
                .await
                .put(head_address, chunk.clone());
        }

        Ok(chunk)
    }

    /// Clear the client's blob cache
    pub async fn clear_blob_cache(&mut self) {
        self.blob_cache.write().await.clear()
    }

    /// Writes raw data to the network
    /// in the form of immutable self encrypted chunks.
    pub async fn write_to_network(&self, data: Bytes, scope: Scope) -> Result<ChunkAddress> {
        let owner = encryption(scope, self.public_key());
        let (head_address, all_chunks) = get_data_chunks(data, owner.as_ref())?;

        let uploader = ChunkUploader::new(UploaderImpl {});
        let _ = uploader.store(all_chunks).await?;

        Ok(head_address)
    }

    ///
    pub fn push_file_to_batch(&self, id: String, path: PathBuf, scope: Scope) {
        self.push_files_to_batch(BTreeMap::from_iter(vec![(id, (path, scope))]))
    }

    ///
    pub fn push_value_to_batch(&self, id: String, value: Bytes, scope: Scope) {
        self.push_values_to_batch(BTreeMap::from_iter(vec![(id, (value, scope))]))
    }

    ///
    pub fn push_files_to_batch(&self, files: BTreeMap<String, (PathBuf, Scope)>) {
        self.push_batch(Batch {
            files,
            ..Default::default()
        })
    }

    ///
    pub fn push_values_to_batch(&self, values: BTreeMap<String, (Bytes, Scope)>) {
        self.push_batch(Batch {
            values,
            ..Default::default()
        })
    }

    // --------------------------------------------
    // ---------- Private helpers -----------------
    // --------------------------------------------

    // Gets and decrypts chunks from the network using nothing else but the secret key, then returns the raw data.
    async fn read_all(&self, secret_key: SecretKey, public: bool) -> Result<Bytes> {
        let encrypted_chunks =
            Self::get_chunks(self.clone(), public, secret_key.keys().into_iter()).await;
        self_encryption::decrypt_full_set(&secret_key, &encrypted_chunks)
            .map_err(Error::SelfEncryption)
    }

    // Gets a subset of chunks from the network, decrypts and
    // reads `len` bytes of the data starting at given `pos` of original file.
    async fn seek(
        &self,
        secret_key: SecretKey,
        public: bool,
        pos: usize,
        len: usize,
    ) -> Result<Bytes> {
        let info = self_encryption::seek_info(secret_key.file_size(), pos, len);

        let all_keys = secret_key.keys();
        let encrypted_chunks = Self::get_chunks(
            self.clone(),
            public,
            info.index_range.map(|i| all_keys[i].clone()),
        )
        .await;

        self_encryption::decrypt_range(&secret_key, &encrypted_chunks, info.relative_pos, len)
            .map_err(Error::SelfEncryption)
    }

    async fn get_chunks(
        reader: Client,
        public: bool,
        keys: impl Iterator<Item = ChunkKey>,
    ) -> Vec<EncryptedChunk> {
        let tasks = keys.map(|key| {
            let reader = reader.clone();
            let address = if public {
                ChunkAddress::Public(key.dst_hash)
            } else {
                ChunkAddress::Private(key.dst_hash)
            };
            task::spawn(async move {
                let chunk = reader.fetch_chunk_from_network(address, false).await?;
                Ok::<EncryptedChunk, Error>(EncryptedChunk {
                    index: key.index,
                    content: chunk.value().clone(),
                })
            })
        });

        join_all(tasks)
            .await
            .into_iter()
            .flatten() // swallows errors
            .flatten()// swallows errors
            .collect_vec()
    }

    /// Takes a chunk and fetches the data map from it.
    /// If the data map is not the root data map of the user's contents,
    /// the process repeats itself until it obtains the root data map.
    async fn unpack(&self, mut chunk: Chunk) -> Result<SecretKey> {
        loop {
            let (public, bytes) = if chunk.is_public() {
                (true, chunk.value().clone())
            } else {
                let owner = encryption(Scope::Private, self.public_key()).ok_or_else(|| {
                    Error::Generic("Could not get an encryption object.".to_string())
                })?;
                (false, owner.decrypt(chunk.value().clone())?)
            };

            match deserialize(&bytes)? {
                SecretKeyLevel::First(secret_key) => {
                    return Ok(secret_key);
                }
                SecretKeyLevel::Additional(secret_key) => {
                    let serialized_chunk = self.read_all(secret_key, public).await?;
                    chunk = deserialize(&serialized_chunk)?;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ChunkAddress;
    use crate::{
        client::utils::{
            random_bytes,
            test_utils::{create_test_client, run_w_backoff_delayed},
        },
        url::Scope,
    };
    use eyre::Result;
    use futures::future::join_all;
    use tokio::time::{Duration, Instant};

    const BLOB_TEST_QUERY_TIMEOUT: u64 = 60;

    // Test storing and getting public Blob.
    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "too heavy for CI"]
    async fn parallel_timings() -> Result<()> {
        let client = create_test_client(Some(BLOB_TEST_QUERY_TIMEOUT)).await?;

        let handles = (0..1000_usize)
            .map(|i| (i, client.clone()))
            .map(|(i, client)| {
                tokio::spawn(async move {
                    let value = random_bytes(1000);
                    let _ = client.write_to_network(value, Scope::Public).await?;
                    println!("Iter: {}", i);
                    let res: Result<()> = Ok(());
                    res
                })
            });

        let results = join_all(handles).await;

        for res1 in results {
            if let Ok(res2) = res1 {
                if res2.is_err() {
                    println!("Error: {:?}", res2);
                }
            } else {
                println!("Error: {:?}", res1);
            }
        }

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "too heavy for CI"]
    async fn one_by_one_timings() -> Result<()> {
        let client = create_test_client(Some(BLOB_TEST_QUERY_TIMEOUT)).await?;

        for i in 0..1000_usize {
            let value = random_bytes(1000);
            let now = Instant::now();
            let _ = client.write_to_network(value, Scope::Public).await?;
            let elapsed = now.elapsed();
            println!("Iter: {}, in {} millis", i, elapsed.as_millis());
        }

        Ok(())
    }

    // Test storing and getting public Blob.
    #[tokio::test(flavor = "multi_thread")]
    async fn public_blob_test() -> Result<()> {
        let client = create_test_client(Some(BLOB_TEST_QUERY_TIMEOUT)).await?;
        // Generate blob
        let blob = random_bytes(10);
        // Store blob
        let head_address = client.write_to_network(blob.clone(), Scope::Public).await?;
        // // check it's the expected Blob address
        // assert_eq!(expected_address, head_address);

        // Assert that the blob was written
        let fetched_data = run_w_backoff_delayed(|| client.read_blob(head_address), 10).await?;
        assert_eq!(blob, fetched_data);

        // Test storing public chunk with the same value.
        // Should not conflict and return same address
        let address = client.write_to_network(blob, Scope::Public).await?;
        assert_eq!(address, head_address);

        Ok(())
    }

    // Test storing, and getting private chunk.
    #[tokio::test(flavor = "multi_thread")]
    async fn private_blob_test() -> Result<()> {
        let client = create_test_client(Some(BLOB_TEST_QUERY_TIMEOUT)).await?;

        let blob = random_bytes(10);

        // Store Blob
        let private_address = client
            .write_to_network(blob.clone(), Scope::Private)
            .await?;

        // Assert that the blob is stored.
        let fetched_data = run_w_backoff_delayed(|| client.read_blob(private_address), 10).await?;
        assert_eq!(blob, fetched_data);

        // Test storing private chunk with the same value.
        // Should not conflict and return same address
        let address = client
            .write_to_network(blob.clone(), Scope::Private)
            .await?;
        assert_eq!(address, private_address);

        // Test storing public chunk with the same value. Should not conflict.
        let public_address = client.write_to_network(blob.clone(), Scope::Public).await?;

        // Assert that the public Blob is stored.
        let fetched_data = run_w_backoff_delayed(|| client.read_blob(public_address), 10).await?;
        assert_eq!(blob, fetched_data);

        Ok(())
    }

    // Test creating and retrieving a 1kb blob.
    #[tokio::test(flavor = "multi_thread")]
    async fn create_and_retrieve_1kb_pub_unencrypted() -> Result<()> {
        let size = 1024;
        gen_data_then_create_and_retrieve(size, true).await?;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn create_and_retrieve_1kb_private_unencrypted() -> Result<()> {
        let size = 1024;
        gen_data_then_create_and_retrieve(size, false).await?;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn create_and_retrieve_1kb_put_pub_retrieve_private() -> Result<()> {
        let size = 1024;
        let data = random_bytes(size);

        let mut client = create_test_client(Some(BLOB_TEST_QUERY_TIMEOUT)).await?;
        let address = client.write_to_network(data, Scope::Public).await?;

        // let's make sure the public chunk is stored
        let _ = run_w_backoff_delayed(|| client.read_blob(address), 10).await?;

        client.query_timeout = Duration::from_secs(5);
        // and now trying to read a private chunk with same address should fail
        let res = client
            .read_blob(ChunkAddress::Private(*address.name()))
            .await;

        assert!(res.is_err());

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn create_and_retrieve_1kb_put_private_retrieve_pub() -> Result<()> {
        let size = 1024;
        let data = random_bytes(size);

        let mut client = create_test_client(Some(BLOB_TEST_QUERY_TIMEOUT)).await?;

        let address = client.write_to_network(data, Scope::Private).await?;

        // let's make sure the private chunk is stored
        let _ = run_w_backoff_delayed(|| client.read_blob(address), 10).await?;

        client.query_timeout = Duration::from_secs(5);

        // and now trying to read a public chunk with same address should fail (timeout)
        let res = client
            .read_blob(ChunkAddress::Public(*address.name()))
            .await;
        assert!(res.is_err());

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn create_and_retrieve_1mb_public() -> Result<()> {
        let size = 1024 * 1024;
        gen_data_then_create_and_retrieve(size, true).await?;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn create_and_retrieve_1mb_private() -> Result<()> {
        let size = 1024 * 1024;
        gen_data_then_create_and_retrieve(size, false).await?;
        Ok(())
    }

    // ----------------------------------------------------------------
    // 10mb (ie. more than 1 chunk)
    // ----------------------------------------------------------------
    #[tokio::test(flavor = "multi_thread")]
    async fn create_and_retrieve_10mb_private() -> Result<()> {
        let size = 1024 * 1024 * 10;
        gen_data_then_create_and_retrieve(size, false).await?;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn create_and_retrieve_10mb_public() -> Result<()> {
        let size = 1024 * 1024 * 10;
        gen_data_then_create_and_retrieve(size, true).await?;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "too heavy for CI"]
    async fn create_and_retrieve_100mb_public() -> Result<()> {
        let size = 1024 * 1024 * 100;
        gen_data_then_create_and_retrieve(size, true).await?;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn seek_in_data() -> Result<()> {
        test_seek(1024).await
    }

    async fn test_seek(size: usize) -> Result<()> {
        // Test read first half
        let data = random_bytes(size);
        let client = create_test_client(Some(BLOB_TEST_QUERY_TIMEOUT)).await?;

        let address = client.write_to_network(data.clone(), Scope::Public).await?;

        let fetched_data =
            run_w_backoff_delayed(|| client.read_blob_from(address, 0, size / 2), 10).await?;
        assert_eq!(fetched_data, data[0..size / 2].to_vec());

        // Test read second half
        let data = random_bytes(size);
        let client = create_test_client(Some(BLOB_TEST_QUERY_TIMEOUT)).await?;

        let address = client.write_to_network(data.clone(), Scope::Public).await?;

        let fetched_data =
            run_w_backoff_delayed(|| client.read_blob_from(address, size / 2, size / 2), 10)
                .await?;
        assert_eq!(fetched_data, data[size / 2..size].to_vec());

        Ok(())
    }

    #[allow(clippy::match_wild_err_arm)]
    async fn gen_data_then_create_and_retrieve(size: usize, public: bool) -> Result<()> {
        let raw_data = random_bytes(size);

        let client = create_test_client(Some(BLOB_TEST_QUERY_TIMEOUT)).await?;

        let address = if public {
            client
                .write_to_network(raw_data.clone(), Scope::Public)
                .await?
        } else {
            client
                .write_to_network(raw_data.clone(), Scope::Private)
                .await?
        };

        // now that it was put to the network we should be able to retrieve it
        let fetched_data = run_w_backoff_delayed(|| client.read_blob(address), 10).await?;
        // then the content should be what we put
        assert_eq!(fetched_data, raw_data);

        // // now let's test Blob data map generation utility returns the correct chunk address
        // let privately_owned = if public {
        //     None
        // } else {
        //     Some(client.public_key())
        // };
        // let (_, head_chunk_address) = Client::encrypt_blob(raw_data, privately_owned).await?;
        // assert_eq!(head_chunk_address, address);

        Ok(())
    }
}
