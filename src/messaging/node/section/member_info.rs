// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::Peer;
use serde::{Deserialize, Serialize};
use xor_name::XorName;

/// Information about a member of our section.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Debug)]
pub struct MemberInfo {
    pub peer: Peer,
    pub state: PeerState,
}

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Debug)]
pub enum PeerState {
    /// Node is active member of the section.
    Joined,
    /// Node went offline.
    Left,
    /// Node was relocated to a different section.
    Relocated(XorName),
}
