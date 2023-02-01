// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{MsgResponse, Session};
use crate::{Error, Result};

use sn_interface::{
    messaging::{
        data::{DataQuery, DataQueryVariant, QueryResponse},
        ClientAuth, Dst, MsgId, MsgKind, WireMsg,
    },
    types::{ChunkAddress, Peer},
};

use bytes::Bytes;
use rand::{rngs::OsRng, seq::SliceRandom};
use tokio::task::JoinSet;
use tracing::{debug, trace, warn};
use xor_name::XorName;

// Number of Elders subset to send queries to
#[cfg(not(feature = "query-happy-path"))]
pub(crate) const NUM_OF_ELDERS_SUBSET_FOR_QUERIES: usize = 3;
#[cfg(feature = "query-happy-path")]
pub(crate) const NUM_OF_ELDERS_SUBSET_FOR_QUERIES: usize = 1;

impl Session {
    /// Get DataSection elders details. Resort to own section if DataSection is not available.
    /// Takes a random subset (NUM_OF_ELDERS_SUBSET_FOR_QUERIES) of the avialable elders as targets
    pub(crate) async fn get_query_elders(
        &self,
        dst: XorName,
    ) -> Result<(bls::PublicKey, Vec<Peer>)> {
        let sap = self.network.read().await.closest(&dst, None).cloned();
        let (section_pk, mut elders) = if let Some(sap) = &sap {
            (sap.section_key(), sap.elders_vec())
        } else {
            return Err(Error::NoNetworkKnowledge(dst));
        };

        elders.shuffle(&mut OsRng);

        // We select the NUM_OF_ELDERS_SUBSET_FOR_QUERIES closest Elders we are querying
        let elders: Vec<_> = elders
            .into_iter()
            .take(NUM_OF_ELDERS_SUBSET_FOR_QUERIES)
            .collect();

        let elders_len = elders.len();
        if elders_len < NUM_OF_ELDERS_SUBSET_FOR_QUERIES && elders_len > 1 {
            return Err(Error::InsufficientElderConnections {
                connections: elders_len,
                required: NUM_OF_ELDERS_SUBSET_FOR_QUERIES,
            });
        }

        Ok((section_pk, elders))
    }

    #[instrument(
        skip(self, auth, payload),
        level = "debug",
        name = "session send query"
    )]
    #[allow(clippy::too_many_arguments)]
    /// Send a `ClientMsg` to the network awaiting for the response.
    pub(crate) async fn send_query(
        &self,
        query: DataQuery,
        auth: ClientAuth,
        payload: Bytes,
        dst_section_info: Option<(bls::PublicKey, Vec<Peer>)>,
    ) -> Result<QueryResponse> {
        let endpoint = self.endpoint.clone();

        let chunk_addr = if let DataQueryVariant::GetChunk(address) = query.variant {
            Some(address)
        } else {
            None
        };

        let dst = query.variant.dst_name();

        let (section_pk, elders) = if let Some(section_info) = dst_section_info {
            section_info
        } else {
            self.get_query_elders(dst).await?
        };

        let elders_len = elders.len();
        let msg_id = MsgId::new();

        debug!(
            "Sending query message {msg_id:?}, from {}, {query:?} to \
            the {elders_len} Elders closest to data name: {elders:?}",
            endpoint.local_addr(),
        );

        let dst = Dst {
            name: dst,
            section_key: section_pk,
        };
        let kind = MsgKind::Client(auth);
        let wire_msg = WireMsg::new_msg(msg_id, payload, kind, dst);

        let send_query_tasks = self.send_query_msg(elders.clone(), wire_msg).await?;

        // TODO:
        // We are now simply accepting the very first valid response we receive,
        // but we may want to revisit this to compare multiple responses and validate them,
        // similar to what we used to do up to the following commit:
        // https://github.com/maidsafe/sn_client/blob/9091a4f1f20565f25d3a8b00571cc80751918928/src/connection_manager.rs#L328
        //
        // For Chunk responses we already validate its hash matches the xorname requested from,
        // so we don't need more than one valid response to prevent from accepting invalid responses
        // from byzantine nodes, however for mutable data (non-Chunk responses) we will
        // have to review the approach.
        self.check_query_responses(msg_id, elders.clone(), chunk_addr, send_query_tasks)
            .await
    }

    async fn check_query_responses(
        &self,
        msg_id: MsgId,
        elders: Vec<Peer>,
        chunk_addr: Option<ChunkAddress>,
        mut send_query_tasks: JoinSet<MsgResponse>,
    ) -> Result<QueryResponse> {
        let mut discarded_responses: usize = 0;
        let mut error_response = None;
        let mut valid_response = None;
        let elders_len = elders.len();

        while let Some(msg_resp) = send_query_tasks.join_next().await {
            let (peer_address, response) = match msg_resp {
                Ok(MsgResponse::QueryResponse(src, resp)) => (src, resp),
                Ok(MsgResponse::CmdResponse(src, resp)) => {
                    debug!("Unexpected cmd response received from {src:?} for {msg_id:?} when awaiting a QueryResponse: {resp:?}");
                    discarded_responses += 1;
                    continue;
                }
                Ok(MsgResponse::Failure(src, error)) => {
                    debug!("Failure occurred with msg {msg_id:?} from {src:?}: {error:?}");
                    discarded_responses += 1;
                    continue;
                }
                Err(join_err) => {
                    warn!("Join failure occurred with msg {msg_id:?}: {join_err:?}");
                    continue;
                }
            };

            // let's see if we have a positive response...
            debug!("Response to {msg_id:?}: {response:?}");

            match *response {
                QueryResponse::GetChunk(Ok(chunk)) => {
                    if let Some(chunk_addr) = chunk_addr {
                        // We are dealing with Chunk query responses, thus we validate its hash
                        // matches its xorname, if so, we don't need to await for more responses
                        debug!("Chunk QueryResponse received is: {chunk:#?}");

                        if chunk_addr.name() == chunk.name() {
                            trace!("Valid Chunk received for {msg_id:?}");
                            valid_response = Some(QueryResponse::GetChunk(Ok(chunk)));
                            break;
                        } else {
                            // the Chunk content doesn't match its XorName,
                            // this is suspicious and it could be a byzantine node
                            warn!("We received an invalid Chunk response from one of the nodes for {msg_id:?}");
                            discarded_responses += 1;
                        }
                    }
                }
                QueryResponse::GetRegister(Err(_))
                | QueryResponse::ReadRegister(Err(_))
                | QueryResponse::GetRegisterPolicy(Err(_))
                | QueryResponse::GetRegisterOwner(Err(_))
                | QueryResponse::GetRegisterUserPermissions(Err(_))
                | QueryResponse::GetChunk(Err(_)) => {
                    debug!(
                        "QueryResponse error #{discarded_responses} for {msg_id:?} received \
                        from {peer_address:?} (but may be overridden by a non-error response \
                        from another elder): {:#?}",
                        &response
                    );
                    error_response = Some(*response);
                    discarded_responses += 1;
                }
                QueryResponse::GetRegister(Ok(ref register)) => {
                    debug!("okay got register from {peer_address:?}");
                    // TODO: properly merge all registers
                    if let Some(QueryResponse::GetRegister(Ok(prior_response))) = &valid_response {
                        if register.size() > prior_response.size() {
                            debug!("longer register");
                            // keep this new register
                            valid_response = Some(*response);
                        }
                    } else {
                        valid_response = Some(*response);
                    }
                }
                QueryResponse::ReadRegister(Ok(_)) => {
                    debug!("okay _read_ register from {peer_address:?}");
                    if valid_response.is_none() {
                        valid_response = Some(*response);
                    }
                }
                QueryResponse::SpentProofShares(Ok(ref spentproof_set)) => {
                    debug!("okay _read_ spentproofs from {peer_address:?}");
                    // TODO: properly merge all registers
                    if let Some(QueryResponse::SpentProofShares(Ok(prior_response))) =
                        &valid_response
                    {
                        if spentproof_set.len() > prior_response.len() {
                            debug!("longer spentproof response retrieved");
                            // keep this new register
                            valid_response = Some(*response);
                        }
                    } else {
                        valid_response = Some(*response);
                    }
                }
                response => {
                    // we got a valid response
                    valid_response = Some(response)
                }
            }
        }

        // we've looped over all responses...
        // if any are valid, lets return it
        if let Some(response) = valid_response {
            debug!("Valid response in!!!: {response:?}");
            return Ok(response);
            // otherwise, if we've got an error in
            // we can return that too
        } else if let Some(response) = error_response {
            if discarded_responses > elders_len / 2 {
                return Ok(response);
            }
        }

        Err(Error::NoResponse {
            msg_id,
            peers: elders,
        })
    }

    #[instrument(skip_all, level = "trace")]
    async fn send_query_msg(
        &self,
        nodes: Vec<Peer>,
        wire_msg: WireMsg,
    ) -> Result<JoinSet<MsgResponse>> {
        let msg_id = wire_msg.msg_id();
        debug!("---> Send msg {msg_id:?} going out.");
        let bytes = wire_msg.serialize()?;

        let mut tasks = JoinSet::new();

        for (peer_index, peer) in nodes.into_iter().enumerate() {
            let session = self.clone();
            let bytes = bytes.clone();

            let _abort_handle = tasks.spawn(async move {
                let mut connect_now = false;
                debug!("Trying to send msg {msg_id:?} to {peer:?}");
                loop {
                    let link = session
                        .peer_links
                        .get_or_create_link(&peer, connect_now, Some(msg_id))
                        .await;
                    match link.send_bi(bytes.clone(), msg_id).await {
                        Ok(recv_stream) => {
                            debug!(
                                "That's {msg_id:?} sent to {peer:?}... starting receive listener"
                            );
                            // let's listen for responses on the bi-stream
                            break session
                                .recv_stream_listener(msg_id, peer, peer_index, recv_stream)
                                .await;
                        }
                        Err(error) if !connect_now => {
                            // Let's retry (only once) to reconnect to this peer and send the msg.
                            error!(
                                "Failed to send {msg_id:?} to {peer:?} on a new \
                                bi-stream: {error:?}. Creating a new connection to retry once ..."
                            );
                            session.peer_links.remove_link_from_peer_links(&peer).await;
                            connect_now = true;
                            continue;
                        }
                        Err(error) => {
                            error!("Error sending {msg_id:?} bidi to {peer:?}: {error:?}");
                            session.peer_links.remove_link_from_peer_links(&peer).await;
                            break MsgResponse::Failure(
                                peer.addr(),
                                Error::FailedToInitateBiDiStream { msg_id, error },
                            );
                        }
                    }
                }
            });
        }

        Ok(tasks)
    }
}
