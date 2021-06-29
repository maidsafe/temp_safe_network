// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

//! SAFE network data types.

mod chunk;
mod errors;
mod keys;
mod map;
/// Register data type
pub mod register;
mod section;
mod sequence;
mod token;
mod utils;

pub use chunk::{
    Address as ChunkAddress, Chunk, Kind as ChunkKind, PrivateChunk, PublicChunk,
    MAX_CHUNK_SIZE_IN_BYTES,
};
pub use errors::{Error, Result};
pub use keys::{
    BlsKeypairShare, Keypair, NodeKeypairs, OwnerType, PublicKey, SecretKey, Signature,
    SignatureShare, Signing,
};
pub use map::{
    Action as MapAction, Address as MapAddress, Entries as MapEntries,
    EntryAction as MapEntryAction, EntryActions as MapEntryActions, Kind as MapKind, Map,
    PermissionSet as MapPermissionSet, Value as MapValue, Values as MapValues,
};
pub use register::Address as RegisterAddress;
pub use section::SectionElders;
pub use sequence::{
    Action as SequenceAction, Address as SequenceAddress, Data as Sequence, DataOp as SequenceOp,
    Entries as SequenceEntries, Entry as SequenceEntry, Index as SequenceIndex,
    Kind as SequenceKind, Permissions as SequencePermissions, Policy as SequencePolicy,
    PrivatePermissions as SequencePrivatePermissions, PrivatePolicy as SequencePrivatePolicy,
    PrivateSeqData, PublicPermissions as SequencePublicPermissions,
    PublicPolicy as SequencePublicPolicy, PublicSeqData, User as SequenceUser,
};
pub use token::Token;

use register::Register;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use xor_name::XorName;

/// Object storing a data variant.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
pub enum Data {
    /// Chunk.
    Chunk(Chunk),
    /// Map.
    Map(Map),
    /// Sequence.
    Sequence(Sequence),
    /// Register.
    Register(Register),
}

/// Object storing an address of data on the network
#[derive(Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Debug, PartialOrd, Ord)]
pub enum DataAddress {
    /// Chunk Address
    Chunk(ChunkAddress),
    /// Map Address
    Map(MapAddress),
    /// Sequence Address
    Sequence(SequenceAddress),
    /// Register Address
    Register(RegisterAddress),
}

impl Data {
    /// Returns true if public.
    pub fn is_public(&self) -> bool {
        match *self {
            Self::Chunk(ref chunk) => chunk.is_public(),
            Self::Map(_) => false,
            Self::Sequence(ref sequence) => sequence.is_public(),
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

impl From<Map> for Data {
    fn from(data: Map) -> Self {
        Self::Map(data)
    }
}

impl From<Sequence> for Data {
    fn from(data: Sequence) -> Self {
        Self::Sequence(data)
    }
}

impl From<Register> for Data {
    fn from(data: Register) -> Self {
        Self::Register(data)
    }
}
