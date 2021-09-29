// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::data::{to_chunk, Blob, BlobAddress, Spot, SpotAddress};
use super::{data::encrypt_blob, Client};
use crate::messaging::data::{DataCmd, DataQuery, QueryResponse};
use crate::types::{Chunk, ChunkAddress, Encryption};
use crate::{
    client::{client_api::data::DataMapLevel, utils::encryption, Error, Result},
    url::Scope,
};

use bincode::deserialize;
use bytes::Bytes;
use futures::future::join_all;
use itertools::Itertools;
use self_encryption::{self, ChunkInfo, DataMap, EncryptedChunk};
use tokio::task;
use tracing::trace;
use xor_name::XorName;

struct HeadChunk {
    chunk: Chunk,
    address: BlobAddress,
}

impl Client {
    /// Read the contents of a blob from the network. The contents are spread across
    /// multiple chunks in the network. This function invokes the self-encryptor and returns
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
        let chunk = self.read_from_network(address.name()).await?;
        let data_map = self.unpack_head_chunk(HeadChunk { chunk, address }).await?;
        self.read_all(data_map).await
    }

    /// Reads a spot from the network. The contents are contained within a single chunk.
    pub async fn read_spot(&self, address: SpotAddress) -> Result<Bytes> {
        let chunk = self.read_from_network(address.name()).await?;
        self.get_bytes(chunk, address.scope())
    }

    /// Read the contents of a blob from the network. The contents are spread across
    /// multiple chunks in the network. This function invokes the self-encryptor and returns
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

        let chunk = self.read_from_network(address.name()).await?;
        let data_map = self.unpack_head_chunk(HeadChunk { chunk, address }).await?;
        self.seek(data_map, position, length).await
    }

    pub(crate) async fn read_from_network(&self, name: &XorName) -> Result<Chunk> {
        trace!("Fetching chunk: {:?}", name);

        let res = self
            .send_query(DataQuery::GetChunk(ChunkAddress(*name)))
            .await?;

        let operation_id = res.operation_id;
        let chunk: Chunk = match res.response {
            QueryResponse::GetChunk(result) => {
                result.map_err(|err| Error::from((err, operation_id)))
            }
            _ => return Err(Error::ReceivedUnexpectedEvent),
        }?;

        Ok(chunk)
    }

    /// Encrypts a binary large object (blob) and returns the resulting address and all chunks.
    /// Does not store anything to the network.
    pub fn encrypt_blob(&self, blob: Blob, scope: Scope) -> Result<(BlobAddress, Vec<Chunk>)> {
        let owner = encryption(scope, self.public_key());
        encrypt_blob(blob.bytes(), owner.as_ref())
    }

    /// Packages a small piece of t(d)ata (spot) and returns the resulting address and the chunk.
    /// The chunk content will be in plain text if it has public scope, or encrypted if it is instead private.
    /// Does not store anything to the network.
    pub fn package_spot(&self, spot: Spot, scope: Scope) -> Result<(SpotAddress, Chunk)> {
        let encryption = encryption(scope, self.public_key());
        let chunk = to_chunk(spot.bytes(), encryption.as_ref())?;
        if chunk.value().len() >= self_encryption::MIN_ENCRYPTABLE_BYTES {
            return Err(Error::Generic("You might need to pad the spot contents and then store it as a `Blob`, as the encryption has made it slightly too big".to_string()));
        }
        let name = *chunk.name();
        let address = if encryption.is_some() {
            SpotAddress::Private(name)
        } else {
            SpotAddress::Public(name)
        };
        Ok((address, chunk))
    }

    /// Directly writes a [`Spot`] to the network in the
    /// form of a single chunk, without any batching.
    pub async fn write_spot_to_network(&self, spot: Spot, scope: Scope) -> Result<SpotAddress> {
        let (address, chunk) = self.package_spot(spot, scope)?;
        self.send_cmd(DataCmd::StoreChunk(chunk)).await?;
        Ok(address)
    }

    /// Directly writes a [`Blob`] to the network in the
    /// form of immutable self encrypted chunks, without any batching.
    pub async fn write_blob_to_network(&self, blob: Blob, scope: Scope) -> Result<BlobAddress> {
        let (head_address, all_chunks) = self.encrypt_blob(blob, scope)?;

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

    // Gets and decrypts chunks from the network using nothing else but the data map, then returns the raw data.
    async fn read_all(&self, data_map: DataMap) -> Result<Bytes> {
        let encrypted_chunks = Self::try_get_chunks(self.clone(), data_map.infos()).await?;
        self_encryption::decrypt_full_set(&data_map, &encrypted_chunks)
            .map_err(Error::SelfEncryption)
    }

    // Gets a subset of chunks from the network, decrypts and
    // reads `len` bytes of the data starting at given `pos` of original file.
    async fn seek(&self, data_map: DataMap, pos: usize, len: usize) -> Result<Bytes> {
        let info = self_encryption::seek_info(data_map.file_size(), pos, len);
        let range = &info.index_range;
        let all_infos = data_map.infos();

        let encrypted_chunks = Self::try_get_chunks(
            self.clone(),
            (range.start..range.end + 1)
                .clone()
                .map(|i| all_infos[i].clone())
                .collect_vec(),
        )
        .await?;

        self_encryption::decrypt_range(&data_map, &encrypted_chunks, info.relative_pos, len)
            .map_err(Error::SelfEncryption)
    }

    async fn try_get_chunks(reader: Client, keys: Vec<ChunkInfo>) -> Result<Vec<EncryptedChunk>> {
        let expected_count = keys.len();

        let tasks = keys.into_iter().map(|key| {
            let reader = reader.clone();
            task::spawn(async move {
                match reader.read_from_network(&key.dst_hash).await {
                    Ok(chunk) => Some(EncryptedChunk {
                        index: key.index,
                        content: chunk.value().clone(),
                    }),
                    Err(e) => {
                        warn!(
                            "Reading chunk {} from network, resulted in error {}.",
                            &key.dst_hash, e
                        );
                        None
                    }
                }
            })
        });

        // This swallowing of errors
        // is basically a compaction into a single
        // error saying "didn't get all chunks".
        let encrypted_chunks = join_all(tasks)
            .await
            .into_iter()
            .flatten()
            .flatten()
            .collect_vec();

        if expected_count > encrypted_chunks.len() {
            Err(Error::NotEnoughChunks(
                expected_count,
                encrypted_chunks.len(),
            ))
        } else {
            Ok(encrypted_chunks)
        }
    }

    /// Extracts a blob DataMapLevel from a head chunk.
    /// If the DataMapLevel is not the first level mapping directly to the user's contents,
    /// the process repeats itself until it obtains the first level DataMapLevel.
    async fn unpack_head_chunk(&self, chunk: HeadChunk) -> Result<DataMap> {
        let HeadChunk { mut chunk, address } = chunk;
        loop {
            let bytes = self.get_bytes(chunk, address.scope())?;

            match deserialize(&bytes)? {
                DataMapLevel::First(data_map) => {
                    return Ok(data_map);
                }
                DataMapLevel::Additional(data_map) => {
                    let serialized_chunk = self.read_all(data_map).await?;
                    chunk = deserialize(&serialized_chunk)?;
                }
            }
        }
    }

    /// If scope == Scope::Private, decrypts contents with the client encryption keys.
    /// Else returns the content bytes.
    fn get_bytes(&self, chunk: Chunk, scope: Scope) -> Result<Bytes> {
        if matches!(scope, Scope::Public) {
            Ok(chunk.value().clone())
        } else {
            let owner = encryption(scope, self.public_key())
                .ok_or_else(|| Error::Generic("Could not get an encryption object.".to_string()))?;
            Ok(owner.decrypt(chunk.value().clone())?)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::client::client_api::blob_apis::Blob;
    use crate::client::utils::test_utils::create_test_client;
    use crate::types::{utils::random_bytes, Keypair};
    use crate::url::Scope;
    use bytes::Bytes;
    use eyre::Result;
    use futures::future::join_all;
    use rand::rngs::OsRng;
    use tokio::time::Instant;

    use super::Spot;

    const MIN_BLOB_SIZE: usize = self_encryption::MIN_ENCRYPTABLE_BYTES;
    const DELAY_DIVIDER: usize = 500_000;

    #[test]
    fn deterministic_chunking() -> Result<()> {
        let keypair = Keypair::new_ed25519(&mut OsRng);
        let blob = random_bytes(MIN_BLOB_SIZE);

        use crate::client::client_api::data::encrypt_blob;
        use crate::client::utils::encryption;
        let owner = encryption(Scope::Private, keypair.public_key());
        let (first_address, mut first_chunks) = encrypt_blob(blob.clone(), owner.as_ref())?;

        first_chunks.sort();

        for _ in 0..100 {
            let owner = encryption(Scope::Private, keypair.public_key());
            let (head_address, mut all_chunks) = encrypt_blob(blob.clone(), owner.as_ref())?;
            assert_eq!(first_address, head_address);
            all_chunks.sort();
            assert_eq!(first_chunks, all_chunks);
        }

        Ok(())
    }

    // Test storing and reading min size blob.
    #[tokio::test(flavor = "multi_thread")]
    async fn store_and_read_3kb() -> Result<()> {
        let client = create_test_client(None).await?;

        let blob = Blob::new(random_bytes(MIN_BLOB_SIZE))?;

        // Store private blob
        let private_address = client
            .write_blob_to_network(blob.clone(), Scope::Private)
            .await?;

        // the larger the file, the longer we have to wait before we start querying
        let delay = tokio::time::Duration::from_secs(usize::max(
            1,
            blob.bytes().len() / DELAY_DIVIDER,
        ) as u64);
        tokio::time::sleep(delay).await;

        // Assert that the blob is stored.
        let read_data = client.read_blob(private_address).await?;

        compare(blob.bytes(), read_data)?;

        // Test storing private blob with the same value.
        // Should not conflict and return same address
        let address = client
            .write_blob_to_network(blob.clone(), Scope::Private)
            .await?;
        assert_eq!(address, private_address);

        // Test storing public blob with the same value. Should not conflict.
        let public_address = client
            .write_blob_to_network(blob.clone(), Scope::Public)
            .await?;

        // Assert that the public blob is stored.
        let read_data = client.read_blob(public_address).await?;

        compare(blob.bytes(), read_data)?;

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn seek_in_data() -> Result<()> {
        for i in 1..5 {
            let size = i * MIN_BLOB_SIZE;

            for divisor in 2..5 {
                let len = size / divisor;
                let blob = Blob::new(random_bytes(size))?;

                // Read first part
                let read_data_1 = {
                    let pos = 0;
                    seek_for_test(blob.clone(), pos, len).await?
                };

                // Read second part
                let read_data_2 = {
                    let pos = len;
                    seek_for_test(blob.clone(), pos, len).await?
                };

                // Join parts
                let read_data: Bytes = [read_data_1, read_data_2]
                    .iter()
                    .flat_map(|bytes| bytes.clone())
                    .collect();

                compare(blob.bytes().slice(0..(2 * len)), read_data)?
            }
        }

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn store_and_read_1kb() -> Result<()> {
        store_and_read_spot(MIN_BLOB_SIZE / 3, Scope::Public).await?;
        store_and_read_spot(MIN_BLOB_SIZE / 3, Scope::Private).await
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn store_and_read_1mb() -> Result<()> {
        store_and_read_blob(1024 * 1024, Scope::Public).await?;
        store_and_read_blob(1024 * 1024, Scope::Private).await
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn store_and_read_10mb() -> Result<()> {
        store_and_read_blob(10 * 1024 * 1024, Scope::Private).await
    }

    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "too heavy for CI"]
    async fn store_and_read_20mb() -> Result<()> {
        store_and_read_blob(20 * 1024 * 1024, Scope::Private).await
    }

    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "too heavy for CI"]
    async fn store_and_read_40mb() -> Result<()> {
        store_and_read_blob(40 * 1024 * 1024, Scope::Private).await
    }

    // Essentially a load test, seeing how much parallel batting the nodes can take.
    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "too heavy for CI"]
    async fn parallel_timings() -> Result<()> {
        let client = create_test_client(None).await?;

        let handles = (0..1000_usize)
            .map(|i| (i, client.clone()))
            .map(|(i, client)| {
                tokio::spawn(async move {
                    let blob = Blob::new(random_bytes(MIN_BLOB_SIZE))?;
                    let _ = client.write_blob_to_network(blob, Scope::Public).await?;
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
        let client = create_test_client(None).await?;

        for i in 0..1000_usize {
            let blob = Blob::new(random_bytes(MIN_BLOB_SIZE))?;
            let now = Instant::now();
            let _ = client.write_blob_to_network(blob, Scope::Public).await?;
            let elapsed = now.elapsed();
            println!("Iter: {}, in {} millis", i, elapsed.as_millis());
        }

        Ok(())
    }

    async fn store_and_read_blob(size: usize, scope: Scope) -> Result<()> {
        let blob = Blob::new(random_bytes(size))?;
        let client = create_test_client(None).await?;
        let address = client.write_blob_to_network(blob.clone(), scope).await?;

        // the larger the file, the longer we have to wait before we start querying
        let delay = tokio::time::Duration::from_secs(usize::max(1, size / DELAY_DIVIDER) as u64);
        tokio::time::sleep(delay).await;

        // now that it was written to the network we should be able to retrieve it
        let read_data = client.read_blob(address).await?;
        // then the content should be what we stored
        compare(blob.bytes(), read_data)?;

        Ok(())
    }

    async fn store_and_read_spot(size: usize, scope: Scope) -> Result<()> {
        let spot = Spot::new(random_bytes(size))?;
        let client = create_test_client(None).await?;
        let address = client.write_spot_to_network(spot.clone(), scope).await?;

        // the larger the size, the longer we have to wait before we start querying
        let delay = tokio::time::Duration::from_secs(usize::max(1, size / DELAY_DIVIDER) as u64);
        tokio::time::sleep(delay).await;

        // now that it was written to the network we should be able to retrieve it
        let read_data = client.read_spot(address).await?;

        // then the content should be what we stored
        compare(spot.bytes(), read_data)?;

        Ok(())
    }

    async fn seek_for_test(blob: Blob, pos: usize, len: usize) -> Result<Bytes> {
        let client = create_test_client(None).await?;
        let address = client
            .write_blob_to_network(blob.clone(), Scope::Public)
            .await?;

        // the larger the file, the longer we have to wait before we start querying
        let delay = tokio::time::Duration::from_secs(usize::max(1, len / DELAY_DIVIDER) as u64);
        tokio::time::sleep(delay).await;

        let read_data = client.read_blob_from(address, pos, len).await?;

        compare(blob.bytes().slice(pos..(pos + len)), read_data.clone())?;

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
}
