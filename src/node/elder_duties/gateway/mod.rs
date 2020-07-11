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
    auth::{Auth, AuthKeysDb, ClientInfo},
    validation::Validation,
};
use crate::{
    cmd::{ConsensusAction, NodeCmd},
    node::Init,
    Config, Messaging, Result,
    keys::NodeKeys,
    msg_decisions::ElderMsgDecisions,
};
use bytes::Bytes;
use log::trace;
use rand::{CryptoRng, Rng};
use routing::Node as Routing;
use safe_nd::{ElderDuty, AuthCmd, ClientAuth, Cmd, Message, MsgEnvelope, NodePublicId, PublicId, Query};
use std::{
    cell::RefCell,
    fmt::{self, Display, Formatter},
    net::SocketAddr,
    rc::Rc,
};

pub(crate) struct Gateway {
    keys: NodeKeys,
    auth: Auth,
    data: Validation,
    messaging: Rc<RefCell<Messaging>>,
}

pub(crate) struct ClientMsg {
    pub client: ClientInfo,
    pub msg: MsgEnvelope,
}

impl Gateway {
    pub fn new(
        keys: NodeKeys,
        config: &Config,
        init_mode: Init,
        routing: Rc<RefCell<Routing>>,
        messaging: Rc<RefCell<Messaging>>,
    ) -> Result<Self> {
        let root_dir = config.root_dir()?;
        let root_dir = root_dir.as_path();
        let auth_keys_db = AuthKeysDb::new(root_dir, init_mode)?;
        let decisions = ElderMsgDecisions::new(keys.clone(), ElderDuty::Gateway);
        let auth = Auth::new(keys.clone(), auth_keys_db, decisions);
        let data = Validation::new(decisions);

        let gateway = Self {
            keys,
            auth,
            data,
            messaging,
        };

        Ok(gateway)
    }

    /// New connection
    pub fn handle_new_connection(&mut self, peer_addr: SocketAddr) {
        self.messaging.handle_new_connection(peer_addr)
    }

    /// Conection failure
    pub fn handle_connection_failure(&mut self, peer_addr: SocketAddr) {
        self.messaging.handle_connection_failure(peer_addr)
    }

    pub fn try_parse_client_msg<R: CryptoRng + Rng>(
        &mut self,
        peer_addr: SocketAddr,
        bytes: &Bytes,
        rng: &mut R,
    ) -> Option<ClientMsg> {
        self.messaging.try_parse_client_msg(peer_addr, bytes, rng)
    }

    pub fn push_to_client(&mut self, msg: MsgEnvelope) -> Option<NodeCmd> {
        self.messaging.send_to_client(msg)
    }

    /// Temporary, while Authenticator is not implemented at app layer.
    /// If a request within NodeCmd::ForwardClientRequest issued by us in `handle_consensused_cmd`
    /// was made by Gateway and destined to our section, this is where the actual request will end up.
    pub fn handle_auth_cmd(
        &mut self,
        _client: PublicId,
        _cmd: AuthCmd,
        msg: MsgEnvelope,
    ) -> Option<NodeCmd> {
        self.auth.finalise(msg)
    }

    /// Basically.. when Gateway nodes have agreed,
    /// they'll forward the request into the network.
    pub fn handle_consensused_cmd(&mut self, cmd: ConsensusAction) -> Option<NodeCmd> {
        use ConsensusAction::*;
        trace!("{}: Consensused {:?}", self, cmd);
        match cmd {
            Forward(msg) => Some(NodeCmd::SendToSection(msg)),
        }
    }

    /// Receive client request
    pub fn handle_client_msg(&mut self, client: PublicId, msg: MsgEnvelope) -> Option<NodeCmd> {
        if let Some(error) = self.auth.verify_client_signature(msg) {
            return Some(error);
        };
        if let Some(error) = self.auth.authorise_app(&client, msg) {
            return Some(error);
        }

        match msg.message {
            Message::Cmd {
                cmd: Cmd::Auth(_),
                id,
                ..
            } => self.auth.initiate(msg),
            Message::Query {
                query: Query::Auth(_),
                id,
                ..
            } => self.auth.list_keys_and_version(msg),
            Message::Cmd {
                cmd: Cmd::Data { cmd, payment },
                id,
                ..
            } => self.data.initiate_write(cmd, msg),
            Message::Query {
                query: Query::Data(data_query),
                id,
                ..
            } => self.data.initiate_read(data_query, msg),
            _ => None, // error..!
        }
    }
}

impl Display for Gateway {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.keys.public_key())
    }
}
