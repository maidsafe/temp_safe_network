// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{action::Action, utils::random_vec};
use bytes::Bytes;
use log::info;
//use pickledb::PickleDb;
use quic_p2p::{Config as QuicP2pConfig, Event, Peer, QuicP2p};
use safe_nd::{Challenge, ClientPublicId, Requester, Response, Signature};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::mpsc::{channel, Receiver},
};
use unwrap::unwrap;

pub(crate) struct SourceElder {
    //client_accounts: PickleDb,
    clients: HashMap<SocketAddr, ClientPublicId>,
    client_candidates: HashMap<SocketAddr, Vec<u8>>,
    quic_p2p: QuicP2p,
}

impl SourceElder {
    pub fn new(config: QuicP2pConfig) -> (Self, Receiver<Event>) {
        let (quic_p2p, event_receiver) = SourceElder::setup_quic_p2p(config);
        (
            Self {
                clients: Default::default(),
                client_candidates: Default::default(),
                quic_p2p,
            },
            event_receiver,
        )
    }

    fn setup_quic_p2p(config: QuicP2pConfig) -> (QuicP2p, Receiver<Event>) {
        let (event_sender, event_receiver) = channel();
        let mut quic_p2p = unwrap!(quic_p2p::Builder::new(event_sender)
            .with_config(config)
            .build());
        let our_conn_info = unwrap!(quic_p2p.our_connection_info());
        info!(
            "QuicP2p started on {}\nwith certificate {:?}",
            our_conn_info.peer_addr, our_conn_info.peer_cert_der
        );
        println!(
            "Our connection info:\n{}\n",
            unwrap!(serde_json::to_string(&our_conn_info))
        );
        (quic_p2p, event_receiver)
    }

    pub fn is_client(&self, peer_addr: &SocketAddr) -> bool {
        self.clients.contains_key(peer_addr)
    }

    pub fn handle_new_connection(&mut self, peer: Peer) -> Option<Action> {
        // If we already know the peer, drop the connection attempt
        if self.clients.contains_key(&peer.peer_addr())
            || self.client_candidates.contains_key(&peer.peer_addr())
        {
            return None;
        }
        // Peer here is assumed to be `Peer::Client` during phase 1, and has been checked at the
        // Vault layer.
        let challenge = random_vec(8);
        match bincode::serialize(&Challenge::Request(challenge.clone())) {
            Ok(msg) => self.quic_p2p.send(peer.clone(), Bytes::from(msg)),
            Err(err) => info!("Unable to serialise message: {}", err),
        }
        let _ = self.client_candidates.insert(peer.peer_addr(), challenge);
        None
    }

    pub fn handle_terminated_connection(&mut self, peer_addr: SocketAddr) -> Option<Action> {
        let _ = self.clients.remove(&peer_addr);
        None
    }

    pub fn handle_established_connection(
        &mut self,
        peer_addr: SocketAddr,
        public_id: ClientPublicId,
        signature: Signature,
    ) -> Option<Action> {
        if let Some(challenge) = self.client_candidates.remove(&peer_addr) {
            match public_id.public_key().verify(&signature, challenge) {
                Ok(()) => {
                    info!("Connection establised with: {} as {}", peer_addr, public_id);
                    let _ = self.clients.insert(peer_addr, public_id);
                    None
                }
                Err(err) => {
                    info!("Failed to establish connection: {}", err);
                    None
                }
            }
        } else {
            info!("Client supplied challenge response without us providing it");
            None
        }
    }

    pub fn handle_client_request(
        &mut self,
        client_id: &ClientPublicId,
        msg: Vec<u8>,
    ) -> Option<Action> {
        unimplemented!();
    }
}
