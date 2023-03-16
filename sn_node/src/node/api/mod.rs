// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

// Public API for applications implementing a SAFE node using sn_node lib/crate.

mod events;
mod node_starter;

pub use events::NodeEvent;
pub use node_starter::{new_node, NodeRef};

pub(crate) use self::events::NodeEventsChannel;
