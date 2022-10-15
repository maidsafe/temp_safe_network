// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{messaging::NUM_OF_ELDERS_SUBSET_FOR_QUERIES, MsgResponse, Session};
use std::collections::BTreeSet;

use crate::{Error, Result};

use qp2p::{RecvStream, UsrMsgBytes};
use sn_interface::{
    at_least_one_correct_elder,
    messaging::{
        data::ClientMsg,
        system::{AntiEntropyKind, NodeMsg},
        AuthKind, AuthorityProof, ClientAuth, Dst, MsgId, MsgType, NodeMsgAuthority,
        SectionTreeUpdate, WireMsg,
    },
    network_knowledge::{NetworkKnowledge, SectionAuthorityProvider},
    types::{log_markers::LogMarker, Peer},
};

use itertools::Itertools;
use rand::{rngs::OsRng, seq::SliceRandom};
use std::net::SocketAddr;
use tokio::sync::mpsc;
use tracing::Instrument;

impl Session {
    #[instrument(skip_all, level = "debug")]
    pub(crate) async fn read_msg_from_recvstream(
        recv_stream: &mut RecvStream,
    ) -> Result<MsgType, Error> {
        let bytes = recv_stream.next().await?;
        let wire_msg = WireMsg::from(bytes)?;
        let msg_type = wire_msg.into_msg()?;

        #[cfg(feature = "traceroute")]
        {
            info!(
                "Message {msg_type} with the Traceroute received at client:\n {:?}",
                wire_msg.traceroute()
            )
        }

        Ok(msg_type)
    }

    // Spawn a task to wait for a single msg incoming on the provided RecvStream
    #[instrument(skip_all, level = "debug")]
    pub(crate) fn spawn_recv_stream_listener(
        mut session: Self,
        msg_id: MsgId,
        peer: Peer,
        mut recv_stream: RecvStream,
        resp_tx: mpsc::Sender<MsgResponse>,
    ) {
        let addr = peer.addr();
        debug!("Waiting for incoming msg on bi-stream from {peer:?}");

        let _handle = tokio::spawn(async move {
            match Self::read_msg_from_recvstream(&mut recv_stream).await {
                Ok(MsgType::Client { msg_id, msg, .. }) => {
                    Self::handle_client_msg(msg_id, msg, peer.addr(), resp_tx).await;
                }
                Ok(MsgType::Node {
                    msg, msg_authority, ..
                }) => {
                    if let Err(err) = session
                        .handle_system_msg(msg, msg_authority, peer, resp_tx)
                        .await
                    {
                        error!(
                            "Error while handling incoming system msg on bi-stream \
                            from {addr:?} for {msg_id:?}: {err:?}"
                        );
                    }
                }
                Err(error) => {
                    error!(
                        "Error while processing incoming msg on bi-stream \
                        from {addr:?} in response to {msg_id:?}: {error:?}"
                    );
                }
            }

            // TODO: ???? once we drop the stream, do we know the connection is closed ???
            trace!("{} to {}", LogMarker::ConnectionClosed, addr);
        })
        .in_current_span();
    }

    async fn handle_system_msg(
        &mut self,
        msg: NodeMsg,
        msg_authority: NodeMsgAuthority,
        src_peer: Peer,
        resp_tx: mpsc::Sender<MsgResponse>,
    ) -> Result<(), Error> {
        // Check that the message can be trusted w.r.t. our known keys
        let known_keys: BTreeSet<_> = self
            .network
            .read()
            .await
            .get_sections_dag()
            .keys()
            .cloned()
            .collect();

        if !NetworkKnowledge::verify_msg_section_key(&msg_authority, &msg, &known_keys) {
            warn!("Message with unknown section key has been dropped, from {src_peer:?}: {msg:?}");
            let pk = msg_authority.src_public_key();
            return Err(Error::UntrustedMessage(pk));
        }

        match msg {
            NodeMsg::AntiEntropy {
                section_tree_update,
                kind:
                    AntiEntropyKind::Redirect { bounced_msg } | AntiEntropyKind::Retry { bounced_msg },
            } => {
                debug!("AE-Redirect/Retry msg received");
                let result = self
                    .handle_ae_msg(section_tree_update, bounced_msg, src_peer, resp_tx)
                    .await;
                if result.is_err() {
                    error!("Failed to handle AE msg from {src_peer:?}, {result:?}");
                }
                result
            }
            msg_type => {
                warn!("Unexpected msg type received: {msg_type:?}");
                Ok(())
            }
        }
    }

    #[instrument(skip(resp_tx), level = "debug")]
    async fn write_msg_response(
        resp_tx: mpsc::Sender<MsgResponse>,
        correlation_id: MsgId,
        src_addr: SocketAddr,
        msg_resp: MsgResponse,
    ) {
        if let Err(err) = resp_tx.send(msg_resp).await {
            // this is not necessarily a problem, the receiver could have closed
            // the channel if enough responses were already received
            warn!(
                "Error reporting from response listener, response received from {src_addr:?} for \
                correlation_id {correlation_id:?}: {err:?}"
            );
        } else {
            debug!(
                "Response received from {src_addr:?} for correlation_id {correlation_id:?} \
                reported from listener"
            );
        }
    }

    // Handle msgs intended for client consumption (re: queries + cmds)
    #[instrument(skip(resp_tx), level = "debug")]
    async fn handle_client_msg(
        msg_id: MsgId,
        msg: ClientMsg,
        src_addr: SocketAddr,
        resp_tx: mpsc::Sender<MsgResponse>,
    ) {
        debug!("ClientMsg with id {msg_id:?} received from {src_addr:?}",);

        let (msg_resp, correlation_id) = match msg {
            ClientMsg::QueryResponse {
                response,
                correlation_id,
            } => {
                trace!(
                    "ClientMsg with id {msg_id:?} is QueryResponse regarding correlation_id \
                    {correlation_id:?} with response {response:?}"
                );

                let resp = MsgResponse::QueryResponse(src_addr, Box::new(response));
                (resp, correlation_id)
            }
            ClientMsg::CmdError {
                error,
                correlation_id,
            } => {
                debug!("CmdError was received for correlation_id {correlation_id:?}: {error:?}");

                let resp = MsgResponse::CmdResponse(src_addr, Some(error));
                (resp, correlation_id)
            }
            ClientMsg::CmdAck { correlation_id } => {
                debug!(
                    "CmdAck was received with id {msg_id:?} regarding correlation_id \
                    {correlation_id:?} from {src_addr:?}"
                );

                let resp = MsgResponse::CmdResponse(src_addr, None);
                (resp, correlation_id)
            }
            _ => {
                warn!("Ignoring unexpected msg type received: {msg:?}");
                return;
            }
        };

        Self::write_msg_response(resp_tx, correlation_id, src_addr, msg_resp).await;
    }

    // Handle Anti-Entropy Redirect or Retry msgs
    #[instrument(skip_all, level = "debug")]
    async fn handle_ae_msg(
        &mut self,
        section_tree_update: SectionTreeUpdate,
        bounced_msg: UsrMsgBytes,
        src_peer: Peer,
        resp_tx: mpsc::Sender<MsgResponse>,
    ) -> Result<(), Error> {
        let target_sap = section_tree_update.signed_sap().value;
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

                debug!(
                    "Resending original message on AE-Redirect with updated details. \
                    Expecting an AE-Retry next"
                );
                self.send_msg(vec![elder], wire_msg, msg_id, false, resp_tx)
                    .await?;
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
        src_peer: Peer,
    ) {
        let sap = section_tree_update.signed_sap().value;
        // Update our network SectionTree based upon passed in knowledge
        match self.network.write().await.update(section_tree_update) {
            Ok(true) => {
                debug!(
                    "Anti-Entropy: updated remote section SAP updated for {:?}",
                    sap.prefix()
                );
            }
            Ok(false) => {
                debug!(
                    "Anti-Entropy: discarded SAP for {:?} since it's the same as \
                    the one in our records: {sap:?}",
                    sap.prefix()
                );
            }
            Err(err) => {
                warn!(
                    "Anti-Entropy: failed to update remote section SAP and section DAG w/ err: {err:?}"
                );
                warn!(
                    "Anti-Entropy: bounced msg dropped. Failed section auth was {:?} sent by: {src_peer:?}",
                    sap.section_key(),
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
                warn!("Unexpected non-ClientMsg returned in AE-Redirect response: {other:?}");
                return Ok(None);
            }
        };

        trace!("Bounced msg ({msg_id:?}) received in an AE response: {service_msg:?}");

        let (target_count, dst_address_of_bounced_msg) = match service_msg.clone() {
            ClientMsg::Cmd(cmd) => (at_least_one_correct_elder(), cmd.dst_name()),
            ClientMsg::Query(query) => (NUM_OF_ELDERS_SUBSET_FOR_QUERIES, query.variant.dst_name()),
            _ => {
                warn!(
                    "Invalid bounced msg {msg_id:?} received in AE response: {service_msg:?}. Msg is of invalid type"
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
                "Final target elders for resending {msg_id:?}: {service_msg:?} msg \
                are {target_elders:?}"
            );
        }

        Ok(Some((msg_id, target_elders, service_msg, dst, auth)))
    }
}
