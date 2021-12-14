// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::Client;
use crate::client::utils::retry;
use crate::client::{connections::QueryResult, errors::Error};
use crate::messaging::{
    data::{DataQuery, ServiceMsg},
    ServiceAuth, WireMsg,
};
use crate::types::{PublicKey, Signature};
use bytes::Bytes;
use tracing::{debug, info_span, Instrument};

// We divide the total query timeout by this number to get a more reasonable starting timeout.
// This also represents the max retries possible _if no backoff were present_, while still
// staying within the max_timeout in practice (it's _probably_ one less than this value...)
const MAX_RETRY_COUNT: f32 = 11.0;

impl Client {
    /// Send a Query to the network and await a response.
    /// Queries are automatically retried using exponential backoff if the timeout is hit.
    #[instrument(skip(self), level = "debug")]
    pub async fn send_query(&self, query: DataQuery) -> Result<QueryResult, Error> {
        self.send_query_with_retry_count(query, MAX_RETRY_COUNT)
            .await
    }

    /// Send a Query to the network and await a response.
    /// Queries are not retried if the timeout is hit.
    #[instrument(skip(self), level = "debug")]
    pub async fn send_query_without_retry(&self, query: DataQuery) -> Result<QueryResult, Error> {
        self.send_query_with_retry_count(query, 1.0).await
    }

    // Send a Query to the network and await a response.
    // Queries are automatically retried using exponential backoff if the timeout is hit
    // This function is a private helper.
    #[instrument(skip(self), level = "debug")]
    async fn send_query_with_retry_count(
        &self,
        query: DataQuery,
        retry_count: f32,
    ) -> Result<QueryResult, Error> {
        let client_pk = self.public_key();
        let msg = ServiceMsg::Query(query.clone());
        let serialised_query = WireMsg::serialize_msg_payload(&msg)?;
        let signature = self.keypair.sign(&serialised_query);

        let starting_query_timeout = self.query_timeout.div_f32(MAX_RETRY_COUNT);
        trace!(
            "Setting up query retry, initial interval is: {:?}",
            starting_query_timeout
        );

        retry(
            || {
                async {
                    debug!(
                        "Attempting {:?} with a query timeout of {:?}",
                        query, starting_query_timeout
                    );
                    let res = tokio::time::timeout(
                        // The max timeout is total_timeout / retry_factor, so we should get
                        // at least lowest_bound_count retries within the total time (if needed)
                        starting_query_timeout,
                        self.send_signed_query(
                            query.clone(),
                            client_pk,
                            serialised_query.clone(),
                            signature.clone(),
                        ),
                    )
                    .await;

                    match res {
                        Ok(Ok(query_result)) => Ok(Ok(query_result)),
                        Ok(Err(error @ Error::InsufficientElderConnections { .. })) => {
                            warn!("Insufficient elder connections during a query attempt");

                            Err(error).map_err(backoff::Error::Transient)
                        }
                        Ok(Err(other_error)) => Err(other_error).map_err(backoff::Error::Permanent),
                        Err(_elapsed) => {
                            Err(Error::QueryTimedOut).map_err(backoff::Error::Transient)
                        }
                    }
                }
                .instrument(info_span!("Attempting a query"))
            },
            starting_query_timeout,
            self.query_timeout,
        )
        .await
        .map_err(|_| {
            debug!(
                "Retries ({}) all failed returning no response for {:?}",
                MAX_RETRY_COUNT, query
            );
            Error::NoResponse
        })?
    }

    /// Send a Query to the network and await a response.
    /// This is to be part of a public API, for the user to
    /// provide the serialised and already signed query.
    pub(crate) async fn send_signed_query(
        &self,
        query: DataQuery,
        client_pk: PublicKey,
        serialised_query: Bytes,
        signature: Signature,
    ) -> Result<QueryResult, Error> {
        debug!("Sending Query: {:?}", query);
        let auth = ServiceAuth {
            public_key: client_pk,
            signature,
        };

        self.session.send_query(query, auth, serialised_query).await
    }
}
