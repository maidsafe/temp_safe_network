// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod node_state;

use super::agreement::SectionAuth;
use serde::{Deserialize, Serialize};

pub use node_state::MembershipState;
pub use node_state::NodeState;

/// Information about members of our section.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Debug)]
pub struct SectionPeers {
    /// Info of members
    pub members: Vec<SectionAuth<NodeState>>,
}
