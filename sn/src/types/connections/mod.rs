// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod connection;

pub(crate) use connection::{NetworkConnection, SendToOneError};

use qp2p::Endpoint;
use std::{collections::BTreeMap, fmt::Debug, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use xor_name::XorName;

type PeerId = (XorName, SocketAddr);

/// This is tailored to the use-case of connecting on send.
/// It keeps a connection instance per node, and is designed to make sure
/// underlying I/O resources are not leaked, overused or left dangling.
#[derive(Clone, Debug)]
pub(crate) struct Connections {
    data: Arc<RwLock<BTreeMap<PeerId, NetworkConnection>>>,
    endpoint: Endpoint,
}

impl Connections {
    pub(crate) fn new(endpoint: Endpoint) -> Self {
        Self {
            data: Arc::new(RwLock::new(BTreeMap::new())),
            endpoint,
        }
    }

    /// Any number of incoming qp2p:Connections can be added.
    /// We will eventually converge to the same one in our comms with the peer.
    pub(crate) async fn add_incoming(&self, id: &PeerId, conn: qp2p::Connection) {
        {
            let data = self.data.read().await;
            if let Some(c) = data.get(id) {
                // node id exists, add to it
                c.add(conn).await;
                return;
            }
            // else still not in list, go ahead and insert
        }

        let mut list = self.data.write().await;
        match list.get(id) {
            // someone else inserted in the meanwhile, add to it
            Some(c) => c.add(conn).await,
            // still not in list, go ahead and insert
            None => {
                let conn = NetworkConnection::new_with(*id, self.endpoint.clone(), conn).await;
                let _ = list.insert(*id, conn);
            }
        }
    }

    /// This method is tailored to the use-case of connecting on send.
    /// I.e. it will not connect here, but on calling send on the returned connection.
    pub(crate) async fn get_or_create(&self, id: &PeerId) -> NetworkConnection {
        if let Some(conn) = self.get(id).await {
            return conn;
        }

        // if id is not in list, the entire list needs to be locked
        // i.e. first conn to any node, will impact all sending at that instant..
        // however, first conn should be a minor part of total time spent using conns,
        // so that is ok
        let mut list = self.data.write().await;
        match list.get(id).cloned() {
            // someone else inserted in the meanwhile, so use that
            Some(conn) => conn,
            // still not in list, go ahead and create + insert
            None => {
                let conn = NetworkConnection::new(*id, self.endpoint.clone());
                let _ = list.insert(*id, conn.clone());
                conn
            }
        }
    }

    /// Drops the connection and all underlying resources.
    pub(crate) async fn disconnect(&self, id: PeerId) {
        let mut list = self.data.write().await;
        let conn = match list.remove(&id) {
            // someone else inserted in the meanwhile, so use that
            Some(conn) => conn,
            // still not in list, go ahead and create + insert
            None => return,
        };
        conn.disconnect().await;
    }

    async fn get(&self, id: &PeerId) -> Option<NetworkConnection> {
        let list = self.data.read().await;
        list.get(id).cloned()
    }
}
