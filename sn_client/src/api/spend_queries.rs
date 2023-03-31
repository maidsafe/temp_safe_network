// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::Client;
use crate::errors::{Error, Result};

use sn_dbc::PublicKey;
use sn_interface::{
    messaging::{
        data::{ClientMsg, DataQuery, Error as ErrorMsg, QueryResponse, SpendQuery},
        ClientAuth, WireMsg,
    },
    types::{
        fees::{RequiredFee, SpendPriority},
        NodeId,
    },
};

use backoff::{backoff::Backoff, ExponentialBackoff};
use futures::future::join_all;
use std::collections::BTreeMap;
use tokio::time::sleep;
use tracing::{debug, info_span};

impl Client {
    /// Return the set of Elder reward keys and the individual fee they ask for processing a spend.
    #[instrument(skip(self), level = "debug")]
    pub async fn get_section_fees(
        &self,
        dbc_id: PublicKey,
        priority: SpendPriority,
    ) -> Result<BTreeMap<NodeId, RequiredFee>> {
        let fee_query = DataQuery::Spentbook(SpendQuery::GetFees { dbc_id, priority });

        let (_, elders) = self
            .session
            .get_all_elders_of_dst(fee_query.dst_name())
            .await?;
        let tasks = elders.into_iter().enumerate().map(|(index, _)| {
            let client = self.clone();
            let query = fee_query.clone();
            tokio::spawn(async move { client.send_fee_query(query, index).await })
        });

        // We just want to receive at least supermajority of results, we don't care about any errors
        // so we log them, but return whatever results we get. If not enough for upper layer, it will error there.
        let results: BTreeMap<_, _> = join_all(tasks)
            .await
            .into_iter()
            .flat_map(|res| {
                if let Err(error) = &res {
                    warn!("Error when joining fee query threads: {error}");
                }
                res
            })
            .flat_map(|res| {
                if let Err(error) = &res {
                    warn!("Error when querying for fees: {error}");
                }
                res
            })
            .filter_map(|(elder, resp)| match resp {
                QueryResponse::GetFees(Ok(fee)) => Some((elder, fee)),
                QueryResponse::GetFees(Err(error)) => {
                    warn!("Fee query unexpectedly failed: {error}");
                    None
                }
                other => {
                    warn!("Unexpected response to fee query: {other:?}");
                    None
                }
            })
            .collect();

        Ok(results)
    }

    /// Send a Query to the network and await a response.
    /// Queries are automatically retried using exponential backoff if the timeout is hit.
    #[instrument(skip(self), level = "debug")]
    async fn send_fee_query(
        &self,
        query: DataQuery,
        elder_index: usize,
    ) -> Result<(NodeId, QueryResponse)> {
        let client_pk = self.public_key();

        trace!("Setting up fee query retry.");

        let span = info_span!("Attempting a fee query.");
        let _ = span.enter();

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

        let dst = query.dst_name();
        let msg = ClientMsg::Query(query.clone());
        let serialised_query = WireMsg::serialize_msg_payload(&msg)?;
        let auth = ClientAuth {
            public_key: client_pk,
            signature: self.keypair.sign(&serialised_query),
        };

        let mut result = Err(Error::ErrorMsg {
            source: ErrorMsg::InconsistentStorageNodeResponses,
        });

        while let Some(delay) = backoff.next_backoff() {
            debug!("Attempting {query:?}");

            // grab up to date destination section from our local network knowledge
            let (section_pk, elders) = self.session.get_all_elders_of_dst(dst).await?;

            let elder = *elders
                .get(elder_index)
                .ok_or(Error::InsufficientElderConnections {
                    connections: elders.len(),
                    required: elder_index + 1,
                })?;

            result = self
                .session
                .send_single_query(
                    query.clone(),
                    auth.clone(),
                    serialised_query.clone(),
                    section_pk,
                    elder,
                )
                .await;

            // if the response is acceptable, return instead of wait/retry loop
            if let Ok((elder, response)) = &result {
                if response.is_error() {
                    warn!(
                        "Fee query errored... querying again until we hit query_timeout ({:?})",
                        self.query_timeout
                    );
                } else {
                    debug!("{query:?} sent and received okay");
                    return Ok((*elder, response.clone()));
                }
            }

            debug!("Sleeping before trying query again: {delay:?} sleep for {query:?}");
            sleep(delay).await;
        }

        // The warning says "last response", because there is no path for it to use the initial result value.
        warn!("Finished trying and last response to {query:?} is {result:?}");

        // we're done trying
        result
    }
}
