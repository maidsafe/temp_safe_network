// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod auth;
mod data_requests;
mod login_packets;
mod messaging;

use self::{
    auth::{Auth, AuthKeysDb},
    data_requests::Validation,
    login_packets::LoginPackets,
    messaging::Messaging,
};
use crate::{
    action::{Action, ConsensusAction},
    chunk_store::LoginPacketChunkStore,
    node::Init,
    rpc::Rpc,
    Config, Result,
};
use bytes::Bytes;
use log::{error, trace};
use rand::{CryptoRng, Rng};
use routing::Node as Routing;
use safe_nd::{
    ClientRequest, MessageId, NodePublicId, NodeRequest, PublicId, Read, Request, Response,
    Signature, SystemOp, Write, XorName,
};
use std::{
    cell::{Cell, RefCell},
    fmt::{self, Display, Formatter},
    net::SocketAddr,
    rc::Rc,
};

pub(crate) struct ClientHandler {
    id: NodePublicId,
    messaging: Messaging,

    auth: Auth,
    login_packets: LoginPackets,
    data: Validation,
}

impl ClientHandler {
    pub fn new(
        id: NodePublicId,
        config: &Config,
        total_used_space: &Rc<Cell<u64>>,
        init_mode: Init,
        routing: Rc<RefCell<Routing>>,
    ) -> Result<Self> {
        let root_dir = config.root_dir()?;
        let root_dir = root_dir.as_path();
        let auth_keys_db = AuthKeysDb::new(root_dir, init_mode)?;
        let login_packets_db = LoginPacketChunkStore::new(
            root_dir,
            config.max_capacity(),
            Rc::clone(&total_used_space),
            init_mode,
        )?;

        let messaging = Messaging::new(id.clone(), routing.clone());
        let auth = Auth::new(id.clone(), auth_keys_db);
        let login_packets = LoginPackets::new(id.clone(), login_packets_db);
        let data = Validation::new(id.clone());

        let client_handler = Self {
            id,
            messaging,
            auth,
            login_packets,
            data,
        };

        Ok(client_handler)
    }

    pub(crate) fn respond_to_client(&mut self, message_id: MessageId, response: Response) {
        self.messaging.respond_to_client(message_id, response);
    }

    pub fn handle_new_connection(&mut self, peer_addr: SocketAddr) {
        self.messaging.handle_new_connection(peer_addr)
    }

    pub fn handle_connection_failure(&mut self, peer_addr: SocketAddr) {
        self.messaging.handle_connection_failure(peer_addr)
    }

    pub fn receive_node_msg(&mut self, src: XorName, rpc: Rpc) -> Option<Action> {
        match rpc {
            Rpc::Request {
                request,
                requester,
                message_id,
                ..
            } => self.finalise_client_request(src, requester, request, message_id),
            Rpc::Response {
                response,
                requester,
                message_id,
                refund: _,
                ..
            } => self
                .messaging
                .relay_reponse_to_client(src, &requester, response, message_id),
            Rpc::Duplicate { .. } => None,
            Rpc::DuplicationComplete { .. } => None,
        }
    }

    /// Basically.. when Gateway nodes have agreed,
    /// they'll forward the request into the network.
    pub fn handle_consensused_action(&mut self, action: ConsensusAction) -> Option<Action> {
        use ConsensusAction::*;
        trace!("{}: Consensused {:?}", self, action,);
        match action {
            Forward {
                request,
                client_public_id,
                message_id,
            } => Some(Action::ForwardClientRequest(Rpc::Request {
                requester: client_public_id,
                request,
                message_id,
                signature: None,
            })),
        }
    }

    pub fn handle_client_message<R: CryptoRng + Rng>(
        &mut self,
        peer_addr: SocketAddr,
        bytes: &Bytes,
        rng: &mut R,
    ) -> Option<Action> {
        let result = self
            .messaging
            .try_parse_client_request(peer_addr, bytes, rng);
        if let Some(result) = result {
            self.process_client_request(
                result.client.public_id,
                result.request,
                result.message_id,
                result.signature,
            )
        } else {
            None
        }
    }

    // on client request
    fn process_client_request(
        &mut self,
        client: PublicId,
        request: Request,
        msg_id: MessageId,
        signature: Option<Signature>,
    ) -> Option<Action> {
        trace!(
            "{}: Received ({:?} {:?}) from {}",
            self,
            request,
            msg_id,
            client.clone()
        );

        if let Some(action) =
            self.auth
                .verify_signature(client.clone(), &request, msg_id, signature)
        {
            return Some(action);
        };
        if let Some(action) = self.auth.authorise_app(&client, &request, msg_id) {
            return Some(action);
        }

        if let Request::Client(client_request) = request {
            match client_request {
                // Temporary
                ClientRequest::Write {
                    write: Write::Account(write),
                    debit_agreement,
                } => self
                    .login_packets
                    .initiate_write(client, write, msg_id, debit_agreement),
                // Temporary
                ClientRequest::Read(Read::Account(read)) => {
                    self.login_packets.read(client, read, msg_id)
                }
                ClientRequest::Read(read) => self.data.initiate_read(read, client, msg_id),
                ClientRequest::Write {
                    write,
                    debit_agreement,
                } => self
                    .data
                    .initiate_write(write, client, msg_id, debit_agreement),
                ClientRequest::System(op) => self.initiate_system_op(op, client, msg_id),
            }
        } else {
            None
        }
    }

    fn initiate_system_op(
        &mut self,
        op: SystemOp,
        client: PublicId,
        msg_id: MessageId,
    ) -> Option<Action> {
        use SystemOp::*;
        match op {
            Transfers(request) => Some(Action::ForwardClientRequest(Rpc::Request {
                request: Request::Node(NodeRequest::System(Transfers(request))),
                requester: client,
                message_id: msg_id,
                signature: None,
            })),
            ClientAuth(request) => self.auth.initiate(client, request, msg_id),
        }
    }

    fn finalise_system_op(
        &mut self,
        op: SystemOp,
        client: PublicId,
        msg_id: MessageId,
    ) -> Option<Action> {
        use SystemOp::*;
        match op {
            Transfers(_) => {
                error!(
                    "{}: Should not be handled at ClientHandler!! Received ({:?} {:?})(client {:?})",
                    self,
                    &op,
                    msg_id,
                    client
                );
                None
            }
            ClientAuth(request) => self.auth.finalise(client, request, msg_id),
        }
    }

    // on consensus
    fn finalise_client_request(
        &mut self,
        src: XorName,
        requester: PublicId,
        request: Request,
        msg_id: MessageId,
    ) -> Option<Action> {
        trace!(
            "{}: Received ({:?} {:?}) from src {} (client {:?})",
            self,
            &request,
            msg_id,
            src,
            requester
        );

        if let Request::Node(node_request) = request {
            match node_request {
                NodeRequest::System(op) => self.finalise_system_op(op, requester, msg_id),
                // Temporary
                NodeRequest::Write(Write::Account(write)) => {
                    self.login_packets.finalise_write(requester, write, msg_id)
                }
                // Temporary
                NodeRequest::Read(Read::Account(read)) => {
                    self.login_packets.read(requester, read, msg_id)
                }
                NodeRequest::Read(_) | NodeRequest::Write(_) => {
                    // error!(
                    //     "{}: Should not receive {:?} as a client handler.",
                    //     self, request
                    // );
                    None
                }
            }
        } else {
            None
        }
    }
}

impl Display for ClientHandler {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.id.name())
    }
}
