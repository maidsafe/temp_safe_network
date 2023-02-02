// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use sn_interface::{
    messaging::{
        data::{ClientDataResponse, ClientMsg},
        system::NodeDataResponse,
        MsgId,
    },
    types::Peer,
};

use lazy_static::lazy_static;
use qp2p::SendStream;
use std::{
    collections::{BTreeMap, BTreeSet},
    env::var,
    str::FromStr,
    sync::Arc,
    time::Duration,
};
use tokio::{sync::RwLock, time::Instant};
use xor_name::XorName;

///
pub(crate) struct DataResponses {
    pending: BTreeMap<MsgId, PendingRequest>,
}

impl DataResponses {
    pub(crate) fn new() -> Self {
        Self {
            pending: BTreeMap::new(),
        }
    }

    pub(crate) fn track(
        &mut self,
        msg_id: MsgId,
        client_name: XorName,
        causing_msg: ClientMsg,
        client_stream: SendStream,
        expected_responders: BTreeSet<Peer>,
    ) {
        if self.pending.contains_key(&msg_id) {
            return;
        }
        let _ = self.pending.insert(
            msg_id,
            PendingRequest::new(causing_msg, client_name, client_stream, expected_responders),
        );
    }

    pub(crate) fn update(
        &mut self,
        peer: Peer,
        response: NodeDataResponse,
    ) -> Option<(ClientDataResponse, XorName, Arc<RwLock<SendStream>>)> {
        let key = response.correlation_id();
        let pending = self.pending.get_mut(key)?;
        let msg = pending.update(peer, response)?;
        Some((msg, pending.client_name, pending.client_stream.clone()))
    }

    /// Removes expired requests and returns peers that didn't respond in time.
    pub(crate) fn clear_expired(&mut self) -> BTreeSet<Peer> {
        let mut timed_out_nodes = BTreeSet::new();

        let keys: Vec<_> = self
            .pending
            .iter()
            .filter_map(|(key, p)| {
                if p.sent.elapsed() >= NODE_RESPONSE_DEFAULT_TIMEOUT {
                    Some(key)
                } else {
                    None
                }
            })
            .copied()
            .collect();

        for key in keys {
            if let Some(pending) = self.pending.remove(&key) {
                timed_out_nodes.extend(pending.missing_responders())
            }
        }

        timed_out_nodes
    }
}

struct PendingRequest {
    sent: Instant,
    msg: ClientMsg,
    client_name: XorName,
    client_stream: Arc<RwLock<SendStream>>,
    received: BTreeMap<Peer, NodeDataResponse>,
    expected_responders: BTreeSet<Peer>,
}

impl PendingRequest {
    fn new(
        msg: ClientMsg,
        client_name: XorName,
        client_stream: SendStream,
        expected_responders: BTreeSet<Peer>,
    ) -> Self {
        Self {
            sent: Instant::now(),
            msg,
            client_name,
            client_stream: Arc::new(RwLock::new(client_stream)),
            received: BTreeMap::new(),
            expected_responders,
        }
    }

    fn update(&mut self, peer: Peer, response: NodeDataResponse) -> Option<ClientDataResponse> {
        if !self.expected_responders.contains(&peer) {
            return None;
        }
        if !self.matches(&response) {
            return None;
        }

        let _ = self.received.insert(peer, response);

        if self.received.len() != self.expected_responders.len() {
            // not enough responses yet
            return None;
        }

        let (succeeded, failed): (Vec<_>, Vec<_>) =
            self.received.values().partition(|r| r.is_success());
        // if any failed, that will be the response
        for response in failed {
            if let Some(to_client) = map_to_client_response(&self.msg, response.clone()) {
                return Some(to_client);
            }
        }
        // else we have a success (we require 100% success for now)
        for response in succeeded {
            if let Some(to_client) = map_to_client_response(&self.msg, response.clone()) {
                return Some(to_client);
            }
        }

        None
        // We previously took the last success after _all_ had succeeded, an equivalent behavior is maintained here.
    }

    fn missing_responders(&self) -> BTreeSet<Peer> {
        let responded = self.received.keys().copied().collect();
        self.expected_responders
            .difference(&responded)
            .copied()
            .collect()
    }

    fn matches(&self, response: &NodeDataResponse) -> bool {
        (matches!(self.msg, ClientMsg::Cmd(_))
            && matches!(response, NodeDataResponse::CmdResponse { .. }))
            || (matches!(self.msg, ClientMsg::Query(_))
                && matches!(response, NodeDataResponse::QueryResponse { .. }))
    }
}

/// Verify what kind of response was received, and if that's the expected type based on
/// the type of msg sent to the nodes, then forward the corresponding response to the client
fn map_to_client_response(
    original_msg: &ClientMsg,
    response: NodeDataResponse,
) -> Option<ClientDataResponse> {
    match original_msg {
        ClientMsg::Query(_) => {
            match response {
                NodeDataResponse::QueryResponse {
                    response,
                    correlation_id,
                } => {
                    // We sent a data query and we received a query response,
                    // so let's forward it to the client
                    debug!("{correlation_id:?} sending query response back to client");
                    Some(ClientDataResponse::QueryResponse {
                        response,
                        correlation_id,
                    })
                }
                NodeDataResponse::CmdResponse {
                    response,
                    correlation_id,
                } => {
                    // TODO: handle this bad response
                    error!("Unexpected response to query from node for {correlation_id:?}: {response:?}");
                    None
                }
            }
        }
        ClientMsg::Cmd(_) => {
            match response {
                NodeDataResponse::CmdResponse {
                    response,
                    correlation_id,
                } => {
                    // We sent a data cmd to store client data and we received a
                    // cmd response, so let's forward it to the client
                    debug!("{correlation_id:?} sending cmd response ACK back to client");
                    Some(ClientDataResponse::CmdResponse {
                        response,
                        correlation_id,
                    })
                }
                NodeDataResponse::QueryResponse {
                    response,
                    correlation_id,
                } => {
                    // TODO: handle this bad response
                    error!("Unexpected response to query from node for {correlation_id:?}: {response:?}");
                    None
                }
            }
        }
    }
}

/// Environment variable to set timeout value (in seconds) for data queries
/// forwarded to Adults. Default value (`NODE_RESPONSE_DEFAULT_TIMEOUT`) is otherwise used.
const ENV_NODE_RESPONSE_TIMEOUT: &str = "SN_NODE_RESPONSE_TIMEOUT";

// Default timeout period set for data queries forwarded to Adult.
// TODO: how to determine this time properly?
const NODE_RESPONSE_DEFAULT_TIMEOUT: Duration = Duration::from_secs(70);

lazy_static! {
    static ref NODE_RESPONSE_TIMEOUT: Duration = match var(ENV_NODE_RESPONSE_TIMEOUT)
        .map(|v| u64::from_str(&v))
    {
        Ok(Ok(secs)) => {
            let timeout = Duration::from_secs(secs);
            info!("{ENV_NODE_RESPONSE_TIMEOUT} env var set, Node data query response timeout set to {timeout:?}");
            timeout
        }
        Ok(Err(err)) => {
            warn!(
                "Failed to parse {ENV_NODE_RESPONSE_TIMEOUT} value, using \
                default value ({NODE_RESPONSE_DEFAULT_TIMEOUT:?}): {err:?}"
            );
            NODE_RESPONSE_DEFAULT_TIMEOUT
        }
        Err(_) => NODE_RESPONSE_DEFAULT_TIMEOUT,
    };
}
