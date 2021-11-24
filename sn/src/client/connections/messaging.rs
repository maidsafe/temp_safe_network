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
use crate::types::utils::write_data_to_disk;
use crate::types::PublicKey;
use bytes::Bytes;
use futures::{future::join_all, stream::FuturesUnordered, TryFutureExt};
use itertools::Itertools;
use qp2p::{Config as QuicP2pConfig, Endpoint};
use rand::rngs::OsRng;
use rand::seq::SliceRandom;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::{
    sync::mpsc::{channel, Sender},
    sync::RwLock,
    task::JoinHandle,
};
use tracing::{debug, error, trace, warn, Instrument};
use xor_name::XorName;

// Number of Elders subset to send queries to
pub(crate) const NUM_OF_ELDERS_SUBSET_FOR_QUERIES: usize = 3;

// Number of bootstrap nodes to attempt to contact per batch (if provided by the node_config)
pub(crate) const NODES_TO_CONTACT_PER_STARTUP_BATCH: usize = 3;

// Root directory for Clients
pub(crate) const SAFE_CLIENT_DIR: &str = ".safe/client";

impl Session {
    /// Acquire a session by bootstrapping to a section, maintaining connections to several nodes.
    #[instrument(skip(err_sender), level = "debug")]
    pub(crate) async fn new(
        client_pk: PublicKey,
        genesis_key: bls::PublicKey,
        qp2p_config: QuicP2pConfig,
        err_sender: Sender<CmdError>,
        local_addr: SocketAddr,
        standard_wait: Duration,
        prefix_map: NetworkPrefixMap,
    ) -> Result<Session, Error> {
        trace!("Trying to bootstrap to the network");

        let endpoint = Endpoint::new_client(local_addr, qp2p_config)?;

        // Create client's root dir
        let root_dir = create_client_root_dir(client_pk).await?;

        // Write our PrefixMap to disk in our root dir
        write_data_to_path(&prefix_map, &root_dir.join("prefix_map")).await?;

        let session = Session {
            client_pk,
            pending_queries: Arc::new(RwLock::new(HashMap::default())),
            incoming_err_sender: Arc::new(err_sender),
            endpoint,
            network: Arc::new(prefix_map),
            ae_redirect_cache: Arc::new(RwLock::new(AeCache::default())),
            ae_retry_cache: Arc::new(RwLock::new(AeCache::default())),
            aggregator: Arc::new(RwLock::new(SignatureAggregator::new())),
            genesis_key,
            initial_connection_check_msg_id: Arc::new(RwLock::new(None)),
            standard_wait,
            root_dir,
        };

        Ok(session)
    }

    /// Send a `ServiceMsg` to the network without awaiting for a response.
    #[instrument(skip(self, auth, payload), level = "debug", name = "session send cmd")]
    pub(crate) async fn send_cmd(
        &self,
        dst_address: XorName,
        auth: ServiceAuth,
        payload: Bytes,
        targets_count: usize,
    ) -> Result<(), Error> {
        let endpoint = self.endpoint.clone();
        // TODO: Consider other approach: Keep a session per section!

        // Get DataSection elders details.
        let (elders, section_pk) =
            if let Some(sap) = self.network.closest_or_opposite(&dst_address, None) {
                let sap_elders: Vec<_> = sap
                    .elders()
                    .map(|elder| elder.addr())
                    .take(targets_count)
                    .collect();

                trace!("{:?} SAP elders found", sap_elders);
                (sap_elders, sap.section_key())
            } else {
                return Err(Error::NoNetworkKnowledge);
            };

        let msg_id = MessageId::new();

        if elders.len() < targets_count {
            return Err(Error::InsufficientElderConnections(
                elders.len(),
                targets_count,
            ));
        }

        debug!(
            "Sending command w/id {:?}, from {}, to {} Elders w/ dst: {:?}",
            msg_id,
            endpoint.public_addr(),
            elders.len(),
            dst_address
        );

        let dst_location = DstLocation::Section {
            name: dst_address,
            section_pk,
        };
        let msg_kind = MsgKind::ServiceMsg(auth);
        let wire_msg = WireMsg::new_msg(msg_id, payload, msg_kind, dst_location)?;

        let res = send_message(
            self.clone(),
            elders.clone(),
            wire_msg,
            self.endpoint.clone(),
            msg_id,
        )
        .await;

        // lets wait for any potential AE response while we're here.
        // TODO: be smart about this. Check AE Retry cache for related msg id eg, continue early if we've seen some.
        // (cannot continue earlier if everything goes okay first time though, which is a shame)
        tokio::time::sleep(self.standard_wait).await;

        trace!("Wait for any cmd response/reaction (AE msgs eg), is over)");
        res
    }

    #[instrument(skip_all, level = "debug")]
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
        let sap = self.network.closest_or_opposite(&dst, None);
        let (section_pk, elders) = if let Some(sap) = &sap {
            (sap.section_key(), sap.elders())
        } else {
            return Err(Error::NoNetworkKnowledge);
        };

        // We select the NUM_OF_ELDERS_SUBSET_FOR_QUERIES closest Elders we are querying
        let chosen_elders: Vec<_> = elders
            .sorted_by(|lhs, rhs| dst.cmp_distance(&lhs.name(), &rhs.name()))
            .map(|elder| elder.addr())
            .take(NUM_OF_ELDERS_SUBSET_FOR_QUERIES)
            .collect();

        let elders_len = chosen_elders.len();
        if elders_len < NUM_OF_ELDERS_SUBSET_FOR_QUERIES && elders_len > 1 {
            return Err(Error::InsufficientElderConnections(
                elders_len,
                NUM_OF_ELDERS_SUBSET_FOR_QUERIES,
            ));
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
            let _handle = tokio::spawn(async move {
                // Insert the response sender
                trace!("Inserting channel for op_id {:?}", op_id);
                let _old = pending_queries_for_thread
                    .write()
                    .await
                    .insert(op_id.clone(), sender);

                trace!("Inserted channel for {:?}", op_id);
            });
        } else {
            warn!("No op_id found for query");
        }

        let failed_sends = std::sync::Arc::new(tokio::sync::Mutex::new(0_usize));

        let dst_location = DstLocation::Section {
            name: dst,
            section_pk,
        };
        let msg_kind = MsgKind::ServiceMsg(auth);
        let wire_msg = WireMsg::new_msg(msg_id, payload, msg_kind, dst_location)?;
        // TODO: prevent this clone
        let priority = wire_msg.clone().into_message()?.priority();
        let msg_bytes = wire_msg.serialize()?;

        // Set up response listeners
        for socket in chosen_elders.clone() {
            let endpoint = endpoint.clone();
            let msg_bytes = msg_bytes.clone();
            let counter_clone = failed_sends.clone();

            let task_handle = tokio::spawn({
                let session = self.clone();
                async move {
                    trace!("queueing query send task to: {:?}", &socket);
                    let result = endpoint
                        .connect_to(&socket)
                        .err_into()
                        .and_then(|(connection, connection_incoming)| async move {
                            Self::spawn_message_listener_thread(
                                session,
                                connection.id(),
                                connection.remote_address(),
                                connection_incoming,
                            );

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
                }
                .instrument(tracing::debug_span!("sending query message"))
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

        let send_failures = *failed_sends.lock().await;
        if send_failures >= 2 {
            let successful_connections = 3 - send_failures;
            error!("Could not send query to enough elders");
            return Err(Error::InsufficientElderConnections(
                successful_connections,
                3,
            ));
        }

        let response = loop {
            let mut error_response = None;
            match (receiver.recv().await, chunk_addr) {
                (Some(QueryResponse::GetChunk(Ok(chunk))), Some(chunk_addr)) => {
                    // We are dealing with Chunk query responses, thus we validate its hash
                    // matches its xorname, if so, we don't need to await for more responses
                    debug!("Chunk QueryResponse received is: {:#?}", chunk);

                    if chunk_addr.name() == chunk.name() {
                        trace!("Valid Chunk received for {:?}", msg_id);
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
                let _handle = tokio::spawn(async move {
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

    #[instrument(skip_all, level = "debug")]
    pub(crate) async fn make_contact_with_nodes(
        &self,
        nodes: Vec<SocketAddr>,
        dst_address: XorName,
        auth: ServiceAuth,
        payload: Bytes,
    ) -> Result<(), Error> {
        let endpoint = self.endpoint.clone();
        // Get DataSection elders details.
        // TODO: we should be able to handle using an pre-existing prefixmap. This is here for when that's in place.
        let (elders_or_adults, section_pk) =
            if let Some(sap) = self.network.closest_or_opposite(&dst_address, None) {
                let mut nodes: Vec<_> = sap
                    .elders()
                    .map(|elder| elder.addr())
                    .take(NUM_OF_ELDERS_SUBSET_FOR_QUERIES)
                    .collect();

                nodes.shuffle(&mut OsRng);

                (nodes, sap.section_key())
            } else {
                // Send message to our bootstrap peer with network's genesis PK.
                (nodes, self.genesis_key)
            };

        let msg_id = MessageId::new();

        debug!(
            "Making initial contact with nodes. Our PublicAddr: {:?}. Using {:?} to {} nodes",
            endpoint.public_addr(),
            msg_id,
            elders_or_adults.len()
        );

        // TODO: Don't use genesis key if we have a full section
        let dst_location = DstLocation::Section {
            name: dst_address,
            section_pk,
        };
        let msg_kind = MsgKind::ServiceMsg(auth);
        let wire_msg = WireMsg::new_msg(msg_id, payload, msg_kind, dst_location)?;

        let initial_contacts = elders_or_adults
            .iter()
            .take(NODES_TO_CONTACT_PER_STARTUP_BATCH)
            .copied()
            .collect();
        send_message(
            self.clone(),
            initial_contacts,
            wire_msg.clone(),
            self.endpoint.clone(),
            msg_id,
        )
        .await?;

        *self.initial_connection_check_msg_id.write().await = Some(msg_id);

        let mut knowledge_checks = 0;
        let mut outgoing_msg_rounds = 1;
        let mut last_start_pos = 0;

        // wait here to give a chance for AE responses to come in and be parsed
        tokio::time::sleep(self.standard_wait).await;

        // If we start with genesis key here, we should wait until we have _at least_ one AE-Retry in
        if section_pk == self.genesis_key {
            // wait until we have _some_ network knowledge
            while self
                .network
                .closest_or_opposite(&dst_address, None)
                .is_none()
            {
                let stats = self.network.known_sections_count();
                debug!("Client still has not received any AE-Retry message... {:?}. Current sections known", stats);

                knowledge_checks += 1;

                if knowledge_checks > 2 {
                    let mut start_pos = outgoing_msg_rounds * NODES_TO_CONTACT_PER_STARTUP_BATCH;

                    if start_pos > elders_or_adults.len() {
                        start_pos = last_start_pos;
                    }

                    last_start_pos = start_pos;

                    let next_batch_end = start_pos + NODES_TO_CONTACT_PER_STARTUP_BATCH;
                    let next_contacts = if next_batch_end > elders_or_adults.len() {
                        elders_or_adults[start_pos..].to_vec()
                    } else {
                        elders_or_adults[start_pos..start_pos + NODES_TO_CONTACT_PER_STARTUP_BATCH]
                            .to_vec()
                    };

                    outgoing_msg_rounds += 1;

                    trace!("Sending out another batch of initial contact msgs to new nodes");
                    send_message(
                        self.clone(),
                        next_contacts,
                        wire_msg.clone(),
                        self.endpoint.clone(),
                        msg_id,
                    )
                    .await?;

                    trace!(
                        "Awaiting a duration of {:?} before trying new nodes",
                        self.standard_wait
                    );
                    tokio::time::sleep(self.standard_wait).await;
                }
            }
        }

        Ok(())
    }
}

#[instrument(skip_all, level = "trace")]
pub(super) async fn send_message(
    session: Session,
    elders: Vec<SocketAddr>,
    wire_msg: WireMsg,
    endpoint: Endpoint,
    msg_id: MessageId,
) -> Result<(), Error> {
    let priority = wire_msg.clone().into_message()?.priority();
    let msg_bytes = wire_msg.serialize()?;

    // Send message to all Elders concurrently
    let mut tasks = Vec::default();

    let successes = Arc::new(RwLock::new(0));

    // clone elders as we want to update them in this process
    for socket in elders.clone() {
        let successes_clone = successes.clone();
        let msg_bytes_clone = msg_bytes.clone();
        let endpoint = endpoint.clone();
        let task_handle: JoinHandle<Result<(), Error>> = tokio::spawn({
            let session = session.clone();
            async move {
                // trace!("About to send cmd message {:?} to {:?}", msg_id, &socket);
                endpoint
                    .connect_to(&socket)
                    .err_into()
                    .and_then(|(connection, connection_incoming)| async move {
                        Session::spawn_message_listener_thread(
                            session,
                            connection.id(),
                            connection.remote_address(),
                            connection_incoming,
                        );
                        connection.send_with(msg_bytes_clone, priority, None).await
                    })
                    .await?;

                *successes_clone.write().await += 1;

                trace!("Sent msg with MsgId {:?} to {:?}", msg_id, &socket);
                Ok(())
            }
            .instrument(tracing::trace_span!("sending message"))
            .in_current_span()
        });
        tasks.push(task_handle);
    }

    // Let's await for all messages to be sent
    let results = join_all(tasks).await;

    for r in results {
        match r {
            Ok(send_result) => {
                if send_result.is_err() {
                    error!("Error during {:?} send: {:?}", msg_id, send_result);
                }
            }
            Err(join_error) => {
                warn!("Tokio join error as we send: {:?}", join_error)
            }
        }
    }

    let failures = elders.len() - *successes.read().await;

    if failures > 0 {
        error!(
            "Sending the message ({:?}) from {} to {}/{} of the elders failed: {:?}",
            msg_id,
            endpoint.public_addr(),
            failures,
            elders.len(),
            elders,
        );
    }

    let successful_sends = *successes.read().await;
    if failures > successful_sends {
        error!("More send errors than success on send_message");
        Err(Error::InsufficientElderConnections(
            elders.len(),
            successful_sends,
        ))
    } else {
        Ok(())
    }
}

#[instrument(skip_all, level = "trace")]
pub(crate) async fn write_data_to_path<T: Serialize>(data: &T, path: &Path) -> Result<(), Error> {
    // Write our PrefixMap to root dir
    if let Err(e) = write_data_to_disk(data, path).await {
        error!("Error writing data for Client at dir {:?}: {:?}", path, e);
    }

    Ok(())
}

#[instrument(skip_all, level = "trace")]
pub(crate) async fn create_client_root_dir(client_pk: PublicKey) -> Result<PathBuf, Error> {
    let mut root_dir = dirs_next::home_dir()
        .ok_or_else(|| Error::Generic("Error opening home dir".to_string()))?;
    root_dir.push(SAFE_CLIENT_DIR);
    root_dir.push(format!("sn_client-{}", client_pk));

    // Create `.safe/client` dir if not present
    tokio::fs::create_dir_all(root_dir.clone())
        .await
        .map_err(|e| Error::Generic(format!("Error creating client root dir: {:?}", e)))?;

    Ok(root_dir)
}
