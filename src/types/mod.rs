// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

//! SAFE network data types.

/// Register data type
pub mod register;
/// Encoding utils
pub mod utils;

mod cf;
mod chunk;
mod errors;
mod keys;
pub mod prefix_map;
mod token;

pub use cf::{map::CFMap, option::CFOption, set::CFSet, value::CFValue};
pub use chunk::{
    Address as ChunkAddress, Chunk, Kind as ChunkKind, PrivateChunk, PublicChunk,
    MAX_CHUNK_SIZE_IN_BYTES,
};
pub use errors::{convert_dt_error_to_error_message, Error, Result};
pub use keys::{
    keypair::{BlsKeypairShare, Keypair, OwnerType, Signing},
    node_keypairs::NodeKeypairs,
    public_key::PublicKey,
    secret_key::SecretKey,
    signature::{Signature, SignatureShare},
};
pub use prefix_map::PrefixMap;
pub use register::Address as RegisterAddress;
pub use token::Token;

use register::Register;
use serde::{Deserialize, Serialize};
use xor_name::XorName;

/// Object storing a data variant.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
pub enum Data {
    /// Chunk.
    Chunk(Chunk),
    /// Register.
    Register(Register),
}

/// Object storing an address of data on the network
#[derive(Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Debug, PartialOrd, Ord)]
pub enum DataAddress {
    /// Chunk Address
    Chunk(ChunkAddress),
    /// Register Address
    Register(RegisterAddress),
}

impl Data {
    /// Returns true if public.
    pub fn is_public(&self) -> bool {
        match *self {
            Self::Chunk(ref chunk) => chunk.is_public(),
            Self::Register(ref register) => register.is_public(),
        }
    }

    /// Returns true if private.
    pub fn is_private(&self) -> bool {
        !self.is_public()
    }
}

impl From<Chunk> for Data {
    fn from(chunk: Chunk) -> Self {
        Self::Chunk(chunk)
    }
}

impl From<Register> for Data {
    fn from(data: Register) -> Self {
        Self::Register(data)
    }
}
