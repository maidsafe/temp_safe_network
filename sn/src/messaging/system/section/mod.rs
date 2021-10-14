// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod candidates;
mod node_state;
mod peer;

pub use candidates::ElderCandidates;
pub use node_state::MembershipState;
pub use node_state::NodeState;
pub use peer::Peer;

use crate::messaging::{system::agreement::SectionAuth, SectionAuthorityProvider};
use bls::PublicKey as BlsPublicKey;
use secured_linked_list::SecuredLinkedList;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use xor_name::XorName;

mod arc_rwlock_serde {
    use serde::de::Deserializer;
    use serde::ser::Serializer;
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;
    use tokio::sync::RwLock;

    pub(crate) fn serialize<S, T>(val: &Arc<RwLock<T>>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
    {
        T::serialize(&futures::executor::block_on(val.read()), s)
    }

    pub(crate) fn deserialize<'de, D, T>(d: D) -> Result<Arc<RwLock<T>>, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
    {
        Ok(Arc::new(RwLock::new(T::deserialize(d)?)))
    }
}

/// Container for storing information about a section.
#[derive(Clone, Debug, Serialize, Deserialize)]
/// All information about a section
pub struct Section {
    /// Network genesis key
    pub genesis_key: BlsPublicKey,
    /// The secured linked list of previous section keys
    #[serde(with = "arc_rwlock_serde")]
    pub chain: Arc<RwLock<SecuredLinkedList>>,
    /// Signed section authority
    #[serde(with = "arc_rwlock_serde")]
    pub section_auth: Arc<RwLock<SectionAuth<SectionAuthorityProvider>>>,
    /// Members of the section
    pub section_peers: SectionPeers,
}

/// Container for storing information about members of our section.
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct SectionPeers {
    /// Members of the section
    pub members: Arc<DashMap<XorName, SectionAuth<NodeState>>>,
}

impl Eq for SectionPeers {}

impl PartialEq for SectionPeers {
    fn eq(&self, _other: &Self) -> bool {
        // TODO: there must be a better way of doing this...
        let mut us: BTreeMap<XorName, SectionAuth<NodeState>> = BTreeMap::default();
        let mut them: BTreeMap<XorName, SectionAuth<NodeState>> = BTreeMap::default();

        for refmulti in self.members.iter() {
            let (key, value) = refmulti.pair();
            let _prev = us.insert(*key, value.clone());
        }

        for refmulti in self.members.iter() {
            let (key, value) = refmulti.pair();
            let _prev = them.insert(*key, value.clone());
        }

        us == them
    }
}
