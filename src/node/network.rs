// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{
    agreement::Proven, prefix_map::PrefixMap, section::SectionAuthorityProvider, signed::Signed,
};
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use xor_name::Prefix;

/// Container for storing information about other sections in the network.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Network {
    // Other sections: maps section prefixes to their latest signed section authority providers.
    pub sections: PrefixMap<OtherSection>,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct OtherSection {
    // If this is signed by our section, then `key_signed` is `None`. If this is signed by our
    // sibling section, then `key_signed` contains the proof of the signing key itself signed by our
    // section.
    pub section_auth: Proven<SectionAuthorityProvider>,
    pub key_signed: Option<Signed>,
}

impl Borrow<Prefix> for OtherSection {
    fn borrow(&self) -> &Prefix {
        &self.section_auth.value.prefix
    }
}
