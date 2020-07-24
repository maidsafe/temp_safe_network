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
        node_ops::{
            GatewayDuty, GroupDecision, MessagingDuty, NodeDuty, NodeOperation, TransferDuty,
        },
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

pub struct Gateway<R: CryptoRng + Rng> {
    keys: NodeKeys,
    auth: Auth,
    data: Validation,
    section: SectionQuerying,
    client_msg_tracking: ClientMsgTracking,
    rng: R,
}

impl<R: CryptoRng + Rng> Gateway<R> {
    pub fn new(info: NodeInfo, section: SectionQuerying, rng: R) -> Result<Self> {
        let auth_keys_db = AuthKeysDb::new(info.root_dir.clone(), info.init_mode)?;

        let wrapping = ElderMsgWrapping::new(info.keys.clone(), ElderDuties::Gateway);
        let auth = Auth::new(info.keys.clone(), auth_keys_db, wrapping.clone());
        let data = Validation::new(wrapping);

        let onboarding = Onboarding::new(info.public_id().clone(), section.clone());
        let client_msg_tracking = ClientMsgTracking::new(info.public_id().clone(), onboarding);

        let gateway = Self {
            keys: info.keys,
            auth,
            data,
            section,
            client_msg_tracking,
            rng,
        };

        Ok(gateway)
    }

    pub fn process(&mut self, cmd: &GatewayDuty) -> Option<NodeOperation> {
        use GatewayDuty::*;
        match cmd {
            ProcessMsg(msg) => wrap(self.process_msg(msg)),
            ProcessClientEvent(event) => self.process_client_event(event),
            ProcessGroupDecision(decision) => wrap(self.process_group_decision(decision)),
        }
    }

    /// Basically.. when Gateway nodes have voted and agreed,
    /// that this is a valid client request to handle locally,
    /// they'll process it locally.
    fn process_group_decision(&mut self, decision: &GroupDecision) -> Option<MessagingDuty> {
        use GroupDecision::*;
        trace!("{}: Group decided on {:?}", self, decision);
        match decision {
            Process(msg) => self.process_msg(msg),
        }
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

    /// This is where client input is parsed.
    fn process_client_event(&mut self, event: &ClientEvent) -> Option<NodeOperation> {
        use ClientEvent::*;
        match event {
            ConnectedTo { peer } => {
                if !self.client_msg_tracking.contains(peer.peer_addr()) {
                    info!("{}: Connected to new client on {}", self, peer.peer_addr());
                }
            }
            ConnectionFailure { peer, .. } => {
                self.client_msg_tracking.remove_client(peer.peer_addr());
            }
            NewMessage { peer, msg } => {
                let parsed = if self.client_msg_tracking.contains(peer.peer_addr()) {
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
                            return wrap(result);
                        }
                        msg
                    }
                    ClientInput::Handshake(request) => {
                        let mut rng = ChaChaRng::from_seed(self.rng.gen());
                        return wrap(self.client_msg_tracking.process_handshake(
                            request,
                            peer.peer_addr(),
                            &mut rng,
                        ));
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
    fn process_client_msg(&mut self, client: PublicId, msg: &MsgEnvelope) -> Option<NodeOperation> {
        if let Some(error) = self.auth.verify_client_signature(msg) {
            return wrap(Some(error));
        };
        if let Some(error) = self.auth.authorise_app(&client, &msg) {
            return wrap(Some(error));
        }

        match &msg.message {
            Message::Cmd {
                cmd: Cmd::Auth(_), ..
            } => wrap(self.auth.initiate(msg)),
            Message::Query {
                query: Query::Auth(_),
                ..
            } => wrap(self.auth.list_keys_and_version(msg)),
            Message::Cmd {
                cmd: Cmd::Data { cmd, .. },
                ..
            } => wrap(self.data.initiate_write(cmd, msg)),
            Message::Query {
                query: Query::Data(data_query),
                ..
            } => wrap(self.data.initiate_read(data_query, msg)),
            Message::Query {
                query: Query::Transfer(_),
                ..
            }
            | Message::Cmd {
                cmd: Cmd::Transfer(_),
                ..
            } => self.process_transfer(msg),
            _ => None, // error..!
        }
    }

    fn process_transfer(&self, msg: &MsgEnvelope) -> Option<NodeOperation> {
        let duty = match &msg.message {
            Message::Query {
                query: Query::Transfer(query),
                ..
            } => TransferDuty::ProcessQuery {
                query: query.clone().into(),
                msg_id: msg.id(),
                origin: msg.origin.address(),
            },
            Message::Cmd {
                cmd: Cmd::Transfer(cmd),
                ..
            } => TransferDuty::ProcessCmd {
                cmd: cmd.clone().into(),
                msg_id: msg.id(),
                origin: msg.origin.address(),
            },
            _ => return None, // error..!
        };
        Some(NodeOperation::RunAsTransfers(duty))
    }
}

fn wrap(option: Option<MessagingDuty>) -> Option<NodeOperation> {
    use NodeDuty::*;
    use NodeOperation::*;
    option.map(|c| RunAsNode(ProcessMessaging(c)))
}

impl<R: CryptoRng + Rng> Display for Gateway<R> {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.keys.public_key())
    }
}
