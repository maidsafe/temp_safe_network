// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{core::NodeContext, CmdChannel, Error, MyNode, Peer, Result};

use sn_interface::{
    messaging::system::NodeMsg,
    network_knowledge::{SectionAuthorityProvider, SectionsDAG},
};

use ed25519_dalek::PublicKey;
use std::{collections::BTreeSet, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use xor_name::{Prefix, XorName};

use super::{flow_ctrl::cmds::Cmd, messaging::Peers};

/// Test interface for sending and receiving messages to and from other nodes.
///
/// A node is a part of the network that can route messages and be a member of a section or group
/// location. Its methods can be used to send requests and responses as either an individual
/// `Node` or as a part of a section or group location.
#[allow(missing_debug_implementations)]
pub struct NodeTestApi {
    node: Arc<RwLock<MyNode>>,
    cmd_channel: CmdChannel,
}

impl NodeTestApi {
    pub(crate) fn new(node: Arc<RwLock<MyNode>>, cmd_channel: CmdChannel) -> Self {
        Self { node, cmd_channel }
    }

    /// Send a message.
    /// Messages sent here, either section to section or node to node.
    async fn send_cmd(&self, cmd: Cmd) -> Result<()> {
        self.cmd_channel
            .send((cmd, vec![]))
            .await
            .map_err(|_| Error::CmdChannelSendError)?;

        Ok(())
    }
}
