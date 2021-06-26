// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! Relocation related messages.

use super::NodeMsg;
use crate::messaging::SectionSigned;
use bls::PublicKey as BlsPublicKey;
pub use ed25519_dalek::{Keypair, Signature, Verifier};
use serde::{Deserialize, Serialize};
use xor_name::XorName;

/// Details of a relocation: which node to relocate, where to relocate it to and what age it should
/// get once relocated.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct RelocateDetails {
    /// Public id of the node to relocate.
    pub pub_id: XorName,
    /// Relocation destination - the node will be relocated to a section whose prefix matches this
    /// name.
    pub dst: XorName,
    /// The BLS key of the destination section used by the relocated node to verify messages.
    pub dst_key: BlsPublicKey,
    /// The age the node will have post-relocation.
    pub age: u8,
}

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
/// Details of a node relocation and new signed name
pub struct RelocatePayload {
    /// Message whose content is Variant::Relocate
    pub details: NodeMsg,
    /// Section authority for the details
    pub section_signed: SectionSigned,
    /// The new name of the node signed using its old public_key, to prove the node identity.
    pub signature_of_new_name_with_old_key: Signature,
}

/// Relocate node of <name> to section <dst>
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct RelocatePromise {
    /// Xorname
    pub name: XorName,
    /// Relocation destination xorname
    pub dst: XorName,
}
