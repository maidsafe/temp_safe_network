// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::agreement::SectionAuth;
use crate::{messaging::SectionAuthorityProvider, types::PrefixMap};
use serde::{Deserialize, Serialize};

/// Container for storing information about other sections in the network.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkDto {
    /// Other sections: maps section prefixes to their latest signed section authority providers.
    pub sections: PrefixMap<SectionAuth<SectionAuthorityProvider>>,
}

impl NetworkDto {
    ///
    pub fn new() -> Self {
        Self {
            sections: PrefixMap::new(),
        }
    }
}
