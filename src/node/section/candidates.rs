// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt::Debug, net::SocketAddr};
use xor_name::{Prefix, XorName};

/// The information about elder candidates in a DKG round.
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Serialize, Deserialize)]
pub struct ElderCandidates {
    /// The section's complete set of elders as a map from their name to their socket address.
    pub elders: BTreeMap<XorName, SocketAddr>,
    /// The section prefix. It matches all the members' names.
    pub prefix: Prefix,
}
