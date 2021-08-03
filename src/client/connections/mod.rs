// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use qp2p::{Config as QuicP2pConfig, Endpoint, QuicP2p};
use std::{borrow::Borrow, collections::HashMap, sync::Arc};
use tokio::sync::mpsc::Sender;
use tokio::sync::RwLock;
use tracing::{debug, trace};

use crate::client::Error;
use crate::messaging::{
    data::{CmdError, QueryResponse},
    node::SigShare,
    signature_aggregator::{Error as AggregatorError, SignatureAggregator},
    MessageId, SectionAuthorityProvider,
};
use crate::types::{PrefixMap, PublicKey};
use xor_name::XorName;

mod listeners;
mod messaging;

type QueryResponseSender = Sender<QueryResponse>;

type PendingQueryResponses = Arc<RwLock<HashMap<OperationId, QueryResponseSender>>>;

pub(crate) struct QueryResult {
    pub(super) response: QueryResponse,
    // TODO: unify this
    pub(super) operation_id: OperationId,
}

#[derive(Clone, Debug)]
pub(super) struct Session {
    client_pk: PublicKey,
    qp2p: QuicP2p<XorName>,
    pending_queries: PendingQueryResponses,
    incoming_err_sender: Arc<Sender<CmdError>>,
    endpoint: Option<Endpoint<XorName>>,
    /// All elders we know about from SectionInfo messages
    network: PrefixMap<SectionAuthorityProvider>,
    aggregator: Arc<RwLock<SignatureAggregator>>,
    is_connecting_to_new_elders: bool,
}

impl Session {
    pub(super) fn new(
        client_pk: PublicKey,
        qp2p_config: QuicP2pConfig,
        err_sender: Sender<CmdError>,
    ) -> Result<Self, Error> {
        debug!("QP2p config: {:?}", qp2p_config);

        let qp2p =
            qp2p::QuicP2p::<XorName>::with_config(Some(qp2p_config), Default::default(), true)?;
        Ok(Self {
            client_pk,
            qp2p,
            pending_queries: Arc::new(RwLock::new(HashMap::default())),
            incoming_err_sender: Arc::new(err_sender),
            endpoint: None,
            network: PrefixMap::new(),
            aggregator: Arc::new(RwLock::new(SignatureAggregator::new())),
            is_connecting_to_new_elders: false,
        })
    }

    /// Get the count of elders we have knowledge of
    #[allow(unused)]
    pub(super) async fn known_elders_count(&self) -> usize {
        self.network.iter().map(|sap| sap.elders.len()).sum()
    }

    pub(super) fn endpoint(&self) -> Result<&Endpoint<XorName>, Error> {
        match self.endpoint.borrow() {
            Some(endpoint) => Ok(endpoint),
            None => {
                trace!("self.endpoint.borrow() was None");
                Err(Error::NotBootstrapped)
            }
        }
    }

    #[allow(unused)]
    pub(crate) async fn aggregate_incoming_message(
        &mut self,
        bytes: Vec<u8>,
        sig_share: SigShare,
    ) -> Result<Option<Vec<u8>>, Error> {
        match self.aggregator.write().await.add(&bytes, sig_share) {
            Ok(key_sig) => {
                if key_sig.public_key.verify(&key_sig.signature, &bytes) {
                    Ok(Some(bytes))
                } else {
                    Err(Error::Aggregation(
                        "Failed to verify aggregated signature".to_string(),
                    ))
                }
            }
            Err(AggregatorError::NotEnoughShares) => Ok(None),
            Err(e) => Err(Error::Aggregation(e.to_string())),
        }
    }
}
