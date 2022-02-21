// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::types::NamedPeer;

use std::collections::BTreeMap;
use xor_name::{Prefix, XorName};

/// The information about elder candidates in a DKG round.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct ElderCandidates {
    prefix: Prefix,
    elders: BTreeMap<XorName, NamedPeer>,
}

impl ElderCandidates {
    pub(crate) fn new(prefix: Prefix, elders: impl IntoIterator<Item = NamedPeer>) -> Self {
        Self {
            prefix,
            elders: elders.into_iter().map(|peer| (peer.name(), peer)).collect(),
        }
    }

    pub(crate) fn prefix(&self) -> Prefix {
        self.prefix
    }

    pub(crate) fn elders(&self) -> impl Iterator<Item = &NamedPeer> + '_ {
        self.elders.values()
    }

    pub(crate) fn names(&self) -> impl Iterator<Item = XorName> + '_ {
        self.elders.keys().copied()
    }

    pub(crate) fn len(&self) -> usize {
        self.elders.len()
    }

    pub(crate) fn contains(&self, name: &XorName) -> bool {
        self.elders.contains_key(name)
    }
}
