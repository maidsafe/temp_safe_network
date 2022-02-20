// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::Core;
use crate::dbs::Error as DbError;
use crate::messaging::data::ServiceMsg;
use crate::messaging::{
    data::{CmdError, DataCmd, DataQuery, Error as ErrorMessage, RegisterRead, RegisterWrite},
    node::NodeMsg,
    AuthorityProof, DstLocation, EndUser, MessageId, MsgKind, ServiceAuth, WireMsg,
};
use crate::routing::{
    error::Result, messages::WireMsgUtils, routing_api::command::Command, section::SectionUtils,
    Event, SectionAuthorityProviderUtils,
};
use bytes::Bytes;
use std::net::SocketAddr;

impl Core {
    /// Forms a command to send the provided node error out
    fn send_error_response(
        &self,
        error: DbError,
        target: EndUser,
        msg_id: MessageId,
    ) -> Result<Vec<Command>> {
        let sending_error = ErrorMessage::InvalidOperation(error.to_string());

        let the_error_msg = ServiceMsg::CmdError {
            error: CmdError::Data(sending_error),
            correlation_id: msg_id,
        };

        let dst = DstLocation::EndUser(target);

        // FIXME: define which signature/authority this message should really carry,
        // perhaps it needs to carry Node signature on a NodeMsg::QueryResponse msg type.
        // Giving a random sig temporarily
        let (msg_kind, payload) = Self::random_client_signature(&the_error_msg)?;
        let wire_msg = WireMsg::new_msg(MessageId::new(), payload, msg_kind, dst)?;

        let command = Command::ParseAndSendWireMsg(wire_msg);

        Ok(vec![command])
    }

    /// Handle regsiter commands
    pub(crate) async fn handle_register_cmd(
        &self,
        msg_id: MessageId,
        register_cmd: RegisterWrite,
        user: EndUser,
        auth: AuthorityProof<ServiceAuth>,
    ) -> Result<Vec<Command>> {
        match self.register_storage.write(register_cmd, auth).await {
            Ok(_) => Ok(vec![]),
            Err(error) => {
                trace!("Problem on writing Register! {:?}", error);
                self.send_error_response(error, user, msg_id)
            }
        }
    }

    /// Handle register reads
    pub(crate) fn handle_register_read(
        &self,
        msg_id: MessageId,
        query: RegisterRead,
        user: EndUser,
        auth: AuthorityProof<ServiceAuth>,
    ) -> Result<Vec<Command>> {
        match self.register_storage.read(&query, auth.node_pk) {
            Ok(response) => {
                if response.failed_with_data_not_found() {
                    // we don't return data not found errors.
                    return Ok(vec![]);
                }

                let msg = ServiceMsg::QueryResponse {
                    response,
                    correlation_id: msg_id,
                };

                // FIXME: define which signature/authority this message should really carry,
                // perhaps it needs to carry Node signature on a NodeMsg::QueryResponse msg type.
                // Giving a random sig temporarily
                let (msg_kind, payload) = Self::random_client_signature(&msg)?;

                let dst = DstLocation::EndUser(user);
                let wire_msg = WireMsg::new_msg(msg_id, payload, msg_kind, dst)?;

                let command = Command::ParseAndSendWireMsg(wire_msg);

                Ok(vec![command])
            }
            Err(error) => {
                trace!("Problem on reading Register! {:?}", error);

                self.send_error_response(error, user, msg_id)
            }
        }
    }

    /// Handle DataMsgs received
    pub(crate) async fn handle_data_msg_received(
        &self,
        msg_id: MessageId,
        msg: ServiceMsg,
        user: EndUser,
        auth: AuthorityProof<ServiceAuth>,
    ) -> Result<Vec<Command>> {
        match msg {
            ServiceMsg::Cmd(DataCmd::Register(register_cmd)) => {
                self.handle_register_cmd(msg_id, register_cmd, user, auth)
                    .await
            }
            ServiceMsg::Query(DataQuery::Register(read)) => {
                self.handle_register_read(msg_id, read, user, auth)
            }
            _ => {
                self.send_event(Event::ServiceMsgReceived {
                    msg_id,
                    msg: Box::new(msg),
                    user,
                    auth,
                })
                .await;

                Ok(vec![])
            }
        }
    }

    /// Handle incoming data msgs, determining if they should be handled at this node or fowrwarded
    // TODO: streamline this as full AE for direct messaging is included.
    pub(crate) async fn handle_data_message(
        &mut self,
        sender: SocketAddr,
        msg_id: MessageId,
        auth: AuthorityProof<ServiceAuth>,
        msg: ServiceMsg,
        dst_location: DstLocation,
        payload: Bytes,
    ) -> Result<Vec<Command>> {
        let is_in_destination = match dst_location.name() {
            Some(dst_name) => {
                let is_in_destination = self.section().prefix().matches(&dst_name);
                if is_in_destination {
                    if let DstLocation::EndUser(EndUser { socket_id, xorname }) = dst_location {
                        if let Some(addr) = self.get_socket_addr(socket_id) {
                            let wire_msg = WireMsg::new_msg(
                                msg_id,
                                payload,
                                MsgKind::ServiceMsg(auth.into_inner()),
                                dst_location,
                            )?;

                            return Ok(vec![Command::SendMessage {
                                recipients: vec![(xorname, *addr)],
                                wire_msg,
                            }]);
                        }
                    }
                }

                is_in_destination
            }
            None => true, // it's a DirectAndUnrouted dst
        };

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

        if is_in_destination {
            // We send this message to be handled by the upper Node layer
            // through the public event stream API

            // This is returned as a command to be handled via spawning
            Ok(vec![Command::HandleServiceMessage {
                msg_id,
                msg,
                user,
                auth,
            }])
        } else {
            // Let's relay the client message then
            let node_msg = NodeMsg::ForwardServiceMsg {
                msg,
                user,
                auth: auth.into_inner(),
            };

            let wire_msg = match WireMsg::single_src(
                &self.node,
                dst_location,
                node_msg,
                self.section.authority_provider().section_key(),
            ) {
                Ok(mut wire_msg) => {
                    wire_msg.set_msg_id(msg_id);
                    wire_msg
                }
                Err(err) => {
                    error!("Failed create node msg {:?}", err);
                    return Ok(vec![]);
                }
            };

            match self.relay_message(wire_msg).await {
                Ok(cmd) => return Ok(vec![cmd]),
                Err(err) => {
                    error!("Failed to relay msg {:?}", err);
                }
            }
            Ok(vec![])
        }
    }
}
