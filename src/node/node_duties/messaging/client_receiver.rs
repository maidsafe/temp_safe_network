// // Copyright 2020 MaidSafe.net limited.
// //
// // This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// // Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// // under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// // KIND, either express or implied. Please review the Licences for the specific language governing
// // permissions and limitations relating to use of the SAFE Network Software.

// use crate::{
//     accumulator::Accumulator,
//     cmd::{GroupDecision, MessagingDuty},
//     messaging::{ClientMessaging, Messaging},
//     node::{
//         adult_duties::AdultDuties,
//         elder_duties::ElderDuties,
//         keys::NodeKeys,
//         msg_analysis::{InboundMsg, InboundMsgAnalysis},
//     },
//     utils, Config, Result,
// };
// use crossbeam_channel::{Receiver, Select};
// use hex_fmt::HexFmt;
// use log::{debug, error, info, trace, warn};
// use rand::{CryptoRng, Rng, SeedableRng};
// use rand_chacha::ChaChaRng;
// use routing::{event::Event as RoutingEvent, Node as Routing, TransportEvent as ClientEvent};
// use safe_nd::{MsgEnvelope, MsgSender, NodeFullId, XorName};
// use std::{
//     cell::{Cell, RefCell},
//     fmt::{self, Display, Formatter},
//     fs,
//     net::SocketAddr,
//     path::PathBuf,
//     rc::Rc,
// };

// struct Receiver {
//     event_receiver: Receiver<RoutingEvent>,
//     client_receiver: Receiver<ClientEvent>,
//     command_receiver: Receiver<Command>,
//     routing: Rc<RefCell<Routing>>,
// }

// pub enum Received {
//     Client(ClientEvent),
//     Network(RoutingEvent),
//     Shutdown,
//     Unknown(ReceivingChannel),
// }

// pub struct ReceivingChannel {
//     pub index: usize,
// }

// impl Receiver {
    
//     pub fn new(event_receiver: Receiver<RoutingEvent>,
//         client_receiver: Receiver<ClientEvent>,
//         command_receiver: Receiver<Command>,
//         routing: Rc<RefCell<Routing>>) -> Self {
//         Self {
//             event_receiver,
//             client_receiver,
//             command_receiver,
//             routing,
//         }
//     }

//     /// Picks up next incoming event.
//     pub fn next(&mut self) -> Option<Received> {
//         let mut sel = Select::new();

//         let mut r_node = self.routing.borrow_mut();
//         r_node.register(&mut sel);
//         let routing_event_rx_idx = sel.recv(&self.event_receiver);
//         let client_network_rx_idx = sel.recv(&self.client_receiver);
//         let command_rx_idx = sel.recv(&self.command_receiver);

//         let selected_operation = sel.ready();
//         drop(r_node);

//         match selected_operation {
//             idx if idx == client_network_rx_idx => {
//                 let event = match self.client_receiver.recv() {
//                     Ok(ev) => ev,
//                     Err(e) => panic!("FIXME: {:?}", e),
//                 };
//                 //self.step_client(event);
//                 Received::Client(event)
//             }
//             idx if idx == routing_event_rx_idx => {
//                 let event = match self.event_receiver.recv() {
//                     Ok(ev) => ev,
//                     Err(e) => panic!("FIXME: {:?}", e),
//                 };
//                 //self.step_routing(event);
//                 Received::Routing(event)
//             }
//             idx if idx == command_rx_idx => {
//                 let command = match self.command_receiver.recv() {
//                     Ok(ev) => ev,
//                     Err(e) => panic!("FIXME: {:?}", e),
//                 };
//                 match command {
//                     Command::Shutdown => Received::Shutdown,
//                 }
//             }
//             index => {
//                 Received::Unknown(ReceivingChannel { index })
//             }
//         }
//     }

// }