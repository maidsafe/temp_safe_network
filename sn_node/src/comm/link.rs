// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::MsgListener;

use qp2p::{Connection, Endpoint, RetryConfig, UsrMsgBytes};
use sn_interface::messaging::MsgId;
use sn_interface::types::{log_markers::LogMarker, Peer};

/// A link to a peer in our network.
///
/// Using the link will open a connection.
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

    pub(crate) async fn new_with(peer: Peer, endpoint: Endpoint, listener: MsgListener) -> Self {
        Self::new(peer, endpoint, listener)
    }

    pub(crate) fn peer(&self) -> &Peer {
        &self.peer
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
    ) -> Result<(), SendToOneError> {
        trace!("Sending on conn: {:?}.", conn.id());

        match conn.send_with(bytes, priority, retry_config).await {
            Ok(()) => Ok(()),
            Err(error) => {
                error!("Error sending out from link... {:?}.", conn.id());

                // dont close just let the conn timeout incase msgs are coming in...
                // it's removed from out Peer tracking, so wont be used again for sending.
                Err(SendToOneError::Send(error))
            }
        }
    }

    /// Send a message using a bi-di stream and await response
    pub(crate) async fn send_on_new_bi_di_stream(
        &mut self,
        bytes: UsrMsgBytes,
        msg_id: MsgId,
    ) -> Result<UsrMsgBytes, SendToOneError> {
        trace!("Sending {msg_id:?} via a bi stream");

        let conn = match self.connect(msg_id).await {
            Ok(conn) => conn,
            Err(err) => {
                error!(
                    "{msg_id:?} Err getting connection during bi stream initialisation to: {:?}.",
                    self.peer()
                );
                return Err(err);
            }
        };

        trace!("connection got to: {:?} {msg_id:?}", self.peer);
        let (mut send_stream, mut recv_stream) =
            match conn.open_bi().await.map_err(SendToOneError::Connection) {
                Ok(streams) => streams,
                Err(stream_opening_err) => {
                    error!("{msg_id:?} Error opening streams {stream_opening_err:?}");
                    // remove that broken conn
                    return Err(stream_opening_err);
                }
            };

        let stream_id = send_stream.id();
        trace!(
            "bidi {stream_id} openeed for {msg_id:?} to: {:?}",
            self.peer
        );
        send_stream.set_priority(10);
        match send_stream.send_user_msg(bytes.clone()).await {
            Ok(_) => {}
            Err(err) => {
                error!(
                    "Error sending bytes {msg_id:?} over stream {stream_id}: {:?}",
                    err
                );
            }
        }

        trace!("{msg_id:?} sent on {stream_id} to: {:?}", self.peer);
        send_stream.finish().await.or_else(|err| match err {
            qp2p::SendError::StreamLost(qp2p::StreamError::Stopped(_)) => Ok(()),
            _ => {
                error!("{msg_id:?} Error finishing up stream {stream_id}: {err:?}");
                Err(SendToOneError::Send(err))
            }
        })?;

        trace!(
            "bidi {stream_id} finished for {msg_id:?} to: {:?}",
            self.peer
        );

        recv_stream
            .next()
            .await
            .map_err(SendToOneError::Recv)?
            .ok_or(SendToOneError::RecvClosed(self.peer))
    }

    // Create fresh connection Link.peer
    pub(crate) async fn connect(&self, msg_id: MsgId) -> Result<Connection, SendToOneError> {
        debug!("{msg_id:?} create conn attempt to {:?}", self.peer);
        let (conn, incoming_msgs) = self
            .endpoint
            .connect_to(&self.peer.addr())
            .await
            .map_err(SendToOneError::Connection)?;

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

/// Errors that can be returned from `Comm::send_to_one`.
#[derive(Debug)]
pub(crate) enum SendToOneError {
    ///
    Connection(qp2p::ConnectionError),
    ///
    Send(qp2p::SendError),
    ///
    Recv(qp2p::RecvError),
    /// Remote peer closed the bi-stream we expected a msg on
    RecvClosed(Peer),
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
