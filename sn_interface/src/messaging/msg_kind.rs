// Copyright 2022 MaidSafe.net limited.
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
/// There are two kinds of messages, messages from clients (apps, browser, cli clients, ...)
/// and messages from Network Nodes
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum MsgKind {
    #[cfg(any(feature = "chunks", feature = "registers"))]
    /// A data message, with the requesting peer's authority.
    /// Authority is needed to access private data, such as reading or writing a private file.
    Client(ClientAuth),
    /// A message from a Node along with its name
    Node(XorName),
}
