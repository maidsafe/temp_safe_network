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
    /// An update to our NetworkKnowledge.
    AntiEntropy(XorName),
    /// A data message, with the requesting peer's authority.
    /// Authority is needed to access private data, such as reading or writing a private file.
    /// is spend tells us if we're dealing with a spend cmd
    /// query index lets us forward the msg to a given index in xorspace
    // TODO: Should query index be a part of Dst?
    Client {
        auth: ClientAuth,
        is_spend: bool,
        query_index: Option<usize>,
    },
    /// A message from a Node along with its name
    Node { name: XorName, is_join: bool },
    /// A data cmd/query response sent from a Node (along with its name).
    DataResponse(XorName),
}

impl MsgKind {
    /// is this an ae msg
    pub fn is_ae_msg(&self) -> bool {
        matches!(self, MsgKind::AntiEntropy(_))
    }
    /// is a client spend cmd
    pub fn is_client_spend(&self) -> bool {
        match self {
            Self::Client { is_spend, .. } => *is_spend,
            _ => false,
        }
    }
    /// return query index
    pub fn query_index(&self) -> &Option<usize> {
        match self {
            Self::Client { query_index, .. } => query_index,
            _ => &None,
        }
    }
}
