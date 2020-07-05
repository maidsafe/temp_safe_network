// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod gateway;
mod metadata;
mod payment;
mod transfers;

use self::{
    gateway::ClientHandler,
    metadata::Metadata,
    payment::DataPayment,
    transfers::{replica_manager::ReplicaManager, Transfers},
};
use crate::{
    cmd::{ConsensusAction, ElderCmd, GatewayCmd, NodeCmd},
    msg::Message,
    node::Init,
    utils, Config, Result,
};
use bytes::Bytes;
use log::{error, trace};
use rand::{CryptoRng, Rng};
use routing::{Node as Routing, RoutingError, SrcLocation};
use safe_nd::{
    GatewayRequest, MessageId, NodePublicId, NodeRequest, PublicId, Request, Response, SystemOp,
    XorName,
};
use std::{
    cell::{Cell, RefCell},
    fmt::{self, Display, Formatter},
    net::SocketAddr,
    rc::Rc,
};
use threshold_crypto::{PublicKey, Signature};

pub(crate) struct ElderDuties {
    id: NodePublicId,
    metadata: Metadata,
    transfers: Transfers,
    gateway: ClientHandler,
    data_payment: DataPayment,
    routing: Rc<RefCell<Routing>>,
}

fn map_gw_cmd(cmd: Option<GatewayCmd>) -> Option<NodeCmd> {
    Some(NodeCmd::Gateway(cmd?))
}

fn map_e_cmd(cmd: Option<ElderCmd>) -> Option<NodeCmd> {
    Some(NodeCmd::Elder(cmd?))
}

impl ElderDuties {
    pub fn new(
        id: NodePublicId,
        config: &Config,
        total_used_space: &Rc<Cell<u64>>,
        init_mode: Init,
        routing: Rc<RefCell<Routing>>,
    ) -> Result<Self> {
        let gateway = ClientHandler::new(
            id.clone(),
            &config,
            &total_used_space,
            init_mode,
            routing.clone(),
        )?;

        let metadata = Metadata::new(
            id.clone(),
            &config,
            &total_used_space,
            init_mode,
            routing.clone(),
        )?;

        let node = routing.borrow();
        let public_key_set = node.public_key_set()?;
        let secret_key_share = node.secret_key_share()?;
        let key_index = node.our_index()?;
        let proof_chain = node.our_history().ok_or(RoutingError::InvalidState)?;
        let replica_manager = ReplicaManager::new(
            secret_key_share,
            key_index,
            public_key_set,
            vec![],
            proof_chain.clone(),
        )?;

        let replica_manager = Rc::new(RefCell::new(replica_manager));
        let transfers = Transfers::new(id.clone(), replica_manager.clone());
        let data_payment = DataPayment::new(id.clone(), replica_manager);

        Ok(Self {
            id,
            gateway,
            metadata,
            transfers,
            data_payment,
            routing: routing.clone(),
        })
    }

    // -------------------------------------------------------------
    // ---------  iffy placing of gateway methods here...  ---------
    // vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv

    pub fn handle_client_message<R: CryptoRng + Rng>(
        &mut self,
        peer_addr: SocketAddr,
        bytes: &Bytes,
        rng: &mut R,
    ) -> Option<NodeCmd> {
        map_gw_cmd(self.gateway.handle_client_message(peer_addr, bytes, rng))
    }

    pub fn respond_to_gateway(&mut self, src: XorName, msg: Message) -> Option<NodeCmd> {
        map_gw_cmd(self.gateway.receive_node_msg(src, msg))
    }

    pub(crate) fn respond_to_client(&mut self, message_id: MessageId, response: Response) {
        self.gateway.respond_to_client(message_id, response);
    }

    pub fn handle_new_connection(&mut self, peer_addr: SocketAddr) {
        self.gateway.handle_new_connection(peer_addr)
    }

    pub fn handle_connection_failure(&mut self, peer_addr: SocketAddr) {
        self.gateway.handle_connection_failure(peer_addr)
    }

    pub fn handle_consensused_cmd(&mut self, action: ConsensusAction) -> Option<NodeCmd> {
        map_gw_cmd(self.gateway.handle_consensused_cmd(action))
    }

    // ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    // ---------  iffy placing of gateway methods here... ----------
    // -------------------------------------------------------------

    /// Name of the node
    /// Age of the node
    pub fn member_left(&mut self, name: XorName, _age: u8) -> Option<Vec<ElderCmd>> {
        self.metadata.trigger_chunk_duplication(XorName(name.0))
    }

    // Update our replica with the latest keys
    pub fn elders_changed(&mut self) -> Option<ElderCmd> {
        let pub_key_set = self.routing.borrow().public_key_set().ok()?.clone();
        let sec_key_share = self.routing.borrow().secret_key_share().ok()?.clone();
        let proof_chain = self.routing.borrow().our_history()?.clone();
        let our_index = self.routing.borrow().our_index().ok()?;
        self.transfers.update_replica_on_churn(
            pub_key_set,
            sec_key_share,
            our_index,
            proof_chain,
        )?;
        None
    }

    pub fn receive_msg(
        &mut self,
        src: SrcLocation,
        msg: Message,
        accumulated_signature: Option<Signature>,
    ) -> Option<ElderCmd> {
        match msg {
            Message::Request {
                request,
                requester,
                message_id,
                ..
            } => self.handle_request(src, requester, request, message_id, accumulated_signature),
            Message::Response {
                response,
                requester,
                message_id,
                proof,
                ..
            } => self.handle_response(src, response, requester, message_id, proof),
            Message::Duplicate { .. } | Message::DuplicationComplete { .. } => {
                self.metadata.receive_msg(src, msg, accumulated_signature)
            }
        }
    }

    fn handle_request(
        &mut self,
        src: SrcLocation,
        requester: PublicId,
        request: Request,
        message_id: MessageId,
        accumulated_signature: Option<Signature>,
    ) -> Option<ElderCmd> {
        trace!(
            "{}: Received ({:?} {:?}) from src {:?} (client {:?})",
            self,
            request,
            message_id,
            src,
            requester
        );
        use Request::*;
        match request.clone() {
            //Client(client) => self.gateway... // Is handled by `fn handle_client_message(..)` above.
            Gateway(write @ GatewayRequest::Write { .. }) => self.data_payment.handle_write(
                src,
                requester,
                write,
                message_id,
                accumulated_signature,
            ),
            Node(NodeRequest::Read(_)) | Node(NodeRequest::Write(_)) => self
                .metadata
                .handle_request(src, requester, request, message_id, accumulated_signature),
            Node(NodeRequest::System(SystemOp::Transfers(req))) => {
                self.transfers.handle_request(requester, req, message_id)
            }
            _ => None,
        }
    }

    fn handle_response(
        &mut self,
        src: SrcLocation,
        response: Response,
        requester: PublicId,
        message_id: MessageId,
        proof: Option<(Request, Signature)>,
    ) -> Option<ElderCmd> {
        use Response::*;
        trace!(
            "{}: Received ({:?} {:?}) from {}",
            self,
            response,
            message_id,
            utils::get_source_name(src),
        );
        if let Some((request, signature)) = proof.as_ref() {
            if !matches!(requester, PublicId::Node(_))
                && self
                    .validate_section_signature(&request, &signature)
                    .is_none()
            {
                error!("Invalid section signature");
                return None;
            }
            match response {
                Write(_) | GetIData(_) => self
                    .metadata
                    .handle_response(src, response, requester, message_id, proof),
                _ => None,
            }
        } else {
            error!("Missing section signature");
            None
        }
    }

    fn public_key(&self) -> Option<PublicKey> {
        Some(self.routing.borrow().public_key_set().ok()?.public_key())
    }

    fn validate_section_signature(&self, request: &Request, signature: &Signature) -> Option<()> {
        if self
            .public_key()?
            .verify(signature, &utils::serialise(request))
        {
            Some(())
        } else {
            None
        }
    }
}

impl Display for ElderDuties {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.id.name())
    }
}
