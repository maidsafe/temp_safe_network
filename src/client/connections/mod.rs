// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use qp2p::Endpoint;
use std::{borrow::Borrow, collections::HashMap, sync::Arc};
use tokio::sync::mpsc::Sender;
use tokio::sync::RwLock;
use tracing::{debug, trace};

use crate::client::Error;
use crate::messaging::{
    data::{CmdError, OperationId, QueryResponse},
    signature_aggregator::SignatureAggregator,
};
use crate::prefix_map::NetworkPrefixMap;
use crate::types::PublicKey;
use std::net::SocketAddr;
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
    // PublicKey of the client
    client_pk: PublicKey,
    endpoint: Option<Endpoint<XorName>>,
    // Channels for sending responses to upper layers
    pending_queries: PendingQueryResponses,
    // Channels for sending errors to upper layer
    incoming_err_sender: Arc<Sender<CmdError>>,
    /// All elders we know about from AE messages
    network: Arc<NetworkPrefixMap>,
    /// Our initial bootstrap node
    bootstrap_peer: SocketAddr,
    /// BLS Signature aggregator for aggregating network messages
    aggregator: Arc<RwLock<SignatureAggregator>>,
    /// Network's genesis key
    genesis_pk: bls::PublicKey,
}

impl Session {
    pub(super) async fn new(
        client_pk: PublicKey,
        local_addr: SocketAddr,
        bootstrap_nodes: &[SocketAddr],
        qp2p_config: qp2p::Config,
        err_sender: Sender<CmdError>,
    ) -> Result<Self, Error> {
        debug!("QP2p config: {:?}", qp2p_config);
        // *****************************************************
        // FIXME: receive the network's genesis pk from the user
        let genesis_pk = bls::SecretKey::random().public_key();
        // *****************************************************

        let (endpoint, incoming_messages, _) = Endpoint::new_client(local_addr, qp2p_config)?;
        let bootstrap_peer = endpoint
            .connect_to_any(bootstrap_nodes)
            .await
            .ok_or(Error::NotBootstrapped)?;

        let session = Self {
            client_pk,
            pending_queries: Arc::new(RwLock::new(HashMap::default())),
            incoming_err_sender: Arc::new(err_sender),
            endpoint: Some(endpoint),
            network: Arc::new(NetworkPrefixMap::new(genesis_pk)),
            bootstrap_peer,
            aggregator: Arc::new(RwLock::new(SignatureAggregator::new())),
            genesis_pk,
        };

        session
            .spawn_message_listener_thread(incoming_messages)
            .await;

        Ok(session)
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
}
