// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod auth;
mod validation;

use self::{
    auth::{Auth, AuthKeysDb},
    validation::Validation,
};
use crate::{
    cmd::{GroupDecision, MessagingDuty},
    messaging::{ClientMessaging, ClientMsg, Onboarding, InputParsing},
    node::keys::NodeKeys,
    node::msg_wrapping::ElderMsgWrapping,
    node::Init,
    Config, Result,
};
use routing::TransportEvent as ClientEvent;
use bytes::Bytes;
use log::trace;
use rand::CryptoRng;
use safe_nd::{Cmd, ElderDuties, Message, MsgEnvelope, PublicId, Query};
use std::{
    fmt::{self, Display, Formatter},
    net::SocketAddr,
};

pub(crate) struct Gateway {
    keys: NodeKeys,
    auth: Auth,
    data: Validation,
    section: SectionQuerying,
    onboarding: Onboarding,
    input_parsing: InputParsing,
    client_msg_tracking: ClientMsgTracking,
}

impl Gateway {
    pub fn new(
        id: NodePublicId,
        keys: NodeKeys,
        config: &Config,
        init_mode: Init,
        section: SectionQuerying,
    ) -> Result<Self> {
        let root_dir = config.root_dir()?;
        let root_dir = root_dir.as_path();
        let auth_keys_db = AuthKeysDb::new(root_dir, init_mode)?;

        let decisions = ElderMsgWrapping::new(keys.clone(), ElderDuties::Gateway);
        let auth = Auth::new(keys.clone(), auth_keys_db, decisions.clone());
        let data = Validation::new(decisions);

        let onboarding = Onboarding::new(id.clone());
        let input_parsing = InputParsing::new();
        let client_msg_tracking = ClientMsgTracking::new(onboarding);

        let gateway = Self {
            keys,
            auth,
            data,
            section,
            onboarding,
            input_parsing,
            client_msg_tracking,
        };

        Ok(gateway)
    }

    pub fn process(&mut self, cmd: &GatewayDuty) -> Option<NodeOperation> {
        use NodeDuty::*;
        use GatewayDuty::*;
        use NodeOperation::*;
        let result = match cmd {
            ProcessMsg(msg) => self.process_msg(msg),
            ProcessClientEvent(event) => self.process_client_event(event),
            ProcessGroupDecision(decision) => self.process_group_decision(decision),
        };
        result.map(|c| RunAsNode(ProcessMessaging(c)))
    }

    fn process_msg(&mut self, msg: &MsgEnvelope) -> Option<MessagingDuty> {
        use AuthCmd::*;
        if let Address::Client(xorname) = &msg.destination() {
            if self.section.handles(&xorname) {
                self.client_msg_tracking.match_outgoing(msg)
            }
            None
        } else if let Message::Cmd {
            cmd: Cmd::Auth(auth_cmd),
            ..
        } = &msg.message {
             /// Temporary, while Authenticator is not implemented at app layer.
            /// If a request within MessagingDuty::ForwardClientRequest issued by us in `handle_group_decision`
            /// was made by Gateway and destined to our section, this is where the actual request will end up.
            return self.auth.finalise(auth_cmd, msg.id(), &msg.origin),
        } else {
            // so, it wasn't really for Gateway after all
            None
        }
    }

    /// Basically.. when Gateway nodes have agreed,
    /// they'll forward the request into the network.
    fn process_group_decision(&mut self, decision: GroupDecision) -> Option<MessagingDuty> {
        use GroupDecision::*;
        trace!("{}: Group decided on {:?}", self, cmd);
        match decision {
            Forward(msg) => Some(MessagingDuty::SendToSection(msg)),
        }
    }

    /// Will only act on this (i.e. return an action) if node is currently an Elder.
    fn process_client_event(&mut self, event: ClientEvent) -> Option<MessagingDuty> {
        use ClientEvent::*;
        let mut rng = ChaChaRng::from_seed(self.rng.gen());
        match event {
            ConnectedTo { peer } => {
                if !self.onboarding.contains(peer.peer_addr()) {
                    info!("{}: Connected to new client on {}", self, peer_addr);
                }
            }
            ConnectionFailure { peer, .. } => {
                self.onboarding.remove_client(peer.peer_addr());
            }
            NewMessage { peer, msg } => {
                let parsed = self.input_parsing.try_parse_client_msg(peer.peer_addr(), &msg, &mut rng)?;
                match parsed {
                    ClientInput::Msgs(msg) => {
                        let result = self.client_msg_tracking.track_incoming(msg.id, peer.peer_addr());
                        if result.is_some() {
                            return result;
                        }
                    }
                    ClientInput::Handshake(hs) => {
                        self.onboarding.process(hs, peer.peer_addr(), &mut rng)
                    }
                }
                
                return self.process_client_msg(parsed.client.public_id, &parsed.msg);
            }
            SentUserMessage { peer, .. } => {
                trace!(
                    "{}: Succesfully sent Message to: {}",
                    self,
                    peer.peer_addr()
                );
            }
            UnsentUserMessage { peer, .. } => {
                info!("{}: Not sent Message to: {}", self, peer.peer_addr());
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
    fn process_client_msg(
        &mut self,
        client: PublicId,
        msg: &MsgEnvelope,
    ) -> Option<MessagingDuty> {
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

        // pub fn push_to_client(&mut self, msg: &MsgEnvelope) -> Option<MessagingDuty> {
    //     // TODO: Handle this result
    //     let _ = self.messaging.send_to_client(msg);
    //     None
    // }

}

impl Display for Gateway {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.keys.public_key())
    }
}
