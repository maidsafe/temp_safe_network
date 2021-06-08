// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{plain_message::PlainMessage, section::MemberInfo, signed::Signed};
use crate::SectionAuthorityProvider;
use ed25519_dalek::{PublicKey, Signature};
use hex_fmt::HexFmt;
use secured_linked_list::SecuredLinkedList;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Borrow,
    collections::BTreeSet,
    fmt::{self, Debug, Formatter},
};
use threshold_crypto::PublicKey as BlsPublicKey;
use xor_name::{Prefix, XorName};

/// SHA3-256 hash digest.
pub type Digest256 = [u8; 32];

/// Unique identified of a DKG session.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct DkgKey {
    pub hash: Digest256,
    pub generation: u64,
}

impl Debug for DkgKey {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "DkgKey({:10}/{})", HexFmt(&self.hash), self.generation)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct DkgFailureSigned {
    pub public_key: PublicKey,
    pub signature: Signature,
}

#[derive(Default, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct DkgFailureSignedSet {
    pub signeds: Vec<DkgFailureSigned>,
    pub non_participants: BTreeSet<XorName>,
}

/// A value together with the signed that it was agreed on by the majority of the section elders.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct Proven<T: Serialize> {
    pub value: T,
    pub signed: Signed,
}

impl<T> Borrow<Prefix> for Proven<T>
where
    T: Borrow<Prefix> + Serialize,
{
    fn borrow(&self) -> &Prefix {
        self.value.borrow()
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub enum Proposal {
    // Proposal to add a node to oursection
    Online {
        member_info: MemberInfo,
        // Previous name if relocated.
        previous_name: Option<XorName>,
        // The key of the destination section that the joining node knows, if any.
        destination_key: Option<BlsPublicKey>,
    },

    // Proposal to remove a node from our section
    Offline(MemberInfo),

    // Proposal to update info about a section. This has two purposes:
    //
    // 1. To signal the completion of a DKG by the elder candidates to the current elders.
    //    This proposal is then signed by the newly generated section key.
    // 2. To update information about other section in the network. In this case the proposal is
    //    signed by an existing key from the chain.
    SectionInfo(SectionAuthorityProvider),

    // Proposal to change the elders (and possibly the prefix) of our section.
    // NOTE: the `SectionAuthorityProvider` is already signed with the new key. This proposal is only to signs the
    // new key with the current key. That way, when it aggregates, we obtain all the following
    // pieces of information at the same time:
    //   1. the new section authority provider
    //   2. the new key
    //   3. the signature of the new section authority provider using the new key
    //   4. the signature of the new key using the current key
    // Which we can use to update the section section authority provider and the section chain at
    // the same time as a single atomic operation without needing to cache anything.
    OurElders(Proven<SectionAuthorityProvider>),

    // Proposal to accumulate the message at the source (that is, our section) and then send it to
    // its destination.
    AccumulateAtSrc {
        message: Box<PlainMessage>,
        proof_chain: SecuredLinkedList,
    },

    // Proposal to change whether new nodes are allowed to join our section.
    JoinsAllowed(bool),
}
