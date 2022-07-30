// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{BlsShareAuth, NodeAuth, ServiceAuth};
use serde::{Deserialize, Serialize};
use xor_name::XorName;

/// Source authority of a message.
///
/// Source of message and authority to send it. Authority is validated by the signature.
/// Messages do not need to sign this field as it is all verifiable (i.e. if the signature validates
/// against the public key and we know the public key then we are good. If the proof is not
/// recognised we can ask for a longer chain that can be recognised).
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum AuthKind {
    #[cfg(any(feature = "chunks", feature = "registers"))]
    /// A data message, with the requesting peer's authority.
    ///
    /// Authority is needed to access private data, such as reading or writing a private file.
    Service(ServiceAuth),

    /// A message from a Node with its own independent authority.
    ///
    /// Node authority is needed when nodes send messages directly to other nodes.
    // FIXME: is the above true? What does is the recieving node validating against?
    Node(NodeAuth),

    /// A message from an Elder node with its share of the section authority.
    ///
    /// Section share authority is needed for messages related to section administration, such as
    /// DKG and relocation.
    NodeBlsShare(BlsShareAuth),
}

impl AuthKind {
    /// The src location of the msg.
    pub fn src_name(&self) -> XorName {
        match self {
            Self::NodeBlsShare(auth) => auth.src_name,
            Self::Node(auth) => crate::types::PublicKey::Ed25519(auth.node_ed_pk).into(),
            #[cfg(any(feature = "chunks", feature = "registers"))]
            Self::Service(auth) => auth.public_key.into(),
        }
    }
}
