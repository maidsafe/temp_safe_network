// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod agreement;
mod anti_entropy;
mod client_msgs;
mod dkg;
mod handover;
mod join;
mod membership;
mod node_msgs;
mod proposal;
mod relocation;
mod serialize;
mod update_section;

use crate::node::{flow_ctrl::cmds::Cmd, Error, MyNode, Result};

use qp2p::SendStream;
use sn_interface::{
    messaging::{MsgType, WireMsg},
    types::Peer,
};

use std::{collections::BTreeSet, sync::Arc};
use tokio::sync::{Mutex, RwLock};

#[derive(Debug, Clone)]
pub(crate) enum Peers {
    Single(Peer),
    Multiple(BTreeSet<Peer>),
}

impl Peers {
    #[allow(unused)]
    pub(crate) fn get(&self) -> BTreeSet<Peer> {
        match self {
            Self::Single(peer) => BTreeSet::from([*peer]),
            Self::Multiple(peers) => peers.clone(),
        }
    }
}

// Message handling
impl MyNode {
    #[instrument(skip(node))]
    pub(crate) async fn handle_msg(
        node: Arc<RwLock<MyNode>>,
        origin: Peer,
        wire_msg: WireMsg,
        send_stream: Option<Arc<Mutex<SendStream>>>,
    ) -> Result<Vec<Cmd>> {
        // Deserialize the payload of the incoming message
        let msg_id = wire_msg.msg_id();
        trace!("Handling msg {msg_id:?}. Validating first...");

        let msg_type = match wire_msg.into_msg() {
            Ok(msg_type) => msg_type,
            Err(error) => {
                error!("Failed to deserialize message payload ({msg_id:?}): {error:?}");
                return Ok(vec![]);
            }
        };

        let context = node.read().await.context();
        trace!("[NODE READ]: Handle msg lock got");
        match msg_type {
            MsgType::Node { msg_id, dst, msg } => {
                // Check for entropy before we proceed further
                // Anything returned here means there's an issue and we should
                // short-circuit below
                let ae_cmds = MyNode::check_ae_on_node_msg(
                    &context,
                    &origin,
                    &msg,
                    &wire_msg,
                    &dst,
                    send_stream.clone(),
                )
                .await?;

                if !ae_cmds.is_empty() {
                    // short circuit and send those AE responses
                    return Ok(ae_cmds);
                }

                MyNode::handle_valid_node_msg(node, context, msg_id, msg, origin, send_stream).await
            }
            MsgType::Client {
                msg_id,
                msg,
                dst,
                auth,
            } => {
                debug!("Valid client msg {msg_id:?}");

                let Some(send_stream) = send_stream else {
                    return Err(Error::NoClientResponseStream)
                };

                // Check for entropy before we proceed further, if AE response was sent
                // to the client we should just short-circuit
                if MyNode::is_ae_sent_to_client(
                    &context,
                    &origin,
                    &wire_msg,
                    &dst,
                    send_stream.clone(),
                )
                .await?
                {
                    return Ok(vec![]);
                }

                trace!("{msg_id:?} No AE needed for client message, proceeding to handle msg");
                MyNode::handle_valid_client_msg(context, msg_id, msg, auth, origin, send_stream)
                    .await
            }
            other @ MsgType::ClientDataResponse { .. } => {
                error!(
                    "Client data response {msg_id:?}, from {}, has been dropped since it's not \
                    meant to be handled by a node: {other:?}",
                    origin.addr()
                );
                Ok(vec![])
            }
            other @ MsgType::NodeDataResponse { .. } => {
                error!(
                    "Node data response {msg_id:?}, from {}, has been dropped since it's not \
                    meant to be handled this way (it is directly forwarded to client): {other:?}",
                    origin.addr()
                );
                Ok(vec![])
            }
        }
    }
}
