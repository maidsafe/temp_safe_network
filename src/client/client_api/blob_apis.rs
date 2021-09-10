// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{data::get_data_chunks, Client};
use crate::messaging::data::{DataCmd, DataQuery, QueryResponse};
use crate::types::{Chunk, ChunkAddress, Encryption};
use crate::{
    client::{client_api::data::SecretKey, utils::encryption, Error, Result},
    url::Scope,
};

use bincode::deserialize;
use bytes::Bytes;
use futures::future::join_all;
use itertools::Itertools;
use self_encryption::{self, ChunkKey, EncryptedChunk, SecretKey as BlobSecretKey};
use tokio::task;
use tracing::trace;
use xor_name::XorName;

struct HeadChunk {
    chunk: Chunk,
    address: BlobAddress,
}

/// Address of a Blob.
#[derive(
    Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize, Debug,
)]
pub enum BlobAddress {
    /// Private namespace.
    Private(XorName),
    /// Public namespace.
    Public(XorName),
}

impl BlobAddress {
    /// The xorname.
    pub fn name(&self) -> &XorName {
        match self {
            Self::Public(name) | Self::Private(name) => name,
        }
    }

    /// The namespace scope of the Blob
    pub fn scope(&self) -> Scope {
        if self.is_public() {
            Scope::Public
        } else {
            Scope::Private
        }
    }

    /// Returns true if public.
    pub fn is_public(self) -> bool {
        matches!(self, BlobAddress::Public(_))
    }

    /// Returns true if private.
    pub fn is_private(self) -> bool {
        !self.is_public()
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
    pub async fn read_blob(&self, address: BlobAddress) -> Result<Bytes>
    where
        Self: Sized,
    {
        let chunk = self.read_from_network(address.name(), false).await?;
        let secret_key = self.unpack_head_chunk(HeadChunk { chunk, address }).await?;
        self.read_all(secret_key).await
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
        address: BlobAddress,
        position: usize,
        length: usize,
    ) -> Result<Bytes>
    where
        Self: Sized,
    {
        trace!(
            "Reading {:?} bytes of blob at: {:?}, starting from position: {:?}",
            &length,
            &address,
            &position,
        );

        let chunk = self.read_from_network(address.name(), false).await?;
        let secret_key = self.unpack_head_chunk(HeadChunk { chunk, address }).await?;
        self.seek(secret_key, position, length).await
    }

    pub(crate) async fn read_from_network(
        &self,
        name: &XorName,
        allow_cache: bool,
    ) -> Result<Chunk> {
        trace!("Fetching chunk: {:?}", name);
        println!("<read_from_network>(1): -> {}", *name);

        let address = ChunkAddress(*name);

        if allow_cache {
            if let Some(chunk) = self.blob_cache.write().await.get(&address) {
                trace!("Chunk retrieved from cache: {:?}", address);
                println!("<read_from_network>(1): cache read.");
                return Ok(chunk.clone());
            }
        }

        println!("<read_from_network>(1): cache miss.");

        let res = self.send_query(DataQuery::GetChunk(address)).await?;

        let operation_id = res.operation_id;
        let chunk: Chunk = match res.response {
            QueryResponse::GetChunk(result) => {
                result.map_err(|err| Error::from((err, operation_id)))
            }
            _ => return Err(Error::ReceivedUnexpectedEvent),
        }?;

        if allow_cache {
            let _ = self.blob_cache.write().await.put(address, chunk.clone());
        }

        Ok(chunk)
    }

    /// Clear the client's blob cache
    pub async fn clear_blob_cache(&mut self) {
        self.blob_cache.write().await.clear()
    }

    /// Directly writes raw data to the network
    /// in the form of immutable self encrypted chunks,
    /// without any batching.
    pub async fn write_to_network(&self, data: Bytes, scope: Scope) -> Result<BlobAddress> {
        let owner = encryption(scope, self.public_key());
        let (head_address, all_chunks) = get_data_chunks(data, owner.as_ref())?;

        let tasks = all_chunks.into_iter().map(|chunk| {
            let writer = self.clone();
            task::spawn(async move { writer.send_cmd(DataCmd::StoreChunk(chunk)).await })
        });

        let _ = join_all(tasks)
            .await
            .into_iter()
            .flatten() // swallows errors
            .collect_vec();

        Ok(head_address)
    }

    // --------------------------------------------
    // ---------- Private helpers -----------------
    // --------------------------------------------

    // Gets and decrypts chunks from the network using nothing else but the secret key, then returns the raw data.
    async fn read_all(&self, secret_key: BlobSecretKey) -> Result<Bytes> {
        let encrypted_chunks = Self::get_chunks(self.clone(), secret_key.keys().into_iter()).await;
        self_encryption::decrypt_full_set(&secret_key, &encrypted_chunks)
            .map_err(Error::SelfEncryption)
    }

    // Gets a subset of chunks from the network, decrypts and
    // reads `len` bytes of the data starting at given `pos` of original file.
    async fn seek(&self, secret_key: BlobSecretKey, pos: usize, len: usize) -> Result<Bytes> {

        let info = self_encryption::seek_info(secret_key.file_size(), pos, len);

        let range = &info.index_range;
        let all_keys = secret_key.keys();

        println!("<seek>(1): secret_key.file_size={} pos={}, len={}, range.start={}, range.end={}", secret_key.file_size(), pos, len, range.start, range.end + 1);

        let encrypted_chunks = Self::get_chunks(
            self.clone(),
            (range.start..range.end + 1)
                .clone()
                .map(|i| all_keys[i].clone()),
        )
        .await;

        println!("<seek>(2): range.len={}, encrypted_chunks.len={}", range.len(), encrypted_chunks.len());

        if range.len() > encrypted_chunks.len() {
            return Err(Error::Generic(format!(
                "Missing chunks! Required {}, but we have {}.",
                range.len(),
                encrypted_chunks.len()
            )));
        }

        self_encryption::decrypt_range(&secret_key, &encrypted_chunks, info.relative_pos, len)
            .map_err(Error::SelfEncryption)
    }

    async fn get_chunks(
        reader: Client,
        keys: impl Iterator<Item = ChunkKey>,
    ) -> Vec<EncryptedChunk> { // <- TODO return Result here
        let tasks = keys.map(|key| {
            let reader = reader.clone();
            task::spawn(async move {
                println!("<get_chunks>(1): trying to read from network..");
                let chunk = reader.read_from_network(&key.dst_hash, true).await?;
                println!("<get_chunks>(2): {} bytes read from network!", chunk.value().len());
                Ok::<EncryptedChunk, Error>(EncryptedChunk {
                    index: key.index,
                    content: chunk.value().clone(),
                })
            })
        });

        // this swallowing of errors
        // is basically a compaction into a single
        // error, that will be raised above this level,
        // basically saying "didn't get all chunks"..
        join_all(tasks)
            .await
            .into_iter()
            .map(|e| {
                if let Err(err) = e {
                    println!("<get_chunks>(3): Error {:?}", err);
                    Result::Err(err)
                } else {e}
            })
            .flatten() // swallows errors <--- TODO spit out errors instead of swallowing them
            .map(|e| {
                if let Err(err) = e {
                    println!("<get_chunks>(4): Error2 {:?}", err);
                    Result::Err(err)
                } else {e}
            })
            .flatten() // swallows errors <--- TODO spit out errors instead of swallowing them
            .map(|a| {println!("<get_chunks>(5): got chunck of len: {:?}", a.content.len()); a})
            .collect_vec()
    }

    /// Extracts a blob secretkey from a head chunk.
    /// If the secretkey is not the first level mapping directly to the user's contents,
    /// the process repeats itself until it obtains the first level secretkey.
    async fn unpack_head_chunk(&self, chunk: HeadChunk) -> Result<BlobSecretKey> {
        let HeadChunk { mut chunk, address } = chunk;
        loop {
            let bytes = if address.is_public() {
                chunk.value().clone()
            } else {
                let owner = encryption(Scope::Private, self.public_key()).ok_or_else(|| {
                    Error::Generic("Could not get an encryption object.".to_string())
                })?;
                owner.decrypt(chunk.value().clone())?
            };

            match deserialize(&bytes)? {
                SecretKey::FirstLevel(secret_key) => {
                    return Ok(secret_key);
                }
                SecretKey::AdditionalLevel(secret_key) => {
                    let serialized_chunk = self.read_all(secret_key).await?;
                    chunk = deserialize(&serialized_chunk)?;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::client::utils::test_utils::{create_test_client, run_w_backoff_delayed};
    use crate::types::{utils::random_bytes, Keypair};
    use crate::url::Scope;
    use bytes::Bytes;
    use eyre::Result;
    use futures::future::join_all;
    use rand::rngs::OsRng;
    use tokio::time::Instant;

    const BLOB_TEST_QUERY_TIMEOUT: u64 = 60;
    const MIN_BLOB_SIZE: usize = self_encryption::MIN_ENCRYPTABLE_BYTES;

    #[test]
    fn deterministic_chunking() -> Result<()> {
        let keypair = Keypair::new_ed25519(&mut OsRng);
        let blob = random_bytes(MIN_BLOB_SIZE);

        use crate::client::client_api::data::get_data_chunks;
        use crate::client::utils::encryption;
        let owner = encryption(Scope::Private, keypair.public_key());
        let (first_address, mut first_chunks) = get_data_chunks(blob.clone(), owner.as_ref())?;

        first_chunks.sort();

        for _ in 0..100 {
            let owner = encryption(Scope::Private, keypair.public_key());
            let (head_address, mut all_chunks) = get_data_chunks(blob.clone(), owner.as_ref())?;
            assert_eq!(first_address, head_address);
            all_chunks.sort();
            assert_eq!(first_chunks, all_chunks);
        }

        Ok(())
    }

    // Test storing and getting public Blob.
    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "too heavy for CI"]
    async fn parallel_timings() -> Result<()> {
        let client = create_test_client(Some(BLOB_TEST_QUERY_TIMEOUT)).await?;

        let handles = (0..1000_usize)
            .map(|i| (i, client.clone()))
            .map(|(i, client)| {
                tokio::spawn(async move {
                    let blob = random_bytes(MIN_BLOB_SIZE);
                    let _ = client.write_to_network(blob, Scope::Public).await?;
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
            let value = random_bytes(MIN_BLOB_SIZE);
            let now = Instant::now();
            let _ = client.write_to_network(value, Scope::Public).await?;
            let elapsed = now.elapsed();
            println!("Iter: {}, in {} millis", i, elapsed.as_millis());
        }

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn seek_in_data() -> Result<()> {
        for i in 1..5 {
            let size = i * MIN_BLOB_SIZE;

            println!("<seek_in_data>(1): <- round={}", i);
            for divisor in 2..5 {
                let len = size / divisor;
                let data = random_bytes(size);

                println!("<seek_in_data>(2): ----- divisor={}, len={}, size={} -----", divisor, len, size);

                // Read first part
                let read_data_1 = {
                    let pos = 0;
                    println!("<seek_in_data>(3): pos={}", pos);
                    seek_data_chunk(data.clone(), pos, len).await?
                };

                // Read second part
                let read_data_2 = {
                    let pos = len;
                    println!("<seek_in_data>(4): pos={}", pos);
                    seek_data_chunk(data.clone(), pos, len).await?
                };

                // Join parts
                let read_data: Bytes = [read_data_1, read_data_2]
                    .iter()
                    .flat_map(|bytes| bytes.clone())
                    .collect();

                compare(data.slice(0..(2 * len)), read_data)?
            }
        }

        Ok(())
    }

    async fn seek_data_chunk(data: Bytes, pos: usize, len: usize) -> Result<Bytes> {
        let client = create_test_client(Some(BLOB_TEST_QUERY_TIMEOUT)).await?;

        let address = client.write_to_network(data.clone(), Scope::Public).await?;

        // the larger the file, the longer we have to wait before we start querying
        let delay = usize::max(1, len / 2_000_000);

        let read_data =
            run_w_backoff_delayed(|| client.read_blob_from(address, pos, len), 10, delay).await?;

        compare(data.slice(pos..(pos + len)), read_data.clone())?;

        Ok(read_data)
    }

    fn compare(original: Bytes, result: Bytes) -> Result<()> {
        assert_eq!(original.len(), result.len());

        for (counter, (a, b)) in original.into_iter().zip(result).enumerate() {
            if a != b {
                return Err(eyre::eyre!(format!("Not equal! Counter: {}", counter)));
            }
        }
        Ok(())
    }

    // Test storing and getting public Blob.
    #[tokio::test(flavor = "multi_thread")]
    async fn public_blob_test() -> Result<()> {
        let client = create_test_client(Some(BLOB_TEST_QUERY_TIMEOUT)).await?;
        // Generate blob
        let blob = random_bytes(MIN_BLOB_SIZE);
        // Store blob
        let public_address = client.write_to_network(blob.clone(), Scope::Public).await?;

        // the larger the file, the longer we have to wait before we start querying
        let delay = usize::max(1, blob.len() / 2_000_000);

        // Assert that the blob was written
        let read_data =
            run_w_backoff_delayed(|| client.read_blob(public_address), 10, delay).await?;
        compare(blob.clone(), read_data)?;

        // Test storing public chunk with the same value.
        // Should not conflict and return same address
        let address = client.write_to_network(blob, Scope::Public).await?;
        assert_eq!(address, public_address);

        Ok(())
    }

    // Test storing, and getting private chunk.
    #[tokio::test(flavor = "multi_thread")]
    async fn private_blob_test() -> Result<()> {
        let client = create_test_client(Some(BLOB_TEST_QUERY_TIMEOUT)).await?;

        let blob = random_bytes(MIN_BLOB_SIZE);

        // Store Blob
        let private_address = client
            .write_to_network(blob.clone(), Scope::Private)
            .await?;

        // the larger the file, the longer we have to wait before we start querying
        let delay = usize::max(1, blob.len() / 2_000_000);

        // Assert that the blob is stored.
        let read_data =
            run_w_backoff_delayed(|| client.read_blob(private_address), 10, delay).await?;
        compare(blob.clone(), read_data)?;

        // Test storing private chunk with the same value.
        // Should not conflict and return same address
        let address = client
            .write_to_network(blob.clone(), Scope::Private)
            .await?;
        assert_eq!(address, private_address);

        // Test storing public chunk with the same value. Should not conflict.
        let public_address = client.write_to_network(blob.clone(), Scope::Public).await?;

        // Assert that the public Blob is stored.
        let read_data =
            run_w_backoff_delayed(|| client.read_blob(public_address), 10, delay).await?;
        compare(blob, read_data)?;

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn create_and_retrieve_1mb_public() -> Result<()> {
        create_and_retrieve(1024 * 1024, Scope::Public).await
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn create_and_retrieve_1mb_private() -> Result<()> {
        create_and_retrieve(1024 * 1024, Scope::Private).await
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn create_and_retrieve_10mb_private() -> Result<()> {
        create_and_retrieve(10 * 1024 * 1024, Scope::Private).await
    }

    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "too heavy for CI"]
    async fn create_and_retrieve_20mb_private() -> Result<()> {
        create_and_retrieve(20 * 1024 * 1024, Scope::Private).await
    }

    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "too heavy for CI"]
    async fn create_and_retrieve_40mb_private() -> Result<()> {
        create_and_retrieve(40 * 1024 * 1024, Scope::Private).await
    }

    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "too heavy for CI"]
    async fn create_and_retrieve_80mb_private() -> Result<()> {
        create_and_retrieve(80 * 1024 * 1024, Scope::Private).await
    }

    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "too heavy for CI"]
    async fn create_and_retrieve_160mb_private() -> Result<()> {
        create_and_retrieve(160 * 1024 * 1024, Scope::Private).await
    }

    async fn create_and_retrieve(size: usize, scope: Scope) -> Result<()> {
        let blob = random_bytes(size);
        let client = create_test_client(Some(BLOB_TEST_QUERY_TIMEOUT)).await?;
        let address = client.write_to_network(blob.clone(), scope).await?;

        // the larger the file, the longer we have to wait before we start querying
        let delay = usize::max(1, size / 2_000_000);

        // now that it was written to the network we should be able to retrieve it
        let read_data = run_w_backoff_delayed(|| client.read_blob(address), 1, delay).await?;
        // then the content should be what we stored
        compare(blob, read_data)?;

        Ok(())
    }
}
