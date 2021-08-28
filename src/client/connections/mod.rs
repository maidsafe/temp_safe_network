// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod listeners;
mod messaging;

use crate::client::Error;
use crate::messaging::{
    data::{CmdError, OperationId, QueryResponse},
    signature_aggregator::SignatureAggregator,
};
use crate::prefix_map::NetworkPrefixMap;
use crate::types::PublicKey;

use qp2p::{Config as QuicP2pConfig, Endpoint};
use std::{borrow::Borrow, collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::{mpsc::Sender, RwLock};
use tracing::trace;
use xor_name::XorName;

type QueryResponseSender = Sender<QueryResponse>;
type PendingQueryResponses = Arc<RwLock<HashMap<OperationId, QueryResponseSender>>>;

pub(crate) struct QueryResult {
    pub(super) response: QueryResponse,
    // TODO: unify this
    pub(super) operation_id: OperationId,
}

#[derive(Clone, Debug)]
pub(super) struct Session {
    local_addr: SocketAddr,
    // PublicKey of the client
    client_pk: PublicKey,
    // Session endpoint.
    endpoint: Option<Endpoint<XorName>>,
    // Qp2p config
    qp2p_config: QuicP2pConfig,
    // Channels for sending responses to upper layers
    pending_queries: PendingQueryResponses,
    // Channels for sending errors to upper layer
    incoming_err_sender: Arc<Sender<CmdError>>,
    /// All elders we know about from AE messages
    network: Arc<NetworkPrefixMap>,
    /// Our initial bootstrap node
    bootstrap_peer: Option<SocketAddr>,
    /// BLS Signature aggregator for aggregating network messages
    aggregator: Arc<RwLock<SignatureAggregator>>,
    /// Network's genesis key
    genesis_pk: bls::PublicKey,
}

impl Session {
    pub(super) fn endpoint(&self) -> Result<&Endpoint<XorName>, Error> {
        match self.endpoint.borrow() {
            Some(endpoint) => Ok(endpoint),
            None => {
                trace!("self.endpoint.borrow() was None");
                Err(Error::NotBootstrapped)
            }
        }
    }
}
