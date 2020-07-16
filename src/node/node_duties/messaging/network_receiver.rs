// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{
    accumulator::Accumulator,
    cmd::{GroupDecision, MessagingDuty},
    messaging::{ClientMessaging, Messaging},
    node::{
        adult_duties::AdultDuties,
        elder_duties::ElderDuties,
        keys::NodeKeys,
        msg_analysis::{InboundMsg, InboundMsgAnalysis},
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

struct NetworkReceiver {
    network_receiver: Receiver<RoutingEvent>,
    routing: Rc<RefCell<Routing>>,
}

pub enum Received {
    Client(ClientEvent),
    Network(RoutingEvent),
    Shutdown,
    Unknown(ReceivingChannel),
}

pub struct ReceivingChannel {
    pub index: usize,
}

impl NetworkReceiver {
    
    pub fn new(network_receiver: Receiver<RoutingEvent>,
        routing: Rc<RefCell<Routing>>) -> Self {
        Self {
            network_receiver,
            client_receiver,
            command_receiver,
            routing,
        }
    }

    /// Picks up next incoming event.
    pub fn next(&mut self) -> Option<Received> {
        let mut sel = Select::new();

        let mut r_node = self.routing.borrow_mut();
        r_node.register(&mut sel);
        let routing_event_index = sel.recv(&self.network_receiver);

        let selected_operation = sel.ready();
        drop(r_node);

        match selected_operation {
            index if index == routing_event_index => {
                let event = match self.network_receiver.recv() {
                    Ok(ev) => ev,
                    Err(e) => panic!("FIXME: {:?}", e),
                };
                //self.step_routing(event);
                Received::Routing(event)
            }
            index => {
                Received::Unknown(ReceivingChannel { index })
            }
        }
    }
}