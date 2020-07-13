// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

pub mod client;

use crate::{cmd::OutboundMsg, utils};
pub use client::{ClientInfo, ClientMessaging, ClientMsg};
use log::{error, info};
use routing::{DstLocation, Node as Routing, SrcLocation};
use safe_nd::{Address, MsgEnvelope, XorName};
use std::{cell::RefCell, collections::BTreeSet, rc::Rc};

pub(super) struct Messaging {
    routing: Rc<RefCell<Routing>>,
}

impl Messaging {
    pub fn new(routing: Rc<RefCell<Routing>>) -> Self {
        Self { routing }
    }

    pub fn send(&self, msg: MsgEnvelope) -> Option<OutboundMsg> {
        match msg.destination() {
            Address::Node(_) => self.send_to_node(msg),
            Address::Section(_) => self.send_to_network(msg),
            Address::Client(_) => Some(OutboundMsg::SendToClient(msg)),
        }
    }

    pub fn send_to_nodes(
        &self,
        targets: BTreeSet<XorName>,
        msg: &MsgEnvelope,
    ) -> Option<OutboundMsg> {
        let name = *self.routing.borrow().id().name();
        for target in targets {
            self.routing
                .borrow_mut()
                .send_message(
                    SrcLocation::Node(name),
                    DstLocation::Node(routing::XorName(target.0)),
                    utils::serialise(&msg),
                )
                .map_or_else(
                    |err| {
                        error!("Unable to send MsgEnvelope to Peer: {:?}", err);
                    },
                    |()| {
                        info!("Sent MsgEnvelope to Peer {:?} from node {:?}", target, name);
                    },
                );
        }
        None
    }

    pub fn send_to_node(&self, msg: MsgEnvelope) -> Option<OutboundMsg> {
        let name = *self.routing.borrow().id().name();
        let dst = match msg.destination() {
            Address::Node(xorname) => DstLocation::Node(routing::XorName(xorname.0)),
            Address::Section(_) => return Some(OutboundMsg::SendToSection(msg)),
            Address::Client(_) => return Some(OutboundMsg::SendToClient(msg)),
        };
        self.routing
            .borrow_mut()
            .send_message(SrcLocation::Node(name), dst, utils::serialise(&msg))
            .map_or_else(
                |err| {
                    error!("Unable to send MsgEnvelope to Peer: {:?}", err);
                    None
                },
                |()| {
                    info!("Sent MsgEnvelope to Peer {:?} from node {:?}", dst, name);
                    None
                },
            )
    }

    pub fn send_to_network(&self, msg: MsgEnvelope) -> Option<OutboundMsg> {
        let name = *self.routing.borrow().id().name();
        let dst = match msg.destination() {
            Address::Node(xorname) => DstLocation::Node(routing::XorName(xorname.0)),
            Address::Client(xorname) | Address::Section(xorname) => {
                DstLocation::Section(routing::XorName(xorname.0))
            }
        };
        self.routing
            .borrow_mut()
            .send_message(SrcLocation::Node(name), dst, utils::serialise(&msg))
            .map_or_else(
                |err| {
                    error!("Unable to send to section: {:?}", err);
                    None
                },
                |()| {
                    info!("Sent to section with: {:?}", msg);
                    None
                },
            )
    }
}
