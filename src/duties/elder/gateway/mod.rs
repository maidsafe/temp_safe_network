// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod auth;
mod messaging;
mod validation;

use self::{
    auth::{Auth, AuthKeysDb, ClientInfo},
    messaging::Messaging,
    validation::Validation,
};
use crate::{
    cmd::{ConsensusAction, ElderCmd, GatewayCmd},
    msg::Message,
    node::Init,
    Config, Result,
};
use bytes::Bytes;
use log::trace;
use rand::{CryptoRng, Rng};
use routing::Node as Routing;
use safe_nd::{
    ClientAuth, ClientRequest, GatewayRequest, MessageId, NodePublicId, PublicId, Request,
    Response, Signature, SystemOp, XorName,
};
use std::{
    cell::RefCell,
    fmt::{self, Display, Formatter},
    net::SocketAddr,
    rc::Rc,
};

pub(crate) struct Gateway {
    id: NodePublicId,
    messaging: Messaging,
    auth: Auth,
    data: Validation,
}

pub(crate) struct ClientMsg {
    pub client: ClientInfo,
    pub request: ClientRequest,
    pub message_id: MessageId,
    pub signature: Option<Signature>,
}

impl Gateway {
    pub fn new(
        id: NodePublicId,
        config: &Config,
        init_mode: Init,
        routing: Rc<RefCell<Routing>>,
    ) -> Result<Self> {
        let root_dir = config.root_dir()?;
        let root_dir = root_dir.as_path();
        let auth_keys_db = AuthKeysDb::new(root_dir, init_mode)?;

        let messaging = Messaging::new(id.clone(), routing);
        let auth = Auth::new(id.clone(), auth_keys_db);
        let data = Validation::new(id.clone());

        let gateway = Self {
            id,
            messaging,
            auth,
            data,
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
    pub(crate) fn respond_to_client(&mut self, message_id: MessageId, response: Response) {
        self.messaging.respond_to_client(message_id, response);
    }

    // pub fn try_parse_client_msg<R: CryptoRng + Rng>(&mut self,
    //     peer_addr: SocketAddr,
    //     bytes: &Bytes,
    //     rng: &mut R,
    // ) -> Option<ClientMsg> {
    //     self
    //         .messaging
    //         .try_parse_client_msg(peer_addr, bytes, rng)
    // }

    /// Receive client request
    pub fn receive_client_request<R: CryptoRng + Rng>(
        &mut self,
        peer_addr: SocketAddr,
        bytes: &Bytes,
        rng: &mut R,
    ) -> Option<GatewayCmd> {
        let request = self.messaging.try_parse_client_msg(peer_addr, bytes, rng)?;

        let client = request.client.public_id;
        let msg_id = request.message_id;
        let signature = request.signature;
        let request = request.request;

        trace!(
            "{}: Received ({:?} {:?}) from {}",
            self,
            request,
            msg_id,
            client
        );

        if let Some(cmd) = self
            .auth
            .verify_signature(client.clone(), &request, msg_id, signature)
        {
            return Some(cmd);
        };
        if let Some(cmd) = self.auth.authorise_app(&client, &request, msg_id) {
            return Some(cmd);
        }

        match request {
            ClientRequest::System(op) => self.initiate_system_op(op, client, msg_id),
            ClientRequest::Read(read) => self.data.initiate_read(read, client, msg_id),
            ClientRequest::Write {
                write,
                debit_agreement,
            } => self
                .data
                .initiate_write(write, client, msg_id, debit_agreement),
        }
    }

    /// Basically.. when Gateway nodes have agreed,
    /// they'll forward the request into the network.
    pub fn handle_consensused_cmd(&mut self, action: ConsensusAction) -> Option<GatewayCmd> {
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
    pub fn handle_request(
        &mut self,
        client: PublicId,
        request: ClientAuth,
        msg_id: MessageId,
    ) -> Option<ElderCmd> {
        wrap(self.auth.finalise(client, request, msg_id)?)
    }

    pub fn receive_node_response(
        &mut self,
        src: XorName,
        client: &PublicId,
        response: Response,
        msg_id: MessageId,
    ) -> Option<GatewayCmd> {
        self.messaging
            .relay_reponse_to_client(src, client, response, msg_id)
    }

    // pub fn receive_node_msg(&mut self, src: XorName, msg: Message) -> Option<GatewayCmd> {
    //     match msg {
    //         Message::Request {
    //             request,
    //             requester,
    //             message_id,
    //             ..
    //         } => self.finalise_client_request(src, requester, request, message_id),
    //         Message::Response {
    //             response,
    //             requester,
    //             message_id,
    //             ..
    //         } => self
    //             .messaging
    //             .relay_reponse_to_client(src, &requester, response, message_id),
    //         Message::Duplicate { .. } => None,
    //         Message::DuplicationComplete { .. } => None,
    //     }
    // }

    fn initiate_system_op(
        &mut self,
        op: SystemOp,
        client: PublicId,
        msg_id: MessageId,
    ) -> Option<GatewayCmd> {
        use SystemOp::*;
        match op {
            ClientAuth(request) => self.auth.initiate(client, request, msg_id),
            Transfers(request) => Some(GatewayCmd::ForwardClientRequest(Message::Request {
                request: Request::Gateway(GatewayRequest::System(Transfers(request))),
                requester: client,
                message_id: msg_id,
                signature: None,
            })),
        }
    }
}

fn wrap(cmd: GatewayCmd) -> Option<ElderCmd> {
    Some(ElderCmd::Gateway(cmd))
}

impl Display for Gateway {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.id.name())
    }
}
