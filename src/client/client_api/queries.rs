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
use tracing::debug;

impl Client {
    // Send a Query to the network and await a response.
    // Queries are automatically retried using exponential backoff if the timeout is hit
    // This function is a helper private to this module.
    pub(crate) async fn send_query(&self, query: DataQuery) -> Result<QueryResult, Error> {
        let client_pk = self.public_key();
        let msg = ServiceMsg::Query(query.clone());
        let serialised_query = WireMsg::serialize_msg_payload(&msg)?;
        let signature = self.keypair.sign(&serialised_query);

        retry(
            || async {
                tokio::time::timeout(
                    self.query_timeout,
                    self.send_signed_query(
                        query.clone(),
                        client_pk,
                        serialised_query.clone(),
                        signature.clone(),
                    ),
                )
                .await
                .map_err(backoff::Error::Transient)
            },
            self.query_timeout.mul_f32(3.0),
        )
        .await
        .map_err(|_| Error::NoResponse)?
    }

    /// Send a Query to the network and await a response
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
