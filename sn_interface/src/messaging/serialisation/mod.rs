// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod wire_msg;
mod wire_msg_header;

pub use self::wire_msg::WireMsg;
#[cfg(feature = "traceroute")]
pub use self::wire_msg::{Entity, Traceroute};
use super::{AuthorityProof, NodeSig, SectionSig, SectionSigShare};

/// Authority of a `NodeMsg`.
/// Src of message and authority to send it. Authority is validated by the signature.
#[derive(Eq, PartialEq, Debug, Clone)]
pub enum NodeMsgAuthority {
    /// Authority of a single peer.
    Node(AuthorityProof<NodeSig>),
    /// Authority of a single peer that uses it's BLS Keyshare to sign the message.
    BlsShare(AuthorityProof<SectionSigShare>),
    /// Authority of a whole section.
    Section(AuthorityProof<SectionSig>),
}

impl NodeMsgAuthority {
    pub fn src_public_key(&self) -> bls::PublicKey {
        match self {
            Self::Node(node_auth) => node_auth.section_pk,
            Self::BlsShare(bls_share_auth) => bls_share_auth.public_key_set.public_key(),
            Self::Section(section_auth) => section_auth.public_key,
        }
    }
}
