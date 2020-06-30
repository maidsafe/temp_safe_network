// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod auth;
mod balances;
mod data_requests;
mod login_packets;
mod messaging;
mod replica_manager;
mod transfers;

use self::{
    auth::{Auth, AuthKeysDb, ClientInfo},
    data_requests::Evaluation,
    login_packets::LoginPackets,
    messaging::Messaging,
    replica_manager::ReplicaManager,
    transfers::Transfers,
};
#[cfg(not(feature = "simulated-payouts"))]
use crate::utils;
use crate::{
    action::{Action, ConsensusAction},
    chunk_store::LoginPacketChunkStore,
    rpc::Rpc,
    vault::Init,
    Config, Result,
};
use bytes::Bytes;
use log::{error, trace};
use rand::{CryptoRng, Rng};
use routing::Node;
use safe_nd::{MessageId, Money, NodePublicId, PublicId, Request, Response, Signature, XorName};
use std::{
    cell::{Cell, RefCell},
    fmt::{self, Display, Formatter},
    net::SocketAddr,
    rc::Rc,
};
use threshold_crypto::{PublicKeySet, SecretKeyShare};

/// The cost to Put a chunk to the network.
pub const COST_OF_PUT: Money = Money::from_nano(1);

pub(crate) struct ClientHandler {
    id: NodePublicId,
    messaging: Messaging,
    transfers: Transfers,
    auth: Auth,
    login_packets: LoginPackets,
    data: Evaluation,
}

impl ClientHandler {
    pub fn new(
        id: NodePublicId,
        config: &Config,
        total_used_space: &Rc<Cell<u64>>,
        init_mode: Init,
        routing_node: Rc<RefCell<Node>>,
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

        let node = routing_node.borrow();
        let public_key_set = node.public_key_set()?;
        let secret_key_share = node.secret_key_share()?;
        let key_index = node.our_index()?;
        let replica_manager =
            ReplicaManager::new(secret_key_share, key_index, public_key_set, vec![])?;

        let messaging = Messaging::new(id.clone(), routing_node.clone());

        let auth = Auth::new(id.clone(), auth_keys_db);
        let transfers = Transfers::new(id.clone(), replica_manager);
        let login_packets = LoginPackets::new(id.clone(), login_packets_db);
        let data = Evaluation::new(id.clone());

        let client_handler = Self {
            id,
            messaging,
            transfers,
            auth,
            login_packets,
            data,
        };

        Ok(client_handler)
    }

    pub fn update_replica_keys(
        &mut self,
        pub_key_set: PublicKeySet,
        sec_key_share: SecretKeyShare,
        index: usize,
    ) -> Option<()> {
        self.transfers
            .update_replica_keys(pub_key_set, sec_key_share, index)
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
                &result.client,
                result.request,
                result.message_id,
                result.signature,
            )
        } else {
            None
        }
    }

    pub fn handle_vault_rpc(&mut self, src: XorName, rpc: Rpc) -> Option<Action> {
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
            } => {
                // TODO: FIX AT2 REFUNDS HERE
                /*
                if let Some(refund_amount) = refund {
                    if let Err(error) = self.transfers.deposit(requester.name(), refund_amount) {
                        error!(
                            "{}: Failed to refund {} coins for {:?}: {:?}",
                            self, refund_amount.signed_transfer.transfer.amount, requester, error,
                        )
                    };
                }
                */

                self.messaging
                    .relay_reponse_to_client(src, &requester, response, message_id)
            }
            Rpc::Duplicate { .. } => None,
            Rpc::DuplicationComplete { .. } => None,
        }
    }

    pub fn handle_consensused_action(&mut self, action: ConsensusAction) -> Option<Action> {
        use ConsensusAction::*;
        trace!("{}: Consensused {:?}", self, action,);
        match action {
            PayAndForward {
                request,
                client_public_id,
                message_id,
                debit_proof,
            } => {
                #[cfg(feature = "simulated-payouts")]
                self.transfers.pay(debit_proof.signed_transfer.transfer);

                #[cfg(not(feature = "simulated-payouts"))]
                {
                    let owner = utils::owner(&client_public_id)?;

                    if let Some(action) = self.transfers.pay_section(
                        debit_proof,
                        *owner.public_key(),
                        &request,
                        message_id,
                    ) {
                        return Some(action);
                    }
                }

                Some(Action::ForwardClientRequest(Rpc::Request {
                    requester: client_public_id,
                    request,
                    message_id,
                    signature: None,
                }))
            }
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
            PayAndProxy {
                request,
                client_public_id,
                message_id,
                put_debit_proof,
                optional_amount_debit_proof,
            } => {
                #[cfg(feature = "simulated-payouts")]
                {
                    self.transfers.pay(put_debit_proof.signed_transfer.transfer);

                    if let Some(proof) = optional_amount_debit_proof {
                        self.transfers.pay(proof.signed_transfer.transfer);
                    }
                }

                #[cfg(not(feature = "simulated-payouts"))]
                {
                    let owner = utils::owner(&client_public_id)?;

                    // Pay for Mutation
                    if let Some(action) = self.transfers.pay_section(
                        put_debit_proof,
                        *owner.public_key(),
                        &request,
                        message_id,
                    ) {
                        return Some(action);
                    }

                    // Pay for optional amount
                    if let Some(proof) = optional_amount_debit_proof {
                        if let Some(action) = self.transfers.pay_section(
                            proof,
                            *owner.public_key(),
                            &request,
                            message_id,
                        ) {
                            return Some(action);
                        }
                    }
                }

                Some(Action::ProxyClientRequest(Rpc::Request {
                    requester: client_public_id,
                    request,
                    message_id,
                    signature: None,
                }))
            }
        }
    }

    // on client request
    fn process_client_request(
        &mut self,
        client: &ClientInfo,
        request: Request,
        message_id: MessageId,
        signature: Option<Signature>,
    ) -> Option<Action> {
        use Request::*;
        trace!(
            "{}: Received ({:?} {:?}) from {}",
            self,
            request,
            message_id,
            client.public_id
        );

        if let Some(action) =
            self.auth
                .verify_signature(&client.public_id, &request, message_id, signature)
        {
            return Some(action);
        };
        if let Some(action) = self
            .auth
            .authorise_app(&client.public_id, &request, message_id)
        {
            return Some(action);
        }

        match request {
            IData(idata_req) => self
                .data
                .immutable
                .process_client_request(client, idata_req, message_id),
            MData(mdata_req) => self
                .data
                .mutable
                .process_client_request(client, mdata_req, message_id),
            SData(sdata_req) => self
                .data
                .sequence
                .process_client_request(client, sdata_req, message_id),
            Money(money_req) => self
                .transfers
                .process_client_request(client, money_req, message_id),
            LoginPacket(login_packet_req) => {
                self.login_packets
                    .process_client_request(client, login_packet_req, message_id)
            }
            Client(client_req) => self
                .auth
                .process_client_request(client, client_req, message_id),
        }
    }

    // on consensus
    fn finalise_client_request(
        &mut self,
        src: XorName,
        requester: PublicId,
        request: Request,
        message_id: MessageId,
    ) -> Option<Action> {
        use Request::*;
        trace!(
            "{}: Received ({:?} {:?}) from src {} (client {:?})",
            self,
            request,
            message_id,
            src,
            requester
        );
        match request {
            LoginPacket(req) => self.login_packets.finalise_client_request(
                src,
                requester,
                req,
                message_id,
                &mut self.transfers,
                &mut self.messaging,
            ),
            Money(req) => self.transfers.finalise_client_request(
                requester,
                req,
                message_id,
                &mut self.messaging,
            ),
            Client(req) => self
                .auth
                .finalise_client_request(requester, req, message_id),
            IData(_) | MData(_) | SData(_) => {
                error!(
                    "{}: Should not receive {:?} as a client handler.",
                    self, request
                );
                None
            }
        }
    }
}

impl Display for ClientHandler {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.id.name())
    }
}
