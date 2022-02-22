// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use dashmap::DashMap;
use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
    future::Future,
    hash::{Hash, Hasher},
    net::SocketAddr,
    sync::Arc,
};
use xor_name::{XorName, XOR_NAME_LEN};

/// Network peer identity.
///
/// When a node knows another node as a `Peer` it's logically connected to it. This is separate from
/// being physically connected at the network layer, which is indicated by the optional `connection`
/// field.
#[derive(Clone)]
pub struct Peer {
    name: XorName,
    addr: SocketAddr,

    // Connections to the peer. There are no guarantees about the state of the connection
    // except that it once connected to this peer's `addr` (e.g. it may already be closed or
    // otherwise unusable).
    connections: Arc<DashMap<usize, qp2p::Connection>>,
}

impl fmt::Debug for Peer {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let connection_ids: Vec<usize> = self.connection_ids();
        f.debug_struct("Peer")
            .field("name", &self.name)
            .field("addr", &self.addr)
            .field("connection_ids", &connection_ids)
            .finish()
    }
}

impl Display for Peer {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{} at {} ({:?})",
            self.name,
            self.addr,
            self.connection_ids()
        )
    }
}

impl Eq for Peer {}

impl Hash for Peer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.addr.hash(state);
    }
}

impl Ord for Peer {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name
            .cmp(&other.name)
            .then_with(|| self.addr.cmp(&other.addr))
    }
}

impl PartialEq for Peer {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.addr == other.addr
    }
}

impl PartialEq<&Self> for Peer {
    fn eq(&self, other: &&Self) -> bool {
        self.name == other.name && self.addr == other.addr
    }
}

impl PartialOrd for Peer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Peer {
    /// Creates a new `Peer` given `Name`, `SocketAddr`.
    pub(crate) fn new(name: XorName, addr: SocketAddr) -> Self {
        Self {
            name,
            addr,
            connections: Arc::new(DashMap::new()),
        }
    }

    /// Returns the `XorName` of the peer.
    pub(crate) fn name(&self) -> XorName {
        self.name
    }

    /// Returns the `SocketAddr`.
    pub(crate) fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Returns the age.
    pub(crate) fn age(&self) -> u8 {
        self.name[XOR_NAME_LEN - 1]
    }

    /// Remove the specified connection, if any.
    pub(crate) fn remove_connection(&self, connection_id: usize) {
        let _ = self.connections.remove(&connection_id);
    }

    /// Get the connection to the peer, if any.
    pub(crate) fn get_connection(&self) -> Option<qp2p::Connection> {
        self.connections
            .iter()
            .next()
            .map(|item| item.value().clone())
    }

    /// Get the connection_ids to the peer, if any.
    pub(crate) fn connection_ids(&self) -> Vec<usize> {
        self.connections.iter().map(|item| *item.key()).collect()
    }

    /// Copy the connections from another peer, if this peer doesn't have one.
    pub(crate) async fn merge_connections(&self, other: &Self) {
        // As a quick sanity check, do nothing if the addresses differ
        if self.addr != other.addr {
            return;
        }
        for item in other.connections.iter() {
            // Another sanity check: the connection itself matches our address
            if self.addr == item.value().remote_address() {
                let _ = self.connections.insert(*item.key(), item.value().clone());
            }
        }
    }

    /// Ensure the peer has a connection, and connect if not.
    ///
    /// This method is tailored to the use-case of connecting on send. In particular, it can be used
    /// to ensure that `connect` is only called once, even if there are many concurrent calls to
    /// `ensure_connection`.
    ///
    /// `is_valid` is used to determine whether to continue with the existing connection, or whether
    /// to call `connect` anyway. For example, setting `is_valid = |_| true` would always use any
    /// existing connection, and so `connect` would only be called once. This mechanism was chosen
    /// so that, e.g. the connection'd ID could be compared to see if a reconnection had already
    /// occurred, or force one otherwise (e.g. by setting
    /// `is_valid = |connection| connection.id() != last_connection_id`).
    pub(crate) async fn ensure_connection<Connect, Fut>(
        &self,
        is_valid: impl Fn(&qp2p::Connection) -> bool,
        connect: Connect,
    ) -> Result<qp2p::Connection, qp2p::ConnectionError>
    where
        Connect: FnOnce(SocketAddr) -> Fut,
        Fut: Future<Output = Result<qp2p::Connection, qp2p::ConnectionError>>,
    {
        for item in self.connections.iter() {
            // TODO: carry out the remove in case of connection is invalid?
            if is_valid(item.value()) {
                return Ok(item.value().clone());
            }
        }

        let new_connection = connect(self.addr).await?;
        let new_connection_id = new_connection.id();
        let _ = self
            .connections
            .insert(new_connection_id, new_connection.clone());
        Ok(new_connection)
    }
}

/// A peer whose name we don't yet know.
///
/// An `UnnamedPeer` represents a connected peer when we don't yet know the peer's name. For
/// example, when we receive a message we don't know the identity of the sender.
///
/// One rough edge to this is that `UnnamedPeer` is also used to represent messages from "ourself".
/// in this case there is no physical connection, and we technically would know our own identity.
/// It's possible that we're self-sending at the wrong level of the API (e.g. we currently serialise
/// and self-deliver a `WireMsg`, when we could instead directly generate the appropriate
/// `Cmd`). One benefit of this is it also works with tests, where we also often don't have an
/// actual connection.
#[derive(Clone, Debug)]
pub struct UnnamedPeer {
    addr: SocketAddr,
    connection: Option<qp2p::Connection>,
}

impl UnnamedPeer {
    pub(crate) fn addressed(addr: SocketAddr) -> Self {
        Self {
            addr,
            connection: None,
        }
    }

    pub(crate) fn connected(connection: qp2p::Connection) -> Self {
        Self {
            addr: connection.remote_address(),
            connection: Some(connection),
        }
    }

    pub(crate) fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub(crate) fn named(self, name: XorName) -> Peer {
        let connections = DashMap::new();
        if let Some(connection) = self.connection {
            let _ = connections.insert(connection.id(), connection);
        }
        Peer {
            name,
            addr: self.addr,
            connections: Arc::new(connections),
        }
    }
}
