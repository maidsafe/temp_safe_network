// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

// use std::collections::BTreeSet;

use super::Client;
use crate::errors::{Error, Result};

use sn_interface::{
    data_copy_count,
    messaging::{
        data::{ClientMsg, DataQuery, QueryResponse},
        ClientAuth, WireMsg,
    },
    types::{Peer, PublicKey, Signature},
};

use backoff::{backoff::Backoff, ExponentialBackoff};
use bytes::Bytes;
use std::collections::BTreeSet;
use tokio::time::sleep;
use tracing::{debug, info_span};

impl Client {
    /// Send a Query to the network and await a response.
    /// Queries are automatically retried using exponential backoff if the timeout is hit.
    #[cfg(not(feature = "check-replicas"))]
    #[instrument(skip(self), level = "debug")]
    pub async fn send_query(&self, query: DataQuery) -> Result<QueryResponse> {
        self.send_query_with_retry(query, true).await
    }

    /// Send a Query to the network and await a response.
    /// Queries are not retried if the timeout is hit.
    #[instrument(skip(self), level = "debug")]
    pub async fn send_query_without_retry(&self, query: DataQuery) -> Result<QueryResponse> {
        self.send_query_with_retry(query, false).await
    }

    // Send a Query to the network and await a response.
    // Queries are automatically retried if the timeout is hit
    // This function is a private helper.
    #[instrument(skip(self), level = "debug")]
    async fn send_query_with_retry(&self, query: DataQuery, retry: bool) -> Result<QueryResponse> {
        let client_pk = self.public_key();

        // Add jitter so not all clients retry at the same rate. This divider will knock on to the overall retry window
        // and should help prevent elders from being conseceutively overwhelmed
        trace!("Setting up query retry");

        let span = info_span!("Attempting a query");
        let _ = span.enter();
        let dst = query.dst_name();
        let mut node_index = 0;
        let max_interval = self.max_backoff_interval;

        let mut backoff = ExponentialBackoff {
            initial_interval: max_interval / 2,
            max_interval,
            max_elapsed_time: self.query_timeout,
            randomization_factor: 1.5,
            ..Default::default()
        };

        // this seems needed for custom settings to take effect
        backoff.reset();

        loop {
            let msg = ClientMsg::Query(query.clone());
            let serialised_query = WireMsg::serialize_msg_payload(&msg)?;
            let signature = self.keypair.sign(&serialised_query);
            debug!("Attempting {query:?} (node_index #{})", node_index);

            // grab up to date destination section from our local network knowledge
            let (section_pk, elders) = self.session.get_query_elders(dst).await?;

            let res = self
                .send_signed_query_to_section(
                    query.clone(),
                    node_index,
                    client_pk,
                    serialised_query.clone(),
                    signature.clone(),
                    Some((section_pk, elders.clone())),
                )
                .await;

            // There should not be more than a certain number of nodes holding
            // copies of the data. Retry the closest node again.
            if !retry || node_index >= data_copy_count() - 1 {
                // we don't want to retry beyond `data_copy_count()` nodes
                return res;
            }

            if let Some(delay) = backoff.next_backoff() {
                // if the response is acceptable, return instead of wait/retry loop
                if let Ok(response) = res {
                    if response.is_data_not_found() {
                        warn!(
                            "Data not found... querying again until we hit query_timeout ({:?})",
                            self.query_timeout
                        );
                    } else {
                        debug!("{query:?} sent and received okay");
                        return Ok(response);
                    }
                }

                // In the next attempt, try the next node, further away.
                node_index += 1;
                debug!("Sleeping before trying query again: {delay:?} sleep for {query:?}");
                sleep(delay).await;
            } else {
                warn!("Finished trying and last response to {query:?} is {res:?}");
                // we're done trying
                return res;
            }
        }
    }

    /// Send a Query to the network and await a response.
    /// This is part of a public API, for the user to
    /// provide the serialised and already signed query.
    pub async fn send_signed_query(
        &self,
        query: DataQuery,
        query_index: usize,
        client_pk: PublicKey,
        serialised_query: Bytes,
        signature: Signature,
    ) -> Result<QueryResponse> {
        debug!("Sending Query: {:?}", query);
        self.send_signed_query_to_section(
            query,
            query_index,
            client_pk,
            serialised_query,
            signature,
            None,
        )
        .await
    }

    // Private helper to send a signed query, with the option to define the destination section.
    // If no destination section is provided, it will be derived from the query content.
    #[allow(clippy::too_many_arguments)]
    async fn send_signed_query_to_section(
        &self,
        query: DataQuery,
        query_index: usize,
        client_pk: PublicKey,
        serialised_query: Bytes,
        signature: Signature,
        dst_section_info: Option<(bls::PublicKey, Vec<Peer>)>,
    ) -> Result<QueryResponse> {
        let auth = ClientAuth {
            public_key: client_pk,
            signature,
        };

        self.session
            .send_query(query, query_index, auth, serialised_query, dst_section_info)
            .await
    }

    /// Send a Query to the network and await a response.
    /// Queries are sent once per each replica, i.e. it sends the query targeting
    /// the replicas (using `node_index`) matching the indexes provided.
    #[instrument(skip(self), level = "debug")]
    pub async fn send_query_to_replicas(
        &self,
        query: DataQuery,
        replicas: &[usize],
    ) -> Result<Vec<(usize, Result<QueryResponse>)>, Error> {
        let client_pk = self.public_key();
        let dst = query.dst_name();

        // grab up to date destination section from our local network knowledge
        let (section_pk, elders) = self.session.get_query_elders(dst).await?;

        // Send queries to the replicas concurrently
        let mut tasks = vec![];
        let unique_indexes = replicas.iter().cloned().collect::<BTreeSet<usize>>();

        let msg = ClientMsg::Query(query.clone());
        let serialised_query = WireMsg::serialize_msg_payload(&msg)?;
        let signature = self.keypair.sign(&serialised_query);

        for node_index in unique_indexes.into_iter() {
            let query = query.clone();
            let sig = signature.clone();
            let serialised_query = serialised_query.clone();
            debug!("Attempting {query:?} @ index: {node_index:?}");

            let client = self.clone();
            let elders_clone = elders.clone();
            tasks.push(async move {
                let result = client
                    .send_signed_query_to_section(
                        query,
                        node_index,
                        client_pk,
                        serialised_query,
                        sig,
                        Some((section_pk, elders_clone)),
                    )
                    .await;

                (node_index, result)
            });
        }

        // Let's await for all queries to be sent
        let results = futures::future::join_all(tasks).await;

        Ok(results)
    }

    /// Send a Query to the network and await a response.
    /// Queries are sent once per each replica, i.e. it sends the query targeting
    /// all replicas (using `node_index`) to make sure the piece of content
    /// is stored in each and all of the expected data replica nodes in the section.
    #[cfg(feature = "check-replicas")]
    #[instrument(skip(self), level = "debug")]
    pub async fn send_query(&self, query: DataQuery) -> Result<QueryResponse, Error> {
        match self.query_all_data_replicas(query.clone()).await {
            Err(Error::DataReplicasCheck(
                error @ crate::errors::DataReplicasCheckError::DifferentResponses { .. },
            )) => {
                warn!("Different responses received for query, we'll retry to send it only once: {error:?}");
                sleep(tokio::time::Duration::from_secs(10)).await;
                let response = self.query_all_data_replicas(query).await;
                debug!("Second attempt to send query to check-replicas: {response:?}");
                response
            }
            other => other,
        }
    }

    #[cfg(feature = "check-replicas")]
    #[instrument(skip(self), level = "debug")]
    async fn query_all_data_replicas(&self, query: DataQuery) -> Result<QueryResponse> {
        use crate::errors::DataReplicasCheckError;
        let span = info_span!("Attempting a query");
        let _ = span.enter();

        // Send queries to all replicas concurrently
        let num_of_replicas = data_copy_count();
        let all_replicas: Vec<usize> = (0..num_of_replicas).collect();
        let results = self
            .send_query_to_replicas(query.clone(), &all_replicas)
            .await?;

        let mut errors = vec![];
        let mut responses = vec![];
        results.into_iter().for_each(|result| match result {
            (node_index, Err(error)) => errors.push((error, node_index)),
            (node_index, Ok(resp)) => responses.push((resp, node_index)),
        });

        if !errors.is_empty() {
            let error_msg = errors.iter().fold(
                format!(
                    "Errors occurred when sending the query to {}/{num_of_replicas} \
                    of the replicas. Errors received: ",
                    errors.len()
                ),
                |acc, (e, i)| format!("{acc}, [ Node-#{i}: {e:?} ]"),
            );
            error!(error_msg);

            return Err(DataReplicasCheckError::ReceivedErrors {
                replicas: num_of_replicas,
                query,
                errors,
            }
            .into());
        }

        if let Some((resp, node_index)) = responses.pop() {
            if responses.iter().all(|(r, _)| r == &resp) {
                return Ok(resp);
            }

            // put the last response back in the list so it's included in the report
            responses.push((resp, node_index));
            let error_msg = responses.iter().fold(
                format!(
                    "Not all responses received are the same when sending query to all \
                    replicas: {query:?}. Responses received: "
                ),
                |acc, (r, i)| format!("{acc}, [ Adult-#{i}: {r:?} ]"),
            );
            error!(error_msg);

            return Err(DataReplicasCheckError::DifferentResponses {
                replicas: num_of_replicas,
                query,
                responses,
            }
            .into());
        }

        Err(DataReplicasCheckError::NoResponse(query).into())
    }
}
