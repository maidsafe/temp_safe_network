// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use std::{collections::BTreeSet, net::SocketAddr, time::Duration};

use bytes::Bytes;
use futures::{future::join_all, stream::FuturesUnordered};
use itertools::Itertools;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::{sync::mpsc::channel, task::JoinHandle, time::timeout};
use tracing::{debug, error, trace, warn};

use crate::client::Error;
use crate::messaging::{
    data::{ChunkRead, DataQuery, QueryResponse},
    section_info::SectionInfoMsg,
    DstLocation, MessageId, MsgKind, SectionAuthorityProvider, ServiceAuth, WireMsg,
};
use crate::types::{Chunk, PrivateChunk, PublicChunk};

use super::{QueryResult, Session};
use crate::routing::XorName;
use bls::SecretKey;
use qp2p::Endpoint;
use tokio::time::sleep;
use xor_name::PrefixMap;

// Number of attempts when retrying to send a message to a node
const NUMBER_OF_RETRIES: usize = 3;
// Number of Elders subset to send queries to
const NUM_OF_ELDERS_SUBSET_FOR_QUERIES: usize = 3;

impl Session {
    /// Bootstrap to the network, connecting with the closest section to
    /// the XorName of our PublicKey.
    pub(crate) async fn bootstrap(&mut self) -> Result<(), Error> {
        trace!(
            "Trying to bootstrap to the network with public_key: {:?}",
            self.client_pk
        );

        let (endpoint, _, mut incoming_messages, _disconnections, mut bootstrapped_peer) =
            self.qp2p.bootstrap().await?;

        self.endpoint = Some(endpoint.clone());
        let mut bootstrap_nodes = endpoint
            .clone()
            .bootstrap_nodes()
            .to_vec()
            .into_iter()
            .collect::<BTreeSet<_>>();

        self.send_get_section_query(XorName::from(self.client_pk), &bootstrapped_peer)
            .await?;

        // Bootstrap and send a handshake request to the bootstrapped peer
        while !self.network.read().await.iter().count() == 1 {
            // This means that the peer we bootstrapped to
            // has responded with a SectionInfo Message
            match timeout(
                Duration::from_secs(30),
                self.process_incoming_message(&mut incoming_messages),
            )
            .await
            {
                Ok(Ok(true)) => {}
                _ => {
                    // Remove the unresponsive peer we bootstrapped to and bootstrap again
                    let _ = bootstrap_nodes.remove(&bootstrapped_peer);
                    bootstrapped_peer = self
                        .qp2p
                        .rebootstrap(
                            &endpoint,
                            &bootstrap_nodes.iter().cloned().collect::<Vec<_>>(),
                        )
                        .await?;
                }
            }
        }

        self.spawn_message_listener_thread(incoming_messages).await;

        debug!(
            "Successfully obtained the initial list of Elders: {:?}",
            self.network
        );

        Ok(())
    }

    /// Send a `ServiceMsg` to the network without awaiting for a response.
    pub(crate) async fn send_cmd(
        &self,
        data_name: XorName,
        auth: ServiceAuth,
        payload: Bytes,
        targets: usize,
    ) -> Result<(), Error> {
        let endpoint = self.endpoint()?.clone();

        // Get DataSection elders details. Resort to own section if DataSection is not available.
        let (elders, section_pk) = self
            .network
            .read()
            .await
            .get_matching(&data_name)
            .map(|(_, sap)| {
                (
                    sap.elders
                        .values()
                        .cloned()
                        .take(targets)
                        .collect::<Vec<SocketAddr>>(),
                    sap.public_key_set.public_key(),
                )
            })
            .ok_or(Error::NotBootstrapped)?;
        let msg_id = MessageId::new();

        debug!(
            "Sending command w/id {:?}, from {}, to {} Elders",
            msg_id,
            endpoint.socket_addr(),
            elders.len()
        );

        trace!(
            "Sending (from {}) dst {:?} w/ id: {:?}",
            endpoint.socket_addr(),
            data_name,
            msg_id
        );

        let dst_location = DstLocation::Section {
            name: data_name,
            section_pk,
        };
        let msg_kind = MsgKind::ServiceMsg(auth);
        let wire_msg = WireMsg::new_msg(msg_id, payload, msg_kind, dst_location)?;

        send_message(elders, wire_msg, self.endpoint.clone(), msg_id).await
    }

    /// Send a `ServiceMsg` to the network awaiting for the response.
    pub(crate) async fn send_query(
        &self,
        query: DataQuery,
        auth: ServiceAuth,
        payload: Bytes,
        msg_id: MessageId,
    ) -> Result<QueryResult, Error> {
        let endpoint = self.endpoint()?.clone();
        let pending_queries = self.pending_queries.clone();

        let chunk_addr = if let DataQuery::Chunk(ChunkRead::Get(address)) = query {
            Some(address)
        } else {
            None
        };

        let data_name = query.dst_name();

        // Get DataSection elders details. Resort to own section if DataSection is not available.
        let (elders, section_pk) = self
            .network
            .read()
            .await
            .get_matching(&data_name)
            .map(|(_, sap)| (sap.elders.clone(), sap.public_key_set.public_key()))
            .ok_or(Error::NotBootstrapped)?;

        // We select the NUM_OF_ELDERS_SUBSET_FOR_QUERIES closest Elders we are querying
        let chosen_elders = elders
            .into_iter()
            .sorted_by(|(lhs_name, _), (rhs_name, _)| data_name.cmp_distance(lhs_name, rhs_name))
            .take(NUM_OF_ELDERS_SUBSET_FOR_QUERIES)
            .map(|(_, addr)| addr)
            .collect::<Vec<SocketAddr>>();

        let wire_msg = WireMsg::new_msg(
            msg_id,
            payload,
            MsgKind::ServiceMsg(auth),
            DstLocation::Section {
                name: data_name,
                section_pk,
            },
        )?;
        let priority = wire_msg.msg_kind().priority();
        let msg_bytes = wire_msg.serialize()?;

        let elders_len = chosen_elders.len();
        if elders_len < NUM_OF_ELDERS_SUBSET_FOR_QUERIES {
            error!(
                "Not enough Elder connections: {}, minimum required: {}",
                elders_len, NUM_OF_ELDERS_SUBSET_FOR_QUERIES
            );
            return Err(Error::InsufficientElderConnections(elders_len));
        }

        let msg_id = MessageId::new();

        debug!(
            "Sending query message {:?}, msg_id: {}, from {}, to the {} Elders closest to data name: {:?}",
            query,
            msg_id,
            endpoint.socket_addr(),
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
                let _ = pending_queries_for_thread
                    .write()
                    .await
                    .insert(op_id, sender);
            });
        }

        let discarded_responses = std::sync::Arc::new(tokio::sync::Mutex::new(0_usize));

        // Set up response listeners
        for socket in chosen_elders {
            let endpoint = endpoint.clone();
            let msg_bytes = msg_bytes.clone();
            let counter_clone = discarded_responses.clone();
            let task_handle = tokio::spawn(async move {
                endpoint.connect_to(&socket).await?;

                // Retry queries that failed due to connection issues only
                let mut result = Err(Error::ElderQuery);
                for attempt in 0..NUMBER_OF_RETRIES + 1 {
                    let msg_bytes_clone = msg_bytes.clone();

                    if let Err(err) = endpoint
                        .send_message(msg_bytes_clone, &socket, priority)
                        .await
                    {
                        error!(
                            "Try #{:?} @ {:?}, failed sending query message: {:?}",
                            attempt + 1,
                            socket,
                            err
                        );
                        result = Err(Error::SendingQuery);
                        if attempt <= NUMBER_OF_RETRIES {
                            let millis = 2_u64.pow(attempt as u32 - 1) * 100;
                            sleep(std::time::Duration::from_millis(millis)).await
                        }
                    } else {
                        trace!("ServiceMsg with id: {:?}, sent to {}", &msg_id, &socket);
                        result = Ok(());
                        break;
                    }
                }

                match &result {
                    Err(err) => {
                        error!("Error sending Query to elder: {:?} ", err);
                        let mut a = counter_clone.lock().await;
                        *a += 1;
                    }
                    Ok(()) => (),
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
        // so we don't need more than one valid response to prevent from accepting invaid responses
        // from byzantine nodes, however for mutable data (non-Chunk esponses) we will
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

                    let xorname = match &chunk {
                        Chunk::Private(priv_chunk) => {
                            *PrivateChunk::new(priv_chunk.value().clone(), *priv_chunk.owner())
                                .name()
                        }
                        Chunk::Public(pub_chunk) => {
                            *PublicChunk::new(pub_chunk.value().clone()).name()
                        }
                    };

                    if *chunk_addr.name() == xorname {
                        trace!("Valid Chunk received for {}", msg_id);
                        break Some(QueryResponse::GetChunk(Ok(chunk)));
                    } else {
                        // the Chunk content doesn't match its Xorname,
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
                    debug!("QueryResponse error received (but may be overridden by a non-error reponse from another elder): {:#?}", &response);
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
                    let _ = pending_queries.clone().write().await.remove(&query_op_id);
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

    // Get section info from the peer we have bootstrapped with.
    pub(crate) async fn send_get_section_query(
        &self,
        name: XorName,
        bootstrapped_peer: &SocketAddr,
    ) -> Result<(), Error> {
        trace!(
            "Querying for section info from bootstrapped node: {:?}",
            bootstrapped_peer
        );

        let msg_id = MessageId::new();
        let query = SectionInfoMsg::GetSectionQuery {
            name,
            is_bootstrapping: true,
        };

        let payload = WireMsg::serialize_msg_payload(&query)?;

        // Random section PK since we have no knowledge in the beginning
        let random_section_pk = SecretKey::random().public_key();

        let dst_location = DstLocation::Section {
            name,
            section_pk: random_section_pk,
        };
        let wire_msg = WireMsg::new_msg(msg_id, payload, MsgKind::SectionInfoMsg, dst_location)?;
        let priority = wire_msg.msg_kind().priority();
        let msg_bytes = wire_msg.serialize()?;

        debug!(
            "Sending query message {:?}, msg_id {}, from {}, to bootstrapped node {} ",
            query,
            msg_id,
            self.endpoint()?.socket_addr(),
            bootstrapped_peer
        );

        self.endpoint()?
            .send_message(msg_bytes, bootstrapped_peer, priority)
            .await?;

        Ok(())
    }

    #[allow(unused)]
    pub(crate) async fn disconnect_from_peers(&self, peers: Vec<SocketAddr>) -> Result<(), Error> {
        for elder in peers {
            self.endpoint()?.disconnect_from(&elder).await?;
        }

        Ok(())
    }
}

pub(crate) async fn send_message(
    elders: Vec<SocketAddr>,
    wire_msg: WireMsg,
    endpoint: Option<Endpoint<XorName>>,
    msg_id: MessageId,
) -> Result<(), Error> {
    let priority = wire_msg.msg_kind().priority();
    let msg_bytes = wire_msg.serialize()?;
    let endpoint = match endpoint {
        Some(ep) => ep,
        None => return Err(Error::NotBootstrapped),
    };

    // Send message to all Elders concurrently
    let mut tasks = Vec::default();

    // clone elders as we want to update them in this process
    for socket in elders {
        let msg_bytes_clone = msg_bytes.clone();
        let endpoint = endpoint.clone();
        let task_handle: JoinHandle<Result<(), Error>> = tokio::spawn(async move {
            trace!("About to send cmd message {:?} to {:?}", msg_id, &socket);
            endpoint.connect_to(&socket).await?;
            endpoint
                .send_message(msg_bytes_clone, &socket, priority)
                .await?;

            trace!("Sent cmd with MsgId {:?}to {:?}", msg_id, &socket);
            Ok(())
        });
        tasks.push(task_handle);
    }

    // Let's await for all messages to be sent
    let results = join_all(tasks).await;

    let mut failures = 0;
    results.iter().for_each(|res| {
        if res.is_err() {
            failures += 1;
        }
    });

    if failures > 0 {
        error!("Sending the message to {} Elders failed", failures);
    }

    Ok(())
}

pub(crate) async fn rebuild_message_for_ae_resend(
    msg_id: MessageId,
    serialized_cmd: Bytes,
    auth: ServiceAuth,
    dst_address: Option<XorName>,
    network: Arc<RwLock<PrefixMap<SectionAuthorityProvider>>>,
) -> Option<(WireMsg, Vec<SocketAddr>)> {
    if let Some(dst_address) = dst_address {
        if let Some((elders, section_pk)) =
            network
                .read()
                .await
                .get_matching(&dst_address)
                .map(|(_, sap)| {
                    (
                        sap.elders.values().cloned().collect::<Vec<SocketAddr>>(),
                        sap.public_key_set.public_key(),
                    )
                })
        {
            // Let's rebuild the message with the updated destination details
            if let Ok(wire_msg) = WireMsg::new_msg(
                msg_id,
                serialized_cmd,
                MsgKind::ServiceMsg(auth),
                DstLocation::Section {
                    name: dst_address,
                    section_pk,
                },
            ) {
                return Some((wire_msg, elders));
            } else {
                error!("Error generating WireMsg on resending");
            }
        } else {
            error!("Error fetching latest elders for resending AE message");
        }
    }
    None
}
