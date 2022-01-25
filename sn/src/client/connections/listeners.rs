// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::Session;

use crate::client::{
    connections::messaging::{send_message, NUM_OF_ELDERS_SUBSET_FOR_QUERIES},
    Error, Result,
};
use crate::messaging::{
    data::{CmdError, DataCmd, ServiceMsg},
    system::{KeyedSig, SectionAuth, SystemMsg},
    AuthorityProof, DstLocation, MessageId, MessageType, MsgKind, ServiceAuth, WireMsg,
};
use crate::node::SectionAuthorityProvider;
use crate::peer::Peer;
use crate::types::{log_markers::LogMarker, utils::compare_and_write_prefix_map_to_disk};
use crate::{at_least_one_correct_elder, elder_count};

use bytes::Bytes;
use itertools::Itertools;
use qp2p::{Close, ConnectionError, ConnectionIncoming, SendError};
use secured_linked_list::SecuredLinkedList;
use std::net::SocketAddr;
use tracing::Instrument;

impl Session {
    // Listen for incoming messages on a connection
    #[instrument(skip_all, level = "debug")]
    pub(crate) fn spawn_message_listener_thread(
        session: Session,
        connection_id: usize,
        connected_peer: Peer,
        mut incoming_messages: ConnectionIncoming,
    ) {
        let src = connected_peer.addr();
        debug!("Listening for incoming messages from {}", connected_peer);

        trace!(
            "{} to {} (id: {})",
            LogMarker::ConnectionOpened,
            src,
            connection_id
        );

        let _handle = tokio::spawn(async move {
            loop {
                match Self::listen_for_incoming_message(src, &mut incoming_messages).await {
                    Ok(Some(msg)) => {
                        if let Err(err) = Self::handle_msg(msg, src, session.clone()).await {
                            error!("Error while handling incoming message: {:?}. Listening for next message...", err);
                        }
                    },
                    Ok(None) => {
                        info!("IncomingMessages listener has closed for connection {}.", connection_id);
                        break;
                    }
                    Err( Error::QuicP2pSend(SendError::ConnectionLost(
                        ConnectionError::Closed(Close::Application { reason, .. }),
                    ))) => {
                        warn!(
                            "Connection was closed by the node: {:?}",
                            String::from_utf8(reason.to_vec())
                        );

                        let _old = session.elder_last_closed_connections.insert(connected_peer.name(), connection_id);
                    },
                    Err(Error::QuicP2p(qp2p_err)) => {
                          // TODO: Can we recover here?
                          info!("Error from Qp2p received, closing listener loop. {:?}", qp2p_err);


                          break;
                    },
                    Err(error) => {
                        error!("Error while processing incoming message: {:?}. Listening for next message...", error);
                    }
                }
            }

            // once the message loop breaks, we know the connection is closed
            trace!("{} to {} (id: {})", LogMarker::ConnectionClosed, src, connection_id);
        }.instrument(info_span!("Listening for incoming msgs from {}", ?src))).in_current_span();
    }

    #[instrument(skip_all, level = "debug")]
    pub(crate) async fn listen_for_incoming_message(
        src: SocketAddr,
        incoming_messages: &mut ConnectionIncoming,
    ) -> Result<Option<MessageType>, Error> {
        if let Some(message) = incoming_messages.next().await? {
            trace!("Incoming message from {:?}", src);
            let msg_type = WireMsg::deserialize(message)?;

            Ok(Some(msg_type))
        } else {
            Ok(None)
        }
    }

    #[instrument(skip_all, level = "debug")]
    pub(crate) async fn handle_msg(
        msg: MessageType,
        src: SocketAddr,
        session: Session,
    ) -> Result<(), Error> {
        match msg {
            MessageType::Service { msg_id, msg, .. } => {
                Self::handle_client_msg(session, msg_id, msg, src)
            }
            MessageType::System {
                msg:
                    SystemMsg::AntiEntropyRedirect {
                        section_auth,
                        section_signed,
                        section_chain,
                        bounced_msg,
                    },
                ..
            } => {
                let result = Self::handle_ae_redirect_msg(
                    session,
                    section_auth.into_state(),
                    section_signed,
                    section_chain,
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
                    section_auth.into_state(),
                    section_signed,
                    bounced_msg,
                    proof_chain,
                    src,
                )
                .await;
                if result.is_err() {
                    warn!("Failed to handle AE-Retry msg from {:?}", src);
                }
                result
            }
            msg_type => {
                warn!("Unexpected message type received: {:?}", msg_type);
                Ok(())
            }
        }
    }

    // Handle messages intended for client consumption (re: queries + commands)
    #[instrument(skip(session), level = "debug")]
    fn handle_client_msg(
        session: Session,
        msg_id: MessageId,
        msg: ServiceMsg,
        src: SocketAddr,
    ) -> Result<(), Error> {
        debug!("ServiceMsg with id {:?} received from {:?}", msg_id, src);
        let queries = session.pending_queries.clone();
        let error_sender = session.incoming_err_sender;

        let _handle = tokio::spawn(async move {
            match msg {
                ServiceMsg::QueryResponse { response, .. } => {
                    // Note that this doesn't remove the sender from here since multiple
                    // responses corresponding to the same message ID might arrive.
                    // Once we are satisfied with the response this is channel is discarded in
                    // ConnectionManager::send_query

                    if let Ok(op_id) = response.operation_id() {
                        if let Some(entry) = queries.get(&op_id) {
                            let all_senders = entry.value();
                            for (_msg_id, sender) in all_senders {
                                trace!("Sending response for query w/{:?} via channel.", op_id);
                                let result = sender.send(response.clone()).await;
                                if result.is_err() {
                                    trace!("Error sending query response on a channel for {:?} op_id {:?}: {:?}. (It has likely been removed)", msg_id, op_id, result)
                                }
                            }
                        } else {
                            // TODO: The trace is only needed when we have an identified case of not finding a channel, but expecting one.
                            // When expecting one, we can log "No channel found for operation", (and then probably at warn or error level).
                            // But when we have received enough responses, we aren't really expecting a channel there, so there is no reason to log anything.
                            // Right now, if we have already received enough responses for a query,
                            // we drop the channels and drop any further responses for that query.
                            // but we should not drop it immediately, but clean it up after a while
                            // and then not log that "no channel was found" when we already had enough responses.
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
                    let _error_from_sender = error_sender.send(error.clone()).await;

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

        Ok(())
    }

    // Handle Anti-Entropy Redirect messages
    #[instrument(skip_all, level = "debug")]
    async fn handle_ae_redirect_msg(
        session: Session,
        target_sap: SectionAuthorityProvider,
        section_signed: KeyedSig,
        section_chain: SecuredLinkedList,
        bounced_msg: Bytes,
        sender: SocketAddr,
    ) -> Result<(), Error> {
        debug!(
            "Received AE-Redirect for from {}, with SAP: {:?}",
            sender, target_sap
        );

        // Try to update our network knowledge first
        Self::update_network_knowledge(
            &session,
            target_sap.clone(),
            section_signed,
            section_chain,
            sender,
        )
        .await;

        if let Some((msg_id, elders, service_msg, dst_location, auth)) =
            Self::new_elder_targets_if_any(session.clone(), bounced_msg.clone(), Some(&target_sap))
                .await?
        {
            if elders.is_empty() {
                debug!("We have already resent this message on an AE-Redirect response. Dropping this instance");
                return Ok(());
            }

            let payload = WireMsg::serialize_msg_payload(&service_msg)?;
            let wire_msg = WireMsg::new_msg(
                msg_id,
                payload,
                MsgKind::ServiceMsg(auth.into_inner()),
                dst_location,
            )?;

            debug!("Resending original message on AE-Redirect with updated details. Expecting an AE-Retry next");

            let endpoint = session.endpoint.clone();
            send_message(session, elders.clone(), wire_msg, endpoint, msg_id).await?;
        }

        Ok(())
    }

    // Handle Anti-Entropy Retry messages
    #[instrument(skip_all, level = "debug")]
    async fn handle_ae_retry_msg(
        session: Session,
        sap: SectionAuthorityProvider,
        section_signed: KeyedSig,
        bounced_msg: Bytes,
        proof_chain: SecuredLinkedList,
        sender: SocketAddr,
    ) -> Result<(), Error> {
        // Try to update our network knowledge first
        Self::update_network_knowledge(&session, sap.clone(), section_signed, proof_chain, sender)
            .await;

        // Extract necessary information for resending
        if let Some((msg_id, elders, service_msg, dst_location, auth)) =
            Self::new_elder_targets_if_any(session.clone(), bounced_msg.clone(), None).await?
        {
            if let Some(id) = *session.clone().initial_connection_check_msg_id.read().await {
                if id == msg_id {
                    trace!(
                        "Retry message recevied from intial client contact probe ({:?}). No need to retry this",
                        msg_id
                    );
                    return Ok(());
                }
            }

            debug!("Received AE-Retry with new SAP: {:?}", sap);

            if elders.is_empty() {
                debug!("We have already responded to this message on an AE-Retry response. Dropping this instance");
                return Ok(());
            }

            let payload = WireMsg::serialize_msg_payload(&service_msg)?;
            let wire_msg = WireMsg::new_msg(
                msg_id,
                payload,
                MsgKind::ServiceMsg(auth.into_inner()),
                dst_location,
            )?;

            debug!("Resending original message via AE-Retry");

            let endpoint = session.endpoint.clone();
            send_message(session, elders.clone(), wire_msg, endpoint, msg_id).await?;
        }

        Ok(())
    }

    /// Update our network knowledge making sure proof chain validates the
    /// new SAP based on currently known remote section SAP or genesis key.
    async fn update_network_knowledge(
        session: &Session,
        sap: SectionAuthorityProvider,
        section_signed: KeyedSig,
        proof_chain: SecuredLinkedList,
        sender: SocketAddr,
    ) {
        match session.network.update(
            SectionAuth {
                value: sap.clone(),
                sig: section_signed,
            },
            &proof_chain,
        ) {
            Ok(true) => {
                debug!(
                    "Anti-Entropy: updated remote section SAP updated for {:?}",
                    sap.prefix()
                );
                // Update the PrefixMap on disk
                if let Err(e) = compare_and_write_prefix_map_to_disk(&session.network).await {
                    error!(
                        "Error writing freshly updated PrefixMap to client dir: {:?}",
                        e
                    );
                }
            }
            Ok(false) => {
                debug!(
                    "Anti-Entropy: discarded SAP for {:?} since it's the same as the one in our records: {:?}",
                    sap.prefix(), sap
                );
            }
            Err(err) => {
                warn!(
                    "Anti-Entropy: failed to update remote section SAP w/ err: {:?}",
                    err
                );
                warn!(
                    "Anti-Entropy: bounced msg dropped. Failed section auth was {:?} sent by: {:?}",
                    sap.section_key(),
                    sender
                );
            }
        }
    }

    /// Checks AE cache to see if we should be forwarding this message (and to whom)
    /// or if it has already been dealt with
    #[instrument(skip_all, level = "debug")]
    async fn new_elder_targets_if_any(
        session: Session,
        bounced_msg: Bytes,
        received_auth: Option<&SectionAuthorityProvider>,
    ) -> Result<
        Option<(
            MessageId,
            Vec<Peer>,
            ServiceMsg,
            DstLocation,
            AuthorityProof<ServiceAuth>,
        )>,
        Error,
    > {
        let is_retry = received_auth.is_none();
        let (msg_id, service_msg, mut dst_location, auth) =
            match WireMsg::deserialize(bounced_msg.clone())? {
                MessageType::Service {
                    msg_id,
                    msg,
                    auth,
                    dst_location,
                } => (msg_id, msg, dst_location, auth),
                other => {
                    warn!(
                        "Unexpected non-serviceMsg returned in AE-Redirect response: {:?}",
                        other
                    );
                    return Ok(None);
                }
            };

        trace!(
            "Bounced message ({:?}) received in an AE response: {:?}",
            msg_id,
            service_msg
        );

        let (target_count, dst_address_of_bounced_msg) = match service_msg.clone() {
            ServiceMsg::Cmd(cmd) => {
                match &cmd {
                    DataCmd::StoreChunk(_) => (at_least_one_correct_elder(), cmd.dst_name()), // stored at Adults, so only 1 correctly functioning Elder need to relay
                    DataCmd::Register(_) => (elder_count(), cmd.dst_name()), // only stored at Elders, all need a copy
                }
            }
            ServiceMsg::Query(query) => (NUM_OF_ELDERS_SUBSET_FOR_QUERIES, query.dst_name()),
            _ => {
                warn!(
                    "Invalid bounced message {:?} received in AE response: {:?}. Message is of invalid type",
                    msg_id,
                    service_msg
                );
                // Early return with random name as we will discard the message at the caller func
                return Ok(None);
            }
        };

        let target_public_key;

        // We normally have received auth when we're in AE-Redirect (where we could not trust enough to update our prefixmap)
        let mut target_elders: Vec<_> = if let Some(auth) = received_auth {
            target_public_key = auth.section_key();
            auth.elders_vec()
                .into_iter()
                .sorted_by(|lhs, rhs| {
                    dst_address_of_bounced_msg.cmp_distance(&lhs.name(), &rhs.name())
                })
                .take(target_count)
                .collect()
        } else {
            // we use whatever is our latest knowledge at this point

            if let Some(sap) = session
                .network
                .closest_or_opposite(&dst_address_of_bounced_msg, None)
            {
                target_public_key = sap.section_key();

                sap.elders_vec().into_iter().take(target_count).collect()
            } else {
                error!("Cannot resend {:?}, no 'received auth' provided, and nothing relevant in session network prefixmap", msg_id);
                return Ok(None);
            }
        };

        let mut the_cache_guard = if is_retry {
            session.ae_retry_cache.write().await
        } else {
            session.ae_redirect_cache.write().await
        };

        let cache_entry =
            the_cache_guard.find(|(candidate_elders, candidate_public_key, candidate_msg)| {
                candidate_elders == &target_elders
                    && candidate_public_key == &target_public_key
                    && candidate_msg == &bounced_msg
            });

        if cache_entry.is_some() {
            // an elder group corresponds to a PK, so as we've sent to this PK, we've sent to these elders...
            debug!("Cache hit! we've sent {:?} before", msg_id);

            target_elders = vec![];
        } else {
            let _old_entry_that_does_not_exist = the_cache_guard.insert((
                target_elders.clone(),
                target_public_key,
                bounced_msg.clone(),
            ));
        }

        // Let's rebuild the message with the updated destination details
        dst_location.set_section_pk(target_public_key);

        if !target_elders.is_empty() {
            debug!(
                "Final target elders for resending {:?} : {:?} message are {:?}",
                msg_id, service_msg, target_elders
            );
        }

        drop(the_cache_guard);

        Ok(Some((
            msg_id,
            target_elders,
            service_msg,
            dst_location,
            auth,
        )))
    }
}
