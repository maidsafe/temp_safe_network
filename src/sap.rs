// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Borrow,
    collections::BTreeMap,
    fmt::{self, Debug, Display, Formatter},
    net::SocketAddr,
};
use threshold_crypto::PublicKeySet;
use xor_name::{Prefix, XorName};

/// A new `SectionAuthorityProvider` is created whenever the elders change,
/// due to an elder being added or removed, or the section splitting or merging.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Serialize, Deserialize)]
pub struct SectionAuthorityProvider {
    /// The section prefix. It matches all the members' names.
    pub prefix: Prefix,
    /// Public key set of the section.
    pub public_key_set: PublicKeySet,
    // The section's complete set of elders as a map from their name to their socket address.
    pub elders: BTreeMap<XorName, SocketAddr>,
}

impl Borrow<Prefix> for SectionAuthorityProvider {
    fn borrow(&self) -> &Prefix {
        &self.prefix
    }
}

impl Debug for SectionAuthorityProvider {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(
            formatter,
            "SectionAuthorityProvider {{ prefix: ({:b}), section_key: {:?}, elders: {{{:?}}} }}",
            self.prefix,
            self.public_key_set.public_key(),
            self.elders.iter().format(", "),
        )
    }
}

impl Display for SectionAuthorityProvider {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{{{}}}/({:b})",
            self.elders.keys().format(", "),
            self.prefix,
        )
    }
}
