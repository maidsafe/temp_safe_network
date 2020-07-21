// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

pub mod state_db;

mod adult_duties;
mod elder_duties;
mod keys;
mod msg_wrapping;
mod node_duties;
mod node_ops;
mod section_querying;

pub use crate::node::state_db::{Command, Init};
use crate::{
    node::{
        keys::NodeKeys,
        node_duties::{
            messaging::{Received, Receiver},
            NodeDuties,
        },
        node_ops::{
            AdultDuty, ElderDuty, GatewayDuty, MetadataDuty, NodeDuty, NodeOperation, PaymentDuty,
            RewardDuty, TransferDuty,
        },
        state_db::{read_state, AgeGroup, NodeInfo},
    },
    utils, Config, Result,
};
use log::warn;
use rand::{CryptoRng, Rng};
use routing::Node as Routing;
use safe_nd::NodeFullId;
use std::{
    cell::RefCell,
    fmt::{self, Display, Formatter},
    net::SocketAddr,
    rc::Rc,
};

/// Main node struct.
pub struct Node<R: CryptoRng + Rng> {
    duties: NodeDuties,
    receiver: Receiver,
    routing: Rc<RefCell<Routing>>,
    rng: R,
}

impl<R: CryptoRng + Rng> Node<R> {
    /// Initialize a new node.
    pub fn new(routing: Routing, receiver: Receiver, config: &Config, mut rng: R) -> Result<Self> {
        let root_dir_buf = config.root_dir()?;
        let root_dir = root_dir_buf.as_path();

        let (age_group, id) = read_state(&root_dir)?.unwrap_or_else(|| {
            let id = NodeFullId::new(&mut rng);
            (Infant, id)
        });

        let routing = Rc::new(RefCell::new(routing));
        let keypair = Rc::new(RefCell::new(utils::key_pair(routing.clone())?));
        let keys = NodeKeys::new(keypair);

        let node_info = NodeInfo {
            id: *id.public_id(),
            keys,
            root_dir: root_dir_buf,
            init_mode: Init::New,
            /// Upper limit in bytes for allowed network storage on this node.
            /// An Adult would be using the space for chunks,
            /// while an Elder uses it for metadata.
            max_storage_capacity: config.max_capacity(),
        };

        let mut duties = NodeDuties::new(id, node_info, routing.clone());

        use AgeGroup::*;
        match age_group {
            Infant => None,
            Adult => duties.process(node_ops::NodeDuty::BecomeAdult),
            Elder => duties.process(node_ops::NodeDuty::BecomeElder),
        };

        let node = Self {
            duties,
            receiver,
            routing,
            rng,
        };

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
        use GatewayDuty::*;
        use NodeDuty::*;
        use NodeOperation::*;
        loop {
            let result = match self.receiver.next() {
                Received::Client(event) => RunAsGateway(ProcessClientEvent(event)),
                Received::Network(event) => RunAsNode(ProcessNetworkEvent(event)),
                Received::Unknown(channel) => {
                    if let Err(err) = self
                        .routing
                        .borrow_mut()
                        .handle_selected_operation(channel.index)
                    {
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
        self.duties.elder_duties()?.rewards().process(duty)
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
        write!(formatter, "{}", self.duties.id())
    }
}
