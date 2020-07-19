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
mod node_ops;
mod state_db;

pub use crate::node::state_db::{Command, Init};
use crate::{
    node::{
        node_ops::{
            GroupDecision, MessagingDuty, NodeDuty, NodeOperation, GatewayDuty, PaymentDuty,
            MetadataDuty, ChunkDuty, RewardDuty, TransferDuty, ElderDuty, AdultDuty,
        },
        node_duties::{NodeDuties, AgeLevel, messaging::{Receiver, Received}},
        adult_duties::AdultDuties,
        elder_duties::ElderDuties,
        keys::NodeKeys,
        state_db::{dump_state, read_state},
    },
    utils, Config, Result,
};
use log::{error, info, warn};
use rand::{CryptoRng, Rng};
use routing::Node as Routing;
use safe_nd::NodeFullId;
use std::{
    cell::{Cell, RefCell},
    fmt::{self, Display, Formatter},
    fs,
    net::SocketAddr,
    rc::Rc,
};

/// Main node struct.
pub struct Node<R: CryptoRng + Rng> {
    id: NodeFullId,
    keys: NodeKeys,
    //root_dir: PathBuf,
    duties: NodeDuties,
    receiver: Receiver,
    routing: Rc<RefCell<Routing>>,
    rng: R,
}

impl<R: CryptoRng + Rng> Node<R> {
    /// Initialize a new node.
    pub fn new(
        routing: Routing,
        receiver: Receiver,
        config: &Config,
        mut rng: R,
    ) -> Result<Self> {
        let mut init_mode = Init::Load;

        let (is_elder, id) = read_state(&config)?.unwrap_or_else(|| {
            let id = NodeFullId::new(&mut rng);
            init_mode = Init::New;
            (false, id)
        });

        let root_dir = config.root_dir()?;
        let root_dir = root_dir.as_path();

        let routing = Rc::new(RefCell::new(routing));
        let keypair = Rc::new(RefCell::new(utils::key_pair(routing.clone())?));
        let keys = NodeKeys::new(keypair);

        let age_level = if is_elder {
            let total_used_space = Rc::new(Cell::new(0));
            let duties = ElderDuties::new(
                id.public_id().clone(),
                keys.clone(),
                &root_dir,
                &total_used_space,
                init_mode,
                routing.clone(),
            )?;
            AgeLevel::Elder(duties)
        } else {
            info!("Initializing new node as Infant");
            AgeLevel::Infant
        };

        let duties = NodeDuties::new(
            keys.clone(),
            age_level, 
            routing.clone(), 
            config.clone(),
        );

        let node = Self {
            id,
            keys,
            duties,
            receiver,
            routing,
            rng,
        };

        let is_elder = matches!(duties, AgeLevel::Elder { .. });
        dump_state(is_elder, root_dir, id)?;

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

    /// Starts the node, and runs the main event loop.
    /// Blocks until the node is terminated, which is done
    /// by client sending in a `Command` to free it.
    pub fn run(&mut self) {
        use NodeOperation::*;
        use GatewayDuty::*;
        use NodeDuty::*;
        loop {
            let result = match self.receiver.next() {
                Received::Client(event) => RunAsGateway(ProcessClientEvent(event)),
                Received::Network(event) => RunAsNode(ProcessNetworkEvent(event)),
                Received::Unknown(channel) => {
                    if let Err(err) = self.routing.borrow_mut().handle_selected_operation(channel.index) {
                        warn!("Could not process operation: {}", err);
                    }
                    continue;
                }
            };
            self.process_while_any(Some(result));
        }
    }

    ///
    pub fn process_while_any(&mut self, op: Option<NodeOperation>) {
        use NodeOperation::*;
        
        let mut next_op = op;
        while let Some(op) = next_op {
            next_op = match op {
                RunAsGateway(duty) => self.run_as_gateway(duty),
                RunAsPayment(duty) => self.run_as_payment(duty),
                RunAsTransfers(duty) => self.run_as_transfers(duty),
                RunAsMetadata(duty) => self.run_as_metadata(duty),
                RunAsRewards(duty) => self.run_as_rewards(duty),
                RunAsAdult(duty) => self.run_as_adult(duty),
                RunAsElder(duty) => self.run_as_elder(duty),
                RunAsNode(duty) => self.run_as_node(duty),
                Unknown => None,
            }
        }
    }

    fn run_as_gateway(&mut self, duty: GatewayDuty) -> Option<NodeOperation> {
        self.duties.elder_duties()?.gateway().process(&duty)
    }

    fn run_as_payment(&mut self, duty: PaymentDuty) -> Option<NodeOperation> {
        self.duties.elder_duties()?.data_payment().process(&duty)
    }

    fn run_as_transfers(&mut self, duty: TransferDuty) -> Option<NodeOperation> {
        self.duties.elder_duties()?.transfers().process(&duty)
    }
    
    fn run_as_metadata(&mut self, duty: MetadataDuty) -> Option<NodeOperation> {
        self.duties.elder_duties()?.metadata().process(&duty)
    }

    fn run_as_rewards(&mut self, duty: RewardDuty) -> Option<NodeOperation> {
        self.duties.elder_duties()?.rewards().process(&duty)
    }

    fn run_as_node(&mut self, duty: NodeDuty) -> Option<NodeOperation> {
        self.duties.process(duty)
    }
    
    fn run_as_elder(&mut self, duty: ElderDuty) -> Option<NodeOperation> {
        self.duties.elder_duties()?.process(duty)
    }

    fn run_as_adult(&mut self, duty: AdultDuty) -> Option<NodeOperation> {
        // if let Some(duties) = self.adult_duties() {
        //     duties.process(&duty)
        // } else {
        //     error!("Invalid message assignment: {:?}", duty);
        //     None
        // }
        self.duties.adult_duties()?.process(&duty)
    }
}

impl<R: CryptoRng + Rng> Display for Node<R> {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.id.public_id())
    }
}
