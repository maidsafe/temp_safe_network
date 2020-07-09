// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod internal_cmds;
mod messaging;
mod remote_msg_eval;

use crate::{
    accumulator::Accumulator,
    cmd::{
        AdultCmd, ConsensusAction, ElderCmd, GatewayCmd, MetadataCmd, NodeCmd, PaymentCmd,
        TransferCmd,
    },
    duties::{adult::AdultDuties, elder::ElderDuties},
    internal_cmds::InternalCmds,
    messaging::Messaging,
    remote_msg_eval::RemoteMsgEvaluation,
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

pub(super) struct RemoteMsgEvaluation {
    routing: Rc<RefCell<Routing>>,
}

impl RemoteMsgEvaluation {
    pub fn new(routing: Rc<RefCell<Routing>>) -> Self {
        Self { routing }
    }

    fn should_forward_to_network(&self, msg: MsgEnvelope) -> bool {
        use Address::*;
        let destined_for_network = match msg.destination() {
            Client(_) => false,
            Node(address) => routing::XorName(address.0) != *self.routing.borrow().id().name(),
            Section(address) => !self.self_is_handler_for(&address),
        };
        let from_client = match msg.most_recent_sender() {
            MsgSender::Client { .. } => true,
            _ => false,
        };
        let is_auth_cmd = match msg.message {
            Message::Cmd {
                cmd: Cmd::Auth { .. },
                ..
            } => true,
            _ => false,
        };
        destined_for_network || (from_client && !is_auth_cmd)
    }

    fn should_go_to_gateway(&self, msg: MsgEnvelope) -> bool {
        let from_client = match msg.most_recent_sender() {
            MsgSender::Client { .. } => true,
            _ => false,
        };
        let is_auth_cmd = match msg.message {
            Message::Cmd {
                cmd: Cmd::Auth { .. },
                ..
            } => true,
            _ => false,
        };
        from_client && is_auth_cmd
    }

    fn should_go_to_data_payment(&self, msg: MsgEnvelope) -> bool {
        let from_gateway_elders = match msg.most_recent_sender() {
            MsgSender::Node {
                duty: Duty::Elder(ElderDuty::Gateway),
                ..
            } => true,
            _ => false,
        };
        let is_data_cmd = match msg.message {
            Message::Cmd {
                cmd: Cmd::Data { .. },
                ..
            } => true,
            _ => false,
        };
        is_data_cmd && from_gateway_elders
    }

    fn should_go_to_metadata_write(&self, msg: MsgEnvelope) -> bool {
        let from_payment_elders = match msg.most_recent_sender() {
            MsgSender::Section {
                duty: Duty::Elder(ElderDuty::Payment),
                ..
            } => true,
            _ => false,
        };
        let is_data_cmd = match msg.message {
            Message::Cmd {
                cmd: Cmd::Data { .. },
                ..
            } => true,
            _ => false,
        };
        is_data_cmd && from_payment_elders
    }

    fn should_go_to_chunk_write(&self, msg: MsgEnvelope) -> bool {
        let from_metadata_elders = match msg.most_recent_sender() {
            MsgSender::Section {
                duty: Duty::Elder(ElderDuty::Metadata),
                ..
            } => true,
            _ => false,
        };
        let is_data_cmd = match msg.message {
            Message::Cmd {
                cmd:
                    Cmd::Data {
                        cmd: DataCmd::Blob(_),
                        ..
                    },
                ..
            } => true,
            _ => false,
        };
        is_data_cmd && from_metadata_elders
    }

    fn should_push_to_client(&self, msg: MsgEnvelope) -> bool {
        match msg.destination() {
            Address::Client(xorname) => self.self_is_handler_for(&xorname),
            _ => false,
        }
    }

    pub fn self_is_handler_for(&self, address: &XorName) -> bool {
        let xorname = routing::XorName(address.0);
        match self.routing.borrow().matches_our_prefix(&xorname) {
            Ok(result) => result,
            _ => false,
        }
    }
}
