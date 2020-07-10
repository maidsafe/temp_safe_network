// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{auth::ClientInfo, ClientMsg};
use crate::{cmd::GatewayCmd, utils};
use bytes::Bytes;
use log::{debug, error, info, trace, warn};
use rand::{CryptoRng, Rng};
use routing::Node;
use safe_nd::{
    DebitAgreementProof, HandshakeRequest, HandshakeResponse, Message, MessageId, NodePublicId,
    PublicId, MsgEnvelope, Signature, XorName, Result,
};
use serde::Serialize;
use std::{
    cell::RefCell,
    collections::{hash_map::Entry, HashMap},
    fmt::{self, Display, Formatter},
    net::SocketAddr,
    rc::Rc,
};

pub(super) struct Messaging {
    id: NodePublicId,
    routing_node: Rc<RefCell<Node>>,
    clients: HashMap<SocketAddr, ClientInfo>,
    pending_msg_ids: HashMap<MessageId, SocketAddr>,
    pending_actions: HashMap<MessageId, MsgEnvelope>,
    // Map of new client connections to the challenge value we sent them.
    client_candidates: HashMap<SocketAddr, (Vec<u8>, PublicId)>,
}

impl Messaging {
    pub fn new(id: NodePublicId, routing_node: Rc<RefCell<Node>>) -> Self {
        Self {
            id,
            routing_node,
            clients: Default::default(),
            pending_msg_ids: Default::default(),
            pending_actions: Default::default(),
            client_candidates: Default::default(),
        }
    }

    pub fn try_parse_client_msg<R: CryptoRng + Rng>(
        &mut self,
        peer_addr: SocketAddr,
        bytes: &Bytes,
        rng: &mut R,
    ) -> Option<ClientMsg> {
        if let Some(client) = self.clients.get(&peer_addr).cloned() {
            self.try_get_client_msg()
        } else {
            self.try_handle_handshake(&bytes, peer_addr);
            None
        }
    }

    fn try_get_client_msg(&mut self, bytes: &Bytes, peer_addr: SocketAddr) -> Option<ClientMsg> {
        let msg = self.try_deserialize_msg(bytes)?;
        if self.shall_handle_request(msg.message.id(), peer_addr) {
            trace!(
                "{}: Received ({:?} {:?}) from {}",
                self,
                "msg.get_type()",
                msg.message.id(),
                client
            );
            return Some(ClientMsg { client, msg });
        }
        None
    }

    fn try_deserialize_msg(&mut self, bytes: &Bytes) -> Option<MsgEnvelope> {
        match bincode::deserialize(&bytes) {
            Ok(msg @ MsgEnvelope { message: Message::Cmd { .. }, .. }) => Some(msg),
            Ok(msg @ MsgEnvelope { message: Message::Query { .. }, .. }) => Some(msg),
            Ok(msg @ MsgEnvelope { message: Message::Event { .. }, .. })
            Ok(msg @ MsgEnvelope { message: Message::CmdError { .. }, .. }) 
            Ok(msg @ MsgEnvelope { message: Message::QueryResponse { .. }, .. }) => {
                info!(
                    "{}: {} invalidly sent {:?}",
                    self, client.public_id, msg
                );
                None
            }
            Err(err) => {
                info!(
                    "{}: Unable to deserialise message from {}: {}",
                    self, client.public_id, err
                );
                None
            }
        }
    }

    fn try_handle_handshake<R: CryptoRng + Rng>(&mut self, bytes: &Bytes, peer_addr: SocketAddr, rng: &mut R) {
        match bincode::deserialize(&bytes) {
            Ok(HandshakeRequest::Bootstrap(client_id)) => {
                self.try_bootstrap(peer_addr, &client_id);
            }
            Ok(HandshakeRequest::Join(client_id)) => {
                self.try_join(peer_addr, client_id, rng);
            }
            Ok(HandshakeRequest::ChallengeResult(signature)) => {
                self.handle_challenge(peer_addr, &signature);
            }
            Err(err) => {
                info!(
                    "{}: Unable to deserialise handshake request from {}: {}",
                    self, peer_addr, err
                );
            }
        }
    }

    pub fn shall_handle_request(&mut self, message_id: MessageId, peer_addr: SocketAddr) -> bool {
        // We could receive a consensused vault msg contains a client request,
        // before receiving the request from that client directly.
        if let Some(msg) = self.pending_actions.remove(&message_id) {
            self.send(
                peer_addr,
                &msg,
            );
            return false;
        }

        if let Entry::Vacant(ve) = self.pending_msg_ids.entry(message_id) {
            let _ = ve.insert(peer_addr);
            true
        } else {
            info!(
                "Pending MessageId {:?} reused - ignoring client message.",
                message_id
            );
            false
        }
    }

    pub fn handle_new_connection(&mut self, peer_addr: SocketAddr) {
        // If we already know the peer, drop the connection attempt.
        if self.clients.contains_key(&peer_addr) || self.client_candidates.contains_key(&peer_addr)
        {
            return;
        }

        info!("{}: Connected to new client on {}", self, peer_addr);
    }

    pub fn handle_connection_failure(&mut self, peer_addr: SocketAddr) {
        if let Some(client) = self.clients.remove(&peer_addr) {
            info!(
                "{}: Disconnected from {:?} on {}",
                self, client.public_id, peer_addr
            );
        } else {
            let _ = self.client_candidates.remove(&peer_addr);
            info!(
                "{}: Disconnected from client candidate on {}",
                self, peer_addr
            );
        }
    }

    // #[allow(unused)]
    // pub fn notify_client(&mut self, client: &XorName, receipt: &DebitAgreementProof) {
    //     for client_id in self.lookup_client_and_its_apps(client) {
    //         self.send_notification_to_client(&client_id, &TransferNotification(receipt.clone()));
    //     }
    // }

    pub fn send_to_client(&mut self, msg: MsgEnvelope) -> Result<()> {
        let msg_id = msg.message.id();
        match msg.destination() {
            Address::Client { .. } => (),
            _ => {
                error!("{} for message-id {:?}, Invalid destination.", self, msg_id);
                return Err(Error::InvalidOperation),
            }
        };
        let peer_addr = match self.pending_msg_ids.remove(&msg_id) {
            Some(peer_addr) => peer_addr,
            None => {
                info!(
                    "{} for message-id {:?}, Unable to find the client to respond to.",
                    self, msg_id
                );
                let _ = self.pending_actions.insert(msg_id, msg);
                return Err(Error::KeyNotFound),
            }
        };

        Ok(self.send(peer_addr, &msg))
    }

    /// Handles a received challenge response.
    ///
    /// Checks that the response contains a valid signature of the challenge we previously sent.
    /// If a client requests the section info, we also send it.
    fn handle_challenge(&mut self, peer_addr: SocketAddr, signature: &Signature) {
        if let Some((challenge, public_id)) = self.client_candidates.remove(&peer_addr) {
            let public_key = match utils::own_key(&public_id) {
                Some(pk) => pk,
                None => {
                    info!(
                        "{}: Client on {} identifies as a node: {}, hence disconnect from it.",
                        self, peer_addr, public_id
                    );
                    if let Err(err) = self
                        .routing_node
                        .borrow_mut()
                        .disconnect_from_client(peer_addr)
                    {
                        warn!("{}: Could not disconnect client: {:?}", self, err);
                    }
                    return;
                }
            };
            match public_key.verify(&signature, challenge) {
                Ok(()) => {
                    info!("{}: Accepted {} on {}.", self, public_id, peer_addr,);
                    let _ = self.clients.insert(peer_addr, ClientInfo { public_id });
                }
                Err(err) => {
                    info!(
                        "{}: Challenge failed for {} on {}: {}",
                        self, public_id, peer_addr, err
                    );
                    if let Err(err) = self
                        .routing_node
                        .borrow_mut()
                        .disconnect_from_client(peer_addr)
                    {
                        warn!("{}: Could not disconnect client: {:?}", self, err);
                    }
                }
            }
        } else {
            info!(
                "{}: {} supplied challenge response without us providing it.",
                self, peer_addr
            );
            if let Err(err) = self
                .routing_node
                .borrow_mut()
                .disconnect_from_client(peer_addr)
            {
                warn!("{}: Could not disconnect client: {:?}", self, err);
            }
        }
    }

    /// Handles a received join request from a client.
    fn try_join<R: CryptoRng + Rng>(
        &mut self,
        peer_addr: SocketAddr,
        client_id: PublicId,
        rng: &mut R,
    ) {
        if !self
            .routing_node
            .borrow()
            .matches_our_prefix(&routing::XorName(client_id.name().0))
            .unwrap_or(false)
        {
            debug!(
                "Client {} ({}) wants to join us but we are not its client handler",
                client_id, peer_addr
            );
            let _ = self
                .routing_node
                .borrow_mut()
                .disconnect_from_client(peer_addr);
        }
        let challenge = utils::random_vec(rng, 8);
        self.send(
            peer_addr,
            &HandshakeResponse::Challenge(PublicId::Node(self.id.clone()), challenge.clone()),
        );
        let _ = self
            .client_candidates
            .insert(peer_addr, (challenge, client_id));
    }

    fn send<T: Serialize>(&mut self, recipient: SocketAddr, msg: &T) {
        let msg = utils::serialise(msg);
        let msg = Bytes::from(msg);

        if let Err(e) = self
            .routing_node
            .borrow_mut()
            .send_message_to_client(recipient, msg, 0)
        {
            warn!(
                "{}: Could not send message to client {}: {:?}",
                self, recipient, e
            );
        }
    }

    // pub(crate) fn send_notification_to_client(
    //     &mut self,
    //     client_id: &PublicId,
    //     notification: &TransferNotification,
    // ) {
    //     let peer_addrs = self.lookup_client_peer_addrs(&client_id);

    //     if peer_addrs.is_empty() {
    //         warn!(
    //             "{}: can't notify {} as none of the instances of the client is connected.",
    //             self, client_id
    //         );
    //         return;
    //     };

    //     for peer_addr in peer_addrs {
    //         self.send(
    //             peer_addr,
    //             &Message::TransferNotification {
    //                 payload: notification.clone(),
    //             },
    //         )
    //     }
    // }

    fn lookup_client_peer_addrs(&self, id: &PublicId) -> Vec<SocketAddr> {
        self.clients
            .iter()
            .filter_map(|(peer_addr, client)| {
                if &client.public_id == id {
                    Some(*peer_addr)
                } else {
                    None
                }
            })
            .collect()
    }

    pub(crate) fn lookup_client_and_its_apps(&self, name: &XorName) -> Vec<PublicId> {
        self.clients
            .values()
            .filter_map(|client| {
                if client.public_id.name() == name {
                    Some(client.public_id.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    }

    fn try_bootstrap(&mut self, peer_addr: SocketAddr, client_id: &PublicId) {
        if !self
            .routing_node
            .borrow()
            .matches_our_prefix(&routing::XorName(client_id.name().0))
            .unwrap_or(false)
        {
            let closest_known_elders = self
                .routing_node
                .borrow()
                .our_elders_sorted_by_distance_to(&routing::XorName(client_id.name().0))
                .into_iter()
                .map(|p2p_node| {
                    let peer_addr = *p2p_node.peer_addr();
                    (XorName(p2p_node.name().0), peer_addr)
                })
                .collect::<Vec<_>>();

            if closest_known_elders.is_empty() {
                warn!(
                    "{}: No closest known elders in any section we know of",
                    self
                );
            } else {
                self.send(peer_addr, &HandshakeResponse::Join(closest_known_elders));
            }
        } else {
            let elders = self
                .routing_node
                .borrow_mut()
                .our_elders()
                .map(|p2p_node| {
                    let peer_addr = *p2p_node.peer_addr();
                    (XorName(p2p_node.name().0), peer_addr)
                })
                .collect::<Vec<_>>();

            self.send(peer_addr, &HandshakeResponse::Join(elders));
        }
    }
}

impl Display for Messaging {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.id.name())
    }
}
