// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

// use super::Session;
// use crate::{
//     client::{
//         connections::messaging::{rebuild_message_for_ae_resend, send_message},
//         Error,
//     },
//     messaging::{
//         data::{CmdError, ServiceMsg},
//         system::{SectionAuth, SystemMsg},
//         MessageId, MessageType, WireMsg,
//     },
// };

use qp2p::IncomingMessages;
use std::net::SocketAddr;
// use tracing::{debug, error, info, trace, warn};

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
        mut session: Session,
        mut incoming_messages: IncomingMessages,
    ) {
        debug!("Listening for incoming messages");
        let _ = tokio::spawn(async move {
            loop {
                session = match Self::get_incoming_message(&mut incoming_messages).await {
                    Ok((src, msg)) => match Self::take_msg(msg, src, session.clone()).await {
                        Ok(session) => session,
                        Err(err) => {
                            error!("Error while processing incoming message: {:?}. Listening for next message...", err);
                            session
                        }
                    },
                    Err(Error::Generic(_)) => {
                        // TODO: FIX error type
                        info!("IncomingMessages listener has closed.");
                        break;
                    }
                    Err(err) => {
                        error!("Error while getting incoming message: {:?}. Listening for next message...", err);
                        session
                    }
                }
            }
        });
    }

    pub(crate) async fn get_incoming_message(
        incoming_messages: &mut IncomingMessages,
    ) -> Result<(SocketAddr, MessageType), Error> {
        if let Some((src, message)) = incoming_messages.next().await {
            let msg_type = WireMsg::deserialize(message)?;
            trace!("Incoming message from {:?}", &src);
            Ok((src, msg_type))
        } else {
            Err(Error::Generic("Nothing..".to_string())) // TODO: FIX error type
        }
    }

    pub(crate) async fn take_msg(
        msg: MessageType,
        src: SocketAddr,
        session: Session,
    ) -> Result<Session, Error> {
        match msg {
            MessageType::Service { msg_id, msg, .. } => {
                return Self::handle_client_msg(session, msg_id, msg, src).await;
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
                let result = Self::handle_ae_redirect_msg(
                    session,
                    section_auth,
                    section_signed,
                    bounced_msg,
                    src,
                )
                .await;
                if result.is_err() {
                    warn!("Failed to handle AE-Redirect");
                }
                result
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
                let result = Self::handle_ae_retry_msg(
                    session,
                    section_auth,
                    section_signed,
                    bounced_msg,
                    proof_chain,
                )
                .await;
                if result.is_err() {
                    warn!("Failed to handle AE-Retry msg");
                }
                result
            }
            msg_type => {
                warn!("Unexpected message type received: {:?}", msg_type);
                Ok(session)
            }
        }
    }

    // Handle messages intended for client consumption (re: queries + commands)
    async fn handle_client_msg(
        session: Session,
        msg_id: MessageId,
        msg: ServiceMsg,
        src: SocketAddr,
    ) -> Result<Session, Error> {
        debug!("ServiceMsg with id {:?} received from {:?}", msg_id, src);
        let queries = session.pending_queries.clone();
        let error_sender = session.incoming_err_sender.clone();

        let _ = tokio::spawn(async move {
            match msg {
                ServiceMsg::QueryResponse { response, .. } => {
                    // Note that this doesn't remove the sender from here since multiple
                    // responses corresponding to the same message ID might arrive.
                    // Once we are satisfied with the response this is channel is discarded in
                    // ConnectionManager::send_query

                    if let Ok(op_id) = response.operation_id() {
                        if let Some(sender) = &queries.read().await.get(&op_id) {
                            trace!("Sending response for query w/{} via channel.", op_id);
                            let _ = sender.send(response).await;
                        } else {
                            // temporarily commenting this out
                            // if we have already received enough responses for a query,
                            // we drop the channels and any drop further responses for that query.
                            // but we should not drop it immediately, but clean it up after a while
                            // and then not log that "no channel was found" when we already had enough responses..
                            //trace!("No channel found for operation {}", op_id);
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

        Ok(session)
    }

    // Handle Antry-Entropy Redirect messages
    async fn handle_ae_redirect_msg(
        session: Session,
        section_auth: SectionAuthorityProvider,
        section_signed: KeyedSig,
        bounced_msg: Bytes,
        sender: SocketAddr,
    ) -> Result<Session, Error> {
        // Check if SAP signature is valid
        if !bincode::serialize(&section_auth)
            .map(|bytes| section_signed.verify(&bytes))
            .unwrap_or(false)
        {
            warn!(
                "Signature returned with SAP in AE-Redirect response is invalid: {:?}",
                section_auth
            );
            return Ok(session);
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
                return Ok(session);
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

        send_message(elders, wire_msg, session.endpoint.clone(), msg_id).await?;

        Ok(session)
    }

    // Handle Antry-Entropy Retry messages
    async fn handle_ae_retry_msg(
        session: Session,
        section_auth: SectionAuthorityProvider,
        section_signed: KeyedSig,
        bounced_msg: Bytes,
        proof_chain: SecuredLinkedList,
    ) -> Result<Session, Error> {
        debug!("Received AE-Retry with new SAP: {:?}", section_auth);
        // Update our network knowledge making sure proof chain
        // validates the new SAP based on currently known remote section SAP.
        match session.network.update(
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
                warn!("Anti-Entropy: failed to update remote section SAP, bounced msg dropped: {:?}, {}", bounced_msg, err);
                return Ok(session);
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
                return Ok(session);
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

        send_message(elders, wire_msg, session.endpoint.clone(), msg_id).await?;

        Ok(session)
    }
}
