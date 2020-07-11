// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{
    accumulator::Accumulator,
    cmd::{NodeCmd, ConsensusAction},
    duties::{adult::AdultDuties, elder::ElderDuties},
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
    Address, Cmd, DataCmd, Duty, ElderDuty, Message, MsgEnvelope, MsgSender, NodeFullId, Query,
    XorName,
};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeSet,
    fmt::{self, Display, Formatter},
    fs,
    net::SocketAddr,
    path::PathBuf,
    rc::Rc,
};
use threshold_crypto::Signature;

pub(super) struct Messaging {
    routing: Rc<RefCell<Routing>>,
}

impl Messaging {
    pub fn new(routing: Rc<RefCell<Routing>>) -> Self {
        Self { routing }
    }

    pub fn send(&self, msg: MsgEnvelope) -> Option<NodeCmd> {
        let name = *self.routing.borrow().id().name();
        let dst = match msg.destination() {
            Address::Node(xorname) => DstLocation::Node(xorname.0),
            Address::Client(xorname) => DstLocation::Client(xorname.0), 
            Address::Section(xorname) => DstLocation::Section(xorname.0),
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

    pub fn send_to_nodes(
        &mut self,
        msgs: BTreeSet<MsgEnvelope>,
    ) -> Option<NodeCmd> {
        for msg in msgs {
            self.send_to_node(msg);
        }
        None
    }

    pub fn send_to_node(&self, msg: MsgEnvelope) {
        let name = *self.routing.borrow().id().name();
        let dst = match msg.destination() {
            Address::Node(xorname) => DstLocation::Node(xorname.0),
            _ => return,
        };
        self.routing
            .borrow_mut()
            .send_message(
                SrcLocation::Node(name),
                dst,
                utils::serialise(&msg),
            )
            .map_or_else(
                |err| {
                    error!("Unable to send MsgEnvelope to Peer: {:?}", err);
                    //None
                },
                |()| {
                    info!("Sent MsgEnvelope to Peer {:?} from node {:?}", target, name);
                    //None
                },
            )
    }

    pub fn send_to_network(&self, msg: MsgEnvelope) -> Option<NodeCmd> {
        let name = *self.routing.borrow().id().name();
        let dst = match msg.destination() {
            Address::Node(xorname) => DstLocation::Node(xorname.0),
            Address::Client(xorname) | Address::Section(xorname) => DstLocation::Section(xorname.0),
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

    // fn forward_client_request(&mut self, msg: MsgEnvelope) -> Option<NodeCmd> {
    //     trace!("{} received a client request {:?}", self, msg);
    //     let msg_clone = msg.clone();
    //     let dst_address = if let MsgEnvelope::Request { ref request, .. } = msg_clone {
    //         match request.dst_address() {
    //             Some(address) => address,
    //             None => {
    //                 error!("{}: Logic error - no data handler address available.", self);
    //                 return None;
    //             }
    //         }
    //     } else {
    //         error!(
    //             "{}: Logic error - expected Request, but got something else.",
    //             self
    //         );
    //         return None;
    //     };

    //     self.forward_to_section(&dst_address, msg)
    // }
}
