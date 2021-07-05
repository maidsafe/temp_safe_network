// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::Core;
use crate::messaging::{
    client::ClientMsg, node::NodeMsg, ClientSigned, DstLocation, EndUser, MessageId, WireMsg,
};
use crate::routing::{
    error::Result,
    messages::WireMsgUtils,
    routing_api::{command::Command, Event},
    section::{SectionAuthorityProviderUtils, SectionUtils},
};
use std::net::SocketAddr;

impl Core {
    pub(crate) async fn handle_forwarded_message(
        &mut self,
        msg_id: MessageId,
        msg: ClientMsg,
        user: EndUser,
        client_signed: ClientSigned,
    ) -> Result<Vec<Command>> {
        self.send_event(Event::ClientMsgReceived {
            msg_id,
            msg: Box::new(msg),
            user,
            client_signed,
        })
        .await;

        Ok(vec![])
    }

    pub(crate) async fn handle_end_user_message(
        &mut self,
        sender: SocketAddr,
        msg_id: MessageId,
        client_signed: ClientSigned,
        msg: ClientMsg,
        dst_location: DstLocation,
    ) -> Result<Vec<Command>> {
        let user = match self.get_enduser_by_addr(&sender) {
            Some(end_user) => {
                debug!(
                    "Message ({}) from client {}, socket id already exists: {:?}",
                    msg_id, sender, end_user
                );
                *end_user
            }
            None => {
                // This is the first time we receive a message from this client
                debug!(
                    "First message ({}) from client {}, creating a socket id",
                    msg_id, sender
                );

                // TODO: remove the enduser registry and simply encrypt socket
                // addr with this node's keypair and use that as the socket id
                match self.try_add_enduser(sender) {
                    Ok(end_user) => end_user,
                    Err(err) => {
                        error!(
                            "Failed to cache client socket address for message {:?}: {:?}",
                            msg, err
                        );
                        return Ok(vec![]);
                    }
                }
            }
        };

        let is_in_destination = match dst_location.name() {
            Some(dst_name) => self.section().prefix().matches(&dst_name),
            None => true, // it's a DirectAndUnrouted dst
        };

        if is_in_destination {
            // We send this message to be handled by the upper Node layer
            // through the public event stream API
            self.handle_forwarded_message(msg_id, msg, user, client_signed)
                .await
        } else {
            // Let's relay the client message then
            let node_msg = NodeMsg::ForwardClientMsg {
                msg,
                user,
                client_signed,
            };

            let wire_msg = match WireMsg::single_src(
                &self.node,
                dst_location,
                node_msg,
                self.section.authority_provider().section_key(),
            ) {
                Ok(msg) => msg,
                Err(err) => {
                    error!("Failed create node msg {:?}", err);
                    return Ok(vec![]);
                }
            };

            match self.relay_message(wire_msg).await {
                Ok(Some(cmd)) => return Ok(vec![cmd]),
                Ok(None) => {
                    error!("Failed to relay msg, no cmd returned.");
                }
                Err(err) => {
                    error!("Failed to relay msg {:?}", err);
                }
            }
            Ok(vec![])
        }
    }
}
