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
use crate::client::connections::messaging::{rebuild_message_for_ae_resend, send_message};
use crate::client::Error;
use crate::messaging::data::Error as DataError;
use crate::messaging::data::ServiceError;
use crate::messaging::{
    data::{CmdError, ServiceMsg},
    MessageId, MessageType, WireMsg,
};
use crate::messaging::{DstLocation, ServiceAuth};

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
                MessageType::Service {
                    msg_id,
                    msg,
                    auth,
                    dst_location,
                } => {
                    self.handle_client_msg(msg_id, msg, src, auth.into_inner(), dst_location)
                        .await
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

    // Handle messages intended for client consumption (re: queries + commands)
    async fn handle_client_msg(
        &mut self,
        msg_id: MessageId,
        msg: ServiceMsg,
        src: SocketAddr,
        auth: ServiceAuth,
        dst_location: DstLocation,
    ) {
        debug!("ServiceMsg with id {:?} received from {:?}", msg_id, src);
        let queries = self.pending_queries.clone();
        let error_sender = self.incoming_err_sender.clone();
        let network = self.network.clone();
        let endpoint = self.endpoint.clone();

        let _ = tokio::spawn(async move {
            debug!("Thread spawned to handle this client message");
            match msg {
                ServiceMsg::QueryResponse { response, .. } => {
                    trace!(
                        "The received query response id is {:?}, msg is {:?}",
                        response.operation_id(),
                        response
                    );

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
                    info!("Received AE-Redirect SAP: {:?}", section_auth);
                    // Update our network knowledge
                    let _ = network
                        .write()
                        .await
                        .insert(section_auth.prefix, section_auth);
                    info!("Updated network knowledge");

                    if let Some((wire_msg, elders)) = rebuild_message_for_ae_resend(
                        msg_id,
                        message.payload,
                        auth,
                        message.dst_location.name(),
                        network,
                    )
                    .await
                    {
                        if let Err(e) = send_message(elders, wire_msg, endpoint, msg_id).await {
                            error!("AE: Error on resending ServiceMsg w/id {:?}: {:?}. Restart the flow", msg_id, e)
                            //     TODO: Remove pending_query channels on query failure.
                        }
                    } else {
                        error!("AE: Error rebuilding message for resending");
                    }
                }
                msg => {
                    warn!("Ignoring unexpected message type received: {:?}", msg);
                }
            };
        });
    }
}
