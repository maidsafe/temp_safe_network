// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use bls::PublicKeySet;
use crdts::merkle_reg::Sha3Hash;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sn_consensus::Generation;
use std::{
    borrow::Borrow,
    collections::BTreeMap,
    fmt::{self, Debug, Display, Formatter},
    net::SocketAddr,
};
use tiny_keccak::{Hasher, Sha3};

use xor_name::{Prefix, XorName};

use crate::messaging::system::{NodeState, SectionSig};

// TODO: we need to maintain a list of nodes who have previously been members of this section (archived nodes)
//       currently, only the final members of the section are preserved on the SAP.

/// Details of section authority.
///
/// A new `SectionAuthorityProvider` is created whenever the elders change, due to an elder being
/// added or removed, or the section splitting or merging.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Serialize, Deserialize)]
pub struct SectionAuthorityProvider {
    /// The section prefix. It matches all the members' names.
    pub prefix: Prefix,
    /// Public key set of the section.
    pub public_key_set: PublicKeySet,
    /// The section's complete set of elders as a map from their name to their socket address.
    pub elders: BTreeMap<XorName, SocketAddr>,
    /// The section members at the time of this elder churn.
    pub members: BTreeMap<XorName, NodeState>,
    /// The membership generation this SAP was instantiated on
    pub membership_gen: Generation,
}

impl Borrow<Prefix> for SectionAuthorityProvider {
    fn borrow(&self) -> &Prefix {
        &self.prefix
    }
}

impl Display for SectionAuthorityProvider {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "sap len:{} generation:{} contains: {{{}}}/({:b})",
            self.elders.len(),
            self.membership_gen,
            self.elders.keys().format(", "),
            self.prefix,
        )
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct SectionInfo {
    pub key: bls::PublicKey,
    pub sig: bls::Signature,
}

impl Debug for SectionInfo {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        let bytes: Vec<u8> = self
            .key
            .to_bytes()
            .into_iter()
            .chain(self.sig.to_bytes().into_iter())
            .collect();
        let hex = hex::encode(bytes);
        let hex: String = hex.chars().into_iter().take(10).collect();
        write!(formatter, "SectionInfo({})", hex)
    }
}

impl Sha3Hash for SectionInfo {
    fn hash(&self, hasher: &mut Sha3) {
        hasher.update(&self.key.to_bytes());
        hasher.update(&self.sig.to_bytes());
    }
}

/// A Merkle DAG of BLS keys where every key is signed by its parent key, except the genesis one.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SectionsDAG {
    pub genesis_key: bls::PublicKey,
    // List of (parent_key, SectionInfo)
    pub sections: Vec<(bls::PublicKey, SectionInfo)>,
}

/// The update to our `NetworkKnowledge` containing the section's `SectionAuthorityProvider` signed
/// by the section and the proof chain to validate the it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SectionTreeUpdate {
    pub section_auth: SectionAuthorityProvider,
    pub section_signed: SectionSig,
    pub proof_chain: SectionsDAG,
}
