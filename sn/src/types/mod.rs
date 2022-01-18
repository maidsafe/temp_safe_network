// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

//! SAFE network data types.

/// Standardised log markers for various events
pub mod log_markers;
/// Register data type
pub mod register;
/// Encoding utils
pub mod utils;

mod address;
mod cache;
mod chunk;
mod errors;
mod keys;
mod token;

pub use address::{
    BytesAddress, ChunkAddress, DataAddress, RegisterAddress, ReplicatedDataAddress,
    SafeKeyAddress, Scope,
};
pub use cache::Cache;
pub use chunk::{Chunk, MAX_CHUNK_SIZE_IN_BYTES};
pub use errors::{convert_dt_error_to_error_message, Error, Result};
pub use keys::{
    keypair::{BlsKeypairShare, Encryption, Keypair, OwnerType, Signing},
    node_keypairs::NodeKeypairs,
    public_key::PublicKey,
    secret_key::SecretKey,
    signature::{Signature, SignatureShare},
};
pub use token::Token;

use crate::messaging::data::{RegisterCmd, ReplicatedRegisterLog};

use serde::{Deserialize, Serialize};
use xor_name::XorName;

///
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum ReplicatedData {
    /// A chunk of data.
    Chunk(Chunk),
    /// A single cmd for a register.
    RegisterWrite(RegisterCmd),
    /// An entire op log of a register.
    RegisterLog(ReplicatedRegisterLog),
}

impl ReplicatedData {
    pub(crate) fn name(&self) -> XorName {
        match self {
            Self::Chunk(chunk) => *chunk.name(),
            Self::RegisterLog(log) => *log.address.name(),
            Self::RegisterWrite(cmd) => *cmd.dst_address().name(),
        }
    }
}
