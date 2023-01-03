// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::ClientAuth;
use serde::{Deserialize, Serialize};
use xor_name::XorName;

/// Message Kind
///
/// There are three kinds of messages, messages from clients (apps, browser, cli clients, ...),
/// response to clients from Network Nodes, and messages among Network Nodes
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum MsgKind {
    /// A data message, with the requesting peer's authority.
    /// Authority is needed to access private data, such as reading or writing a private file.
    Client(ClientAuth),
    /// A data response sent from a Node (along with its name) to the client
    ClientDataResponse(XorName),
    /// A message from a Node along with its name
    Node(XorName),
    /// A message from a Node waiting to join along with its name
    NodeJoin(XorName),
    /// A data response sent from an Adult (along with its name) to Elders
    NodeDataResponse(XorName),
}
