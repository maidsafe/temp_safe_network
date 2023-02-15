// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use std::time::Duration;

use super::{CommEvent, MsgFromPeer};

use sn_interface::{
    messaging::{MsgKind, WireMsg},
    types::{log_markers::LogMarker, Peer},
};

use qp2p::{Connection, ConnectionIncoming, IncomingConnections};
use tokio::{sync::mpsc::Sender, task};

#[tracing::instrument(skip_all)]
pub(crate) fn listen_for_connections(
    comm_events_sender: Sender<CommEvent>,
    mut incoming_connections: IncomingConnections,
) {
    let _handle = task::spawn(async move {
        while let Some((connection, incoming_msgs)) = incoming_connections.next().await {
            trace!(
                "{}: from {:?} with connection_id {}",
                LogMarker::IncomingConnection,
                connection.remote_address(),
                connection.id()
            );

            let _handle = task::spawn(listen_for_msgs(
                comm_events_sender.clone(),
                connection,
                incoming_msgs,
            ));
        }
    });
}

#[tracing::instrument(skip_all)]
pub(crate) async fn listen_for_msgs(
    comm_events: Sender<CommEvent>,
    conn: Connection,
    mut incoming_msgs: ConnectionIncoming,
) {
    let conn_id = conn.id();
    let remote_address = conn.remote_address();

    loop {
        let msg = incoming_msgs.next_with_stream().await.transpose();

        // this is after the work, but the send should slow us down enough...
        // but this way we dont hold if over the `next` call, which may not occur until we gegt a msg in
        let capacity = comm_events.capacity();
        trace!("capacity in comms: {capacity:?}");
        let reserved_sender = match comm_events.reserve().await {
            Ok(p) => p,
            Err(error) => {
                error!("Could not reserve sender for CommEvent: {error:?}");
                break;
            }
        };
        if let Some(result) = msg {
            match result {
                Ok((msg_bytes, send_stream)) => {
                    let stream_info = if let Some(stream) = &send_stream {
                        format!(" on {}", stream.id())
                    } else {
                        "".to_string()
                    };
                    debug!(
                    "New msg arrived over conn_id={conn_id} from {remote_address:?}{stream_info}"
                    );

                    let wire_msg = match WireMsg::from(msg_bytes.0) {
                        Ok(wire_msg) => wire_msg,
                        Err(error) => {
                            // TODO: should perhaps rather drop this connection.. as it is a spam vector
                            debug!("Failed to deserialize message received from {remote_address:?}{stream_info}: {error:?}");
                            continue;
                        }
                    };

                    let src_name = match wire_msg.kind() {
                        MsgKind::Client { auth, .. } => auth.public_key.into(),
                        MsgKind::Node { name, .. } | MsgKind::ClientDataResponse(name) => *name,
                    };

                    let peer = Peer::new(src_name, remote_address);
                    let msg_id = wire_msg.msg_id();
                    debug!(
                    "Msg {msg_id:?} received, over conn_id={conn_id}, from: {peer:?}{stream_info} was: {wire_msg:?}"
                    );

                    let msg_event = CommEvent::Msg(MsgFromPeer {
                        sender: peer,
                        wire_msg,
                        send_stream,
                        is_response: false,
                    });

                    // handle the message first
                    reserved_sender.send(msg_event);
                }
                Err(error) => {
                    warn!("Error on connection {conn_id} with {remote_address}: {error:?}");
                }
            }
        } else {
            // we loop rest here
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    trace!(%conn_id, %remote_address, "{}", LogMarker::ConnectionClosed);
}

pub(crate) async fn response_msg_received(
    wire_msg: WireMsg,
    peer: Peer,
    send_stream: Option<qp2p::SendStream>,
    comm_events: Sender<CommEvent>,
) {
    let msg_id = wire_msg.msg_id();
    let msg_event = CommEvent::Msg(MsgFromPeer {
        sender: peer,
        wire_msg,
        send_stream,
        is_response: true,
    });

    // handle the message first
    if let Err(error) = comm_events.send(msg_event).await {
        error!("Error pushing msg {msg_id:?} onto internal msg handling channel: {error:?}");
    }
}
