// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::client::{Error, Result};
use crate::types::{Chunk, ChunkAddress as Address, Encryption, PrivateChunk, PublicChunk};
use bincode::serialize;
use bytes::Bytes;
use rayon::prelude::*;
use self_encryption::{ChunkKey, DataMap, EncryptedChunk};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub(crate) enum DataMapLevel {
    // Holds the data map that is returned after writing the chunks
    // of the source data to the network.
    Final(DataMap),
    // Holds the data map returned after a writing a
    // serialized chunk that holds an _additional_ level of data map
    // which is pointing to chunks resulting from chunking up a data map.
    Additional(DataMap),
}

#[allow(unused)]
pub(crate) fn get_file_chunks(
    path: &Path,
    encryption: Option<&impl Encryption>,
) -> Result<(Address, Vec<Chunk>)> {
    let chunks = encrypt_file(path);
    pack_root_map(chunks, encryption)
}

pub(crate) fn get_data_chunks(
    data: Bytes,
    encryption: Option<&impl Encryption>,
) -> Result<(Address, Vec<Chunk>)> {
    let chunks = encrypt_data(data);
    pack_root_map(chunks, encryption)
}

// Returns the topmost chunk through which the entire
// data tree can be accessed (i.e. the root chunk), and all the other encrypted chunks.
// The root chunk must be stored in a safe way, since it is the key to the entire file.
pub(crate) fn pack_root_map(
    encrypted_chunks: Vec<EncryptedChunk>,
    encryption: Option<&impl Encryption>,
) -> Result<(Address, Vec<Chunk>)> {
    // get data map
    let keys = encrypted_chunks.iter().map(|c| c.key.clone()).collect();

    // if root chunk content was too big, it will have been
    // chunked up into additional chunks, and now we have a new root chunk
    // which points to all of those (extra root) chunks.. and so on
    let (data_map, additional_chunks) = pack(keys, encryption)?;

    let all_chunks: Vec<_> = encrypted_chunks
        .par_iter()
        .map(|c| to_chunk(c.content.clone(), encryption))
        .flatten() // swallows errors!
        .chain(additional_chunks) // drops errors
        .collect();

    Ok((data_map, all_chunks))
}

fn encrypt_file(file: &Path) -> Vec<EncryptedChunk> {
    let bytes = Bytes::from(std::fs::read(file).unwrap());
    self_encryption::encrypt(bytes).unwrap()
}

fn encrypt_data(bytes: Bytes) -> Vec<EncryptedChunk> {
    self_encryption::encrypt(bytes).unwrap()
}

/// Takes the "Root data map" and returns a chunk that is acceptable by the network
///
/// If the root data map chunk is too big, it is self-encrypted and the resulting data map is put into a chunk.
/// The above step is repeated as many times as required until the chunk size is valid.
fn pack(
    keys: Vec<ChunkKey>,
    encryption: Option<&impl Encryption>,
) -> Result<(Address, Vec<Chunk>)> {
    let mut chunks = vec![];
    let mut chunk_content = Bytes::from(serialize(&DataMapLevel::Final(DataMap::Chunks(keys)))?);

    loop {
        let chunk = to_chunk(chunk_content, encryption)?;
        // If data map chunk is less that 1MB return it so it can be directly sent to the network
        if chunk.validate_size() {
            let address = *chunk.address();
            chunks.push(chunk);
            chunks.reverse();
            // returns the address of the last data map, and all the chunks produced
            return Ok((address, chunks));
        } else {
            let serialized_chunk = Bytes::from(serialize(&chunk)?);
            let data_map_chunks =
                self_encryption::encrypt(serialized_chunk).map_err(Error::SelfEncryption)?;
            let additional_chunks = data_map_chunks
                .iter()
                .map(|c| to_chunk(c.content.clone(), encryption))
                .flatten();
            chunks.extend(additional_chunks);
            let keys = data_map_chunks.into_iter().map(|c| c.key).collect();
            chunk_content =
                Bytes::from(serialize(&DataMapLevel::Additional(DataMap::Chunks(keys)))?);
        }
    }
}

fn to_chunk(chunk_content: Bytes, encryption: Option<&impl Encryption>) -> Result<Chunk> {
    let chunk: Chunk = if let Some(encryption) = encryption {
        // strictly, we do not need to encrypt this if it's not going to be the
        // last level, since it will then instead be self-encrypted.
        // But we can just as well do it, for now.. (which also lets us avoid some edge case handling).
        let encrypted_content = encryption.encrypt(chunk_content)?;
        PrivateChunk::new(encrypted_content, *encryption.public_key()).into()
    } else {
        PublicChunk::new(chunk_content).into()
    };

    Ok(chunk)
}

// /// Takes a chunk and fetches the data map from it.
// /// If the data map is not the root data map of the user's contents,
// /// the process repeats itself until it obtains the root data map.
// pub(crate) async fn unpack(&self, mut chunk: Chunk) -> Result<DataMap> {
//     loop {
//         let public = chunk.is_public();
//         match deserialize(chunk.value())? {
//             DataMapLevel::Root(data_map) => {
//                 return Ok(data_map);
//             }
//             DataMapLevel::Child(data_map) => {
//                 let serialized_chunk = self
//                     .read_all(data_map, public)
//                     .await?;
//                 chunk = deserialize(&serialized_chunk)?;
//             }
//         }
//     }
// }
