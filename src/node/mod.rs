// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod adult_duties;
mod services;
mod elder_duties;
mod keys;

use self::{adult_duties::AdultDuties, elder_duties::ElderDuties, keys::NodeKeys, services::{
    section_members::SectionMembers, 
    duty_finder::{InboundMsg, InboundMsgAnalysis}}];
use crate::{
    accumulator::Accumulator,
    cmd::{GroupDecision, OutboundMsg},
    utils, Config, messaging::Messaging, Result,
};
use crossbeam_channel::{Receiver, Select};
use hex_fmt::HexFmt;
use log::{debug, error, info, trace, warn};
use rand::{CryptoRng, Rng, SeedableRng};
use rand_chacha::ChaChaRng;
use routing::{
    event::Event as RoutingEvent, Node as Routing, SrcLocation, TransportEvent as ClientEvent,
};
use safe_nd::{Address, MsgEnvelope, MsgSender, NodeFullId, XorName};
use std::{
    cell::{Cell, RefCell},
    fmt::{self, Display, Formatter},
    fs,
    net::SocketAddr,
    path::PathBuf,
    rc::Rc,
};

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
    keys: NodeKeys,
    root_dir: PathBuf,
    state: State,
    event_receiver: Receiver<RoutingEvent>,
    client_receiver: Receiver<ClientEvent>,
    command_receiver: Receiver<Command>,
    routing: Rc<RefCell<Routing>>,
    messaging: Rc<RefCell<Messaging>>,
    msg_analysis: InboundMsgAnalysis,
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

        let root_dir = config.root_dir()?;
        let root_dir = root_dir.as_path();

        let routing = Rc::new(RefCell::new(routing));
        let keypair = Rc::new(RefCell::new(utils::key_pair(routing.clone())?));
        let keys = NodeKeys::new(keypair);

        let messaging = Messaging::new(routing.clone());
        let messaging = Rc::new(RefCell::new(messaging));

        let state = if is_elder {
            let total_used_space = Rc::new(Cell::new(0));
            let duties = ElderDuties::new(
                keys.clone(),
                &config,
                &total_used_space,
                init_mode,
                routing.clone(),
                messaging.clone(),
            )?;
            State::Elder {
                duties,
                accumulator: Accumulator::new(routing.clone()),
            }
        } else {
            info!("Initializing new node as Infant");
            State::Infant
        };

        let msg_analysis = InboundMsgAnalysis::new(routing.clone());

        let node = Self {
            id,
            keys: keys.clone(),
            root_dir: root_dir.to_path_buf(),
            state,
            event_receiver,
            client_receiver,
            command_receiver,
            routing,
            messaging,
            msg_analysis,
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
    pub fn is_adult(&self) -> bool {
        if let State::Adult { .. } = self.state {
            true
        } else {
            false
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

    fn step_routing(&mut self, event: RoutingEvent) {
        debug!("Received routing event: {:?}", event);
        let result = self.process_network_event(event);
        self.send_while_any(result);
    }

    fn step_client(&mut self, event: ClientEvent) {
        let result = self.process_client_event(event);
        self.send_while_any(result);
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
            self.keys.clone(),
            &config,
            &total_used_space,
            Init::New,
            self.routing.clone(),
            self.messaging.clone(),
        )?;
        self.state = State::Elder {
            duties,
            accumulator: Accumulator::new(self.routing.clone()),
        };
        Ok(())
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

    fn process_network_event(&mut self, event: RoutingEvent) -> Option<OutboundMsg> {
        match event {
            RoutingEvent::Consensus(custom_event) => {
                match bincode::deserialize::<GroupDecision>(&custom_event) {
                    Ok(consensused_cmd) => self
                        .elder_duties()?
                        .gateway()
                        .handle_consensused_cmd(consensused_cmd),
                    Err(e) => {
                        error!("Invalid GroupDecision passed from Routing: {:?}", e);
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
                if let Some(cmds) = self.elder_duties()?.member_left(XorName(name.0), age) {
                    for cmd in cmds {
                        let result = self.send(cmd);
                        self.send_while_any(result);
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
                // this here is where we query source section for the reward counter
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
                    "Received MsgEnvelope: {:8?}\n Sent from {:?} to {:?}",
                    HexFmt(&content),
                    src,
                    dst
                );
                self.handle_serialized_msg(src, content)
            }
            RoutingEvent::EldersChanged { .. } => self.elder_duties()?.elders_changed(),
            // Ignore all other events
            _ => None,
        }
    }

    fn handle_serialized_msg(&mut self, src: SrcLocation, content: Vec<u8>) -> Option<OutboundMsg> {
        match bincode::deserialize::<MsgEnvelope>(&content) {
            Ok(msg) => self.handle_remote_msg(msg),
            Err(e) => {
                error!(
                    "Error deserializing routing MsgEnvelope into msg type: {:?}",
                    e
                );
                None
            }
        }
    }

    fn handle_remote_msg(&mut self, msg: MsgEnvelope) -> Option<OutboundMsg> {
        use InboundMsg::*;
        match self.msg_analysis.evaluate(msg) {
            Accumulate(msg) => self.accumulate_msg(msg),
            PushToClient(msg) => OutboundMsg::SendToClient(msg),
            ForwardToNetwork(msg) => OutboundMsg::SendToSection(msg),
            RunAtGateway(msg) => self.elder_duties()?.gateway().handle_auth_cmd(msg),
            RunAtPayment(msg) => self.elder_duties()?.data_payment().pay_for_data(msg),
            RunAtMetadata(msg) => self.elder_duties()?.metadata().receive_msg(msg),
            RunAtAdult(msg) => self.adult_duties()?.receive_msg(msg),
            RunAtRewards(msg) => unimplemented!(),
            Unknown => {
                error!("Unknown message destination: {:?}", msg.message.id());
                None
            }
        }
    }

    fn accumulate_msg(&mut self, msg: MsgEnvelope) -> Option<OutboundMsg> {
        let id = self.keys.public_key();
        info!(
            "{}: Accumulating signatures for {:?}",
            &id,
            msg.message.id()
        );
        if let Some((accumulated_msg, signature)) = self.accumulator()?.accumulate_cmd(msg) {
            info!(
                "Got enough signatures for {:?}",
                accumulated_msg.message.id()
            );
            // upgrade sender to Section, since it accumulated
            let sender = match msg.most_recent_sender() {
                MsgSender::Node { duty, .. } => MsgSender::Section {
                    id,
                    duty: *duty,
                    signature,
                },
                _ => return None, // invalid use case, we only accumulate from Nodes
            };
            // consider msg.pop_proxy() to remove the Node
            // or we just set the last proxy always
            self.handle_remote_msg(msg.with_proxy(sender))
        } else {
            None
        }
    }

    /// Will only act on this (i.e. return an action) if node is currently an Elder.
    fn process_client_event(&mut self, event: ClientEvent) -> Option<OutboundMsg> {
        use ClientEvent::*;
        let mut rng = ChaChaRng::from_seed(self.rng.gen());
        let elder_duties = self.elder_duties()?;
        match event {
            ConnectedTo { peer } => elder_duties
                .gateway()
                .handle_new_connection(peer.peer_addr()),
            ConnectionFailure { peer, .. } => {
                elder_duties
                    .gateway()
                    .handle_connection_failure(peer.peer_addr());
            }
            NewMessage { peer, msg } => {
                let gateway = elder_duties.gateway();
                let parsed = gateway.try_parse_client_msg(peer.peer_addr(), &msg, &mut rng)?;
                return gateway.handle_client_msg(parsed.client.public_id, parsed.msg);
            }
            SentUserMessage { peer, .. } => {
                trace!(
                    "{}: Succesfully sent Message to: {}",
                    self,
                    peer.peer_addr()
                );
            }
            UnsentUserMessage { peer, .. } => {
                info!("{}: Not sent Message to: {}", self, peer.peer_addr());
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

    fn accumulator(&mut self) -> Option<&mut Accumulator> {
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

    pub fn send_while_any(&mut self, cmd: Option<OutboundMsg>) {
        let mut next_cmd = cmd;
        while let Some(cmd) = next_cmd {
            next_cmd = self.send(cmd);
        }
    }

    pub fn vote_for(&mut self, cmd: GroupDecision) -> Option<OutboundMsg> {
        self.routing
            .borrow_mut()
            .vote_for_user_event(utils::serialise(&cmd))
            .map_or_else(
                |_err| {
                    error!("Cannot vote. node is not an elder");
                    None
                },
                |()| None,
            )
    }

    pub fn send(&mut self, outbound: OutboundMsg) -> Option<OutboundMsg> {
        use OutboundMsg::*;
        match outbound {
            SendToClient(msg) => {
                if self.is_handler_for(msg.origin.address()) {
                    self.elder_duties()?.gateway().push_to_client(msg)
                } else {
                    Some(SendToSection(msg))
                }
            }
            SendToNode(msg) => self.messaging.borrow_mut().send_to_node(msg),
            SendToAdults { targets, msg } => self.messaging.borrow_mut().send_to_nodes(targets, msg),
            SendToSection(msg) => self.messaging.borrow_mut().send_to_network(msg),
            VoteFor(decision) => self.vote_for(decision),
        }
    }

    fn is_handler_for(&self, address: Address) -> bool {
        match address {
            Address::Client(xorname) => self.msg_analysis.self_is_handler_for(xorname),
            _ => false,
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
