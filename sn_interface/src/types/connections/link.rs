// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::Peer;

use crate::messaging::MsgId;
use crate::types::log_markers::LogMarker;
use qp2p::{Connection, Endpoint, RecvStream, UsrMsgBytes};
// Required for docs
#[allow(unused_imports)]
use qp2p::RetryConfig;
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::RwLock;
use xor_name::XorName;

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
#[derive(Clone, Debug)]
pub struct Link {
    peer: Peer,
    endpoint: Endpoint,
    connections: Arc<RwLock<BTreeMap<ConnId, Connection>>>,
}

impl Link {
    pub(crate) fn new(peer: Peer, endpoint: Endpoint) -> Self {
        Self {
            peer,
            endpoint,
            connections: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    #[allow(unused)]
    pub(crate) fn name(&self) -> XorName {
        self.peer.name()
    }

    pub async fn send_bi(
        &self,
        bytes: UsrMsgBytes,
        msg_id: MsgId,
    ) -> Result<RecvStream, SendToOneError> {
        debug!("sending bidi msg out... {msg_id:?} ");
        let conn = self.get_or_connect(msg_id).await?;
        debug!("conenction got {msg_id:?}");
        let (mut send_stream, recv_stream) =
            conn.open_bi().await.map_err(SendToOneError::Connection)?;

        debug!("{msg_id:?} bidi opened");
        send_stream.set_priority(10);
        send_stream
            .send_user_msg(bytes.clone())
            .await
            .map_err(SendToOneError::Send)?;
        debug!("{msg_id:?} bidi msg sent");

        send_stream.finish().await.or_else(|err| match err {
            qp2p::SendError::StreamLost(qp2p::StreamError::Stopped(_)) => Ok(()),
            _ => Err(SendToOneError::Send(err)),
        })?;
        debug!("{msg_id:?} bidi finished");
        Ok(recv_stream)
    }

    // Get a connection or create a fresh one
    async fn get_or_connect(&self, msg_id: MsgId) -> Result<Connection, SendToOneError> {
        debug!("Attempting to get conn read lock... {msg_id:?}");
        let empty_conns = self.connections.read().await.is_empty();
        debug!("lockgottt {msg_id:?}");
        if empty_conns {
            // read again
            // first caller will find none again, but the subsequent callers
            // will access only after the first one finished creating a new connection
            // thus will find a connection here:
            debug!("{msg_id:?} creating conn with {:?}", self.peer);
            self.create_connection(msg_id).await
        } else {
            debug!("{msg_id:?} connections do exist...");
            // TODO: add in simple connection check when available.
            // we can then remove dead conns easily and return only valid conns
            let connections = self.connections.read().await;
            let conn = connections.iter().next().map(|(_, c)| c.clone());
            // we have to drop connection read here before we attempt to create (and write) connections
            drop(connections);
            Ok(conn.unwrap_or(self.create_connection(msg_id).await?))
        }
    }

    /// Is this Link currently connected?
    pub(crate) async fn is_connected(&self) -> bool {
        // self.remove_expired().await;
        !self.connections.read().await.is_empty()
    }

    /// Uses qp2p to create a connection and stores it on Self
    async fn create_connection(&self, msg_id: MsgId) -> Result<Connection, SendToOneError> {
        // grab write lock to prevent many many conns being opened at once
        let mut conns = self.connections.write().await;

        debug!("{msg_id:?} creating connnnn to {:?}", self.peer);
        let (conn, _incoming_msgs) = self
            .endpoint
            .connect_to(&self.peer.addr())
            .await
            .map_err(SendToOneError::Connection)?;

        debug!("conn creating done {:?}", self.peer);
        trace!(
            "{} to {} (id: {})",
            LogMarker::ConnectionOpened,
            conn.remote_address(),
            conn.id()
        );

        let _ = conns.insert(conn.id(), conn.clone());

        Ok(conn)
    }
}

/// Errors that can be returned from `Comm::send_to_one`.
#[derive(Debug)]
pub enum SendToOneError {
    ///
    Connection(qp2p::ConnectionError),
    ///
    Send(qp2p::SendError),
    #[cfg(feature = "chaos")]
    /// ChaosNoConn
    ChaosNoConnection,
    /// Sending failed repeatedly
    SendRepeatedlyFailed,
}

impl SendToOneError {
    ///
    #[allow(unused)]
    pub(crate) fn is_local_close(&self) -> bool {
        matches!(
            self,
            Self::Connection(qp2p::ConnectionError::Closed(qp2p::Close::Local))
                | Self::Send(qp2p::SendError::ConnectionLost(
                    qp2p::ConnectionError::Closed(qp2p::Close::Local)
                ))
        )
    }
}
