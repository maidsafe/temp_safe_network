// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! Comms for the SAFE Network.
//! All comms with nodes are done though this.

// For quick_error
#![recursion_limit = "256"]
#![doc(
    html_logo_url = "https://github.com/maidsafe/QA/raw/master/Images/maidsafe_logo.png",
    html_favicon_url = "https://maidsafe.net/img/favicon.ico",
    test(attr(deny(warnings)))
)]
// Forbid some very bad patterns. Forbid is stronger than `deny`, preventing us from suppressing the
// lint with `#[allow(...)]` et-all.
#![forbid(
    arithmetic_overflow,
    mutable_transmutes,
    no_mangle_const_items,
    unknown_crate_types,
    unsafe_code
)]
// Turn on some additional warnings to encourage good style.
#![warn(
    missing_debug_implementations,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results,
    clippy::unicode_not_nfc,
    clippy::unwrap_used
)]

#[macro_use]
extern crate tracing;

mod error;
mod listener;
mod peer_session;

pub use self::error::{Error, Result};

use self::peer_session::PeerSession;

use sn_interface::{
    messaging::{
        data::ClientDataResponse as ClientResponse,
        system::{NodeDataResponse as NodeResponse, NodeMsgType},
        Dst, MsgId, MsgKind, MsgType, WireMsg,
    },
    types::Peer,
};

use qp2p::{Endpoint, SendStream, UsrMsgBytes};

use std::{
    collections::{BTreeMap, BTreeSet},
    net::SocketAddr,
};
use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    task,
};

/// Standard channel size, to allow for large swings in throughput
static STANDARD_CHANNEL_SIZE: usize = 100_000;

/// Events from the comm module.
#[derive(Debug)]
pub enum CommEvent {
    /// A msg was received.
    Msg(MsgFromPeer),
    /// A send error occurred.
    Error {
        /// The sender/recipient that failed.
        peer: Peer,
        /// The failure type.
        error: Error,
    },
}

/// Internal comm cmds.
#[derive(custom_debug::Debug)]
enum CommCmd {
    Send {
        msg_id: MsgId,
        peer: Peer,
        #[debug(skip)]
        bytes: UsrMsgBytes,
    },
    SetTargets(BTreeSet<Peer>),
    SendWithBiResponse {
        peer: Peer,
        msg_id: MsgId,
        bytes: UsrMsgBytes,
    },
    SendAndRespondOnStream {
        msg_id: MsgId,
        msg_type: NodeMsgType,
        #[debug(skip)]
        node_bytes: BTreeMap<Peer, UsrMsgBytes>,
        dst_stream: (Dst, SendStream),
    },
}

/// A msg received on the wire.
#[derive(Debug)]
pub struct MsgFromPeer {
    /// The peer that sent us the msg.
    pub sender: Peer,
    /// The msg that we received.
    pub wire_msg: WireMsg,
    /// An optional stream to return msgs on, if
    /// this msg came on a bidi-stream.
    pub send_stream: Option<SendStream>,
}

/// Communication component of the node to interact with other nodes.
///
/// Any failed sends are tracked via `CommEvent::Error`, which will track issues for any peers
/// in the section (otherwise ignoring failed send to out of section nodes or clients).
#[derive(Clone, Debug)]
pub struct Comm {
    our_endpoint: Endpoint,
    cmd_sender: Sender<CommCmd>,
}

impl Comm {
    /// Creates a new instance of Comm with an endpoint
    /// and starts listening to the incoming messages from other nodes.
    pub async fn new(local_addr: SocketAddr) -> Result<(Self, Receiver<CommEvent>)> {
        // creates an endpoint to listen to the incoming messages from other nodes
        let (our_endpoint, incoming_connections) = Endpoint::builder()
            .addr(local_addr)
            .idle_timeout(70_000)
            .server()?;

        trace!("Creating comms..");
        // comm_events_receiver will be used by upper layer to receive all msgs comming in from the network
        let (comm_events_sender, comm_events_receiver) = mpsc::channel(STANDARD_CHANNEL_SIZE);
        let (cmd_sender, cmd_receiver) = mpsc::channel(STANDARD_CHANNEL_SIZE);

        // listen for msgs/connections to our endpoint
        listener::listen_for_connections(comm_events_sender.clone(), incoming_connections);

        process_cmds(our_endpoint.clone(), cmd_receiver, comm_events_sender);

        Ok((
            Self {
                our_endpoint,
                cmd_sender,
            },
            comm_events_receiver,
        ))
    }

    /// The socket address of our endpoint.
    pub fn socket_addr(&self) -> SocketAddr {
        self.our_endpoint.local_addr()
    }

    /// Closes the endpoint.
    pub fn close_endpoint(&self) {
        self.our_endpoint.close()
    }

    /// Sets the available targets to be only those in the passed in set.
    pub fn set_comm_targets(&self, targets: BTreeSet<Peer>) {
        // We only remove sessions by calling this function,
        // No removals are made even if we failed to send using all peer session's connections,
        // as it's our source of truth for known and connectable peers.
        let sender = self.cmd_sender.clone();
        let _handle = task::spawn(async move { sender.send(CommCmd::SetTargets(targets)).await });
    }

    /// Sends the payload on a new or existing connection.
    #[tracing::instrument(skip(self, bytes))]
    pub fn send_out_bytes(&self, peer: Peer, msg_id: MsgId, bytes: UsrMsgBytes) {
        let sender = self.cmd_sender.clone();
        let _handle = task::spawn(async move {
            sender
                .send(CommCmd::Send {
                    msg_id,
                    peer,
                    bytes,
                })
                .await
        });
    }

    /// Sends the payload on a new bidi-stream and returns the response.
    #[tracing::instrument(skip(self, bytes))]
    pub fn send_with_bi_response(&self, peer: Peer, msg_id: MsgId, bytes: UsrMsgBytes) {
        let sender = self.cmd_sender.clone();
        let _handle = task::spawn(async move {
            sender
                .send(CommCmd::SendWithBiResponse {
                    msg_id,
                    peer,
                    bytes,
                })
                .await
        });
    }

    /// Sends the payload on a new bidi-stream and returns the response.
    #[tracing::instrument(skip(node_bytes))]
    pub fn send_and_respond_on_stream(
        &self,
        msg_id: MsgId,
        msg_type: NodeMsgType,
        node_bytes: BTreeMap<Peer, UsrMsgBytes>,
        dst_stream: (Dst, SendStream),
    ) {
        let sender = self.cmd_sender.clone();
        let _handle = task::spawn(async move {
            sender
                .send(CommCmd::SendAndRespondOnStream {
                    msg_id,
                    msg_type,
                    node_bytes,
                    dst_stream,
                })
                .await
        });
    }

    // /// Sends the payload on a new bidi-stream and returns the response.
    // #[tracing::instrument(skip(bytes))]
    // pub async fn send_out_bytes_to_peer_and_return_response(
    //     &self,
    //     peer: Peer,
    //     msg_id: MsgId,
    //     bytes: UsrMsgBytes,
    // ) -> Result<WireMsg> {
    //     let sender = self.cmd_sender.clone();
    //     let _ = task::spawn(async move {
    //         sender
    //             .send(CommCmd::SendAndRespondOnStream {
    //                 msg_id,
    //                 node_bytes: BTreeMap::from([(peer, bytes)]),
    //                 client_stream,
    //                 dst,
    //             })
    //             .await
    //     });

    //     unimplemented!();
    // }
}

fn process_cmds(
    our_endpoint: Endpoint,
    mut update_receiver: Receiver<CommCmd>,
    comm_events: Sender<CommEvent>,
) {
    let _handle = task::spawn(async move {
        let mut sessions = BTreeMap::<Peer, PeerSession>::new();
        // let sessions = Arc::new(DashMap::<Peer, PeerSession>::new());
        while let Some(cmd) = update_receiver.recv().await {
            trace!("Comms cmd handling: {cmd:?}");
            match cmd {
                // This is the only place that mutates `sessions`.
                CommCmd::SetTargets(targets) => {
                    // Drops sessions that not among the targets.
                    sessions.retain(|p, _| targets.contains(p));
                    // Adds new sessions for each new target.
                    targets.iter().for_each(|peer| {
                        if sessions.get(peer).is_none() {
                            let session = PeerSession::new(*peer, our_endpoint.clone());
                            let _ = sessions.insert(*peer, session);
                        }
                    });
                }
                CommCmd::Send {
                    msg_id,
                    peer,
                    bytes,
                } => {
                    let session = match sessions.get(&peer) {
                        Some(session) => session.clone(),
                        None => {
                            error!(
                                "Sending message (msg_id: {msg_id:?}) to {peer:?} failed: unknown node."
                            );
                            send_error(
                                peer,
                                Error::ConnectingToUnknownNode(msg_id),
                                comm_events.clone(),
                            );
                            continue;
                        }
                    };
                    send(msg_id, session, bytes, comm_events.clone());
                }
                CommCmd::SendWithBiResponse {
                    peer,
                    msg_id,
                    bytes,
                } => {
                    // TODO: use `NODE_RESPONSE_TIMEOUT`
                    let session = match sessions.get(&peer) {
                        Some(session) => session.clone(),
                        None => {
                            error!(
                                "Sending message (msg_id: {msg_id:?}) to {peer:?} failed: unknown node."
                            );
                            send_error(
                                peer,
                                Error::ConnectingToUnknownNode(msg_id),
                                comm_events.clone(),
                            );
                            continue;
                        }
                    };
                    send_with_bi_response(msg_id, session, bytes, comm_events.clone());
                }
                CommCmd::SendAndRespondOnStream {
                    msg_id,
                    msg_type,
                    node_bytes,
                    dst_stream,
                } => {
                    let node_bytes = node_bytes
                        .into_iter()
                        .filter_map(|(peer, bytes)| {
                            debug!("Trying to get {peer:?} session in order to send: {msg_id:?}", );
                            match sessions.get(&peer) {
                                Some(session) => Some((session.clone(), bytes)),
                                None => {
                                    error!(
                                        "Sending message (msg_id: {msg_id:?}) to {peer:?} failed: unknown node."
                                    );
                                    send_error(peer, Error::ConnectingToUnknownNode(msg_id), comm_events.clone());
                                    None
                                }
                            }
                        }).collect();
                    send_and_respond_on_stream(
                        msg_id,
                        msg_type,
                        node_bytes,
                        dst_stream,
                        comm_events.clone(),
                    );
                }
            }
        }
    });
}

fn send(msg_id: MsgId, session: PeerSession, bytes: UsrMsgBytes, comm_events: Sender<CommEvent>) {
    let _handle = task::spawn(async move {
        let (h, d, p) = &bytes;
        let bytes_len = h.len() + d.len() + p.len();
        let peer = session.peer();
        trace!("Sending message bytes ({bytes_len} bytes) w/ {msg_id:?} to {peer:?}");
        match session.send(msg_id, bytes).await {
            Ok(()) => {
                trace!("Msg {msg_id:?} sent to {peer:?}");
            }
            Err(error) => {
                error!("Sending message (msg_id: {msg_id:?}) to {peer:?} failed: {error}");
                send_error(peer, Error::FailedSend(msg_id), comm_events.clone());
            }
        }
    });
}

fn send_with_bi_response(
    msg_id: MsgId,
    session: PeerSession,
    bytes: UsrMsgBytes,
    comm_events: Sender<CommEvent>,
) {
    let _handle = task::spawn(async move {
        let (h, d, p) = &bytes;
        let bytes_len = h.len() + d.len() + p.len();
        let peer = session.peer();
        trace!("Sending message bytes ({bytes_len} bytes) w/ {msg_id:?} to {peer:?}");
        let node_response_bytes = match session.send_with_bi_return_response(bytes, msg_id).await {
            Ok(response_bytes) => {
                debug!("Peer response from {peer:?} is in for {msg_id:?}");
                response_bytes
            }
            Err(error) => {
                error!("Sending message (msg_id: {msg_id:?}) to {peer:?} failed: {error}");
                send_error(peer, Error::FailedSend(msg_id), comm_events.clone());
                return;
            }
        };
        match WireMsg::from(node_response_bytes) {
            Ok(wire_msg) => {
                listener::msg_received(wire_msg, peer, None, comm_events.clone());
            }
            Err(error) => {
                error!("Failed sending {msg_id:?} to {peer:?}: {error:?}");
                send_error(peer, Error::InvalidMsgReceived(msg_id), comm_events.clone());
            }
        };
    });
}

fn send_and_respond_on_stream(
    msg_id: MsgId,
    msg_type: NodeMsgType,
    node_bytes: Vec<(PeerSession, UsrMsgBytes)>,
    dst_stream: (Dst, SendStream),
    comm_events: Sender<CommEvent>,
) {
    let _handle = task::spawn(async move {
        // responses from those we send to
        let mut all_received = vec![];

        for (session, bytes) in node_bytes {
            let peer = session.peer();
            let node_response_bytes =
                match session.send_with_bi_return_response(bytes, msg_id).await {
                    Ok(response_bytes) => response_bytes,
                    Err(error) => {
                        error!("Failed sending {msg_id:?} to {peer:?}: {error:?}");
                        send_error(peer, Error::FailedSend(msg_id), comm_events.clone());
                        // continue with next node
                        continue;
                    }
                };

            debug!("Response from node {peer:?} is in for {msg_id:?}");

            match WireMsg::from(node_response_bytes) {
                Ok(received) => {
                    match received.into_msg() {
                        Ok(msg) => all_received.push((msg, peer)),
                        Err(_error) => {
                            send_error(
                                peer,
                                Error::InvalidMsgReceived(msg_id),
                                comm_events.clone(),
                            );
                            // continue with next received msg
                        }
                    };
                }
                Err(error) => {
                    error!("Failed sending {msg_id:?} to {peer:?}: {error:?}");
                    send_error(peer, Error::InvalidMsgReceived(msg_id), comm_events.clone());
                    // continue with next received msg
                }
            };
        }

        let (dst, mut stream) = dst_stream;
        for (received, peer) in all_received {
            match map_to_client_response(msg_type, msg_id, received, dst) {
                Some(bytes) => match stream.send_user_msg(bytes).await {
                    Ok(()) => (),
                    Err(error) => {
                        send_error(peer, Error::from(error), comm_events.clone());
                        // continue with next received msg
                    }
                },
                None => {
                    send_error(peer, Error::InvalidMsgReceived(msg_id), comm_events.clone());
                    // continue with next received msg
                }
            }
        }
    });
}

/// Verify what kind of response was received, and if that's the expected type based on
/// the type of msg sent to the nodes, then return the corresponding response to the client.
fn map_to_client_response(
    sent: NodeMsgType,
    correlation_id: MsgId,
    received: MsgType,
    dst: Dst,
) -> Option<UsrMsgBytes> {
    let response = match sent {
        NodeMsgType::DataQuery => {
            match received {
                MsgType::NodeDataResponse {
                    msg: NodeResponse::QueryResponse { response, .. },
                    ..
                } => {
                    // We sent a data query and we received a query response,
                    // so let's forward it to the client
                    debug!("{correlation_id:?} sending query response back to client");
                    ClientResponse::QueryResponse {
                        response,
                        correlation_id,
                    }
                }
                other_resp => {
                    error!("Unexpected response to query from node for {correlation_id:?}: {other_resp:?}");
                    return None;
                }
            }
        }
        NodeMsgType::StoreData => {
            match received {
                MsgType::NodeDataResponse {
                    msg: NodeResponse::CmdResponse { response, .. },
                    ..
                } => {
                    // We sent a data cmd to store client data and we received a
                    // cmd response, so let's forward it to the client
                    debug!("{correlation_id:?} sending cmd response ACK back to client");
                    ClientResponse::CmdResponse {
                        response,
                        correlation_id,
                    }
                }
                other_resp => {
                    error!("Unexpected response to cmd from node for {correlation_id:?}: {other_resp:?}");
                    return None;
                }
            }
        }
    };

    let kind = MsgKind::ClientDataResponse(dst.name);
    let payload = WireMsg::serialize_msg_payload(&response).ok()?;
    let wire_msg = WireMsg::new_msg(correlation_id, payload, kind, dst);

    wire_msg.serialize().ok()
}

fn send_error(peer: Peer, error: Error, comm_events: Sender<CommEvent>) {
    let _handle = task::spawn(async move {
        let error_msg =
            format!("Failed to send error {error} of peer {peer} on comm event channel ");
        match comm_events.send(CommEvent::Error { peer, error }).await {
            Ok(()) => (),
            Err(err) => error!("{error_msg} due to {err}."),
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    use sn_interface::{
        messaging::{
            data::{ClientMsg, DataQuery, DataQueryVariant},
            ClientAuth, Dst, MsgId, MsgKind,
        },
        types::{ChunkAddress, Keypair, Peer},
    };

    use assert_matches::assert_matches;
    use eyre::Result;
    use futures::future;
    use std::{net::Ipv4Addr, time::Duration};
    use tokio::{
        net::UdpSocket,
        sync::mpsc::{self, Receiver},
        time,
    };

    const TIMEOUT: Duration = Duration::from_secs(1);

    #[tokio::test]
    async fn successful_send() -> Result<()> {
        let (comm, _rx) = Comm::new(local_addr()).await?;

        let (peer0, mut rx0) = new_peer().await?;
        let (peer1, mut rx1) = new_peer().await?;

        // add peers as known members
        comm.set_comm_targets([peer0, peer1].into());

        let peer0_msg = new_test_msg(dst(peer0))?;
        let peer1_msg = new_test_msg(dst(peer1))?;

        comm.send_out_bytes(peer0, peer0_msg.msg_id(), peer0_msg.serialize()?);
        comm.send_out_bytes(peer1, peer1_msg.msg_id(), peer1_msg.serialize()?);

        if let Some(bytes) = rx0.recv().await {
            assert_eq!(WireMsg::from(bytes)?, peer0_msg);
        }

        if let Some(bytes) = rx1.recv().await {
            assert_eq!(WireMsg::from(bytes)?, peer1_msg);
        }

        Ok(())
    }

    #[tokio::test]
    async fn failed_send() -> Result<()> {
        let (comm, mut rx) = Comm::new(local_addr()).await?;

        let invalid_peer = get_invalid_peer().await?;
        let invalid_addr = invalid_peer.addr();
        let msg = new_test_msg(dst(invalid_peer))?;
        comm.send_out_bytes(invalid_peer, msg.msg_id(), msg.serialize()?);

        if let Some(CommEvent::Error { peer, error }) = rx.recv().await {
            // the peer is still not set as a known member thus it should have failed
            assert_matches!(error, Error::ConnectingToUnknownNode(_));
            assert_eq!(peer.addr(), invalid_addr);
        }

        // let's add the peer as a known member and check again
        comm.set_comm_targets([invalid_peer].into());

        comm.send_out_bytes(invalid_peer, msg.msg_id(), msg.serialize()?);

        if let Some(CommEvent::Error { peer, error }) = rx.recv().await {
            assert_matches!(error, Error::FailedSend(_));
            assert_eq!(peer.addr(), invalid_addr);
        }

        Ok(())
    }

    #[tokio::test]
    async fn send_after_reconnect() -> Result<()> {
        let (send_comm, _rx) = Comm::new(local_addr()).await?;

        let (recv_endpoint, mut incoming_connections) = Endpoint::builder()
            .addr(local_addr())
            .idle_timeout(70_000)
            .server()?;
        let recv_addr = recv_endpoint.local_addr();
        let name = xor_name::rand::random();
        let peer = Peer::new(name, recv_addr);
        let msg0 = new_test_msg(dst(peer))?;

        // add peer as a known member
        send_comm.set_comm_targets([peer].into());

        send_comm.send_out_bytes(peer, msg0.msg_id(), msg0.serialize()?);

        let mut msg0_received = false;

        // Receive one message and disconnect from the peer
        {
            if let Some((_, mut incoming_msgs)) = incoming_connections.next().await {
                if let Some(msg) = time::timeout(TIMEOUT, incoming_msgs.next()).await?? {
                    assert_eq!(WireMsg::from(msg.0)?, msg0);
                    msg0_received = true;
                }
                // connection dropped here
            }
            assert!(msg0_received);
        }

        let msg1 = new_test_msg(dst(peer))?;
        send_comm.send_out_bytes(peer, msg1.msg_id(), msg1.serialize()?);

        let mut msg1_received = false;

        if let Some((_, mut incoming_msgs)) = incoming_connections.next().await {
            if let Some(msg) = time::timeout(TIMEOUT, incoming_msgs.next()).await?? {
                assert_eq!(WireMsg::from(msg.0)?, msg1);
                msg1_received = true;
            }
        }

        assert!(msg1_received);

        Ok(())
    }

    #[tokio::test]
    async fn incoming_connection_lost() -> Result<()> {
        let (comm0, mut rx0) = Comm::new(local_addr()).await?;
        let addr0 = comm0.socket_addr();

        let (comm1, _rx1) = Comm::new(local_addr()).await?;

        let peer = Peer::new(xor_name::rand::random(), addr0);
        let msg = new_test_msg(dst(peer))?;

        // add peer as a known member
        comm1.set_comm_targets([peer].into());

        // Send a message to establish the connection
        comm1.send_out_bytes(peer, msg.msg_id(), msg.serialize()?);

        assert_matches!(rx0.recv().await, Some(CommEvent::Msg(MsgFromPeer { .. })));

        // Drop `comm1` to cause connection lost.
        drop(comm1);

        assert_matches!(time::timeout(TIMEOUT, rx0.recv()).await, Err(_));

        Ok(())
    }

    fn dst(peer: Peer) -> Dst {
        Dst {
            name: peer.name(),
            section_key: bls::SecretKey::random().public_key(),
        }
    }

    fn new_test_msg(dst: Dst) -> Result<WireMsg> {
        let src_keypair = Keypair::new_ed25519();

        let query = DataQueryVariant::GetChunk(ChunkAddress(xor_name::rand::random()));
        let query = DataQuery {
            node_index: 0,
            variant: query,
        };
        let query = ClientMsg::Query(query);
        let payload = WireMsg::serialize_msg_payload(&query)?;

        let auth = ClientAuth {
            public_key: src_keypair.public_key(),
            signature: src_keypair.sign(&payload),
        };

        Ok(WireMsg::new_msg(
            MsgId::new(),
            payload,
            MsgKind::Client(auth),
            dst,
        ))
    }

    async fn new_peer() -> Result<(Peer, Receiver<UsrMsgBytes>)> {
        let (endpoint, mut incoming_connections) = Endpoint::builder()
            .addr(local_addr())
            .idle_timeout(70_000)
            .server()?;
        let addr = endpoint.local_addr();

        let (tx, rx) = mpsc::channel(1);

        let _handle = tokio::task::spawn(async move {
            while let Some((_, mut incoming_messages)) = incoming_connections.next().await {
                while let Ok(Some(msg)) = incoming_messages.next().await {
                    let _ = tx.send(msg.0).await;
                }
            }
        });

        Ok((Peer::new(xor_name::rand::random(), addr), rx))
    }

    async fn get_invalid_peer() -> Result<Peer> {
        let socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).await?;
        let addr = socket.local_addr()?;

        // Keep the socket alive to keep the address bound, but don't read/write to it so any
        // attempt to connect to it will fail.
        let _handle = tokio::task::spawn(async move {
            debug!("get invalid peer");
            future::pending::<()>().await;
            let _ = socket;
        });

        Ok(Peer::new(xor_name::rand::random(), addr))
    }

    fn local_addr() -> SocketAddr {
        (Ipv4Addr::LOCALHOST, 0).into()
    }
}
