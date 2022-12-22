// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod listeners;
mod messaging;

use crate::{connections::PeerLinks, Error, Result};

use sn_interface::{
    messaging::data::{CmdResponse, QueryResponse},
    network_knowledge::SectionTree,
};

use qp2p::{Config as QuicP2pConfig, Endpoint};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;

// Use for internal communication between the bi-stream response listener threads
// and the thread waiting to aggregate responses
#[derive(Debug)]
pub(super) enum MsgResponse {
    CmdResponse(SocketAddr, Box<CmdResponse>),
    QueryResponse(SocketAddr, Box<QueryResponse>),
    Failure(SocketAddr, Error),
}

#[derive(Debug)]
pub struct QueryResult {
    pub response: QueryResponse,
}

impl QueryResult {
    /// Returns true if the QueryResponse is DataNotFound
    pub(crate) fn data_was_found(&self) -> bool {
        let found = !self.response.is_data_not_found();

        debug!("was the data found??? {found:?}, {self:?}");

        found
    }
}

#[derive(Clone, Debug)]
pub(super) struct Session {
    // Session endpoint.
    pub(super) endpoint: Endpoint,
    /// All elders we know about from AE messages
    pub(super) network: Arc<RwLock<SectionTree>>,
    /// Links to nodes
    peer_links: PeerLinks,
}

impl Session {
    /// Acquire a session by bootstrapping to a section, maintaining connections to several nodes.
    #[instrument(level = "debug")]
    pub(crate) fn new(
        mut qp2p_config: QuicP2pConfig,
        local_addr: SocketAddr,
        network_contacts: SectionTree,
    ) -> Result<Self> {
        qp2p_config.max_concurrent_bidi_streams = Some(500);
        let endpoint = Endpoint::new_client(local_addr, qp2p_config)?;
        let peer_links = PeerLinks::new(endpoint.clone());

        let session = Self {
            endpoint,
            network: Arc::new(RwLock::new(network_contacts)),
            peer_links,
        };

        Ok(session)
    }
}
