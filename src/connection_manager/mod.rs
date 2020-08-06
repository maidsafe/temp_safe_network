// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod connection_group;
mod response_manager;
use tokio::time::timeout;

use crate::{
    client::SafeKey, client::TransferActor, network_event::NetworkEvent, network_event::NetworkTx,
    CoreError,
};
use connection_group::ConnectionGroup;
use futures::lock::Mutex;
use log::{error, trace};
use quic_p2p::Config as QuicP2pConfig;
use safe_nd::{DebitAgreementProof, Message, PublicId, QueryResponse};
use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
    time::Duration,
};

const CONNECTION_TIMEOUT_SECS: u64 = 30;

/// Initialises `QuicP2p` instance. Establishes new connections.
/// Contains a reference to crossbeam channel provided by quic-p2p for capturing the events.
#[derive(Clone, Debug)]
pub struct ConnectionManager {
    inner: Arc<Mutex<Inner>>,
}

impl ConnectionManager {
    /// Create a new connection manager.
    pub fn new(mut config: QuicP2pConfig, net_tx: &NetworkTx) -> Result<Self, CoreError> {
        config.port = Some(0); // Make sure we always use a random port for client connections.

        let inner = Arc::new(Mutex::new(Inner {
            config,
            groups: HashMap::default(),
            net_tx: net_tx.clone(),
        }));

        Ok(Self { inner })
    }

    /// Returns `true` if this connection manager is already connected to a Client Handlers
    /// group serving the provided public ID.
    pub async fn has_connection_to(&self, pub_id: &PublicId) -> bool {
        let inner = self.inner.lock().await;
        inner.groups.contains_key(&pub_id)
    }

    /// Send `message` via the `ConnectionGroup` specified by our given `pub_id`.
    pub async fn send_query(
        &mut self,
        pub_id: &PublicId,
        msg: &Message,
    ) -> Result<QueryResponse, CoreError> {
        self.inner.lock().await.send_query(pub_id, msg).await
    }

    /// Send `message` via the `ConnectionGroup` specified by our given `pub_id`, does not expect response
    pub async fn send_cmd(&mut self, pub_id: &PublicId, msg: &Message) -> Result<(), CoreError> {
        self.inner.lock().await.send_cmd(pub_id, msg).await
    }

    /// Send `message` via the `ConnectionGroup` specified by our given `pub_id`. Wait for DebitAgreementProof generation before replying
    pub async fn send_for_validation(
        &mut self,
        pub_id: &PublicId,
        msg: &Message,
        transfer_actor: &mut TransferActor,
    ) -> Result<DebitAgreementProof, CoreError> {
        self.inner
            .lock()
            .await
            .send_for_validation(pub_id, msg, transfer_actor)
            .await
    }

    /// Connect to Client Handlers that manage the provided ID.
    pub async fn bootstrap(&mut self, full_id: SafeKey) -> Result<(), CoreError> {
        self.inner.lock().await.bootstrap(full_id).await
    }

    /// Reconnect to the network.
    pub fn restart_network(&mut self) {
        unimplemented!();
    }

    /// Disconnect from a group.
    pub async fn disconnect(&mut self, pub_id: &PublicId) -> Result<(), CoreError> {
        self.inner.lock().await.disconnect(pub_id).await
    }
}

struct Inner {
    config: QuicP2pConfig,
    groups: HashMap<PublicId, ConnectionGroup>,
    net_tx: NetworkTx,
}

impl Drop for Inner {
    fn drop(&mut self) {
        // Disconnect from all groups gracefully
        trace!("Dropped ConnectionManager - terminating gracefully");
        let _ = self.net_tx.unbounded_send(NetworkEvent::Disconnected);
    }
}

impl Inner {
    async fn bootstrap(&mut self, full_id: SafeKey) -> Result<(), CoreError> {
        trace!("Trying to bootstrap with group {:?}", full_id.public_id());

        let (connected_tx, connected_rx) = futures::channel::oneshot::channel();

        if let Entry::Vacant(value) = self.groups.entry(full_id.public_id()) {
            let _ = value
                .insert(ConnectionGroup::new(self.config.clone(), full_id, connected_tx).await?);

            match timeout(Duration::from_secs(CONNECTION_TIMEOUT_SECS), connected_rx).await {
                Ok(response) => response.map_err(|err| CoreError::from(format!("{}", err)))?,
                Err(_) => Err(CoreError::from(
                    "Connection timed out when bootstrapping to the network",
                )),
            }
        } else {
            trace!("Group {} is already connected", full_id.public_id());
            Ok(())
        }
    }

    async fn send_query(
        &mut self,
        pub_id: &PublicId,
        msg: &Message,
    ) -> Result<QueryResponse, CoreError> {
        let conn_group = self.groups.get_mut(&pub_id).ok_or_else(|| {
            CoreError::Unexpected(
                "No connection group found - did you call `bootstrap`?".to_string(),
            )
        })?;

        conn_group.send_query(msg).await
    }

    async fn send_cmd(&mut self, pub_id: &PublicId, msg: &Message) -> Result<(), CoreError> {
        let msg_id = msg.id();

        let conn_group = self.groups.get_mut(&pub_id).ok_or_else(|| {
            CoreError::Unexpected(
                "No connection group found - did you call `bootstrap`?".to_string(),
            )
        })?;

        conn_group.send_cmd(msg_id, msg).await
    }

    async fn send_for_validation(
        &mut self,
        pub_id: &PublicId,
        msg: &Message,
        transfer_actor: &mut TransferActor,
    ) -> Result<DebitAgreementProof, CoreError> {
        let msg_id = msg.id();

        let conn_group = self.groups.get_mut(&pub_id).ok_or_else(|| {
            CoreError::Unexpected(
                "No connection group found - did you call `bootstrap`?".to_string(),
            )
        })?;

        let proof = conn_group
            .send_for_validation(&msg_id, msg, transfer_actor)
            .await?;

        Ok(proof)
    }

    /// Disconnect from a group.
    pub async fn disconnect(&mut self, pub_id: &PublicId) -> Result<(), CoreError> {
        trace!("Disconnecting group {:?}", pub_id);

        let group = self.groups.remove(&pub_id);

        if let Some(mut group) = group {
            group.close().await.map(move |res| {
                // Drop the group once it's disconnected
                let _ = group;
                res
            })
        } else {
            error!("No group found for {}", pub_id); // FIXME: handle properly
            Ok(())
        }
    }
}
