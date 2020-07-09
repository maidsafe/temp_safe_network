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
    cmd::{ConsensusAction, ElderCmd, GatewayCmd},
    msg::Message,
    node::Init,
    Config, Result,
    Messaging,
};
use bytes::Bytes;
use log::trace;
use rand::{CryptoRng, Rng};
use routing::Node as Routing;
use safe_nd::{
    AuthCmd, AuthQuery, ClientAuth, ClientRequest, Cmd, GatewayRequest, Message, MessageId,
    NodePublicId, PublicId, Query, Signature, SystemOp, XorName,
};
use std::{
    cell::RefCell,
    fmt::{self, Display, Formatter},
    net::SocketAddr,
    rc::Rc,
};

pub(crate) struct Gateway {
    id: NodePublicId,
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
        id: NodePublicId,
        config: &Config,
        init_mode: Init,
        routing: Rc<RefCell<Routing>>,
        messaging: Rc<RefCell<Messaging>>,
    ) -> Result<Self> {
        let root_dir = config.root_dir()?;
        let root_dir = root_dir.as_path();
        let auth_keys_db = AuthKeysDb::new(root_dir, init_mode)?;
        let auth = Auth::new(id.clone(), auth_keys_db);
        let data = Validation::new(id.clone());

        let gateway = Self {
            id,
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

    /// Respond to client
    pub(crate) fn send_to_client(&mut self, message_id: MessageId, response: Response) {
        self.messaging.send_to_client(message_id, response);
    }

    pub fn try_parse_client_msg<R: CryptoRng + Rng>(&mut self,
        peer_addr: SocketAddr,
        bytes: &Bytes,
        rng: &mut R,
    ) -> Option<ClientMsg> {
        self
            .messaging
            .try_parse_client_msg(peer_addr, bytes, rng)
    }

    /// Receive client request
    pub fn handle_client_msg(
        &mut self,
        client: PublicId,
        msg: MsgEnvelope,
    ) -> Option<NodeCmd> {
        if let MsgSender::Client { signature, .. } = msg.origin {
            if let Some(cmd) =
                self.auth
                    .verify_signature(client.clone(), &request, msg_id, signature)
            {
                return Some(cmd);
            };
        }

        if let Some(cmd) = self.auth.authorise_app(&client, &request, msg_id) {
            return Some(cmd);
        }

        match msg.message {
            Message::Cmd {
                cmd: Cmd::Auth(auth_cmd),
                id,
                ..
            } => self.auth.cmd(client, auth_cmd, id),
            Message::Query {
                query: Query::Auth(auth_query),
                id,
                ..
            } => self.auth.query(client, auth_query, id),
            Message::Cmd {
                cmd: Cmd::Data { cmd, payment },
                id,
                ..
            } => self.data.initiate_write(cmd, client, id, payment),
            Message::Query {
                query: Query::Data(data_query),
                id,
                ..
            } => self.data.initiate_read(client, data_query, id),
            _ => None, // error..!
        }
    }

    /// Basically.. when Gateway nodes have agreed,
    /// they'll forward the request into the network.
    pub fn handle_consensused_cmd(&mut self, action: ConsensusAction) -> Option<NodeCmd> {
        use ConsensusAction::*;
        trace!("{}: Consensused {:?}", self, action,);
        match action {
            Forward {
                request,
                client_public_id,
                message_id,
            } => Some(GatewayCmd::ForwardClientRequest(Message::Request {
                requester: client_public_id,
                request,
                message_id,
                signature: None,
            })),
        }
    }

    /// If a request within GatewayCmd::ForwardClientRequest issued by us in `handle_consensused_cmd`
    /// was a GatewayRequest destined to our section, this is where the actual request will end up.
    pub fn handle_auth_cmd(
        &mut self,
        client: PublicId,
        cmd: AuthCmd,
        msg_id: MessageId,
    ) -> Option<NodeCmd> {
        wrap(self.auth.finalise(client, cmd, msg_id)?)
    }

    pub fn push_to_client(
        &mut self,
        msg: MsgEnvelope,
    ) -> Option<NodeCmd> {
        self.messaging.send_to_client(msg)
    }
}

fn wrap(cmd: GatewayCmd) -> Option<NodeCmd> {
    Some(ElderCmd::Gateway(cmd))
}

impl Display for Gateway {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.id.name())
    }
}
