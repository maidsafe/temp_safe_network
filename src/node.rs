// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{
    accumulator::Accumulator,
    action::{Action, ConsensusAction},
    duties::{adult::AdultDuties, elder::ElderDuties},
    rpc::Rpc as Message,
    utils, Config, Result,
};
use crossbeam_channel::{Receiver, Select};
use hex_fmt::HexFmt;
use log::{debug, error, info, trace, warn};
use rand::{CryptoRng, Rng, SeedableRng};
use rand_chacha::ChaChaRng;
use routing::{
    event::Event as RoutingEvent, DstLocation, Node as Routing, Prefix, SrcLocation,
    TransportEvent as ClientEvent,
};
use safe_nd::{
    ClientAuth, ClientRequest, NodeFullId, NodeRequest, PublicId, Read, Request, Response,
    SystemOp, Write, XorName,
};
use std::borrow::Cow;
use std::{
    cell::{Cell, RefCell},
    fmt::{self, Display, Formatter},
    fs,
    net::SocketAddr,
    path::PathBuf,
    rc::Rc,
};
use threshold_crypto::Signature;

const STATE_FILENAME: &str = "state";

#[allow(clippy::large_enum_variant)]
enum State {
    Infant,
    Adult {
        duties: AdultDuties,
        accumulator: Accumulator,
    },
    Elder {
        duties: ElderDuties,
        accumulator: Accumulator,
    },
}

/// Specifies whether to try loading cached data from disk, or to just construct a new instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Init {
    Load,
    New,
}

/// Command that the user can send to a running node to control its execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    /// Shutdown the vault
    Shutdown,
}

/// Main node struct.
pub struct Node<R: CryptoRng + Rng> {
    id: NodeFullId,
    root_dir: PathBuf,
    state: State,
    event_receiver: Receiver<RoutingEvent>,
    client_receiver: Receiver<ClientEvent>,
    command_receiver: Receiver<Command>,
    routing: Rc<RefCell<Routing>>,
    rng: R,
}

impl<R: CryptoRng + Rng> Node<R> {
    /// Create and start vault. This will block until a `Command` to free it is fired.
    pub fn new(
        routing: Routing,
        event_receiver: Receiver<RoutingEvent>,
        client_receiver: Receiver<ClientEvent>,
        config: &Config,
        command_receiver: Receiver<Command>,
        mut rng: R,
    ) -> Result<Self> {
        let mut init_mode = Init::Load;

        let (is_elder, id) = Self::read_state(&config)?.unwrap_or_else(|| {
            let id = NodeFullId::new(&mut rng);
            init_mode = Init::New;
            (false, id)
        });

        #[cfg(feature = "mock_parsec")]
        {
            // trace!(
            //     "creating node {:?} with routing_id {:?}",
            //     id.public_id().name(),
            //     routing.id()
            // );
        }

        let root_dir = config.root_dir()?;
        let root_dir = root_dir.as_path();

        let routing = Rc::new(RefCell::new(routing));

        let state = if is_elder {
            let total_used_space = Rc::new(Cell::new(0));
            let duties = ElderDuties::new(
                id.public_id().clone(),
                &config,
                &total_used_space,
                init_mode,
                routing.clone(),
            )?;
            State::Elder {
                duties,
                accumulator: Accumulator::new(routing.clone()),
            }
        } else {
            info!("Initializing new node as Infant");
            State::Infant
        };

        let node = Self {
            id,
            root_dir: root_dir.to_path_buf(),
            state,
            event_receiver,
            client_receiver,
            command_receiver,
            routing,
            rng,
        };
        node.dump_state()?;
        Ok(node)
    }

    /// Returns our connection info.
    pub fn our_connection_info(&mut self) -> Result<SocketAddr> {
        self.routing
            .borrow_mut()
            .our_connection_info()
            .map_err(From::from)
    }

    /// Returns whether routing node is in elder state.
    pub fn is_elder(&mut self) -> bool {
        self.routing.borrow().is_elder()
    }

    /// Returns whether node is in adult state.
    #[allow(unused)]
    fn is_adult(&self) -> bool {
        if let State::Adult { .. } = self.state {
            true
        } else {
            false
        }
    }

    /// Runs the main event loop. Blocks until the node is terminated.
    pub fn run(&mut self) {
        loop {
            let mut sel = Select::new();

            let mut r_node = self.routing.borrow_mut();
            r_node.register(&mut sel);
            let routing_event_rx_idx = sel.recv(&self.event_receiver);
            let client_network_rx_idx = sel.recv(&self.client_receiver);
            let command_rx_idx = sel.recv(&self.command_receiver);

            let selected_operation = sel.ready();
            drop(r_node);

            match selected_operation {
                idx if idx == client_network_rx_idx => {
                    let event = match self.client_receiver.recv() {
                        Ok(ev) => ev,
                        Err(e) => panic!("FIXME: {:?}", e),
                    };
                    self.step_client(event);
                }
                idx if idx == routing_event_rx_idx => {
                    let event = match self.event_receiver.recv() {
                        Ok(ev) => ev,
                        Err(e) => panic!("FIXME: {:?}", e),
                    };
                    self.step_routing(event);
                }
                idx if idx == command_rx_idx => {
                    let command = match self.command_receiver.recv() {
                        Ok(ev) => ev,
                        Err(e) => panic!("FIXME: {:?}", e),
                    };
                    match command {
                        Command::Shutdown => break,
                    }
                }
                idx => {
                    if let Err(err) = self.routing.borrow_mut().handle_selected_operation(idx) {
                        warn!("Could not process operation: {}", err);
                    }
                }
            }
        }
    }

    fn promote_to_adult(&mut self) -> Result<()> {
        let mut config = Config::default();
        config.set_root_dir(self.root_dir.clone());
        let total_used_space = Rc::new(Cell::new(0));
        let duties = AdultDuties::new(
            self.id.public_id().clone(),
            &config,
            &total_used_space,
            Init::New,
            self.routing.clone(),
        )?;
        self.state = State::Adult {
            duties,
            accumulator: Accumulator::new(self.routing.clone()),
        };
        Ok(())
    }

    fn promote_to_elder(&mut self) -> Result<()> {
        let mut config = Config::default();
        config.set_root_dir(self.root_dir.clone());
        let total_used_space = Rc::new(Cell::new(0));
        let duties = ElderDuties::new(
            self.id.public_id().clone(),
            &config,
            &total_used_space,
            Init::New,
            self.routing.clone(),
        )?;
        self.state = State::Elder {
            duties,
            accumulator: Accumulator::new(self.routing.clone()),
        };
        Ok(())
    }

    /// Processes any outstanding network events and returns. Does not block.
    /// Returns whether at least one event was processed.
    pub fn poll(&mut self) -> bool {
        let mut _processed = false;
        loop {
            let mut sel = Select::new();
            let mut r_node = self.routing.borrow_mut();
            r_node.register(&mut sel);
            let routing_event_rx_idx = sel.recv(&self.event_receiver);
            let client_network_rx_idx = sel.recv(&self.client_receiver);
            let command_rx_idx = sel.recv(&self.command_receiver);

            if let Ok(selected_operation) = sel.try_ready() {
                drop(r_node);

                match selected_operation {
                    idx if idx == client_network_rx_idx => {
                        let event = match self.client_receiver.recv() {
                            Ok(ev) => ev,
                            Err(e) => panic!("FIXME: {:?}", e),
                        };
                        self.step_client(event);
                        _processed = true;
                    }
                    idx if idx == routing_event_rx_idx => {
                        let event = match self.event_receiver.recv() {
                            Ok(ev) => ev,
                            Err(e) => panic!("FIXME: {:?}", e),
                        };
                        self.step_routing(event);
                        _processed = true;
                    }
                    idx if idx == command_rx_idx => {
                        let command = match self.command_receiver.recv() {
                            Ok(ev) => ev,
                            Err(e) => panic!("FIXME: {:?}", e),
                        };
                        match command {
                            Command::Shutdown => (),
                        }
                        _processed = true;
                    }
                    idx => {
                        if let Err(err) = self.routing.borrow_mut().handle_selected_operation(idx) {
                            warn!("Could not process operation: {}", err);
                            break;
                        }
                    }
                }
            } else {
                break;
            }
        }

        _processed
    }

    fn step_routing(&mut self, event: RoutingEvent) {
        debug!("Received routing event: {:?}", event);
        let mut maybe_action = self.handle_routing_event(event);
        while let Some(action) = maybe_action {
            maybe_action = self.handle_action(action);
        }
    }

    fn step_client(&mut self, event: ClientEvent) {
        let mut maybe_action = self.handle_client_event(event);
        while let Some(action) = maybe_action {
            maybe_action = self.handle_action(action);
        }
    }

    fn handle_routing_event(&mut self, event: RoutingEvent) -> Option<Action> {
        match event {
            RoutingEvent::Consensus(custom_event) => {
                match bincode::deserialize::<ConsensusAction>(&custom_event) {
                    Ok(consensus_action) => self
                        .elder_duties()?
                        .handle_consensused_action(consensus_action),
                    Err(e) => {
                        error!("Invalid ConsensusAction passed from Routing: {:?}", e);
                        None
                    }
                }
            }
            RoutingEvent::Promoted => self.promote_to_elder().map_or_else(
                |err| {
                    error!("Error when promoting node to Elder: {:?}", err);
                    None
                },
                |()| {
                    info!("Vault promoted to Elder");
                    None
                },
            ),
            RoutingEvent::MemberLeft { name, age } => {
                trace!("A node has left the section. Node: {:?}", name);
                let next_actions = self.elder_duties()?.member_left(XorName(name.0), age);
                if let Some(actions) = next_actions {
                    for action in actions {
                        let _ = self.handle_action(action);
                    }
                };
                None
            }
            RoutingEvent::MemberJoined { .. } => {
                trace!("New member has joined the section");
                let elder_count = self.routing.borrow().our_elders().count();
                let adult_count = self.routing.borrow().our_adults().count();
                info!("No. of Elders: {}", elder_count);
                info!("No. of Adults: {}", adult_count);
                None
            }
            RoutingEvent::Connected(_) => self.promote_to_adult().map_or_else(
                |err| {
                    error!("Error when promoting node to Adult: {:?}", err);
                    None
                },
                |()| {
                    info!("Vault promoted to Adult");
                    None
                },
            ),
            RoutingEvent::MessageReceived { content, src, dst } => {
                info!(
                    "Received message: {:8?}\n Sent from {:?} to {:?}",
                    HexFmt(&content),
                    src,
                    dst
                );
                self.handle_routing_message(src, content)
            }
            RoutingEvent::EldersChanged { .. } => self.elder_duties()?.elders_changed(),
            // Ignore all other events
            _ => None,
        }
    }

    fn accumulate_msg(&mut self, src: SrcLocation, msg: Message) -> Option<Action> {
        let id = *self.routing.borrow().id().name();
        info!(
            "{}: Accumulating signatures for {:?}",
            &id,
            msg.message_id()
        );
        if let Some((accumulated_msg, signature)) = self.accumulator_mut()?.accumulate_request(msg)
        {
            info!(
                "Got enough signatures for {:?}",
                accumulated_msg.message_id()
            );
            let prefix = match src {
                SrcLocation::Node(name) => Prefix::<routing::XorName>::new(32, name),
                SrcLocation::Section(prefix) => prefix,
            };
            self.process_locally(
                SrcLocation::Section(prefix),
                accumulated_msg,
                Some(signature),
            )
        } else {
            None
        }
    }

    fn process_locally(
        &mut self,
        src: SrcLocation,
        msg: Message,
        accumulated_sig: Option<Signature>,
    ) -> Option<Action> {
        match self.state {
            State::Elder { .. } => self.elder_duties()?.receive_msg(src, msg, accumulated_sig),
            State::Adult { .. } => self.adult_duties()?.receive_msg(src, msg, accumulated_sig),
            _ => None,
        }
    }

    fn handle_routing_message(&mut self, src: SrcLocation, message: Vec<u8>) -> Option<Action> {
        match bincode::deserialize::<Message>(&message) {
            Ok(msg) => match &msg {
                Message::Request {
                    request,
                    requester,
                    signature,
                    ..
                } => {
                    if matches!(requester, PublicId::Node(_)) {
                        if let Some((_, signature)) = signature.clone() {
                            self.process_locally(src, msg, Some(signature.0))
                        } else {
                            error!("Signature missing from duplication GET request");
                            None
                        }
                    } else {
                        match request {
                            Request::Node(NodeRequest::Read(Read::Blob(_)))
                            | Request::Node(NodeRequest::Write(Write::Blob(_))) => {
                                self.accumulate_msg(src, msg)
                            }
                            other => unimplemented!("Should not receive: {:?}", other),
                        }
                    }
                }
                Message::Response { response, .. } => match response {
                    Response::Write(_) | Response::GetIData(_) => {
                        self.process_locally(src, msg, None)
                    }
                    _ => unimplemented!("Should not receive: {:?}", response),
                },
                Message::Duplicate { .. } => self.accumulate_msg(src, msg),
                Message::DuplicationComplete { .. } => self.process_locally(src, msg, None),
            },
            Err(e) => {
                error!("Error deserializing routing message into msg type: {:?}", e);
                None
            }
        }
    }

    fn handle_client_event(&mut self, event: ClientEvent) -> Option<Action> {
        use ClientEvent::*;
        let mut rng = ChaChaRng::from_seed(self.rng.gen());
        let elder_duties = self.elder_duties()?;
        match event {
            ConnectedTo { peer } => elder_duties.handle_new_connection(peer.peer_addr()),
            ConnectionFailure { peer, .. } => {
                elder_duties.handle_connection_failure(peer.peer_addr());
            }
            NewMessage { peer, msg } => {
                return elder_duties.handle_client_message(peer.peer_addr(), &msg, &mut rng);
            }
            SentUserMessage { peer, .. } => {
                trace!(
                    "{}: Succesfully sent message to: {}",
                    self,
                    peer.peer_addr()
                );
            }
            UnsentUserMessage { peer, .. } => {
                info!("{}: Not sent message to: {}", self, peer.peer_addr());
            }
            BootstrapFailure | BootstrappedTo { .. } => {
                error!("unexpected bootstrapping client event")
            }
            Finish => {
                info!("{}: Received Finish event", self);
            }
        }
        None
    }

    #[allow(dead_code)]
    fn vote_for_action(&mut self, action: &ConsensusAction) -> Option<Action> {
        self.routing
            .borrow_mut()
            .vote_for_user_event(utils::serialise(&action))
            .map_or_else(
                |_err| {
                    error!("Cannot vote. node is not an elder");
                    None
                },
                |()| None,
            )
    }

    fn handle_action(&mut self, action: Action) -> Option<Action> {
        trace!("{} handle action {:?}", self, action);
        use Action::*;
        match action {
            // Bypass client requests
            // VoteFor(action) => self.vote_for_action(&action),
            VoteFor(action) => self.elder_duties()?.handle_consensused_action(action),
            ForwardClientRequest(msg) => self.forward_client_request(msg),
            RespondToOurDataHandlers { rpc } => {
                // TODO - once Routing is integrated, we'll construct the full message to send
                //        onwards, and then if we're also part of the data handlers, we'll call that
                //        same handler which Routing will call after receiving a message.

                self.respond_to_data_handlers(rpc)
            }
            RespondToClientHandlers { sender, rpc } => {
                let client_name = utils::requester_address(&rpc);

                // TODO - once Routing is integrated, we'll construct the full message to send
                //        onwards, and then if we're also part of the client handlers, we'll call that
                //        same handler which Routing will call after receiving a message.
                debug!("Responded to client handlers with {:?}", &rpc);
                if self.self_is_handler_for(&client_name) {
                    return self.elder_duties()?.respond_to_gateway(sender, rpc);
                }
                None
            }
            SendToPeers { targets, rpc } => {
                let mut next_action = None;
                for target in targets {
                    if target == *self.id.public_id().name() {
                        info!("Vault is one of the targets. Accumulating message locally");
                        next_action = self.accumulate_msg(
                            SrcLocation::Node(routing::XorName(target.0)),
                            rpc.clone(),
                        );
                    } else {
                        // Always None
                        let _ = self.send_message_to_peer(target, rpc.clone());
                    }
                }
                next_action
            }
            RespondToClient {
                message_id,
                response,
            } => {
                self.elder_duties()?.respond_to_client(message_id, response);
                None
            }
        }
    }

    fn respond_to_data_handlers(&self, msg: Message) -> Option<Action> {
        let name = *self.routing.borrow().id().name();
        self.routing
            .borrow_mut()
            .send_message(
                SrcLocation::Node(name),
                DstLocation::Section(name),
                utils::serialise(&msg),
            )
            .map_or_else(
                |err| {
                    error!("Unable to respond to our data handlers: {:?}", err);
                    None
                },
                |()| {
                    info!("Responded to our data handlers with: {:?}", &msg);
                    None
                },
            )
    }

    fn send_message_to_peer(&self, target: XorName, msg: Message) -> Option<Action> {
        let name = *self.routing.borrow().id().name();
        self.routing
            .borrow_mut()
            .send_message(
                SrcLocation::Node(name),
                DstLocation::Node(routing::XorName(target.0)),
                utils::serialise(&msg),
            )
            .map_or_else(
                |err| {
                    error!("Unable to send message to Peer: {:?}", err);
                    None
                },
                |()| {
                    info!("Sent message to Peer {:?} from node {:?}", target, name);
                    None
                },
            )
    }

    fn forward_client_request(&mut self, msg: Message) -> Option<Action> {
        trace!("{} received a client request {:?}", self, msg);
        let dst_address = if let Message::Request { ref request, .. } = msg {
            match request.dst_address() {
                Some(address) => address,
                None => {
                    // temporary, while Authenticator is not
                    // its own app using Map to store these things.
                    if self.is_client_auth_write(request) {
                        Cow::Borrowed(self.id.public_id().name())
                    } else {
                        error!("{}: Logic error - no data handler address available.", self);
                        return None;
                    }
                }
            }
        } else {
            error!(
                "{}: Logic error - expected Request, but got something else.",
                self
            );
            return None;
        };

        // TODO - once Routing is integrated, we'll construct the full message to send
        //        onwards, and then if we're also part of the data handlers, we'll call that
        //        same handler which Routing will call after receiving a message.

        if self.self_is_handler_for(&dst_address) {
            if let Message::Request { .. } = &msg {
                return self.process_locally(
                    SrcLocation::Node(routing::XorName(rand::random())), // dummy xorname
                    msg,
                    None,
                );
            }
        }
        error!("{}: Logic error - unexpected msg.", self);
        return None;
    }

    fn is_client_auth_write(&self, request: &Request) -> bool {
        match request {
            Request::Client(ClientRequest::System(SystemOp::ClientAuth(
                ClientAuth::DelAuthKey { .. },
            )))
            | Request::Client(ClientRequest::System(SystemOp::ClientAuth(
                ClientAuth::InsAuthKey { .. },
            ))) => true,
            _ => false,
        }
    }

    #[allow(unused)]
    fn is_client_req(&self, request: &Request) -> bool {
        match request {
            Request::Client(_) => true,
            _ => false,
        }
    }

    #[allow(unused)]
    fn is_node_req(&self, request: &Request) -> bool {
        match request {
            Request::Node(_) => true,
            _ => false,
        }
    }

    #[allow(unused)]
    fn is_client_handler_req(&self, request: &Request) -> bool {
        match request {
            Request::Client(ClientRequest::System(SystemOp::Transfers(_))) // Temporary! Is really at Section(TransferReplicas).
            | Request::Client(ClientRequest::System(SystemOp::ClientAuth(_)))
            | Request::Client(ClientRequest::Read(Read::Account(_)))
            | Request::Client(ClientRequest::Write { write: Write::Account(_), .. })
            | Request::Node(NodeRequest::System(SystemOp::ClientAuth(_)))
            | Request::Node(NodeRequest::Read(Read::Account(_)))
            | Request::Node(NodeRequest::Write(Write::Account(_))) => true,
            _ => false,
        }
    }

    #[allow(unused)]
    fn is_data_handler_req(&self, request: &Request) -> bool {
        !self.is_client_handler_req(request) // temporary simplification
    }

    #[allow(unused)]
    fn is_transfer_handler_req(&self, request: &Request) -> bool {
        match request {
            Request::Client(ClientRequest::System(SystemOp::Transfers(_))) => true,
            _ => false,
        }
    }

    fn self_is_handler_for(&self, address: &XorName) -> bool {
        let xorname = routing::XorName(address.0);
        match self.routing.borrow().matches_our_prefix(&xorname) {
            Ok(result) => result,
            _ => false,
        }
    }

    #[allow(unused)]
    fn accumulator(&self) -> Option<&Accumulator> {
        match &self.state {
            State::Infant => None,
            State::Elder {
                ref accumulator, ..
            } => Some(accumulator),
            State::Adult {
                ref accumulator, ..
            } => Some(accumulator),
        }
    }

    fn accumulator_mut(&mut self) -> Option<&mut Accumulator> {
        match &mut self.state {
            State::Infant => None,
            State::Elder {
                ref mut accumulator,
                ..
            } => Some(accumulator),
            State::Adult {
                ref mut accumulator,
                ..
            } => Some(accumulator),
        }
    }

    fn adult_duties(&mut self) -> Option<&mut AdultDuties> {
        match &mut self.state {
            State::Adult { ref mut duties, .. } => Some(duties),
            _ => None,
        }
    }

    fn elder_duties(&mut self) -> Option<&mut ElderDuties> {
        match &mut self.state {
            State::Elder { ref mut duties, .. } => Some(duties),
            _ => None,
        }
    }

    fn dump_state(&self) -> Result<()> {
        let path = self.root_dir.join(STATE_FILENAME);
        let is_elder = matches!(self.state, State::Elder { .. });
        Ok(fs::write(path, utils::serialise(&(is_elder, &self.id)))?)
    }

    /// Returns Some((is_elder, ID)) or None if file doesn't exist.
    fn read_state(config: &Config) -> Result<Option<(bool, NodeFullId)>> {
        let path = config.root_dir()?.join(STATE_FILENAME);
        if !path.is_file() {
            return Ok(None);
        }
        let contents = fs::read(path)?;
        Ok(Some(bincode::deserialize(&contents)?))
    }
}

impl<R: CryptoRng + Rng> Display for Node<R> {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.id.public_id())
    }
}
