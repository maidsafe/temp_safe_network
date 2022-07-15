// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod listeners;
mod messaging;

use crate::Result;
use sn_interface::{
    messaging::{
        data::{CmdError, OperationId, QueryResponse},
        MsgId,
    },
    network_knowledge::prefix_map::NetworkPrefixMap,
    types::PeerLinks,
};

use dashmap::DashMap;
use qp2p::{Config as QuicP2pConfig, Endpoint};
use secured_linked_list::SecuredLinkedList;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::{mpsc::Sender, RwLock};

// Here we dont track the msg_id across the network, but just use it as a local identifier to remove the correct listener
type PendingQueryResponses = Arc<DashMap<OperationId, Vec<(MsgId, QueryResponseSender)>>>;
type QueryResponseSender = Sender<QueryResponse>;

type CmdResponse = (SocketAddr, Option<CmdError>);
type PendingCmdAcks = Arc<DashMap<MsgId, Sender<CmdResponse>>>;

#[derive(Debug)]
pub struct QueryResult {
    pub response: QueryResponse,
    pub operation_id: OperationId,
}

#[derive(Clone, Debug)]
pub(super) struct Session {
    // Session endpoint.
    endpoint: Endpoint,
    // Channels for sending responses to upper layers
    pending_queries: PendingQueryResponses,
    // Channels for sending errors to upper layer
    #[allow(dead_code)]
    incoming_err_sender: Arc<Sender<CmdError>>,
    // Channels for sending CmdAck to upper layers
    pending_cmds: PendingCmdAcks,
    /// All elders we know about from AE messages
    network: Arc<NetworkPrefixMap>,
    /// A DAG containing all section chains of the whole network that we are aware of
    all_sections_chains: Arc<RwLock<SecuredLinkedList>>,
    /// Initial network comms MsgId
    initial_connection_check_msg_id: Arc<RwLock<Option<MsgId>>>,
    /// Standard time to await potential AE messages:
    cmd_ack_wait: Duration,
    /// Links to nodes
    peer_links: PeerLinks,
}

impl Session {
    /// Acquire a session by bootstrapping to a section, maintaining connections to several nodes.
    #[instrument(skip(err_sender), level = "debug")]
    pub(crate) fn new(
        genesis_key: bls::PublicKey,
        qp2p_config: QuicP2pConfig,
        err_sender: Sender<CmdError>,
        local_addr: SocketAddr,
        cmd_ack_wait: Duration,
        prefix_map: NetworkPrefixMap,
    ) -> Result<Session> {
        let endpoint = Endpoint::new_client(local_addr, qp2p_config)?;
        let peer_links = PeerLinks::new(endpoint.clone());

        let session = Session {
            pending_queries: Arc::new(DashMap::default()),
            incoming_err_sender: Arc::new(err_sender),
            pending_cmds: Arc::new(DashMap::default()),
            endpoint,
            network: Arc::new(prefix_map),
            initial_connection_check_msg_id: Arc::new(RwLock::new(None)),
            cmd_ack_wait,
            peer_links,
            all_sections_chains: Arc::new(RwLock::new(SecuredLinkedList::new(genesis_key))),
        };

        Ok(session)
    }
}
