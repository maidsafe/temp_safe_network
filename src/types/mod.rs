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

mod cache;
mod chunk;
mod errors;
mod keys;
mod token;

pub use cache::Cache;
pub use chunk::{Address as ChunkAddress, Chunk, MAX_CHUNK_SIZE_IN_BYTES};
pub use errors::{convert_dt_error_to_error_message, Error, Result};
pub use keys::{
    keypair::{BlsKeypairShare, Encryption, Keypair, OwnerType, Signing},
    node_keypairs::NodeKeypairs,
    public_key::PublicKey,
    secret_key::SecretKey,
    signature::{Signature, SignatureShare},
};
pub use register::Address as RegisterAddress;
pub use token::Token;

use serde::{Deserialize, Serialize};
use xor_name::XorName;

/// Object storing an address of data on the network
#[derive(Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Debug, PartialOrd, Ord)]
pub enum DataAddress {
    /// Chunk Address
    Chunk(ChunkAddress),
    /// Register Address
    Register(RegisterAddress),
}
