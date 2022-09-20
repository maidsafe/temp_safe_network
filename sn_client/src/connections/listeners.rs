// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::Session;

use crate::{
    connections::{messaging::NUM_OF_ELDERS_SUBSET_FOR_QUERIES, PendingCmdAcks},
    Error, Result,
};

use dashmap::DashSet;
use qp2p::UsrMsgBytes;
use sn_interface::{
    at_least_one_correct_elder,
    messaging::{
        data::{ClientMsg, Error as ErrorMsg},
        system::{AntiEntropyKind, NodeMsg},
        AuthKind, AuthorityProof, ClientAuth, Dst, MsgId, MsgType, WireMsg,
    },
    network_knowledge::{SectionAuthorityProvider, SectionTreeUpdate},
    types::{log_markers::LogMarker, Peer},
};

use itertools::Itertools;
use qp2p::{Close, ConnectionError, ConnectionIncoming as IncomingMsgs, SendError};
use rand::{rngs::OsRng, seq::SliceRandom};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::Instrument;

impl Session {
    // Listen for incoming msgs on a connection
    #[instrument(skip_all, level = "debug")]
    pub(crate) fn spawn_msg_listener_thread(
        session: Self,
        peer: Peer,
        conn: qp2p::Connection,
        mut incoming_msgs: IncomingMsgs,
    ) {
        let mut first = true;
        let addr = peer.addr();
        let connection_id = conn.id();

        debug!("Listening for incoming msgs from {:?}", peer);

        let _handle = tokio::spawn(async move {
            loop {
                match Self::listen_for_incoming_msg(addr, &mut incoming_msgs).await {
                    Ok(Some(msg)) => {
                        if first {
                            first = false;
                            session.peer_links.add_incoming(&peer, conn.clone()).await;
                        }

                        if let Err(err) = Self::handle_msg(msg, peer, session.clone()).await {
                            error!("Error while handling incoming msg: {:?}. Listening for next msg...", err);
                        }
                    },
                    Ok(None) => {
                        // once the msg loop breaks, we know this specific connection is closed
                        break;
                    }
                    Err(Error::QuicP2pSend{ peer, error: SendError::ConnectionLost(
                        ConnectionError::Closed(Close::Application { reason, .. }),
                    )}) => {
                        warn!(
                            "Connection was closed by the node {}: {:?}",peer,
                            String::from_utf8(reason.to_vec())
                        );

                        break;

                    },
                    Err(Error::QuicP2p(qp2p_err)) => {
                        // TODO: Can we recover here?
                        info!("Error from Qp2p received, closing listener loop. {:?}", qp2p_err);
                        break;
                    },
                    Err(error) => {
                        error!("Error while processing incoming msg: {:?}. Listening for next msg...", error);
                    }
                }
            }

            session.peer_links.remove(&peer).await;
            // once the msg loop breaks, we know the connection is closed
            trace!("{} to {} (id: {})", LogMarker::ConnectionClosed, addr, connection_id);

        }.instrument(info_span!("Listening for incoming msgs from {}", ?addr))).in_current_span();
    }

    #[instrument(skip_all, level = "debug")]
    pub(crate) async fn listen_for_incoming_msg(
        src: SocketAddr,
        incoming_msgs: &mut IncomingMsgs,
    ) -> Result<Option<MsgType>, Error> {
        if let Some(bytes) = incoming_msgs.next().await? {
            trace!("Incoming msg from {:?}", src);
            let wire_msg = WireMsg::from(bytes)?;
            let msg_type = wire_msg.into_msg()?;

            #[cfg(feature = "traceroute")]
            {
                info!(
                    "Message {} with the Traceroute received at client:\n {:?}",
                    msg_type,
                    wire_msg.traceroute()
                )
            }

            Ok(Some(msg_type))
        } else {
            Ok(None)
        }
    }

    #[instrument(skip_all, level = "debug")]
    pub(crate) async fn handle_msg(
        msg: MsgType,
        src_peer: Peer,
        mut session: Self,
    ) -> Result<(), Error> {
        match msg.clone() {
            MsgType::Client { msg_id, msg, .. } => {
                Self::handle_client_msg(session, msg_id, msg, src_peer)
            }
            MsgType::Node { msg, .. } => session.handle_system_msg(msg, src_peer).await,
        }
    }

    async fn handle_system_msg(&mut self, msg: NodeMsg, sender: Peer) -> Result<(), Error> {
        match msg {
            NodeMsg::AntiEntropy {
                section_tree_update,
                kind:
                    AntiEntropyKind::Redirect { bounced_msg } | AntiEntropyKind::Retry { bounced_msg },
            } => {
                debug!("AE-Redirect/Retry msg received");
                let result = self
                    .handle_ae_msg(section_tree_update, bounced_msg, sender)
                    .await;
                if result.is_err() {
                    error!("Failed to handle AE msg from {sender:?}, {result:?}");
                }
                result
            }
            msg_type => {
                warn!("Unexpected msg type received: {:?}", msg_type);
                Ok(())
            }
        }
    }

    #[instrument(skip(cmds), level = "debug")]
    fn write_cmd_response(
        cmds: PendingCmdAcks,
        correlation_id: MsgId,
        src: SocketAddr,
        error: Option<ErrorMsg>,
    ) {
        if error.is_some() {
            debug!("CmdError was received for {correlation_id:?}: {:?}", error);
        }

        if let Some(mut received_acks) = cmds.get_mut(&correlation_id) {
            let acks = received_acks.value_mut();

            let _prior = acks.insert((src, error));
        } else {
            let received = DashSet::new();
            let _nonexistent = received.insert((src, error));
            let _non_prior = cmds.insert(correlation_id, Arc::new(received));
        }
    }

    // Handle msgs intended for client consumption (re: queries + cmds)
    #[instrument(skip(session), level = "debug")]
    fn handle_client_msg(
        session: Self,
        msg_id: MsgId,
        msg: ClientMsg,
        src_peer: Peer,
    ) -> Result<(), Error> {
        debug!(
            "ClientMsg with id {:?} received from {:?}",
            msg_id,
            src_peer.addr()
        );
        let queries = session.pending_queries.clone();
        let cmds = session.pending_cmds;

        let _handle = tokio::spawn(async move {
            match msg {
                ClientMsg::QueryResponse {
                    response,
                    correlation_id,
                } => {
                    trace!(
                        "ClientMsg with id {:?} is QueryResponse regarding correlation_id {:?} with response {:?}",
                        msg_id,
                        correlation_id,
                        response,
                    );
                    // Note that this doesn't remove the sender from here since multiple
                    // responses corresponding to the same msg ID might arrive.
                    // Once we are satisfied with the response this is channel is discarded in
                    // ConnectionManager::send_query

                    if let Ok(op_id) = response.operation_id() {
                        debug!("OpId of {msg_id:?} is {op_id:?}");
                        if let Some(entry) = queries.get_mut(&op_id) {
                            debug!("op id: {op_id:?} exists in pending queries...");
                            let received = entry.value();

                            debug!("inserting response : {response:?}");
                            // we can acutally have many responses per peer if they're different
                            // this could be a fail, and then an Ok aftewards from a different adult.
                            let _prior = received.insert((src_peer.addr(), response));

                            debug!("received now looks like: {:?}", received);
                        } else {
                            debug!("op id: {op_id:?} does not exist in pending queries...");
                            let received = DashSet::new();
                            let _prior = received.insert((src_peer.addr(), response));
                            let _prev = queries.insert(op_id, Arc::new(received));
                            debug!("op_id added :{op_id:?}")
                        }
                    } else {
                        warn!(
                            "Ignoring query response without operation id: {:?} {:?}",
                            msg_id, response
                        );
                    }
                }
                ClientMsg::CmdError {
                    error,
                    correlation_id,
                } => {
                    Self::write_cmd_response(cmds, correlation_id, src_peer.addr(), Some(error));
                }
                ClientMsg::CmdAck { correlation_id } => {
                    debug!(
                        "CmdAck was received with id {:?} regarding correlation_id {:?} from {:?}",
                        msg_id,
                        correlation_id,
                        src_peer.addr()
                    );
                    Self::write_cmd_response(cmds, correlation_id, src_peer.addr(), None);
                }
                _ => {
                    warn!("Ignoring unexpected msg type received: {:?}", msg);
                }
            };
        });

        Ok(())
    }

    // Handle Anti-Entropy Redirect or Retry msgs
    #[instrument(skip_all, level = "debug")]
    async fn handle_ae_msg(
        &mut self,
        section_tree_update: SectionTreeUpdate,
        bounced_msg: UsrMsgBytes,
        src_peer: Peer,
    ) -> Result<(), Error> {
        let target_sap = section_tree_update.signed_sap.value.clone();
        debug!("Received Anti-Entropy from {src_peer}, with SAP: {target_sap:?}");

        // Try to update our network knowledge first
        self.update_network_knowledge(section_tree_update, src_peer)
            .await;

        if let Some((msg_id, elders, service_msg, dst, auth)) =
            Self::new_target_elders(bounced_msg.clone(), &target_sap).await?
        {
            let ae_msg_src_name = src_peer.name();
            // here we send this to only one elder for each AE message we get in. We _should_ have one per elder we sent to.
            // deterministically send to most elder based upon sender
            let target_elder = elders
                .iter()
                .sorted_by(|lhs, rhs| ae_msg_src_name.cmp_distance(&lhs.name(), &rhs.name()))
                .cloned()
                .collect_vec()
                .pop();

            // there should always be one
            if let Some(elder) = target_elder {
                let payload = WireMsg::serialize_msg_payload(&service_msg)?;
                let wire_msg =
                    WireMsg::new_msg(msg_id, payload, AuthKind::Client(auth.into_inner()), dst);

                debug!("Resending original message on AE-Redirect with updated details. Expecting an AE-Retry next");

                self.send_msg(vec![elder], wire_msg, msg_id, false).await?;
            } else {
                error!("No elder determined for resending AE message");
            }
        }

        Ok(())
    }

    /// Update our network knowledge making sure proof chain validates the
    /// new SAP based on currently known remote section SAP or genesis key.
    async fn update_network_knowledge(
        &mut self,
        section_tree_update: SectionTreeUpdate,
        sender: Peer,
    ) {
        let sap = section_tree_update.signed_sap.value.clone();
        // Update our network PrefixMap based upon passed in knowledge
        match self.network.write().await.update(section_tree_update) {
            Ok(true) => {
                debug!(
                    "Anti-Entropy: updated remote section SAP updated for {:?}",
                    sap.prefix()
                );
            }
            Ok(false) => {
                debug!(
                    "Anti-Entropy: discarded SAP for {:?} since it's the same as the one in our records: {:?}",
                    sap.prefix(), sap
                );
            }
            Err(err) => {
                warn!(
                    "Anti-Entropy: failed to update remote section SAP and section DAG w/ err: {:?}",
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

    /// Checks AE cache to see if we should be forwarding this msg (and to whom)
    /// or if it has already been dealt with
    #[instrument(skip_all, level = "debug")]
    async fn new_target_elders(
        bounced_msg: UsrMsgBytes,
        received_auth: &SectionAuthorityProvider,
    ) -> Result<Option<(MsgId, Vec<Peer>, ClientMsg, Dst, AuthorityProof<ClientAuth>)>, Error> {
        let (msg_id, service_msg, dst, auth) = match WireMsg::deserialize(bounced_msg)? {
            MsgType::Client {
                msg_id,
                msg,
                auth,
                dst,
            } => (msg_id, msg, dst, auth),
            other => {
                warn!(
                    "Unexpected non-ClientMsg returned in AE-Redirect response: {:?}",
                    other
                );
                return Ok(None);
            }
        };

        trace!(
            "Bounced msg ({:?}) received in an AE response: {:?}",
            msg_id,
            service_msg
        );

        let (target_count, dst_address_of_bounced_msg) = match service_msg.clone() {
            ClientMsg::Cmd(cmd) => (at_least_one_correct_elder(), cmd.dst_name()),
            ClientMsg::Query(query) => (NUM_OF_ELDERS_SUBSET_FOR_QUERIES, query.variant.dst_name()),
            _ => {
                warn!(
                    "Invalid bounced msg {:?} received in AE response: {:?}. Msg is of invalid type",
                    msg_id,
                    service_msg
                );
                // Early return with random name as we will discard the msg at the caller func
                return Ok(None);
            }
        };

        let target_public_key;

        // We normally have received auth when we're in AE-Redirect
        let mut target_elders: Vec<_> = {
            target_public_key = received_auth.section_key();

            received_auth
                .elders_vec()
                .into_iter()
                .sorted_by(|lhs, rhs| {
                    dst_address_of_bounced_msg.cmp_distance(&lhs.name(), &rhs.name())
                })
                .take(target_count)
                .collect()
        };
        // shuffle so elders sent to is random for better availability
        target_elders.shuffle(&mut OsRng);

        // Let's rebuild the msg with the updated destination details
        let dst = Dst {
            name: dst.name,
            section_key: target_public_key,
        };

        if !target_elders.is_empty() {
            debug!(
                "Final target elders for resending {:?} : {:?} msg are {:?}",
                msg_id, service_msg, target_elders
            );
        }

        Ok(Some((msg_id, target_elders, service_msg, dst, auth)))
    }
}
