// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::MsgListener;

use crate::node::Result;
use qp2p::{Connection, Endpoint};
use sn_interface::messaging::MsgId;
use sn_interface::types::{log_markers::LogMarker, Peer};

/// A link to a peer in our network.
/// TODO: deprecate and just use send on qp2p as needed
///
#[derive(Clone)]
pub(crate) struct Link {
    peer: Peer,
    endpoint: Endpoint,
    listener: MsgListener,
}

impl Link {
    pub(crate) fn new(peer: Peer, endpoint: Endpoint, listener: MsgListener) -> Self {
        Self {
            peer,
            endpoint,
            listener,
        }
    }

    pub(crate) fn peer(&self) -> &Peer {
        &self.peer
    }

    pub(crate) async fn connect(&self, msg_id: MsgId) -> Result<Connection> {
        debug!("{msg_id:?} create conn attempt to {:?}", self.peer);
        let (conn, incoming_msgs) = self.endpoint.connect_to(&self.peer.addr()).await?;

        trace!(
            "{msg_id:?}: {} to {} (id: {})",
            LogMarker::ConnectionOpened,
            conn.remote_address(),
            conn.id()
        );

        self.listener.listen(conn.clone(), incoming_msgs);

        Ok(conn)
    }
}
