// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::MsgListener;

use dashmap::DashMap;
use qp2p::{Connection, Endpoint, RetryConfig, UsrMsgBytes};
use sn_interface::types::{log_markers::LogMarker, Peer};
use std::sync::Arc;

type ConnId = String;

/// A link to a peer in our network.
///
/// The upper layers will add incoming connections to the link,
/// and use the link to send msgs.
/// Using the link will open a connection if there is none there.
/// The link is a way to keep connections to a peer in one place
/// and use them efficiently; converge to a single one regardless of concurrent
/// comms initiation between the peers, and so on.
/// Unused connections will expire, so the Link is cheap to keep around.
/// The Link is kept around as long as the peer is deemed worth to keep contact with.
#[derive(Clone)]
pub(crate) struct Link {
    peer: Peer,
    endpoint: Endpoint,
    pub(crate) connections: LinkConnections,
    listener: MsgListener,
}

pub(crate) type LinkConnections = Arc<DashMap<ConnId, Connection>>;

impl Link {
    pub(crate) fn new(peer: Peer, endpoint: Endpoint, listener: MsgListener) -> Self {
        Self {
            peer,
            endpoint,
            connections: Arc::new(DashMap::new()),
            listener,
        }
    }

    pub(crate) async fn new_with(
        peer: Peer,
        endpoint: Endpoint,
        listener: MsgListener,
        conn: Connection,
    ) -> Self {
        let mut instance = Self::new(peer, endpoint, listener);
        instance.insert(conn).await;
        instance
    }

    pub(crate) fn peer(&self) -> &Peer {
        &self.peer
    }

    pub(crate) async fn add(&mut self, conn: Connection) {
        self.insert(conn).await;
    }

    /// Send a message to the peer using the given configuration.
    ///
    /// See [`send`](Self::send) if you want to send with the default configuration.
    #[instrument(skip_all)]
    pub(crate) async fn send_with_connection(
        bytes: UsrMsgBytes,
        priority: i32,
        retry_config: Option<&RetryConfig>,
        conn: Connection,
        connections: LinkConnections,
    ) -> Result<(), SendToOneError> {
        trace!(
            "We have {} open connections to node {:?}.",
            connections.len(),
            conn.id()
        );

        match conn.send_with(bytes, priority, retry_config).await {
            Ok(()) => Ok(()),
            Err(error) => {
                error!(
                    "Error sending out from link... We have {} open connections to node {:?}.",
                    connections.len(),
                    conn.id()
                );
                // clean up failing connections at once, no nead to leak it outside of here
                // next send (e.g. when retrying) will use/create a new connection
                let id = &conn.id();
                // We could write just `self.connections.remove(id)`, but the library warns for `unused_results`.
                {
                    // Timeouts etc should register instantly so we should clean those up fair fast
                    let _ = connections.remove(id);
                }
                // dont close just let the conn timeout incase msgs are coming in...
                // it's removed from out Peer tracking, so wont be used again for sending.
                Err(SendToOneError::Send(error))
            }
        }
    }

    pub(crate) async fn get_or_connect(
        &mut self,
        should_establish_new_connection: bool,
    ) -> Result<Connection, SendToOneError> {
        if self.connections.is_empty() {
            if should_establish_new_connection {
                self.create_connection().await
            } else {
                Err(SendToOneError::NoConnection)
            }
        } else {
            // let mut fastest_conn = None;
            if let Some(conn) = self.connections.iter().next() {
                return Ok(conn.value().clone());
            }
            // This should not be possible to hit...
            Err(SendToOneError::NoConnection)
        }
    }

    async fn create_connection(&mut self) -> Result<Connection, SendToOneError> {
        let (conn, incoming_msgs) = self
            .endpoint
            .connect_to(&self.peer.addr())
            .await
            .map_err(SendToOneError::Connection)?;

        trace!(
            "{} to {} (id: {})",
            LogMarker::ConnectionOpened,
            conn.remote_address(),
            conn.id()
        );

        self.insert(conn.clone()).await;

        self.listener.listen(conn.clone(), incoming_msgs);

        Ok(conn)
    }

    async fn insert(&mut self, conn: Connection) {
        let id = conn.id();

        let _ = self.connections.insert(id, conn);
    }
}

/// Errors that can be returned from `Comm::send_to_one`.
#[derive(Debug)]
pub(crate) enum SendToOneError {
    ///
    Connection(qp2p::ConnectionError),
    ///
    Send(qp2p::SendError),
    /// No Connection Exists to send on, as required by should_establish_new_connection
    NoConnection,
}

impl SendToOneError {
    ///
    #[allow(unused)]
    pub(crate) fn is_local_close(&self) -> bool {
        matches!(
            self,
            SendToOneError::Connection(qp2p::ConnectionError::Closed(qp2p::Close::Local))
                | SendToOneError::Send(qp2p::SendError::ConnectionLost(
                    qp2p::ConnectionError::Closed(qp2p::Close::Local)
                ))
        )
    }
}
