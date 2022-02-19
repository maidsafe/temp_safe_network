// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::NodeId;

use crate::types::log_markers::LogMarker;

use bytes::Bytes;
use priority_queue::DoublePriorityQueue;
use qp2p::{ConnectionIncoming as IncomingMsgs, Endpoint, RetryConfig};
use std::{
    collections::BTreeMap,
    net::SocketAddr,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};
use tokio::sync::RwLock;

type Priority = u64;
type ConnId = usize;

/// Errors that can be returned from `Comm::send_to_one`.
#[derive(Debug)]
pub(crate) enum SendToOneError {
    ///
    Connection(qp2p::ConnectionError),
    ///
    Send(qp2p::SendError),
}

impl SendToOneError {
    ///
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

///
#[derive(Clone, Debug)]
pub(crate) struct Connection {
    id: NodeId,
    data: Arc<RwLock<BTreeMap<ConnId, qp2p::Connection>>>,
    queue: Arc<RwLock<DoublePriorityQueue<ConnId, Priority>>>,
    counter: Arc<AtomicU64>,
    endpoint: Endpoint,
}

impl Connection {
    ///
    pub(crate) fn new(id: NodeId, endpoint: Endpoint) -> Self {
        Self {
            id,
            data: Arc::new(RwLock::new(BTreeMap::new())),
            queue: Arc::new(RwLock::new(DoublePriorityQueue::new())),
            counter: Arc::new(AtomicU64::new(u64::MAX)),
            endpoint,
        }
    }

    ///
    pub(crate) async fn new_with(id: NodeId, endpoint: Endpoint, conn: qp2p::Connection) -> Self {
        let instance = Self::new(id, endpoint);
        instance.insert(conn).await;
        instance
    }

    ///
    pub(crate) async fn add(&self, conn: qp2p::Connection) {
        self.insert(conn).await;
    }

    /// A stable identifier for the connection.
    ///
    /// This ID will not change for the lifetime of the connection.
    #[allow(unused)]
    pub(crate) fn id(&self) -> &NodeId {
        &self.id
    }

    /// The address of the remote peer.
    #[allow(unused)]
    pub(crate) fn remote_address(&self) -> SocketAddr {
        self.id.1
    }

    /// Send a message to the peer with default retry configuration.
    ///
    /// The message will be sent on a unidirectional QUIC stream, meaning the application is
    /// responsible for correlating any anticipated responses from incoming streams.
    ///
    /// The priority will be `0` and retry behaviour will be determined by the
    /// [`Config`](crate::Config) that was used to construct the [`Endpoint`] this connection
    /// belongs to. See [`send_with`](Self::send_with) if you want to send a message with specific
    /// configuration.
    #[allow(unused)]
    pub(crate) async fn send<F: Fn(qp2p::Connection, IncomingMsgs)>(
        &self,
        msg: Bytes,
        listen: F,
    ) -> Result<(), SendToOneError> {
        self.send_with(msg, 0, None, listen).await
    }

    /// Send a message to the peer using the given configuration.
    ///
    /// See [`send`](Self::send) if you want to send with the default configuration.
    pub(crate) async fn send_with<F: Fn(qp2p::Connection, IncomingMsgs)>(
        &self,
        msg: Bytes,
        priority: i32,
        retry_config: Option<&RetryConfig>,
        listen: F,
    ) -> Result<(), SendToOneError> {
        let conn = self.get_or_connect(listen).await?;
        match conn.send_with(msg, priority, retry_config).await {
            Ok(()) => Ok(()),
            Err(error) => {
                let id = &conn.id();
                {
                    let _ = self.data.write().await.remove(id);
                }
                {
                    let _ = self.queue.write().await.remove(id);
                }
                conn.close(Some(format!("{:?}", error)));
                Err(SendToOneError::Send(error))
            }
        }
    }

    async fn get_or_connect<F: Fn(qp2p::Connection, IncomingMsgs)>(
        &self,
        listen: F,
    ) -> Result<qp2p::Connection, SendToOneError> {
        let res = { self.queue.read().await.peek_min().map(|(id, _prio)| *id) };
        match res {
            None => self.create_connection(listen).await,
            Some(id) => {
                let res = { self.data.read().await.get(&id).cloned() };
                match res {
                    Some(conn) => {
                        self.touch(conn.id()).await;
                        Ok(conn)
                    }
                    None => self.create_connection(listen).await,
                }
            }
        }
    }

    async fn create_connection<F: Fn(qp2p::Connection, IncomingMsgs)>(
        &self,
        listen: F,
    ) -> Result<qp2p::Connection, SendToOneError> {
        let (conn, incoming_msgs) = self
            .endpoint
            .connect_to(&self.id.1)
            .await
            .map_err(SendToOneError::Connection)?;

        trace!(
            "{} to {} (id: {})",
            LogMarker::ConnectionOpened,
            conn.remote_address(),
            conn.id()
        );

        self.insert(conn.clone()).await;

        listen(conn.clone(), incoming_msgs);

        Ok(conn)
    }

    async fn insert(&self, conn: qp2p::Connection) {
        let id = conn.id();

        {
            let _ = self.data.write().await.insert(id, conn);
        }
        {
            let _ = self.queue.write().await.push(id, self.priority().await);
        }

        let len = { self.queue.read().await.len() as u8 };
        if len == u8::MAX {
            if let Some((evicted, _)) = self.queue.write().await.pop_max() {
                if let Some(conn) = self.data.write().await.remove(&evicted) {
                    trace!("Connection evicted: {}", evicted);
                    conn.close(Some("Connection evicted.".to_string()));
                }
            }
        }
    }

    async fn touch(&self, id: ConnId) {
        let _ = self
            .queue
            .write()
            .await
            .change_priority(&id, self.priority().await);
    }

    async fn priority(&self) -> Priority {
        let prio = self.counter.fetch_sub(1, Ordering::SeqCst);
        if prio == 0 {
            // after u64::MAX connections to this peer, we will see this small hiccup..
            // empty the cache when we overflow
            let mut queue = self.queue.write().await;
            let to_keep = match queue.peek_min() {
                Some((id, _)) => self.data.write().await.remove(id),
                None => None,
            };
            queue.clear();
            let mut data = self.data.write().await;
            data.clear();
            if let Some(conn) = to_keep {
                let id = conn.id();
                let _ = data.insert(id, conn);
                let _ = queue.push(id, self.counter.fetch_sub(1, Ordering::SeqCst));
            }
            self.counter.fetch_sub(1, Ordering::SeqCst)
        } else {
            prio
        }
    }
}
