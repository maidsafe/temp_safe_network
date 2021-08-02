// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::Core;
use crate::dbs::convert_to_error_message as convert_db_error_to_error_message;
use crate::messaging::data::ServiceMsg;
use crate::messaging::NodeAuth;
use crate::messaging::{
    data::{
        ChunkRead, CmdError, DataCmd, DataQuery, Error as ErrorMessage, QueryResponse,
        RegisterRead, RegisterWrite,
    },
    node::{NodeMsg, NodeQueryResponse},
    AuthorityProof, DstLocation, EndUser, MessageId, MsgKind, ServiceAuth, WireMsg,
};
use crate::routing::core::capacity::CHUNK_COPY_COUNT;
use crate::routing::peer::PeerUtils;
use crate::routing::{
    error::Result, messages::WireMsgUtils, routing_api::command::Command, section::SectionUtils,
    SectionAuthorityProviderUtils,
};
// use bls::PublicKey;
use crate::types::PublicKey;
use bytes::Bytes;
use itertools::Itertools;
use std::collections::BTreeSet;
use std::net::SocketAddr;
use xor_name::XorName;

impl Core {
    /// Forms a command to send the provided node error out
    pub(crate) fn send_cmd_error_response(
        &self,
        error: CmdError,
        target: EndUser,
        msg_id: MessageId,
    ) -> Result<Vec<Command>> {
        // let sending_error = ErrorMessage::InvalidOperation(error.to_string());

        let the_error_msg = ServiceMsg::CmdError {
            error,
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
    pub(crate) async fn handle_register_write(
        &self,
        msg_id: MessageId,
        register_write: RegisterWrite,
        user: EndUser,
        auth: AuthorityProof<ServiceAuth>,
    ) -> Result<Vec<Command>> {
        match self.register_storage.write(register_write, auth).await {
            Ok(_) => Ok(vec![]),
            Err(error) => {
                trace!("Problem on writing Register! {:?}", error);
                let error = convert_db_error_to_error_message(error);

                let error = CmdError::Data(error);
                self.send_cmd_error_response(error, user, msg_id)
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
        match self.register_storage.read(&query, auth.public_key) {
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
                let error = convert_db_error_to_error_message(error);
                let error = CmdError::Data(error);

                self.send_cmd_error_response(error, user, msg_id)
            }
        }
    }

    /// Handle chunk read
    pub(crate) fn handle_chunk_query_response_at_adult(
        &self,
        msg_id: MessageId,
        query: ChunkRead,
        requester: PublicKey,
        user: EndUser,
    ) -> Result<Vec<Command>> {
        // TODO ensure we track liveness.

        debug!(">>>> HANDLE chunk read at adult");
        // let pk = auth.into_inner().public_key;
        match self.chunk_storage.read(&query, requester) {
            Ok(response) => {
                let msg = NodeMsg::NodeQueryResponse {
                    response,
                    correlation_id: msg_id,
                    user,
                };

                // Setup node authority on this response and send this back to our elders
                let section_pk = *self.section().chain().last_key();
                let dst = DstLocation::Section {
                    name: query.dst_address(),
                    section_pk,
                };

                let payload = WireMsg::serialize_msg_payload(&msg)?;

                let auth = NodeAuth::authorize(section_pk, &self.node().keypair, &payload);
                let msg_kind = MsgKind::NodeAuthMsg(auth.into_inner());

                let wire_msg = WireMsg::new_msg(msg_id, payload, msg_kind, dst)?;

                let command = Command::ParseAndSendWireMsg(wire_msg);

                Ok(vec![command])
            }
            Err(error) => {
                error!("Problem on reading chunk! {:?}", error);
                // Nothing to do, we've had a bad time here...
                Ok(vec![])
            }
        }
    }

    /// Handle chunk read
    /// Records response in liveness tracking
    /// Forms a response to send to the requester
    pub(crate) fn handle_chunk_query_response_at_elder(
        &self,
        // msg_id: MessageId,
        correlation_id: MessageId,
        response: NodeQueryResponse,
        user: EndUser,
        sending_nodes_pk: PublicKey,
    ) -> Result<Vec<Command>> {
        // TODO ensure we track liveness.

        let msg_id = MessageId::new();
        debug!(
            ">>>>>> HANDLE chunk read @ elders from {:?} ",
            sending_nodes_pk
        );

        let NodeQueryResponse::GetChunk(response) = response;
        // let pk = auth.into_inner().public_key;
        let query_response = QueryResponse::GetChunk(response);

        if query_response.failed_with_data_not_found() {
            // we don't return data not found errors.
            debug!(">>>> {:?}, saying data not found", sending_nodes_pk);

            match query_response.operation_id() {
                Ok(op_id) => {
                    let node_id = XorName::from(sending_nodes_pk);
                    self.liveness.remove_black_eye(&node_id, op_id)
                }
                Err(error) => {
                    warn!("Node problems noted when retrieving data: {:?}", error)
                }
            }
            return Ok(vec![]);
        }

        let msg = ServiceMsg::QueryResponse {
            response: query_response,
            correlation_id,
        };

        // FIXME: define which signature/authority this message should really carry,
        // perhaps it needs to carry Node signature on a NodeMsg::QueryResponse msg type.
        // Giving a random sig temporarily
        let (msg_kind, payload) = Self::random_client_signature(&msg)?;

        let dst = DstLocation::EndUser(user);
        let wire_msg = WireMsg::new_msg(msg_id, payload, msg_kind, dst)?;

        let command = Command::ParseAndSendWireMsg(wire_msg);
        debug!(">>>> Should be sending to client now... {:?}", dst);

        Ok(vec![command])
    }

    /// Handle ServiceMsgs received from EndUser
    pub(crate) async fn handle_service_msg_received(
        &self,
        msg_id: MessageId,
        msg: ServiceMsg,
        user: EndUser,
        auth: AuthorityProof<ServiceAuth>,
    ) -> Result<Vec<Command>> {
        let requester = auth.public_key;
        let service_auth = auth.into_inner();
        let verified = match WireMsg::verify_sig(service_auth, msg.clone()) {
            Ok(verified) => verified,
            Err(_e) => {
                let error = CmdError::Data(ErrorMessage::InvalidOwner(requester));
                return self.send_cmd_error_response(error, user, msg_id);
            }
        };

        match msg {
            // Register
            // Commands to be handled at elder.
            ServiceMsg::Cmd(DataCmd::Register(register_write)) => {
                self.handle_register_write(msg_id, register_write, user, verified)
                    .await
            }
            ServiceMsg::Query(DataQuery::Register(read)) => {
                self.handle_register_read(msg_id, read, user, verified)
            }
            // These will only be received at elders.
            // These reads/writes are for adult nodes...
            ServiceMsg::Cmd(DataCmd::Chunk(chunk_write)) => {
                self.write_chunk_to_adults(chunk_write, msg_id, verified, user)
                    .await
            }
            ServiceMsg::Query(DataQuery::Chunk(read)) => {
                self.read_chunk_from_adults(&read, msg_id, verified, user)
                    .await
            }
            _ => {
                warn!("!!!! Unexpected ServiceMsg received in routing. Was not sent to node layer: {:?}", msg);
                Ok(vec![])
            }
        }
    }

    // Used to fetch the list of holders for a given chunk.
    pub(crate) async fn get_chunk_holder_adults(&self, target: &XorName) -> BTreeSet<XorName> {
        let full_adults = self.full_adults().await;
        // TODO, tidy this up,  was basically non_full_adults_closest_to func in adult_reader

        // TODO: reuse our_adults_sorted_by_distance_to API when core is merged into upper layer
        let adults = self
            .section()
            .adults()
            .copied()
            .map(|p2p_node| *p2p_node.name())
            .collect::<Vec<_>>();

        adults
            .into_iter()
            .sorted_by(|lhs, rhs| target.cmp_distance(lhs, rhs))
            .filter(|name| !full_adults.contains(name))
            .take(CHUNK_COPY_COUNT)
            .collect::<BTreeSet<_>>()
    }

    /// Handle incoming data msgs, determining if they should be handled at this node or fowrwarded
    // TODO: streamline this as full AE for direct messaging is included.
    pub(crate) async fn handle_service_message(
        &mut self,
        sender: SocketAddr,
        msg_id: MessageId,
        auth: AuthorityProof<ServiceAuth>,
        msg: ServiceMsg,
        dst_location: DstLocation,
        payload: Bytes,
    ) -> Result<Vec<Command>> {
        debug!(">>>>> Incoming service msg...");
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
            debug!(">>> In destination");
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
