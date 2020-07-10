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
        NodeCmd, ConsensusAction, NodeCmd, GatewayCmd, MetadataCmd, NodeCmd, PaymentCmd,
        TransferCmd,
    },
    duties::{adult::AdultDuties, elder::ElderDuties},
    internal_cmds::InternalCmds,
    messaging::Messaging,
    remote_msg_eval::RemoteMsgEval,
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

pub(crate) struct RemoteMsgEval {
    routing: Rc<RefCell<Routing>>,
}

pub(crate) enum EvalOptions {
    ForwardToNetwork(MsgEnvelope),
    RunAtGateway(MsgEnvelope),
    RunAtPayment(MsgEnvelope),
    AccumulateForMetadata(MsgEnvelope),
    RunAtMetadata(MsgEnvelope),
    AccumulateForAdult(MsgEnvelope),
    RunAtAdult(MsgEnvelope),
    PushToClient(MsgEnvelope),
    RunAtRewards(MsgEnvelope),
    Unknown,
}

impl RemoteMsgEval {
    pub fn new(routing: Rc<RefCell<Routing>>) -> Self {
        Self { routing }
    }

    pub fn evaluate(msg: MsgEnvelope) -> EvalOptions {
        if self.should_forward_to_network(msg) {
            // Any type of msg that is not process locally.
            EvalOptions::ForwardToNetwork(msg)
        } else if self.should_go_to_gateway() {
            // Client auth operations (Temporarily handled here, will be at app layer (Authenticator)).
            // Gateway Elders should just execute and return the result, for client to accumulate.
            EvalOptions::RunAtGateway(msg)
        } else if self.should_go_to_data_payment(msg) {
            // Incoming msg from `Gateway`!
            EvalOptions::RunAtPayment(msg) // Payment Elders should just execute and send onwards.
        } else if self.should_accumulate_for_metadata_write(msg) {
            // Incoming msg from `Payment`!
            EvalOptions::AccumulateForMetadata(msg) // Metadata Elders accumulate the msgs from Payment Elders.
        } else if self.should_go_to_metadata_write(msg) {
            // Accumulated msg from `Payment`!
            EvalOptions::RunAtMetadata(msg)
        } else if self.should_accumulate_for_chunk_write(msg) {
            // Incoming msg from `Metadata`!
            EvalOptions::AccumulateForAdult(msg) // Adults accumulate the msgs from Metadata Elders.
        } else if self.should_go_to_chunk_write(msg) {
            // Accumulated msg from `Metadata`!
            EvalOptions::RunAtAdult(msg)
        } else if self.should_push_to_client(msg) {
            // From network to client!
            EvalOptions::PushToClient(msg)
        } else if self.should_go_to_rewards() {
            EvalOptions::RunAtRewards(msg)
        } else {
            EvalOptions::Unknown
        }
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

    /// We do not accumulate these request, they are executed
    /// at once (i.e. payment carried out) and sent on to 
    /// Metadata section. (They however, will accumulate those msgs.)
    /// The reason for this is that the payment request is already signed
    /// by the client and validated by its replicas, 
    /// so there is no reason to accumulate it here.
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

    /// The individual Payment Elder nodes send their msgs
    /// to Metadata section, where it is accumulated.
    fn should_accumulate_for_metadata_write(&self, msg: MsgEnvelope) -> bool {
        let from_payment_elder = match msg.most_recent_sender() {
            MsgSender::Node {
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
        is_data_cmd && from_payment_elder
    }

    /// After the data write sent from Payment Elders has been 
    /// accumulated (can be seen since the sender is `Section`),
    /// it is time to actually carry out the write operation.
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

    /// Adults accumulate the write requests from Elders.
    fn should_accumulate_for_chunk_write(&self, msg: MsgEnvelope) -> bool {
        let from_metadata_elders = match msg.most_recent_sender() {
            MsgSender::Node {
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

    /// When the write requests from Elders has been accumulated
    /// at an Adult, it is time to carry out the write operation.
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
