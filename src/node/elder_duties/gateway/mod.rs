// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod auth;
mod client;
mod validation;

use self::{
    auth::{Auth, AuthKeysDb},
    validation::Validation,
};
use crate::{
    node::elder_duties::gateway::client::{
        try_deserialize_handshake, try_deserialize_msg, ClientInput, ClientMsgTracking, Onboarding,
    },
    node::keys::NodeKeys,
    node::msg_wrapping::ElderMsgWrapping,
    node::state_db::NodeInfo,
    node::{
        node_ops::{GatewayDuty, GroupDecision, MessagingDuty, NodeDuty, NodeOperation},
        section_querying::SectionQuerying,
    },
    Result,
};
use log::{error, info, trace};
use rand::{CryptoRng, Rng, SeedableRng};
use rand_chacha::ChaChaRng;
use routing::TransportEvent as ClientEvent;
use safe_nd::{Address, Cmd, ElderDuties, Message, MsgEnvelope, PublicId, Query};
use std::fmt::{self, Display, Formatter};

pub(crate) struct Gateway<R: CryptoRng + Rng> {
    keys: NodeKeys,
    auth: Auth,
    data: Validation,
    section: SectionQuerying,
    onboarding: Onboarding,
    client_msg_tracking: ClientMsgTracking,
    rng: R,
}

impl<R: CryptoRng + Rng> Gateway<R> {
    pub fn new(info: NodeInfo, section: SectionQuerying, rng: R) -> Result<Self> {
        let auth_keys_db = AuthKeysDb::new(info.root_dir, info.init_mode)?;

        let wrapping = ElderMsgWrapping::new(info.keys.clone(), ElderDuties::Gateway);
        let auth = Auth::new(info.keys.clone(), auth_keys_db, wrapping.clone());
        let data = Validation::new(wrapping);

        let onboarding = Onboarding::new(info.id.clone(), section);
        let client_msg_tracking = ClientMsgTracking::new(info.id, onboarding);

        let gateway = Self {
            keys: info.keys,
            auth,
            data,
            section,
            onboarding,
            client_msg_tracking,
            rng,
        };

        Ok(gateway)
    }

    pub fn process(&mut self, cmd: &GatewayDuty) -> Option<NodeOperation> {
        use GatewayDuty::*;
        use NodeDuty::*;
        use NodeOperation::*;
        let result = match cmd {
            ProcessMsg(msg) => self.process_msg(msg),
            ProcessClientEvent(event) => self.process_client_event(event),
            ProcessGroupDecision(decision) => self.process_group_decision(decision),
        };
        result.map(|c| RunAsNode(ProcessMessaging(c)))
    }

    fn process_msg(&mut self, msg: &MsgEnvelope) -> Option<MessagingDuty> {
        if let Address::Client(xorname) = &msg.destination() {
            if self.section.handles(&xorname) {
                return self.client_msg_tracking.match_outgoing(msg);
            }
            None
        } else if let Message::Cmd {
            cmd: Cmd::Auth(auth_cmd),
            ..
        } = &msg.message
        {
            // Temporary, while Authenticator is not implemented at app layer.
            // If a request within MessagingDuty::ForwardClientRequest issued by us in `handle_group_decision`
            // was made by Gateway and destined to our section, this is where the actual request will end up.
            return self.auth.finalise(auth_cmd, msg.id(), &msg.origin);
        } else {
            // so, it wasn't really for Gateway after all
            None
        }
    }

    /// Basically.. when Gateway nodes have agreed,
    /// they'll forward the request into the network.
    fn process_group_decision(&mut self, decision: &GroupDecision) -> Option<MessagingDuty> {
        use GroupDecision::*;
        trace!("{}: Group decided on {:?}", self, decision);
        match decision {
            Forward(msg) => Some(MessagingDuty::SendToSection(msg.clone())),
        }
    }

    /// This is where client input is parsed.
    fn process_client_event(&mut self, event: &ClientEvent) -> Option<MessagingDuty> {
        use ClientEvent::*;
        let mut rng = ChaChaRng::from_seed(self.rng.gen());
        match event {
            ConnectedTo { peer } => {
                if !self.onboarding.contains(peer.peer_addr()) {
                    info!("{}: Connected to new client on {}", self, peer.peer_addr());
                }
            }
            ConnectionFailure { peer, .. } => {
                self.onboarding.remove_client(peer.peer_addr());
            }
            NewMessage { peer, msg } => {
                let parsed = if self.onboarding.contains(peer.peer_addr()) {
                    try_deserialize_msg(msg)
                } else {
                    try_deserialize_handshake(msg, peer.peer_addr())
                }?;
                let parsed = match parsed {
                    ClientInput::Msg(msg) => {
                        let result = self
                            .client_msg_tracking
                            .track_incoming(msg.id(), peer.peer_addr());
                        if result.is_some() {
                            return result;
                        }
                        msg
                    }
                    ClientInput::Handshake(request) => {
                        return self.onboarding.process(request, peer.peer_addr(), &mut rng);
                    }
                };

                return self.process_client_msg(parsed.public_id, &parsed.msg);
            }
            SentUserMessage { peer, .. } => {
                trace!(
                    "{}: Succesfully sent Message to: {}",
                    self,
                    peer.peer_addr()
                );
            }
            UnsentUserMessage { peer, .. } => {
                info!("{}: Could not send message to: {}", self, peer.peer_addr());
            }
            BootstrapFailure | BootstrappedTo { .. } => {
                error!("Unexpected bootstrapping client event")
            }
            Finish => {
                info!("{}: Received Finish event", self);
            }
        }
        None
    }

    /// Process a msg from a client.
    fn process_client_msg(&mut self, client: PublicId, msg: &MsgEnvelope) -> Option<MessagingDuty> {
        if let Some(error) = self.auth.verify_client_signature(msg) {
            return Some(error);
        };
        if let Some(error) = self.auth.authorise_app(&client, &msg) {
            return Some(error);
        }

        match &msg.message {
            Message::Cmd {
                cmd: Cmd::Auth(_), ..
            } => self.auth.initiate(msg),
            Message::Query {
                query: Query::Auth(_),
                ..
            } => self.auth.list_keys_and_version(msg),
            Message::Cmd {
                cmd: Cmd::Data { cmd, .. },
                ..
            } => self.data.initiate_write(cmd, msg),
            Message::Query {
                query: Query::Data(data_query),
                ..
            } => self.data.initiate_read(data_query, msg),
            _ => None, // error..!
        }
    }
}

impl<R: CryptoRng + Rng> Display for Gateway<R> {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.keys.public_key())
    }
}
