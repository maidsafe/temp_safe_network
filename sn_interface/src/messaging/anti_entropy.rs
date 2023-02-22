// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::system::SectionDecisions;

use crate::network_knowledge::SectionTreeUpdate;

use qp2p::UsrMsgBytes;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, custom_debug::Debug, serde::Serialize, serde::Deserialize)]
pub enum AntiEntropyMsg {
    /// Probes the network by sending a message to a random or chosen dst triggering an AE flow.
    /// Sends the current section key of target section which we know
    /// This expects a response, even if we're up to date.
    Probe(bls::PublicKey),
    /// An update to our NetworkKnowledge.
    AntiEntropy {
        /// The kind of anti-entropy response.
        kind: AntiEntropyKind,
        /// The update containing the current `SectionAuthorityProvider`
        /// and the section chain truncated from the triggering msg's dst section_key or genesis_key
        /// if the the dst section_key is not a direct ancestor to our section_key
        section_tree_update: SectionTreeUpdate,
    },
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, custom_debug::Debug)]
pub enum AntiEntropyKind {
    /// This AE message is sent to a node or client when a message with outdated section
    /// information was received, attaching the bounced message so
    /// the node or client can resend it with up to date destination information.
    Retry {
        #[debug(skip)]
        bounced_msg: UsrMsgBytes,
    },
    /// This AE message is sent to a node or client when a message needs to be sent to a
    /// different and/or closest section, attaching the bounced message so the node or client
    /// can resend it to the correct section with up to date destination information.
    Redirect {
        #[debug(skip)]
        bounced_msg: UsrMsgBytes,
    },
    /// This AE message is sent to update a node or client when we notice they are behind
    Update { section_decisions: SectionDecisions },
}
