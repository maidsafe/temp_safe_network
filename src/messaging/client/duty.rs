// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use serde::{Deserialize, Serialize};

///
#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum Duty {
    ///
    Adult(AdultDuties),
    ///
    Elder,
    ///
    Node(NodeDuties),
}

/// Duties of a Node.
#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum NodeDuties {
    /// Config of a node, such as reward wallet.
    NodeConfig,
}

/// Duties of an Adult.
#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum AdultDuties {
    /// Keeping and serving chunks.
    ChunkStorage,
    /// Replicating chunks.
    ChunkReplication,
}
