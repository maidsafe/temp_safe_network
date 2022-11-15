// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{MsgResponse, Session};

use crate::{Error, Result};

use qp2p::{RecvStream, UsrMsgBytes};
use sn_interface::{
    messaging::{
        data::{ClientMsg, ClientMsgResponse},
        system::{AntiEntropyKind, NodeMsg},
        AuthorityProof, ClientAuth, Dst, MsgId, MsgKind, MsgType, WireMsg,
    },
    network_knowledge::{SectionAuthorityProvider, SectionTreeUpdate},
    types::{log_markers::LogMarker, Peer},
};

use itertools::Itertools;
use rand::{rngs::OsRng, seq::SliceRandom};
use xor_name::XorName;

// Maximum number of times we'll re-send a msg upon receiving an AE response for it
const MAX_AE_RETRIES_TO_ATTEMPT: u8 = 5;

// If the msg was resent due to AE response, we internally pass the information
// about where the msg was resent to, and the bi-stream to read the response on.
struct MsgResent {
    new_peer: Peer,
    new_recv_stream: RecvStream,
}

impl Session {
    #[instrument(skip_all, level = "debug")]
    async fn read_msg_from_recvstream(recv_stream: &mut RecvStream) -> Result<MsgType, Error> {
        let bytes = recv_stream.next().await?;
        let wire_msg = WireMsg::from(bytes)?;
        let msg_type = wire_msg.into_msg()?;

        Ok(msg_type)
    }

    // Wait for a msg response incoming on the provided RecvStream
    #[instrument(skip_all, level = "debug")]
    pub(crate) async fn recv_stream_listener(
        &self,
        correlation_id: MsgId,
        mut peer: Peer,
        peer_index: usize,
        mut recv_stream: RecvStream,
    ) -> MsgResponse {
        // Unless we receive AntiEntropy responses, which require re-sending the
        // message, the first msg received is the response we expect and return
        let mut attempt = 0;
        let result = loop {
            let addr = peer.addr();
            if attempt > MAX_AE_RETRIES_TO_ATTEMPT {
                break MsgResponse::Failure(
                    addr,
                    Error::AntiEntropyMaxRetries {
                        msg_id: correlation_id,
                        retries: attempt - 1,
                    },
                );
            }

            let stream_id = recv_stream.id();
            debug!("Waiting for response msg on {stream_id} from {peer:?} for {correlation_id:?}, attempt #{attempt}");

            match Self::read_msg_from_recvstream(&mut recv_stream).await {
                Ok(MsgType::ClientMsgResponse { msg_id, msg, .. }) => {
                    break Self::handle_client_msg(msg_id, msg, peer, correlation_id).await;
                }
                Ok(MsgType::Node { msg_id, msg, .. }) => match self
                    .handle_system_msg(msg_id, msg, peer, peer_index, correlation_id)
                    .await
                {
                    Ok(MsgResent {
                        new_peer,
                        new_recv_stream,
                    }) => {
                        recv_stream = new_recv_stream;
                        trace!("{} to {}", LogMarker::StreamClosed, addr);
                        peer = new_peer;
                        attempt += 1;
                        continue;
                    }
                    Err(err) => break MsgResponse::Failure(addr, err),
                },
                Ok(msg @ MsgType::Client { .. }) => {
                    warn!("Unexpected ClientMsg type received for {correlation_id:?}: {msg:?}");
                    break MsgResponse::Failure(
                        addr,
                        Error::UnexpectedMsgType {
                            correlation_id,
                            peer,
                            msg,
                        },
                    );
                }
                Err(err) => break MsgResponse::Failure(addr, err),
            }
        };

        trace!("{} to {}", LogMarker::StreamClosed, peer.addr());
        result
    }

    async fn handle_system_msg(
        &self,
        msg_id: MsgId,
        msg: NodeMsg,
        src_peer: Peer,
        src_peer_index: usize,
        correlation_id: MsgId,
    ) -> Result<MsgResent> {
        match msg {
            NodeMsg::AntiEntropy {
                section_tree_update,
                kind:
                    AntiEntropyKind::Redirect { bounced_msg } | AntiEntropyKind::Retry { bounced_msg },
            } => {
                debug!("AE Redirect/Retry msg with id {msg_id:?} received for {correlation_id:?}");
                self.handle_ae_msg(
                    section_tree_update,
                    bounced_msg,
                    src_peer,
                    src_peer_index,
                    correlation_id,
                )
                .await
            }
            other_msg => {
                warn!("Unexpected NodeMsg type with id {msg_id:?} received for {correlation_id:?}: {other_msg:?}");
                Err(Error::UnexpectedNodeMsg {
                    correlation_id,
                    peer: src_peer,
                    msg: other_msg,
                })
            }
        }
    }

    // Handle msgs intended for client consumption (re: queries + cmds)
    #[instrument(level = "debug")]
    async fn handle_client_msg(
        msg_id: MsgId,
        msg: ClientMsgResponse,
        src_peer: Peer,
        correlation_id: MsgId,
    ) -> MsgResponse {
        let src_addr = src_peer.addr();
        debug!("ClientMsg with id {msg_id:?} received from {src_addr:?}",);

        match msg {
            ClientMsgResponse::QueryResponse {
                response,
                correlation_id,
            } => {
                trace!(
                    "ClientMsgResponse with id {msg_id:?} is QueryResponse regarding correlation_id \
                    {correlation_id:?} with response {response:?}"
                );
                MsgResponse::QueryResponse(src_addr, Box::new(response))
            }
            ClientMsgResponse::CmdResponse {
                response,
                correlation_id,
            } => {
                trace!(
                    "ClientMsgResponse with id {msg_id:?} is CmdAck regarding correlation_id \
                    {correlation_id:?} with response {response:?}"
                );
                MsgResponse::CmdResponse(src_addr, Box::new(response))
            }
        }
    }

    // Handle Anti-Entropy Redirect or Retry msgs
    #[instrument(skip_all, level = "debug")]
    async fn handle_ae_msg(
        &self,
        section_tree_update: SectionTreeUpdate,
        bounced_msg: UsrMsgBytes,
        src_peer: Peer,
        src_peer_index: usize,
        correlation_id: MsgId,
    ) -> Result<MsgResent> {
        let target_sap = section_tree_update.signed_sap.value.clone();
        debug!("Received Anti-Entropy from {src_peer}, with SAP: {target_sap:?}");

        // Try to update our network knowledge first
        self.update_network_knowledge(section_tree_update, src_peer)
            .await;

        let (msg_id, elders, service_msg, dst, auth) =
            Self::new_target_elders(src_peer, bounced_msg.clone(), &target_sap, correlation_id)
                .await?;

        debug!("{msg_id:?} AE bounced msg going out again. Resending original message (sent to {src_peer:?}) to new section eldere");

        // The actual order of elders doesn't really matter. All that matters is we pass each AE response
        // we get through the same hoops, to then be able to ping a new elder on a 1-1 basis for the src_peer
        // we initially targetted.
        let deterministic_ordering = XorName::from_content(
            b"Arbitrary string that we use to sort new SAP elders consistently",
        );

        // here we send this to only one elder for each AE message we get in. We _should_ have one per elder we sent to.
        // deterministically sent to closest elder based upon the initial sender index
        let ordered_elders = elders
            .iter()
            .sorted_by(|lhs, rhs| deterministic_ordering.cmp_distance(&lhs.name(), &rhs.name()))
            .cloned()
            .collect_vec();

        let target_elder = ordered_elders.get(src_peer_index);

        // there should always be one
        if let Some(elder) = target_elder {
            let payload = WireMsg::serialize_msg_payload(&service_msg)?;
            let wire_msg =
                WireMsg::new_msg(msg_id, payload, MsgKind::Client(auth.into_inner()), dst);
            let bytes = wire_msg.serialize()?;

            debug!("Resending original message {msg_id:?} received on AE Redirect/Retry with updated details.");

            let link = self.peer_links.get_or_create_link(elder, false).await;
            let new_recv_stream = link
                .send_bi(bytes, msg_id)
                .await
                .map_err(|_| Error::FailedToInitateBiDiStream(msg_id))?;

            Ok(MsgResent {
                new_peer: *elder,
                new_recv_stream,
            })
        } else {
            Err(Error::AntiEntropyNoSapElders)
        }
    }

    /// Update our network knowledge making sure proof chain validates the
    /// new SAP based on currently known remote section SAP or genesis key.
    async fn update_network_knowledge(
        &self,
        section_tree_update: SectionTreeUpdate,
        src_peer: Peer,
    ) {
        debug!("Attempting to update our knowledge...");
        let sap = section_tree_update.signed_sap.value.clone();
        let mut network = self.network.write().await;
        debug!("Attempting to update our knowledge... WRITE LOCK GOT");
        // Update our network SectionTree based upon passed in knowledge
        match network.update(section_tree_update) {
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
    #[allow(clippy::type_complexity)]
    async fn new_target_elders(
        src_peer: Peer,
        bounced_msg: UsrMsgBytes,
        received_auth: &SectionAuthorityProvider,
        correlation_id: MsgId,
    ) -> Result<(MsgId, Vec<Peer>, ClientMsg, Dst, AuthorityProof<ClientAuth>), Error> {
        let (msg_id, service_msg, dst, auth) = match WireMsg::deserialize(bounced_msg)? {
            MsgType::Client {
                msg_id,
                msg,
                auth,
                dst,
            } => (msg_id, msg, dst, auth),
            msg @ MsgType::ClientMsgResponse { .. } | msg @ MsgType::Node { .. } => {
                warn!("Unexpected bounced msg received in AE response: {msg:?}");
                return Err(Error::UnexpectedMsgType {
                    correlation_id,
                    peer: src_peer,
                    msg,
                });
            }
        };

        trace!("Bounced msg {msg_id:?} received in an AE response: {service_msg:?}");
        let dst_address_of_bounced_msg = match service_msg.clone() {
            ClientMsg::Cmd(cmd) => cmd.dst_name(),
            ClientMsg::Query(query) => query.variant.dst_name(),
        };

        let target_public_key = received_auth.section_key();

        // We normally have received auth when we're in AE-Redirect
        let mut target_elders: Vec<_> = received_auth
            .elders_vec()
            .into_iter()
            .sorted_by(|lhs, rhs| dst_address_of_bounced_msg.cmp_distance(&lhs.name(), &rhs.name()))
            .collect();

        if target_elders.is_empty() {
            Err(Error::AntiEntropyNoSapElders)
        } else {
            // shuffle so elders sent to is random for better availability
            target_elders.shuffle(&mut OsRng);

            // Let's rebuild the msg with the updated destination details
            let dst = Dst {
                name: dst.name,
                section_key: target_public_key,
            };
            debug!(
                "Final target elders for resending {msg_id:?}: {service_msg:?} msg \
                are {target_elders:?}"
            );
            Ok((msg_id, target_elders, service_msg, dst, auth))
        }
    }
}
