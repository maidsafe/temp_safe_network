// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod adult_duties;
mod elder_duties;
mod node_duties;
mod keys;
mod msg_wrapping;
mod section_querying;

use crate::{
    cmd::{GroupDecision, MessagingDuty},
    messaging::{ClientMessaging, Messaging, Receiver, Received},
    node::{
        node_duties::{NodeDuties, NetworkEvents},
        adult_duties::AdultDuties,
        elder_duties::ElderDuties,
        keys::NodeKeys,
        msg_analysis::{NodeOperation, InboundMsgAnalysis},
    },
    utils, Config, Result,
};
use crossbeam_channel::{Receiver, Select};
use hex_fmt::HexFmt;
use log::{debug, error, info, trace, warn};
use rand::{CryptoRng, Rng, SeedableRng};
use rand_chacha::ChaChaRng;
use routing::{event::Event as RoutingEvent, Node as Routing, TransportEvent as ClientEvent};
use safe_nd::{MsgEnvelope, MsgSender, NodeFullId, XorName};
use std::{
    cell::{Cell, RefCell},
    fmt::{self, Display, Formatter},
    fs,
    net::SocketAddr,
    path::PathBuf,
    rc::Rc,
};

const STATE_FILENAME: &str = "state";

/// Main node struct.
pub struct Node<R: CryptoRng + Rng> {
    id: NodeFullId,
    keys: NodeKeys,
    //root_dir: PathBuf,
    duties: Duties,
    receiver: Receiver,
    routing: Rc<RefCell<Routing>>,
    rng: R,
}

impl<R: CryptoRng + Rng> Node<R> {
    /// Create and start vault. This will block until a `Command` to free it is fired.
    pub fn new(
        routing: Routing,
        receiver: Receiver,
        config: &Config,
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

        let age_based_duties = if is_elder {
            let total_used_space = Rc::new(Cell::new(0));
            let duties = ElderDuties::new(
                keys.clone(),
                &config,
                &total_used_space,
                init_mode,
                routing.clone(),
                ClientMessaging::new(id.public_id().clone(), routing.clone()),
            )?;
            AgeBasedDuties::Elder(duties)
        } else {
            info!("Initializing new node as Infant");
            AgeBasedDuties::Infant
        };

        let duties = Duties::new(
            keys.clone(),
            age_based_duties, 
            routing.clone(), 
            config.clone(),
        );

        let node = Self {
            id,
            keys,
            //root_dir: root_dir.to_path_buf(),
            duties,
            receiver,
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

    fn node_duties(&mut self) -> &mut Duties {
        &mut self.duties
    }

    fn adult_duties(&mut self) -> Option<&mut AdultDuties> {
        use AgeBasedDuties::*;
        match &mut self.duties.age_based {
            Adult(ref mut duties) => Some(duties),
            _ => None,
        }
    }

    fn elder_duties(&mut self) -> Option<&mut ElderDuties> {
        use AgeBasedDuties::*;
        match &mut self.duties.age_based {
            Elder(ref mut duties) => Some(duties),
            _ => None,
        }
    }

    /// Runs the main event loop. Blocks until the node is terminated.
    pub fn run(&mut self) {
        use NodeOperation::*;
        use GatewayDuty::*;
        use NodeDuty::*;
        loop {
            let result = match self.receiver.next() {
                Received::Client(event) => Some(RunAsGateway(ProcessClientEvent(event))),
                Received::Network(event) => Some(RunAsNode(ProcessNetworkEvent(event)))),
                Received::Unknown(channel) => {
                    if let Err(err) = self.routing.borrow_mut().handle_selected_operation(channel.index) {
                        warn!("Could not process operation: {}", err);
                    }
                    None
                }
            };
            self.process_while_any(result);
        }
    }

    fn accumulate_msg(&mut self, msg: &MsgEnvelope) -> Option<MessagingDuty> {
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
            self.handle_remote_msg(&msg.with_proxy(sender))
        } else {
            None
        }
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

    ///
    pub fn process_while_any(&mut self, op: Option<NodeOperation>) {
        use NodeOperation::*;
        
        let mut next_op = op;
        while let Some(op) = next_op {
            next_op = match op {
                Accumulate(msg) => self.node_duties().accumulation().process(&msg),
                RunAsMessaging(duty) => self.node_duties().process(NodeDuty::ProcessMessaging(&duty)),
                RunAsGateway(duty) => self
                    .elder_duties()?
                    .gateway()
                    .process(&duty),
                RunAsPayment(duty) => self.elder_duties()?.data_payment().pay_for_data(&duty),
                RunAsMetadata(duty) => self.elder_duties()?.metadata().receive_msg(&duty),
                RunAsChunks(duty) => self.adult_duties()?.receive_msg(&duty),
                RunAsRewards(duty) => self.elder_duties()?.receive_reward_msg(&duty),
                RunAsTransfers(duty) => self.elder_duties()?.transfers().receive_msg(&duty),
                RunAsElder(duty) => self.elder_duties()?.process(duty),
                RunAsNode(duty) => self.node_duties().process(duty),
                Unknown => {
                    error!("Unknown message destination: {:?}", msg.message.id());
                    None
                }
            }
        }
    }

    ///
    fn vote_for(&mut self, cmd: GroupDecision) -> Option<MessagingDuty> {
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

    // ///
    // fn send(&mut self, outbound: MessagingDuty) -> Option<MessagingDuty> {
    //     use MessagingDuty::*;
    //     match outbound {
    //         SendToClient(msg) => {
    //             if self.msg_analysis.is_dst_for(&msg) {
    //                 self.elder_duties()?.gateway().push_to_client(&msg)
    //             } else {
    //                 Some(SendToSection(msg))
    //             }
    //         }
    //         SendToNode(msg) => self.messaging.borrow_mut().send_to_node(msg),
    //         SendToAdults { targets, msg } => {
    //             self.messaging.borrow_mut().send_to_nodes(targets, &msg)
    //         }
    //         SendToSection(msg) => self.messaging.borrow_mut().send_to_network(msg),
    //         VoteFor(decision) => self.vote_for(decision),
    //     }
    // }

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
