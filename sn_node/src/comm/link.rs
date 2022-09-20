// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::MsgListener;

use dashmap::DashMap;
use priority_queue::DoublePriorityQueue;
use qp2p::{Connection, UsrMsgBytes};
use qp2p::{Endpoint, RetryConfig};
use sn_interface::types::{log_markers::LogMarker, Peer};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;

type Priority = u64;
type ConnId = String;

// Capacity is needed since we cannot control how many connections
// another node opens to us (however, if they run the same code that
// we do, they would open very few connections).
// 255 is way more than we need and expect, thus gives ample room for
// unforseen bursts, but at the same time puts a sane cap on the max
// number continuously held for an - obviously malfunctioning - peer (i.e. edge case).
const CAPACITY: usize = 255;
// How long before quiet connections are cleaned up.
const UNUSED_TTL: Duration = Duration::from_secs(120);

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
pub(crate) struct Link {
    peer: Peer,
    endpoint: Endpoint,
    pub(crate) connections: LinkConnections,
    pub(crate) queue: LinkQueue,
    access_counter: u64,
    listener: MsgListener,
    expiration_check: Instant,
}

pub(crate) type LinkConnections = Arc<DashMap<ConnId, ExpiringConn>>;
pub(crate) type LinkQueue = Arc<RwLock<DoublePriorityQueue<ConnId, Priority>>>;

impl Link {
    pub(crate) fn new(peer: Peer, endpoint: Endpoint, listener: MsgListener) -> Self {
        Self {
            peer,
            endpoint,
            connections: Arc::new(DashMap::new()),
            queue: Arc::new(RwLock::new(DoublePriorityQueue::new())),
            access_counter: 0,
            listener,
            expiration_check: expiration(),
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

    /// Disposes of the link and all underlying resources.
    /// Also any clones of this link that are held, will be cleaned up.
    /// This is due to the fact that we do never leak the `Connection` outside of this struct,
    /// since that struct is cloneable and uses Arc internally.
    pub(crate) async fn disconnect(&mut self) {
        self.queue.write().await.clear();
        for refmulti in self.connections.iter() {
            let item = refmulti.value();
            item.conn
                .close(Some("We disconnected from peer.".to_string()));
        }
        self.connections.clear();
    }

    /// Send a message to the peer using the given configuration.
    ///
    /// See [`send`](Self::send) if you want to send with the default configuration.
    #[instrument(skip_all)]
    pub(crate) async fn send_with(
        // &mut self,
        bytes: UsrMsgBytes,
        priority: i32,
        retry_config: Option<&RetryConfig>,
        conn: Connection,
        connections: LinkConnections,
        queue: LinkQueue,
    ) -> Result<(), SendToOneError> {
        // trace!(
        //     "We have {} open connections to node {:?}.",
        //     self.queue.len(),
        //     self.peer
        // );

        match conn.send_with(bytes, priority, retry_config).await {
            Ok(()) => Ok(()),
            Err(error) => {
                // clean up failing connections at once, no nead to leak it outside of here
                // next send (e.g. when retrying) will use/create a new connection
                let id = &conn.id();
                // We could write just `self.queue.remove(id)`, but the library warns for `unused_results`.
                {
                    let _ = connections.remove(id);
                    let _ = queue.write().await.remove(id);
                }
                conn.close(Some(format!("{:?}", error)));
                Err(SendToOneError::Send(error))
            }
        }
    }

    pub(crate) async fn get_or_connect(&mut self) -> Result<Connection, SendToOneError> {
        // get the most recently used connection
        let q = self.queue.read().await;
        let conn = q.peek_max().map(|(id, _prio)| id.clone());
        // drop the initial immutable borrow
        // so we can create conn if required
        drop(q);
        match conn {
            None => {
                // if should_establish_new_connection {
                self.create_connection().await
                // } else {
                //     Err(SendToOneError::NoConnection)
                // }
            }
            Some(id) => self.read_conn(id).await,
        }
    }

    // /// Is this Link currently connected?
    // #[allow(unused)]
    // pub(crate) async fn is_connected(&self) -> bool {
    //     // get the most recently used connection

    //     self.queue
    //     .read().await
    //         .peek_max()
    //         .and_then(|(id, _)| self.connections.get(id))
    //         .map(|conn| !conn.expired())
    //         .unwrap_or(false)
    // }

    async fn read_conn(&mut self, id: ConnId) -> Result<Connection, SendToOneError> {
        let conn = self.connections.get_mut(&id);

        let mut existing_conn = None;
        // let mut should_create_conn = false;
        if let Some(ref entry) = conn {
            let item = entry.value();
            // let id = item.conn.id();
            let connec = item.conn.clone();

            existing_conn = Some(connec);
            // drop(item);
            // drop(entry);
        }

        drop(conn);

        if let Some(conn) = existing_conn {
            self.touch(id).await;
            return Ok(conn);
        }

        // if should_create_conn {
        self.create_connection().await
        // }
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

        let _ = self.connections.insert(id.clone(), ExpiringConn::new(conn));

        let prio = self.priority().await;
        let _ = self.queue.write().await.push(id, prio);
    }

    async fn touch(&mut self, id: ConnId) {
        let prio = self.priority().await;
        let _ = self.queue.write().await.change_priority(&id, prio);

        if let Some(mut conn) = self.connections.get_mut(&id) {
            conn.touch()
        }
    }

    async fn priority(&mut self) -> Priority {
        if self.access_counter == u64::MAX {
            // after u64::MAX connections to this peer (very unlikely), we need to update the prios
            let sorted_queue = self.queue.read().await.clone().into_sorted_iter();
            // update all prios, starting from zero prio again
            for (new_prio, (id, _old_prio)) in sorted_queue.enumerate() {
                let _ = self
                    .queue
                    .write()
                    .await
                    .change_priority(&id, new_prio as u64);
            }

            self.access_counter = self.queue.read().await.len() as u64;
        }

        self.access_counter = self.access_counter.saturating_add(1);

        self.access_counter
    }

    /// Remove expired connections.
    pub(crate) async fn remove_expired(&mut self) {
        if Instant::now() < self.expiration_check {
            return;
        }

        self.expiration_check = expiration();

        let mut expired_ids = vec![];

        let read_queue = self.queue.read().await.clone();

        // the iter is sorted from lowest to highest
        for (id, _old_prio) in read_queue.into_sorted_iter() {
            if 1 + expired_ids.len() >= self.queue.read().await.len() {
                break;
            }

            if let Some(conn) = self.connections.get_mut(&id) {
                if conn.expired() {
                    expired_ids.push(id);
                }
            }
        }

        for id in expired_ids {
            let _ = self.queue.write().await.remove(&id);

            if let Some((_, item)) = self.connections.remove(&id) {
                trace!("Connection expired: {}", item.conn.id());
                item.conn.close(Some("Connection expired.".to_string()));
            }
        }

        self.drop_excess().await;
    }

    /// Are we still connected here  after cleanup
    /// retruens true if a connection remains
    pub(crate) async fn is_connected_after_cleanup(&mut self) -> bool {
        self.remove_expired().await;
        !self.connections.is_empty()
    }
    /// Remove connections that exceed capacity, oldest first.
    async fn drop_excess(&mut self) {
        let mut queue = self.queue.write().await;
        if queue.len() >= CAPACITY {
            // remove the least recently used connections
            if let Some((evicted_id, _)) = queue.pop_min() {
                if let Some((evicted_id, item)) = self.connections.remove(&evicted_id) {
                    trace!("Connection evicted: {}", evicted_id);
                    item.conn.close(Some("Connection evicted.".to_string()));
                }
            }
        }
    }
}

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

#[derive(Clone, Debug)]
pub(crate) struct ExpiringConn {
    conn: Connection,
    expiry: Instant,
}

impl ExpiringConn {
    fn new(conn: Connection) -> Self {
        ExpiringConn {
            conn,
            expiry: expiration(),
        }
    }

    fn expired(&self) -> bool {
        self.expiry < Instant::now()
    }

    fn touch(&mut self) {
        self.expiry = expiration();
    }
}

fn expiration() -> Instant {
    Instant::now() + UNUSED_TTL
}
