// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{QueryResult, Session};

use super::AeCache;
use crate::client::Error;
use crate::messaging::{
    data::{CmdError, DataQuery, QueryResponse},
    signature_aggregator::SignatureAggregator,
    DstLocation, MessageId, MsgKind, ServiceAuth, WireMsg,
};
use crate::prefix_map::NetworkPrefixMap;
use crate::types::PublicKey;

use bytes::Bytes;
use futures::{future::join_all, stream::FuturesUnordered, TryFutureExt};
use itertools::Itertools;
use qp2p::{Config as QuicP2pConfig, Endpoint};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    net::SocketAddr,
    sync::Arc,
};
use tokio::{
    sync::mpsc::{channel, Sender},
    sync::RwLock,
    task::JoinHandle,
};
use tracing::{debug, error, trace, warn};
use xor_name::XorName;

// Number of Elders subset to send queries to
pub(crate) const NUM_OF_ELDERS_SUBSET_FOR_QUERIES: usize = 3;
// Number of attempts to make when trying to bootstrap to a section
const NUM_OF_BOOTSTRAPPING_ATTEMPTS: u8 = 3;

impl Session {
    /// Acquire a session by bootstrapping to a section, maintaining connections to several nodes.
    pub(crate) async fn bootstrap(
        client_pk: PublicKey,
        genesis_key: bls::PublicKey,
        qp2p_config: QuicP2pConfig,
        err_sender: Sender<CmdError>,
        bootstrap_nodes: BTreeSet<SocketAddr>,
        local_addr: SocketAddr,
    ) -> Result<Session, Error> {
        trace!(
            "Trying to bootstrap to the network with public_key: {:?}",
            client_pk
        );
        debug!("QP2p config: {:?}", qp2p_config);

        let (endpoint, incoming_messages, _) = Endpoint::new_client(local_addr, qp2p_config)?;
        let bootstrap_nodes = bootstrap_nodes.iter().copied().collect_vec();
        let bootstrap_peer = endpoint
            .connect_to_any(&bootstrap_nodes)
            .await
            .ok_or(Error::NotBootstrapped)?;

        let session = Session {
            client_pk,
            pending_queries: Arc::new(RwLock::new(HashMap::default())),
            incoming_err_sender: Arc::new(err_sender),
            endpoint,
            network: Arc::new(NetworkPrefixMap::new(genesis_key)),
            ae_cache: Arc::new(RwLock::new(AeCache::default())),
            aggregator: Arc::new(RwLock::new(SignatureAggregator::new())),
            bootstrap_peer: bootstrap_peer.remote_address(),
            genesis_key,
        };

        Self::spawn_message_listener_thread(session.clone(), incoming_messages).await;

        Ok(session)
    }

    /// Tries to bootstrap a client to a section. If there is a failure then it retries.
    /// After a maximum of three attempts if the boostrap process still fails, the unresponsive
    /// node is removed from the list and an error is returned.
    pub(crate) async fn attempt_bootstrap(
        client_pk: PublicKey,
        genesis_key: bls::PublicKey,
        qp2p_config: qp2p::Config,
        mut bootstrap_nodes: BTreeSet<SocketAddr>,
        local_addr: SocketAddr,
        err_sender: Sender<CmdError>,
    ) -> Result<Session, Error> {
        let mut attempts = 0;
        loop {
            match Session::bootstrap(
                client_pk,
                genesis_key,
                qp2p_config.clone(),
                err_sender.clone(),
                bootstrap_nodes.clone(),
                local_addr,
            )
            .await
            {
                Ok(session) => break Ok(session),
                Err(err) => {
                    attempts += 1;
                    if let Error::BootstrapToPeerFailed(failed_peer) = err {
                        // Remove the unresponsive peer we boostrapped to and bootstrap again
                        let _ = bootstrap_nodes.remove(&failed_peer);
                    }
                    if attempts < NUM_OF_BOOTSTRAPPING_ATTEMPTS {
                        trace!(
                            "Error connecting to network! {:?}\nRetrying... ({})",
                            err,
                            attempts
                        );
                    } else {
                        break Err(err);
                    }
                }
            }
        }
    }

    /// Send a `ServiceMsg` to the network without awaiting for a response.
    pub(crate) async fn send_cmd(
        &self,
        dst_address: XorName,
        auth: ServiceAuth,
        payload: Bytes,
        targets: usize,
    ) -> Result<(), Error> {
        let endpoint = self.endpoint.clone();

        // TODO: Consider other approach: Keep a session per section!

        // Get DataSection elders details.
        let (elders, section_pk) = if let Some(sap) = self.network.closest_or_opposite(&dst_address)
        {
            (
                sap.value
                    .elders
                    .values()
                    .cloned()
                    .take(targets)
                    .collect::<Vec<SocketAddr>>(),
                sap.value.public_key_set.public_key(),
            )
        } else {
            // Send message to our bootstrap peer with network's genesis PK.
            (vec![self.bootstrap_peer], self.genesis_key)
        };

        let msg_id = MessageId::new();

        debug!(
            "Sending command w/id {:?}, from {}, to {} Elders",
            msg_id,
            endpoint.public_addr(),
            elders.len()
        );

        trace!(
            "Sending (from {}) dst {:?} w/ id: {:?}",
            endpoint.public_addr(),
            dst_address,
            msg_id
        );

        let dst_location = DstLocation::Section {
            name: dst_address,
            section_pk,
        };
        let msg_kind = MsgKind::ServiceMsg(auth);
        let wire_msg = WireMsg::new_msg(msg_id, payload, msg_kind, dst_location)?;

        send_message(elders.clone(), wire_msg, self.endpoint.clone(), msg_id).await
    }

    /// Send a `ServiceMsg` to the network awaiting for the response.
    pub(crate) async fn send_query(
        &self,
        query: DataQuery,
        auth: ServiceAuth,
        payload: Bytes,
    ) -> Result<QueryResult, Error> {
        let endpoint = self.endpoint.clone();
        let pending_queries = self.pending_queries.clone();

        let chunk_addr = if let DataQuery::GetChunk(address) = query {
            Some(address)
        } else {
            None
        };

        let dst = query.dst_name();

        // Get DataSection elders details. Resort to own section if DataSection is not available.
        let (elders, section_pk) = if let Some(sap) = self.network.closest_or_opposite(&dst) {
            (sap.value.elders, sap.value.public_key_set.public_key())
        } else {
            let mut bootstrapped_peer = BTreeMap::new();
            let _ = bootstrapped_peer.insert(XorName::random(), self.bootstrap_peer);
            // Send message to our bootstrap peer with the network's genesis PK.
            (bootstrapped_peer, self.genesis_key)
        };

        // We select the NUM_OF_ELDERS_SUBSET_FOR_QUERIES closest Elders we are querying
        let chosen_elders = elders
            .into_iter()
            .sorted_by(|(lhs_name, _), (rhs_name, _)| dst.cmp_distance(lhs_name, rhs_name))
            .map(|(_, addr)| addr)
            .take(NUM_OF_ELDERS_SUBSET_FOR_QUERIES)
            .collect::<Vec<SocketAddr>>();

        let elders_len = chosen_elders.len();
        if elders_len < NUM_OF_ELDERS_SUBSET_FOR_QUERIES && elders_len > 1 {
            error!(
                "Not enough Elder connections: {}, minimum required: {}",
                elders_len, NUM_OF_ELDERS_SUBSET_FOR_QUERIES
            );
            return Err(Error::InsufficientElderConnections(elders_len));
        }

        let msg_id = MessageId::new();

        debug!(
            "Sending query message {:?}, msg_id: {:?}, from {}, to the {} Elders closest to data name: {:?}",
            query,
            msg_id,
            endpoint.public_addr(),
            elders_len,
            chosen_elders
        );

        // We send the same message to all Elders concurrently
        let tasks = FuturesUnordered::new();
        let (sender, mut receiver) = channel::<QueryResponse>(7);

        let pending_queries_for_thread = pending_queries.clone();
        if let Ok(op_id) = query.operation_id() {
            let _ = tokio::spawn(async move {
                // Insert the response sender
                trace!("Inserting channel for {:?}", op_id);
                let _old = pending_queries_for_thread
                    .write()
                    .await
                    .insert(op_id.clone(), sender);

                trace!("Inserted channel for {:?}", op_id);
            });
        } else {
            warn!("No op_id found for query");
        }

        let discarded_responses = std::sync::Arc::new(tokio::sync::Mutex::new(0_usize));

        let dst_location = DstLocation::Section {
            name: dst,
            section_pk,
        };
        let msg_kind = MsgKind::ServiceMsg(auth);
        let wire_msg = WireMsg::new_msg(msg_id, payload, msg_kind, dst_location)?;
        let priority = wire_msg.msg_kind().priority();
        let msg_bytes = wire_msg.serialize()?;

        // Set up response listeners
        for socket in chosen_elders.clone() {
            let endpoint = endpoint.clone();
            let msg_bytes = msg_bytes.clone();
            let counter_clone = discarded_responses.clone();
            let task_handle = tokio::spawn(async move {
                trace!("queueing query send task to: {:?}", &socket);
                let result = endpoint
                    .connect_to(&socket)
                    .err_into()
                    .and_then(|connection| async move {
                        connection.send_with(msg_bytes, priority, None).await
                    })
                    .await;
                match &result {
                    Err(err) => {
                        error!("Error sending Query to elder: {:?} ", err);
                        let mut a = counter_clone.lock().await;
                        *a += 1;
                    }
                    Ok(()) => trace!("ServiceMsg with id: {:?}, sent to {}", &msg_id, &socket),
                }
                result
            });

            tasks.push(task_handle);
        }

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
        let mut discarded_responses: usize = 0;

        // Send all queries concurrently
        let results = join_all(tasks).await;

        for result in results {
            if let Err(err) = result {
                error!("Error spawning task to send query: {:?} ", err);
                discarded_responses += 1;
            }
        }

        let response = loop {
            let mut error_response = None;
            match (receiver.recv().await, chunk_addr) {
                (Some(QueryResponse::GetChunk(Ok(chunk))), Some(chunk_addr)) => {
                    // We are dealing with Chunk query responses, thus we validate its hash
                    // matches its xorname, if so, we don't need to await for more responses
                    debug!("Chunk QueryResponse received is: {:#?}", chunk);

                    if chunk_addr.name() == chunk.name() {
                        trace!("Valid Chunk received for {}", msg_id);
                        break Some(QueryResponse::GetChunk(Ok(chunk)));
                    } else {
                        // the Chunk content doesn't match its XorName,
                        // this is suspicious and it could be a byzantine node
                        warn!("We received an invalid Chunk response from one of the nodes");
                        discarded_responses += 1;
                    }
                }
                // Erring on the side of positivity. \
                // Saving error, but not returning until we have more responses in
                // (note, this will overwrite prior errors, so we'll just return whichever was last received)
                (response @ Some(QueryResponse::GetChunk(Err(_))), Some(_))
                | (response @ Some(QueryResponse::GetRegister((Err(_), _))), None)
                | (response @ Some(QueryResponse::GetRegisterPolicy((Err(_), _))), None)
                | (response @ Some(QueryResponse::GetRegisterOwner((Err(_), _))), None)
                | (response @ Some(QueryResponse::GetRegisterUserPermissions((Err(_), _))), None) =>
                {
                    debug!("QueryResponse error received (but may be overridden by a non-error response from another elder): {:#?}", &response);
                    error_response = response;
                    discarded_responses += 1;
                }
                (Some(response), _) => {
                    debug!("QueryResponse received is: {:#?}", response);
                    break Some(response);
                }
                (None, _) => {
                    debug!("QueryResponse channel closed.");
                    break None;
                }
            }
            if discarded_responses == elders_len {
                break error_response;
            }
        };

        debug!(
            "Response obtained for query w/id {:?}: {:?}",
            msg_id, response
        );

        if let Some(query) = &response {
            if let Ok(query_op_id) = query.operation_id() {
                let _ = tokio::spawn(async move {
                    // Remove the response sender
                    trace!("Removing channel for {:?}", query_op_id);
                    let _old_channel = pending_queries.clone().write().await.remove(&query_op_id);
                });
            }
        }

        match response {
            Some(response) => {
                let operation_id = response
                    .operation_id()
                    .map_err(|_| Error::UnknownOperationId)?;
                Ok(QueryResult {
                    response,
                    operation_id,
                })
            }
            None => Err(Error::NoResponse),
        }
    }

    #[allow(unused)]
    pub(crate) async fn disconnect_from_peers(&self, peers: Vec<SocketAddr>) -> Result<(), Error> {
        for elder in peers {
            self.endpoint.disconnect_from(&elder).await;
        }
        Ok(())
    }
}

pub(crate) async fn send_message(
    elders: Vec<SocketAddr>,
    wire_msg: WireMsg,
    endpoint: Endpoint<XorName>,
    msg_id: MessageId,
) -> Result<(), Error> {
    let priority = wire_msg.msg_kind().priority();
    let msg_bytes = wire_msg.serialize()?;

    // Send message to all Elders concurrently
    let mut tasks = Vec::default();

    // clone elders as we want to update them in this process
    for socket in elders.clone() {
        let msg_bytes_clone = msg_bytes.clone();
        let endpoint = endpoint.clone();
        let task_handle: JoinHandle<Result<(), Error>> = tokio::spawn(async move {
            trace!("About to send cmd message {:?} to {:?}", msg_id, &socket);
            endpoint
                .connect_to(&socket)
                .err_into()
                .and_then(|connection| async move {
                    connection.send_with(msg_bytes_clone, priority, None).await
                })
                .await?;

            trace!("Sent cmd with MsgId {:?} to {:?}", msg_id, &socket);
            Ok(())
        });
        tasks.push(task_handle);
    }

    // Let's await for all messages to be sent
    let results = join_all(tasks).await;

    let mut failures = 0;
    results.iter().for_each(|res| {
        if res.is_err() {
            error!("Client error contacting node was: {:?}", res);
            failures += 1;
        }
    });

    if failures > 0 {
        error!(
            "Sending the message to {}/{} of the elders failed",
            failures,
            elders.len()
        );
    }

    Ok(())
}
