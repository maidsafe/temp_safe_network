// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use std::net::SocketAddr;

use qp2p::IncomingMessages;
use tracing::{debug, error, info, trace, warn};

use super::Session;
use crate::client::connections::messaging::send_message;
use crate::client::Error;
use crate::messaging::data::Error as DataError;
use crate::messaging::data::ServiceError;
use crate::messaging::{
    data::{CmdError, ServiceMsg},
    section_info::{GetSectionResponse, SectionInfoMsg},
    DstLocation, MessageId, MessageType, MsgKind, WireMsg,
};

impl Session {
    // Listen for incoming messages on a connection
    pub(crate) async fn spawn_message_listener_thread(
        &self,
        mut incoming_messages: IncomingMessages,
    ) {
        debug!("Listening for incoming messages");
        let mut session = self.clone();
        let _ = tokio::spawn(async move {
            loop {
                match session
                    .process_incoming_message(&mut incoming_messages)
                    .await
                {
                    Ok(true) => (),
                    Ok(false) => {
                        info!("IncomingMessages listener has closed.");
                        break;
                    }
                    Err(err) => {
                        error!("Error while processing incoming message: {:?}. Listening for next message...", err);
                    }
                }
            }
        });
    }

    pub(crate) async fn process_incoming_message(
        &mut self,
        incoming_messages: &mut IncomingMessages,
    ) -> Result<bool, Error> {
        if let Some((src, message)) = incoming_messages.next().await {
            let message_type = WireMsg::deserialize(message)?;
            trace!("Incoming message from {:?}", &src);
            match message_type {
                MessageType::SectionInfo { msg, .. } => {
                    if let Err(error) = self.handle_section_info_msg(msg).await {
                        error!("Error handling network info message: {:?}", error);
                    }
                }
                MessageType::Service { msg_id, msg, .. } => {
                    self.handle_client_msg(msg_id, msg, src).await
                }
                msg_type => {
                    warn!("Unexpected message type received: {:?}", msg_type);
                }
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    // =================== Private helpers ===================

    // Handle received network info messages
    async fn handle_section_info_msg(&mut self, msg: SectionInfoMsg) -> Result<(), Error> {
        trace!("Handling network info message {:?}", msg);

        match &msg {
            SectionInfoMsg::GetSectionResponse(GetSectionResponse::Success(sap))
            | SectionInfoMsg::GetSectionResponse(GetSectionResponse::Redirect(sap)) => {
                debug!("GetSectionResponse::Success!");
                let _ = self.network.write().await.insert(sap.clone());
                Ok(())
            }
            SectionInfoMsg::GetSectionQuery { .. } => Err(Error::UnexpectedMessageOnJoin(format!(
                "bootstrapping failed since an invalid response ({:?}) was received",
                msg
            ))),
        }
    }

    // Handle messages intended for client consumption (re: queries + commands)
    async fn handle_client_msg(&mut self, msg_id: MessageId, msg: ServiceMsg, src: SocketAddr) {
        debug!("ServiceMsg with id {:?} received from {:?}", msg_id, src);
        let queries = self.pending_queries.clone();
        let error_sender = self.incoming_err_sender.clone();
        let network = self.network.clone();
        let client_pk = self.client_pk;
        let endpoint = self.endpoint.clone();

        let _ = tokio::spawn(async move {
            debug!("Thread spawned to handle this client message");
            match msg {
                ServiceMsg::QueryResponse { response, .. } => {
                    trace!("The received query response is {:?}", response);

                    // Note that this doesn't remove the sender from here since multiple
                    // responses corresponding to the same message ID might arrive.
                    // Once we are satisfied with the response this is channel is discarded in
                    // ConnectionManager::send_query

                    if let Ok(op_id) = response.operation_id() {
                        debug!("Query response (op_id is: {})", op_id);

                        if let Some(sender) = &queries.read().await.get(&op_id) {
                            trace!("Sending response for query w/{} via channel.", op_id);
                            let _ = sender.send(response).await;
                        } else {
                            trace!("No channel found for operation {}", op_id);
                        }
                    } else {
                        warn!("Ignoring query response without operation id");
                    }
                }
                ServiceMsg::CmdError {
                    error,
                    correlation_id,
                    ..
                } => {
                    debug!(
                        "CmdError was received for Message w/ID: {:?}, sending on error channel",
                        correlation_id
                    );
                    warn!("CmdError received is: {:?}", error);
                    let _ = error_sender.send(error.clone()).await;

                    match error {
                        CmdError::Data(_error) => {
                            // do nothing just yet
                        }
                    }
                }
                ServiceMsg::ServiceError(ServiceError {
                    reason: Some(DataError::WrongDestination),
                    sap: Some(section_auth),
                    source_message: Some(message),
                }) => {
                    {
                        // Update our network knowledge
                        let _ = network.write().await.insert(section_auth);
                    }
                    // We need to deserialize to check if the original message was tampered
                    // and verify the ServiceAuth.
                    if let Ok(the_original_message) = WireMsg::deserialize(message.clone()) {
                        match the_original_message {
                            MessageType::Service {
                                msg_id, msg, auth, ..
                            } => {
                                // Verify that the authority has not changed
                                if let Ok(serialized_cmd) = WireMsg::serialize_msg_payload(&msg) {
                                    if client_pk == auth.public_key
                                        && client_pk
                                            .verify(&auth.signature, serialized_cmd.clone())
                                            .is_ok()
                                    {
                                        // In case of queries, we do not expect a response right here. It will be handled at the
                                        // original caller `send_query`.
                                        if let Some(dst_address) = msg.dst_address() {
                                            if let Some((elders, section_pk)) =
                                                network.read().await.get_matching(&dst_address).map(
                                                    |sap| {
                                                        (
                                                            sap.elders
                                                                .values()
                                                                .cloned()
                                                                .collect::<Vec<SocketAddr>>(),
                                                            sap.public_key_set.public_key(),
                                                        )
                                                    },
                                                )
                                            {
                                                // Let's rebuild the message with the updated destination details
                                                if let Ok(wire_msg) = WireMsg::new_msg(
                                                    msg_id,
                                                    serialized_cmd,
                                                    MsgKind::ServiceMsg(auth.into_inner()),
                                                    DstLocation::Section {
                                                        name: dst_address,
                                                        section_pk,
                                                    },
                                                ) {
                                                    if let Ok(msg_bytes) = wire_msg.serialize() {
                                                        if let Some(endpoint) = endpoint {
                                                            if let Err(e) = send_message(
                                                                elders,
                                                                msg_bytes,
                                                                endpoint.clone(),
                                                                msg_id,
                                                            )
                                                            .await
                                                            {
                                                                error!("Error on resending ServiceMsg w/id {:?}: {:?}. Restart the flow", msg_id, e)
                                                                //     TODO: Remove pending_query channels on query failure.
                                                            }
                                                        } else {
                                                            error!("AE: No endpoint found");
                                                        }
                                                    } else {
                                                        error!("AE: Error serializing wire_msg on resending message");
                                                    }
                                                } else {
                                                    error!("AE: Error rebuilding wire_msg on resending message");
                                                }
                                            }
                                        } else {
                                            error!("No Dst_Address found on the received rebounded ServiceError. Only Commands and Queries have destination address");
                                        }
                                    } else {
                                        warn!("Failed to prove authenticity of the original message w/id {:?} on AE resend", msg_id);
                                    }
                                } else {
                                    error!(
                                        "Error serializing ServiceMsg w/id {:?} on AE checks.",
                                        msg_id
                                    );
                                }
                            }
                            _ => error!("Received invalid MessageType for ServiceError"),
                        }
                    } else {
                        error!("Error deserializing received ServiceError's source message");
                    }
                }
                msg => {
                    warn!("Ignoring unexpected message type received: {:?}", msg);
                }
            };
        });
    }
}
