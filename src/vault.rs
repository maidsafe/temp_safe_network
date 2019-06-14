// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//use crate::{coins_handler::CoinsHandler, destination_elder::DestinationElder};
use crate::{action::Action, adult::Adult, error::Result, source_elder::SourceElder};
use log::{info, trace};
//use pickledb::PickleDb;
use quic_p2p::{Config as QuickP2pConfig, Event, Peer};
use safe_nd::{Challenge, ClientPublicId, Message, Request, Requester, Signature};
use std::{net::SocketAddr, sync::mpsc::Receiver};
use unwrap::unwrap;

#[allow(clippy::large_enum_variant)]
enum State {
    Elder {
        src: SourceElder,
        //dst: DestinationElder,
        //coins_handler: CoinsHandler,
    },
    Adult(Adult),
}

/// Main vault struct.
pub struct Vault {
    //id: NodeFullId,
    state: State,
    event_receiver: Option<Receiver<Event>>,
}

impl Vault {
    /// Construct a new vault instance.
    pub fn new(config: QuickP2pConfig) -> Result<Self> {
        let (src, event_receiver) = SourceElder::new(config);
        let event_receiver = Some(event_receiver);

        Ok(Self {
            //id: Default::default(),
            state: State::Elder { src },
            event_receiver,
        })
    }

    /// Run the main event loop.  Blocks until the vault is terminated.
    pub fn run(&mut self) {
        if let Some(event_receiver) = self.event_receiver.take() {
            for event in event_receiver.iter() {
                let mut some_action = self.handle_quic_p2p_event(event);
                while let Some(action) = some_action {
                    some_action = self.handle_action(action);
                }
            }
        } else {
            info!("Event receiver not available!");
        }
    }

    fn handle_quic_p2p_event(&mut self, event: Event) -> Option<Action> {
        match event {
            Event::ConnectedTo { peer } => match &peer {
                Peer::Node { .. } => None,
                Peer::Client { .. } => {
                    info!("Connected to {:?}", peer);
                    self.source_elder_mut()
                        .and_then(|source_elder| source_elder.handle_new_connection(peer))
                }
            },
            Event::ConnectionFailure { peer_addr } => {
                info!("Disconnected from {}", peer_addr);
                self.source_elder_mut()
                    .and_then(|source_elder| source_elder.handle_terminated_connection(peer_addr))
            }
            Event::NewMessage { peer_addr, msg } => {
                if self
                    .source_elder()
                    .map(|source_elder| source_elder.is_client(&peer_addr))
                    .unwrap_or(false)
                {
                    match bincode::deserialize(&msg) {
                        Ok(msg) => {
                            info!("Received message from {}", peer_addr);
                            self.handle_message(peer_addr, msg)
                        }
                        Err(err) => {
                            info!("Unable to deserialise message: {}", err);
                            None
                        }
                    }
                } else {
                    match bincode::deserialize(&msg) {
                        Ok(Challenge::Response(public_id, signature)) => {
                            trace!("Received challenge response from {}", peer_addr);
                            self.handle_challenge(peer_addr, public_id, signature)
                        }
                        Ok(Challenge::Request(_)) => {
                            info!("Received unexpected challenge request");
                            None
                        }
                        Err(err) => {
                            info!("Unable to deserialise challenge: {}", err);
                            None
                        }
                    }
                }
            }
            event => {
                info!("Unexpected event: {}", event);
                None
            }
        }
    }

    fn handle_message(&self, peer_addr: SocketAddr, msg: Message) -> Option<Action> {
        match msg {
            Message::Request {
                request,
                message_id,
                signature,
            } => None,
            Message::Response {
                response,
                message_id,
            } => None,
        }
    }

    fn handle_challenge(
        &mut self,
        peer_addr: SocketAddr,
        public_id: ClientPublicId,
        signature: Signature,
    ) -> Option<Action> {
        self.source_elder_mut().and_then(|source_elder| {
            source_elder.handle_established_connection(peer_addr, public_id, signature)
        })
    }

    fn handle_action(&mut self, action: Action) -> Option<Action> {
        match action {
            Action::ClientRequest { client_id, msg } => self.handle_client_request(&client_id, msg),
        }
    }

    fn handle_client_request(
        &mut self,
        client_id: &ClientPublicId,
        msg: Vec<u8>,
    ) -> Option<Action> {
        self.source_elder_mut()
            .and_then(|source_elder| source_elder.handle_client_request(client_id, msg))
    }

    fn source_elder(&self) -> Option<&SourceElder> {
        match &self.state {
            State::Elder { ref src, .. } => Some(src),
            State::Adult(_) => None,
        }
    }

    fn source_elder_mut(&mut self) -> Option<&mut SourceElder> {
        match &mut self.state {
            State::Elder { ref mut src, .. } => Some(src),
            State::Adult(_) => None,
        }
    }
}
