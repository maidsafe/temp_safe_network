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
use crate::client::{connections::messaging::send_message, Error};
use crate::messaging::{
    data::{CmdError, ServiceMsg},
    system::{KeyedSig, SectionAuth, SystemMsg},
    DstLocation, MessageId, MessageType, MsgKind, SectionAuthorityProvider, WireMsg,
};
use crate::types::PublicKey;
use bytes::Bytes;
use secured_linked_list::SecuredLinkedList;
use xor_name::XorName;

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
                MessageType::Service { msg_id, msg, .. } => {
                    self.handle_client_msg(msg_id, msg, src).await;
                }
                MessageType::System {
                    msg:
                        SystemMsg::AntiEntropyRedirect {
                            section_auth,
                            section_signed,
                            bounced_msg,
                        },
                    ..
                } => {
                    if let Err(err) = self
                        .handle_ae_redirect_msg(section_auth, section_signed, bounced_msg, src)
                        .await
                    {
                        warn!("Failed to handle AE-Redirect msg: {:?}", err);
                    }
                }
                MessageType::System {
                    msg:
                        SystemMsg::AntiEntropyRetry {
                            section_auth,
                            section_signed,
                            bounced_msg,
                            proof_chain,
                        },
                    ..
                } => {
                    if let Err(err) = self
                        .handle_ae_retry_msg(section_auth, section_signed, bounced_msg, proof_chain)
                        .await
                    {
                        warn!("Failed to handle AE-Retry msg: {:?}", err);
                    }
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
    async fn handle_client_msg(&mut self, msg_id: MessageId, msg: ServiceMsg, src: SocketAddr) {
        debug!("ServiceMsg with id {:?} received from {:?}", msg_id, src);
        let queries = self.pending_queries.clone();
        let error_sender = self.incoming_err_sender.clone();

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
                msg => {
                    warn!("Ignoring unexpected message type received: {:?}", msg);
                }
            };
        });
    }

    // Handle Anti-Entropy Redirect messages
    async fn handle_ae_redirect_msg(
        &self,
        section_auth: SectionAuthorityProvider,
        section_signed: KeyedSig,
        bounced_msg: Bytes,
        sender: SocketAddr,
    ) -> Result<(), Error> {
        // Check if SAP signature is valid
        if !bincode::serialize(&section_auth)
            .map(|bytes| section_signed.verify(&bytes))
            .unwrap_or(false)
        {
            warn!(
                "Signature returned with SAP in AE-Redirect response is invalid: {:?}",
                section_auth
            );
            return Ok(());
        }

        let (msg_id, service_msg, auth) = match WireMsg::deserialize(bounced_msg)? {
            MessageType::Service {
                msg_id, msg, auth, ..
            } => (msg_id, msg, auth),
            other => {
                warn!(
                    "Unexpected non-serviceMsg returned in AE-Redirect response: {:?}",
                    other
                );
                return Ok(());
            }
        };
        debug!(
            "Received AE-Redirect for {:?}, from {}, with SAP: {:?}",
            msg_id, sender, section_auth
        );

        debug!(
            "Bounced message ({:?}) received in AE-Redirect response: {:?}",
            msg_id, service_msg
        );
        let message = WireMsg::serialize_msg_payload(&service_msg)?;

        // TODO: we cannot trust these Elders belong to the network we are intended
        // to connect to (based on the genesis key we know). We could send the genesis key
        // as the destination section key and that should cause an AE-Retry response,
        // which we could use to verify the SAP we receive an trust.
        let elders = section_auth
            .elders
            .values()
            .cloned()
            .collect::<Vec<SocketAddr>>();
        let section_pk = section_auth.public_key_set.public_key();

        // Let's rebuild the message with the updated destination details
        let wire_msg = WireMsg::new_msg(
            msg_id,
            message,
            MsgKind::ServiceMsg(auth.into_inner()),
            DstLocation::Section {
                name: XorName::from(PublicKey::Bls(section_pk)),
                section_pk,
            },
        )?;

        send_message(elders, wire_msg, self.endpoint.clone(), msg_id).await
    }

    // Handle Anti-Entropy Retry messages
    async fn handle_ae_retry_msg(
        &self,
        section_auth: SectionAuthorityProvider,
        section_signed: KeyedSig,
        bounced_msg: Bytes,
        proof_chain: SecuredLinkedList,
    ) -> Result<(), Error> {
        debug!("Received AE-Retry with new SAP: {:?}", section_auth);
        // Update our network knowledge making sure proof chain
        // validates the new SAP based on currently known remote section SAP.
        match self.network.update(
            SectionAuth {
                value: section_auth.clone(),
                sig: section_signed,
            },
            &proof_chain,
        ) {
            Ok(updated) => {
                if updated {
                    debug!(
                        "Anti-Entropy: updated remote section SAP updated for {:?}",
                        section_auth.prefix
                    );
                } else {
                    debug!(
                        "Anti-Entropy: discarded SAP for {:?} since it's the same as the one in our records: {:?}",
                        section_auth.prefix, section_auth
                    );
                }
            }
            Err(err) => {
                warn!("Anti-Entropy: failed to update remote section SAP, bounced msg dropped: {:?}, {:?}", bounced_msg, err);
                return Ok(());
            }
        }

        let (msg_id, service_msg, mut dst_location, auth) = match WireMsg::deserialize(bounced_msg)?
        {
            MessageType::Service {
                msg_id,
                msg,
                auth,
                dst_location,
            } => (msg_id, msg, dst_location, auth),
            other => {
                warn!(
                    "Unexpected non-serviceMsg returned in AE response: {:?}",
                    other
                );
                return Ok(());
            }
        };

        debug!(
            "Bounced message ({:?}) received in AE response: {:?}",
            msg_id, service_msg
        );
        let payload = WireMsg::serialize_msg_payload(&service_msg)?;

        // Let's rebuild the message with the updated destination details
        let elders = section_auth
            .elders
            .values()
            .cloned()
            .collect::<Vec<SocketAddr>>();
        dst_location.set_section_pk(section_auth.public_key_set.public_key());

        let wire_msg = WireMsg::new_msg(
            msg_id,
            payload,
            MsgKind::ServiceMsg(auth.into_inner()),
            dst_location,
        )?;

        send_message(elders, wire_msg, self.endpoint.clone(), msg_id).await
    }
}
