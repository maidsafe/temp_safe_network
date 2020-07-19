// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::utils;
use bytes::Bytes;
use log::{debug, error, info, trace, warn};
use rand::{CryptoRng, Rng};
use routing::Node as Routing;
use safe_nd::{
    Address, Error, HandshakeRequest, HandshakeResponse, Message, MessageId, MsgEnvelope,
    MsgSender, NodePublicId, PublicId, Result, Signature, XorName,
};
use serde::Serialize;
use std::{
    cell::RefCell,
    collections::{hash_map::Entry, HashMap},
    fmt::{self, Display, Formatter},
    net::SocketAddr,
    rc::Rc,
};

#[derive(Clone, Debug)]
pub struct ClientInfo {
    pub public_id: PublicId,
}

impl Display for ClientInfo {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.public_id.name())
    }
}

pub enum OnboardingDecision {
    Continue(OnboardingStep),
    Disconnect(SocketAddr),
}

pub struct OnboardingStep {
    peer_address: SocketAddr,
    response: HandshakeResponse,
}

pub struct Onboarding {
    id: NodePublicId,
    clients: HashMap<SocketAddr, ClientInfo>,
    // Map of new client connections to the challenge value we sent them.
    client_candidates: HashMap<SocketAddr, (Vec<u8>, PublicId)>,
}

impl Onboarding {
    pub fn new(id: NodePublicId) -> Self {
        Self {
            id,
            clients: HashMap::<SocketAddr, ClientInfo>::new(),
            client_candidates: Default::default(),
        }
    }

    /// Query
    pub fn contains(&mut self, peer_addr: SocketAddr) -> bool {
        self.clients.contains_key(&peer_addr) || self.client_candidates.contains_key(&peer_addr)
    }

    pub fn remove_client(&mut self, peer_addr: SocketAddr) {
        if let Some(client) = self.clients.remove(&peer_addr) {
            // info!(
            //     "{}: Disconnected from {:?} on {}",
            //     self, client.public_id, peer_addr
            // );
        } else {
            let _ = self.client_candidates.remove(&peer_addr);
            // info!(
            //     "{}: Disconnected from client candidate on {}",
            //     self, peer_addr
            // );
        }
    }

    pub fn process<R: CryptoRng + Rng>(
        &mut self,
        handshake: HandshakeRequest,
        peer_addr: SocketAddr,
        rng: &mut R,
    ) -> Option<MessagingDuty> {
        match handshake {
            HandshakeRequest::Bootstrap(client_id) => self.try_bootstrap(peer_addr, &client_id),
            HandshakeRequest::Join(client_id) => self.try_join(peer_addr, client_id, rng),
            HandshakeRequest::ChallengeResult(signature) => self.receive_challenge_response(peer_addr, &signature),
        }
    }

    fn try_bootstrap(&self, peer_addr: SocketAddr, client_id: &PublicId) -> Option<MessagingDuty> {
        let elders = if self
            .section
            .matches_our_prefix(client_id.name())
        {
            self.section.our_elders()
                .into_iter()
                .map(|elder_address| {
                    let peer_addr = *p2p_node.peer_addr();
                    (elder_address, peer_addr)
                })
                .collect::<Vec<_>>()
        } else {
            let closest_known_elders = self
                .section
                .our_elders_sorted_by_distance_to(client_id.name())
                .into_iter()
                .map(|p2p_node| {
                    let peer_addr = *p2p_node.peer_addr();
                    (XorName(p2p_node.name().0), peer_addr)
                })
                .collect::<Vec<_>>()

            if closest_known_elders.is_empty() {
                warn!(
                    "{}: No closest known elders in any section we know of",
                    self
                );
                return None;
            } else {
                elders
            }
        };
        Some(MessagingDuty::SendHandshake { 
            address: peer_addr, 
            response: HandshakeResponse::Join(elders)
        })
    }
    
    /// Handles a received join request from a client.
    fn try_join<R: CryptoRng + Rng>(
        &mut self,
        peer_addr: SocketAddr,
        client_id: PublicId,
        rng: &mut R,
    ) -> Option<MessagingDuty> {
        if self.section.matches_our_prefix(client_id.name())
        {
            let challenge = utils::random_vec(rng, 8);
            let _ = self
                .client_candidates
                .insert(peer_addr, (challenge.clone(), client_id));
            Some(MessagingDuty::SendHandshake { 
                address: peer_addr, 
                response: HandshakeResponse::Challenge(PublicId::Node(self.id.clone()), challenge)
            })
        } else {
            debug!(
                "Client {} ({}) wants to join us but we are not its client handler",
                client_id, peer_addr
            );
            Some(MessagingDuty::DisconnectClient(peer_addr))
        }
    }

    /// Handles a received challenge response.
    ///
    /// Checks that the response contains a valid signature of the challenge we previously sent.
    /// If a client requests the section info, we also send it.
    fn receive_challenge_response(&mut self, peer_addr: SocketAddr, signature: &Signature) -> Option<OnboardingDecision> {
        if let Some((challenge, public_id)) = self.client_candidates.remove(&peer_addr) {
            let public_key = match utils::own_key(&public_id) {
                Some(pk) => pk,
                None => {
                    info!(
                        "{}: Client on {} identifies as a node: {}, hence disconnect from it.",
                        self, peer_addr, public_id
                    );
                    return Some(MessagingDuty::DisconnectClient(peer_addr));
                }
            };
            match public_key.verify(&signature, challenge) {
                Ok(()) => {
                    info!("{}: Accepted {} on {}.", self, public_id, peer_addr,);
                    let _ = self.clients.insert(peer_addr, ClientInfo { public_id });
                    None
                }
                Err(err) => {
                    info!(
                        "{}: Challenge failed for {} on {}: {}",
                        self, public_id, peer_addr, err
                    );
                    Some(MessagingDuty::DisconnectClient(peer_addr))
                }
            }
        } else {
            info!(
                "{}: {} supplied challenge response without us providing it.",
                self, peer_addr
            );
            Some(MessagingDuty::DisconnectClient(peer_addr))
        }
    }

    // fn send<T: Serialize>(&mut self, recipient: SocketAddr, msg: &T) {
    //     let msg = utils::serialise(msg);
    //     let msg = Bytes::from(msg);

    //     if let Err(e) = self
    //         .routing
    //         .borrow_mut()
    //         .send_message_to_client(recipient, msg, 0)
    //     {
    //         warn!(
    //             "{}: Could not send message to client {}: {:?}",
    //             self, recipient, e
    //         );
    //     }
    // }

    // pub fn notify_client(&mut self, client: &XorName, receipt: &DebitAgreementProof) {
    //     for client_id in self.lookup_client_and_its_apps(client) {
    //         self.send_notification_to_client(&client_id, &TransferNotification(receipt.clone()));
    //     }
    // }

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

    fn lookup_client_and_its_apps(&self, name: &XorName) -> Vec<PublicId> {
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

}

impl Display for Onboarding {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.id.name())
    }
}
