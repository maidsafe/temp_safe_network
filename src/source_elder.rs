// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod account;
mod balance;

use self::{
    account::AccountsDb,
    balance::{Balance, BalancesDb},
};
use crate::{
    action::Action,
    chunk_store::{error::Error as ChunkStoreError, LoginPacketChunkStore},
    quic_p2p::{self, Config as QuicP2pConfig, Event, NodeInfo, Peer, QuicP2p},
    rpc::Rpc,
    utils,
    vault::Init,
    Config, Error, Result,
};
use bytes::Bytes;
use crossbeam_channel::{self, Receiver};
use lazy_static::lazy_static;
use log::{error, info, trace, warn};
use safe_nd::{
    AData, ADataAddress, AppPermissions, Challenge, Coins, Error as NdError, IData, IDataAddress,
    IDataKind, LoginPacket, MData, Message, MessageId, NodePublicId, PublicId, PublicKey, Request,
    Response, Result as NdResult, Signature, Transaction, TransactionId, XorName,
};
use serde::Serialize;
use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::{self, Display, Formatter},
    net::SocketAddr,
    rc::Rc,
};
use unwrap::unwrap;

lazy_static! {
    /// The cost to Put a chunk to the network.
    pub static ref COST_OF_PUT: Coins = unwrap!(Coins::from_nano(1_000_000_000));
}

#[derive(Clone, Debug)]
struct ClientInfo {
    public_id: PublicId,
    has_balance: bool,
}

pub(crate) struct SourceElder {
    id: NodePublicId,
    accounts: AccountsDb,
    balances: BalancesDb,
    clients: HashMap<SocketAddr, ClientInfo>,
    // Map of new client connections to the challenge value we sent them.
    client_candidates: HashMap<SocketAddr, Vec<u8>>,
    quic_p2p: QuicP2p,
    login_packets: LoginPacketChunkStore,
}

impl SourceElder {
    pub fn new(
        id: NodePublicId,
        config: &Config,
        total_used_space: &Rc<RefCell<u64>>,
        init_mode: Init,
    ) -> Result<(Self, Receiver<Event>)> {
        let accounts = AccountsDb::new(config.root_dir(), init_mode)?;
        let balances = BalancesDb::new(config.root_dir(), init_mode)?;
        let (quic_p2p, event_receiver) = Self::setup_quic_p2p(config.quic_p2p_config())?;
        let login_packets = LoginPacketChunkStore::new(
            config.root_dir(),
            config.max_capacity(),
            Rc::clone(&total_used_space),
            init_mode,
        )?;
        let src_elder = Self {
            id,
            accounts,
            balances,
            clients: Default::default(),
            client_candidates: Default::default(),
            quic_p2p,
            login_packets,
        };

        Ok((src_elder, event_receiver))
    }

    fn setup_quic_p2p(config: &QuicP2pConfig) -> Result<(QuicP2p, Receiver<Event>)> {
        let (event_sender, event_receiver) = crossbeam_channel::unbounded();
        let mut quic_p2p = quic_p2p::Builder::new(event_sender)
            .with_config(config.clone())
            .build()?;
        let our_conn_info = quic_p2p.our_connection_info()?;
        info!(
            "QuicP2p started on {}\nwith certificate {:?}",
            our_conn_info.peer_addr, our_conn_info.peer_cert_der
        );
        println!(
            "Our connection info:\n{}\n",
            unwrap!(serde_json::to_string(&our_conn_info))
        );
        Ok((quic_p2p, event_receiver))
    }

    pub fn our_connection_info(&mut self) -> Result<NodeInfo> {
        Ok(self.quic_p2p.our_connection_info()?)
    }

    pub fn handle_new_connection(&mut self, peer: Peer) {
        // If we already know the peer, drop the connection attempt.
        if self.clients.contains_key(&peer.peer_addr())
            || self.client_candidates.contains_key(&peer.peer_addr())
        {
            return;
        }

        let peer_addr = match peer {
            Peer::Node { node_info } => {
                info!(
                    "{}: Rejecting connection attempt by node on {}",
                    self, node_info.peer_addr
                );
                self.quic_p2p.disconnect_from(node_info.peer_addr);
                return;
            }
            Peer::Client { peer_addr } => peer_addr,
        };

        let challenge = utils::random_vec(8);
        self.send(peer.clone(), &Challenge::Request(challenge.clone()));
        let _ = self.client_candidates.insert(peer.peer_addr(), challenge);
        info!("{}: Connected to new client on {}", self, peer_addr);
    }

    pub fn handle_connection_failure(&mut self, peer_addr: SocketAddr, error: Error) {
        info!("{}: {}", self, error);
        if let Some(client) = self.clients.remove(&peer_addr) {
            info!(
                "{}: Disconnected from {:?} on {}",
                self, client.public_id, peer_addr
            );
        } else {
            let _ = self.client_candidates.remove(&peer_addr);
            info!(
                "{}: Disconnected from client candidate on {}",
                self, peer_addr
            );
        }
    }

    pub fn handle_client_message(&mut self, peer_addr: SocketAddr, bytes: Bytes) -> Option<Action> {
        if let Some(client) = self.clients.get(&peer_addr).cloned() {
            match bincode::deserialize(&bytes) {
                Ok(Message::Request {
                    request,
                    message_id,
                    signature,
                }) => {
                    return self.handle_client_request(&client, request, message_id, signature);
                }
                Ok(Message::Response { response, .. }) => {
                    info!(
                        "{}: {} invalidly sent {:?}",
                        self, client.public_id, response
                    );
                }
                Err(err) => {
                    info!(
                        "{}: Unable to deserialise message from {}: {}",
                        self, client.public_id, err
                    );
                }
            }
        } else {
            match bincode::deserialize(&bytes) {
                Ok(Challenge::Response(public_id, signature)) => {
                    self.handle_challenge(peer_addr, public_id, signature);
                }
                Ok(Challenge::Request(_)) => {
                    info!(
                        "{}: Received unexpected challenge request from {}",
                        self, peer_addr
                    );
                    self.quic_p2p.disconnect_from(peer_addr);
                }
                Err(err) => {
                    info!(
                        "{}: Unable to deserialise challenge from {}: {}",
                        self, peer_addr, err
                    );
                }
            }
        }
        None
    }

    #[allow(clippy::cognitive_complexity)]
    fn handle_client_request(
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
        if let Some(sig) = signature.as_ref() {
            if !self.is_valid_client_signature(&client.public_id, &request, &message_id, sig) {
                return None;
            }
        }

        let dbg_msg = format!(
            "{}: ({:?}/{:?}) from {} is unsigned",
            self, request, message_id, client.public_id
        );
        let has_signature = || {
            if (&signature).is_none() {
                warn!("{}", dbg_msg);
                return None;
            }
            Some(())
        };

        // TODO - remove this
        #[allow(unused)]
        match request {
            //
            // ===== Immutable Data =====
            //
            PutIData(chunk) => {
                has_signature()?;
                self.handle_put_idata(client, chunk, message_id)
            }
            GetIData(address) => {
                // TODO: We don't check for the existence of a valid signature for published data,
                // since it's free for anyone to get.  However, as a means of spam prevention, we
                // could change this so that signatures are required, and the signatures would need
                // to match a pattern which becomes increasingly difficult as the client's
                // behaviour is deemed to become more "spammy". (e.g. the get requests include a
                // `seed: [u8; 32]`, and the client needs to form a sig matching a required pattern
                // by brute-force attempts with varying seeds)
                if address.kind() != IDataKind::Pub {
                    has_signature()?;
                }
                self.handle_get_idata(client, address, message_id)
            }
            DeleteUnpubIData(address) => {
                has_signature()?;
                self.handle_delete_unpub_idata(client, address, message_id)
            }
            //
            // ===== Mutable Data =====
            //
            PutMData(chunk) => {
                has_signature()?;
                self.handle_put_mdata(client, chunk, message_id)
            }
            MutateSeqMDataEntries { .. }
            | MutateUnseqMDataEntries { .. }
            | DeleteMData(..)
            | SetMDataUserPermissions { .. }
            | DelMDataUserPermissions { .. } => {
                has_signature()?;
                self.handle_mdata_mutation(request, client, message_id)
            }
            GetMData(..)
            | GetMDataVersion(..)
            | GetMDataShell(..)
            | GetMDataValue { .. }
            | ListMDataPermissions(..)
            | ListMDataUserPermissions { .. }
            | ListMDataEntries(..)
            | ListMDataKeys(..)
            | ListMDataValues(..) => {
                has_signature()?;
                self.handle_get_mdata(request, client, message_id)
            }
            //
            // ===== Append Only Data =====
            //
            PutAData(chunk) => {
                has_signature()?;
                self.handle_put_adata(client, chunk, message_id)
            }
            GetAData(ref address)
            | GetADataShell { ref address, .. }
            | GetADataRange { ref address, .. }
            | GetADataIndices(ref address)
            | GetADataLastEntry(ref address)
            | GetADataOwners { ref address, .. }
            | GetADataPermissions { ref address, .. }
            | GetPubADataUserPermissions { ref address, .. }
            | GetUnpubADataUserPermissions { ref address, .. }
            | GetADataValue { ref address, .. } => {
                if !utils::adata::is_published(address) {
                    has_signature()?;
                }
                self.handle_get_adata(client, request, message_id)
            }
            DeleteAData(address) => {
                has_signature()?;
                self.handle_delete_adata(client, address, message_id)
            }
            AddPubADataPermissions { ref address, .. }
            | AddUnpubADataPermissions { ref address, .. }
            | SetADataOwner { ref address, .. } => {
                has_signature()?;
                self.handle_mutate_adata(client, request, message_id)
            }
            AppendSeq { ref append, .. } => {
                if !utils::adata::is_published(&append.address) {
                    has_signature()?;
                }
                if !utils::adata::is_sequential(&append.address) {
                    self.send_response_to_client(
                        &client.public_id,
                        message_id,
                        Response::Mutation(Err(NdError::InvalidOperation)),
                    );
                    return None;
                }
                self.handle_mutate_adata(client, request, message_id)
            }
            AppendUnseq(ref append) => {
                if !utils::adata::is_published(&append.address) {
                    has_signature()?;
                }
                if utils::adata::is_sequential(&append.address) {
                    self.send_response_to_client(
                        &client.public_id,
                        message_id,
                        Response::Mutation(Err(NdError::InvalidOperation)),
                    );
                    return None;
                }
                self.handle_mutate_adata(client, request, message_id)
            }
            //
            // ===== Coins =====
            //
            TransferCoins {
                destination,
                amount,
                transaction_id,
            } => {
                has_signature()?;
                self.handle_transfer_coins(
                    &client.public_id,
                    message_id,
                    destination,
                    amount,
                    transaction_id,
                )
            }
            GetBalance => {
                has_signature()?;
                let balance = self
                    .balance(client.public_id.name())
                    .or_else(|| Coins::from_nano(0).ok())?;
                let response = Response::GetBalance(Ok(balance));
                self.send_response_to_client(&client.public_id, message_id, response);
                None
            }
            CreateBalance {
                new_balance_owner,
                amount,
                transaction_id,
            } => {
                has_signature()?;
                self.handle_create_balance(
                    &client.public_id,
                    message_id,
                    new_balance_owner,
                    amount,
                    transaction_id,
                )
            }
            //
            // ===== Login packets =====
            //
            CreateLoginPacket(login_packet) => {
                has_signature()?;
                self.handle_create_login_packet_client_req(
                    &client.public_id,
                    login_packet,
                    message_id,
                )
            }
            CreateLoginPacketFor {
                new_owner,
                amount,
                transaction_id,
                new_login_packet,
            } => {
                has_signature()?;
                unimplemented!();
                // self.handle_create_balance()
                // THEN
                // self.handle_create_login_packet_req()
            }
            UpdateLoginPacket(ref updated_login_packet) => {
                has_signature()?;
                self.handle_update_login_packet_req(
                    &client.public_id,
                    updated_login_packet,
                    message_id,
                )
            }
            GetLoginPacket(ref address) => {
                has_signature()?;
                self.handle_get_login_packet_req(&client.public_id, address, message_id)
            }
            //
            // ===== Client (Owner) to SrcElders =====
            //
            ListAuthKeysAndVersion => {
                has_signature()?;
                self.handle_list_auth_keys_and_version(client, message_id)
            }
            InsAuthKey {
                key,
                version,
                permissions,
            } => {
                has_signature()?;
                self.handle_ins_auth_key(client, key, version, permissions, message_id)
            }
            DelAuthKey { key, version } => {
                has_signature()?;
                self.handle_del_auth_key(client, key, version, message_id)
            }
        }
    }

    fn is_valid_client_signature(
        &self,
        client_id: &PublicId,
        request: &Request,
        message_id: &MessageId,
        signature: &Signature,
    ) -> bool {
        let pub_key = match utils::own_key(client_id) {
            Some(pk) => pk,
            None => {
                error!("{}: Logic error.  This should be unreachable.", self);
                return false;
            }
        };
        match pub_key.verify(signature, utils::serialise(&(request, message_id))) {
            Ok(_) => true,
            Err(error) => {
                warn!(
                    "{}: ({:?}/{:?}) from {} is invalid: {}",
                    self, request, message_id, client_id, error
                );
                false
            }
        }
    }

    fn handle_get_mdata(
        &mut self,
        request: Request,
        client: &ClientInfo,
        message_id: MessageId,
    ) -> Option<Action> {
        Some(Action::ForwardClientRequest(Rpc::Request {
            requester: client.public_id.clone(),
            request,
            message_id,
        }))
    }

    fn handle_mdata_mutation(
        &mut self,
        request: Request,
        client: &ClientInfo,
        message_id: MessageId,
    ) -> Option<Action> {
        let owner = utils::owner(&client.public_id)?;
        self.pay(
            &client.public_id,
            owner.public_key(),
            &request,
            message_id,
            *COST_OF_PUT,
        )?;

        Some(Action::ForwardClientRequest(Rpc::Request {
            requester: client.public_id.clone(),
            request,
            message_id,
        }))
    }

    fn handle_put_mdata(
        &mut self,
        client: &ClientInfo,
        chunk: MData,
        message_id: MessageId,
    ) -> Option<Action> {
        let owner = utils::owner(&client.public_id)?;

        // Assert that the owner's public key has been added to the chunk, to avoid Apps
        // putting chunks which can't be retrieved by their Client owners.
        if chunk.owner() != *owner.public_key() {
            trace!(
                "{}: {} attempted PutMData with invalid owners field.",
                self,
                client.public_id
            );
            self.send_response_to_client(
                &client.public_id,
                message_id,
                Response::Mutation(Err(NdError::InvalidOwners)),
            );
            return None;
        }

        let request = Request::PutMData(chunk);
        self.pay(
            &client.public_id,
            owner.public_key(),
            &request,
            message_id,
            *COST_OF_PUT,
        )?;

        Some(Action::ForwardClientRequest(Rpc::Request {
            requester: client.public_id.clone(),
            request,
            message_id,
        }))
    }

    fn handle_put_idata(
        &mut self,
        client: &ClientInfo,
        chunk: IData,
        message_id: MessageId,
    ) -> Option<Action> {
        let owner = utils::owner(&client.public_id)?;

        // Assert that if the request was for UnpubIData, that the owner's public key has
        // been added to the chunk, to avoid Apps putting chunks which can't be retrieved
        // by their Client owners.
        if let IData::Unpub(ref unpub_chunk) = &chunk {
            if unpub_chunk.owner() != owner.public_key() {
                trace!(
                    "{}: {} attempted Put UnpubIData with invalid owners field.",
                    self,
                    client.public_id
                );
                self.send_response_to_client(
                    &client.public_id,
                    message_id,
                    Response::Mutation(Err(NdError::InvalidOwners)),
                );
                return None;
            }
        }

        let request = Request::PutIData(chunk);
        self.pay(
            &client.public_id,
            owner.public_key(),
            &request,
            message_id,
            *COST_OF_PUT,
        )?;

        Some(Action::ForwardClientRequest(Rpc::Request {
            requester: client.public_id.clone(),
            request,
            message_id,
        }))
    }

    fn handle_get_idata(
        &mut self,
        client: &ClientInfo,
        address: IDataAddress,
        message_id: MessageId,
    ) -> Option<Action> {
        Some(Action::ForwardClientRequest(Rpc::Request {
            requester: client.public_id.clone(),
            request: Request::GetIData(address),
            message_id,
        }))
    }

    fn handle_delete_unpub_idata(
        &mut self,
        client: &ClientInfo,
        address: IDataAddress,
        message_id: MessageId,
    ) -> Option<Action> {
        if address.kind() == IDataKind::Pub {
            self.send_response_to_client(
                &client.public_id,
                message_id,
                Response::Mutation(Err(NdError::InvalidOperation)),
            );
            return None;
        }
        Some(Action::ForwardClientRequest(Rpc::Request {
            requester: client.public_id.clone(),
            request: Request::DeleteUnpubIData(address),
            message_id,
        }))
    }

    fn handle_get_adata(
        &mut self,
        client: &ClientInfo,
        request: Request,
        message_id: MessageId,
    ) -> Option<Action> {
        Some(Action::ForwardClientRequest(Rpc::Request {
            requester: client.public_id.clone(),
            request,
            message_id,
        }))
    }

    fn handle_put_adata(
        &mut self,
        client: &ClientInfo,
        chunk: AData,
        message_id: MessageId,
    ) -> Option<Action> {
        let owner = utils::owner(&client.public_id)?;
        // TODO - Should we replace this with a adata.check_permission call in destination_elder.
        // That would be more consistent, but on the other hand a check here stops spam earlier.
        if chunk.check_is_last_owner(*owner.public_key()).is_err() {
            trace!(
                "{}: {} attempted Put AppendOnlyData with invalid owners.",
                self,
                client.public_id
            );
            self.send_response_to_client(
                &client.public_id,
                message_id,
                Response::Mutation(Err(NdError::InvalidOwners)),
            );
            return None;
        }

        let request = Request::PutAData(chunk);
        self.pay(
            &client.public_id,
            owner.public_key(),
            &request,
            message_id,
            *COST_OF_PUT,
        )?;

        Some(Action::ForwardClientRequest(Rpc::Request {
            requester: client.public_id.clone(),
            request,
            message_id,
        }))
    }

    fn handle_delete_adata(
        &mut self,
        client: &ClientInfo,
        address: ADataAddress,
        message_id: MessageId,
    ) -> Option<Action> {
        if utils::adata::is_published(&address) {
            self.send_response_to_client(
                &client.public_id,
                message_id,
                Response::Mutation(Err(NdError::InvalidOperation)),
            );
            return None;
        }
        let request = Request::DeleteAData(address);
        if client.has_balance {
            Some(Action::ForwardClientRequest(Rpc::Request {
                requester: client.public_id.clone(),
                request,
                message_id,
            }))
        } else {
            self.send_response_to_client(
                &client.public_id,
                message_id,
                Response::Mutation(Err(NdError::AccessDenied)),
            );
            None
        }
    }

    fn handle_mutate_adata(
        &mut self,
        client: &ClientInfo,
        request: Request,
        message_id: MessageId,
    ) -> Option<Action> {
        if client.has_balance {
            Some(Action::ForwardClientRequest(Rpc::Request {
                requester: client.public_id.clone(),
                request,
                message_id,
            }))
        } else {
            self.send_response_to_client(
                &client.public_id,
                message_id,
                Response::Mutation(Err(NdError::AccessDenied)),
            );
            None
        }
    }

    /// Handles a received challenge response.
    ///
    /// Checks that the response contains a valid signature of the challenge we previously sent.
    fn handle_challenge(
        &mut self,
        peer_addr: SocketAddr,
        public_id: PublicId,
        signature: Signature,
    ) {
        let public_key = match utils::own_key(&public_id) {
            Some(pk) => pk,
            None => {
                info!(
                    "{}: Client on {} identifies as a node: {}",
                    self, peer_addr, public_id
                );
                self.quic_p2p.disconnect_from(peer_addr);
                return;
            }
        };
        if let Some(challenge) = self.client_candidates.remove(&peer_addr) {
            match public_key.verify(&signature, challenge) {
                Ok(()) => {
                    let has_balance = self.has_balance(&public_id);
                    info!(
                        "{}: Accepted {} on {}. Has balance: {}",
                        self, public_id, peer_addr, has_balance
                    );
                    let _ = self.clients.insert(
                        peer_addr,
                        ClientInfo {
                            public_id,
                            has_balance,
                        },
                    );
                }
                Err(err) => {
                    info!(
                        "{}: Challenge failed for {} on {}: {}",
                        self, public_id, peer_addr, err
                    );
                    self.quic_p2p.disconnect_from(peer_addr);
                }
            }
        } else {
            info!(
                "{}: {} on {} supplied challenge response without us providing it.",
                self, public_id, peer_addr
            );
            self.quic_p2p.disconnect_from(peer_addr);
        }
    }

    fn has_balance(&self, public_id: &PublicId) -> bool {
        match public_id {
            PublicId::Client(pub_id) => self.balances.exists(pub_id.name()),
            PublicId::App(app_pub_id) => {
                self.balances.exists(app_pub_id.owner().name())
                    && self.accounts.app_permissions(app_pub_id).is_some()
            }
            PublicId::Node(_) => {
                error!("{}: Logic error. This should be unreachable.", self);
                false
            }
        }
    }

    pub fn handle_vault_message(&mut self, src: XorName, message: Rpc) -> Option<Action> {
        match message {
            Rpc::Request {
                request,
                requester,
                message_id,
            } => self.handle_vault_request(src, requester, request, message_id),
            Rpc::Response {
                response,
                requester,
                message_id,
            } => self.handle_response(src, requester, response, message_id),
        }
    }

    fn handle_vault_request(
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
        // TODO - remove this
        #[allow(unused)]
        match request {
            CreateLoginPacket(ref login_packet) => {
                self.handle_create_login_packet_vault_req(requester, login_packet, message_id)
            }
            PutIData(_)
            | GetIData(_)
            | DeleteUnpubIData(_)
            | PutMData(_)
            | GetMData(_)
            | GetMDataValue { .. }
            | DeleteMData(_)
            | GetMDataShell(_)
            | GetMDataVersion(_)
            | ListMDataEntries(_)
            | ListMDataKeys(_)
            | ListMDataValues(_)
            | SetMDataUserPermissions { .. }
            | DelMDataUserPermissions { .. }
            | ListMDataPermissions(_)
            | ListMDataUserPermissions { .. }
            | MutateSeqMDataEntries { .. }
            | MutateUnseqMDataEntries { .. }
            | PutAData(_)
            | GetAData(_)
            | GetADataShell { .. }
            | GetADataValue { .. }
            | DeleteAData(_)
            | GetADataRange { .. }
            | GetADataIndices(_)
            | GetADataLastEntry(_)
            | GetADataPermissions { .. }
            | GetPubADataUserPermissions { .. }
            | GetUnpubADataUserPermissions { .. }
            | GetADataOwners { .. }
            | AddPubADataPermissions { .. }
            | AddUnpubADataPermissions { .. }
            | SetADataOwner { .. }
            | AppendSeq { .. }
            | AppendUnseq(_)
            | TransferCoins { .. }
            | GetBalance
            | ListAuthKeysAndVersion
            | InsAuthKey { .. }
            | DelAuthKey { .. }
            | CreateBalance { .. }
            | CreateLoginPacketFor { .. }
            | UpdateLoginPacket { .. }
            | GetLoginPacket(..) => {
                error!(
                    "{}: Should not receive {:?} as a source elder.",
                    self, request
                );
                None
            }
        }
    }

    /// Handle response from the destination elders.
    fn handle_response(
        &mut self,
        dst_elders: XorName,
        requester: PublicId,
        response: Response,
        message_id: MessageId,
    ) -> Option<Action> {
        use Response::*;
        trace!(
            "{}: Received ({:?} {:?}) to {} from {}",
            self,
            response,
            message_id,
            requester,
            dst_elders
        );

        match response {
            // Transfer the response from destination elders to clients
            GetIData(..)
            | GetAData(..)
            | GetADataShell(..)
            | GetADataRange(..)
            | GetADataIndices(..)
            | GetADataLastEntry(..)
            | GetADataOwners(..)
            | GetPubADataUserPermissions(..)
            | GetUnpubADataUserPermissions(..)
            | GetUnpubADataPermissionAtIndex(..)
            | GetPubADataPermissionAtIndex(..)
            | GetADataValue(..)
            | GetMData(..)
            | GetMDataShell(..)
            | GetMDataVersion(..)
            | ListUnseqMDataEntries(..)
            | ListSeqMDataEntries(..)
            | ListMDataKeys(..)
            | ListSeqMDataValues(..)
            | ListUnseqMDataValues(..)
            | ListMDataUserPermissions(..)
            | ListMDataPermissions(..)
            | GetSeqMDataValue(..)
            | GetUnseqMDataValue(..)
            | Mutation(..)
            | Transaction(..) => {
                self.send_response_to_client(&requester, message_id, response);
                None
            }
            //
            // ===== Invalid =====
            //
            GetLoginPacket(_) | GetBalance(_) | ListAuthKeysAndVersion(_) => {
                error!(
                    "{}: Should not receive {:?} as a source elder.",
                    self, response
                );
                None
            }
        }
    }

    fn handle_create_balance(
        &mut self,
        public_id: &PublicId,
        message_id: MessageId,
        owner_key: PublicKey,
        amount: Coins,
        transaction_id: TransactionId,
    ) -> Option<Action> {
        let result = self
            .withdraw_coins_for_transfer(public_id.name(), amount)
            .and_then(|cost| {
                self.create_balance(owner_key, amount).map_err(|error| {
                    self.refund(public_id.name(), cost);
                    error
                })
            })
            .map(|_| Transaction {
                id: transaction_id,
                amount,
            });

        self.send_response_to_client(public_id, message_id, Response::Transaction(result));
        None
    }

    fn handle_transfer_coins(
        &mut self,
        public_id: &PublicId,
        message_id: MessageId,
        destination: XorName,
        amount: Coins,
        transaction_id: TransactionId,
    ) -> Option<Action> {
        let result = self
            .withdraw_coins_for_transfer(public_id.name(), amount)
            .and_then(|cost| {
                self.deposit(&destination, amount).map_err(|error| {
                    self.refund(public_id.name(), cost);
                    error
                })
            })
            .map(|_| Transaction {
                id: transaction_id,
                amount,
            });

        self.send_response_to_client(public_id, message_id, Response::Transaction(result));
        None
    }

    fn withdraw_coins_for_transfer(
        &mut self,
        balance_name: &XorName,
        amount: Coins,
    ) -> Result<Coins, NdError> {
        match self.withdraw(balance_name, amount) {
            Ok(()) => Ok(amount),
            Err(error) => {
                // Note: in phase 1, we proceed even if there are insufficient funds.
                trace!("{}: Unable to withdraw {} coins: {}", self, amount, error);
                Ok(unwrap!(Coins::from_nano(0)))
            }
        }
    }

    fn create_balance(&mut self, owner_key: PublicKey, amount: Coins) -> Result<(), NdError> {
        if self.balances.exists(&owner_key) {
            info!(
                "{}: Failed to create balance for {:?}: already exists.",
                self, owner_key
            );

            Err(NdError::BalanceExists)
        } else {
            let balance = Balance { coins: amount };
            self.put_balance(&owner_key, &balance)?;
            for client in self
                .clients
                .values_mut()
                .filter(|client| client.public_id.name() == &XorName::from(owner_key))
            {
                client.has_balance = true;
            }
            Ok(())
        }
    }

    fn refund(&mut self, balance_name: &XorName, amount: Coins) {
        if let Err(error) = self.deposit(balance_name, amount) {
            error!(
                "{}: Failed to refund {} coins to balance of {:?}: {:?}.",
                self, amount, balance_name, error
            );
        }
    }

    fn send<T: Serialize>(&mut self, recipient: Peer, msg: &T) {
        let msg = utils::serialise(msg);
        let msg = Bytes::from(msg);
        self.quic_p2p.send(recipient, msg)
    }

    fn send_response_to_client(
        &mut self,
        client_id: &PublicId,
        message_id: MessageId,
        response: Response,
    ) {
        let peer_addr = if let Some(peer_addr) = self.lookup_client_peer_addr(client_id) {
            *peer_addr
        } else {
            info!("{}: client {} not found", self, client_id);
            return;
        };

        self.send(
            Peer::Client { peer_addr },
            &Message::Response {
                response,
                message_id,
            },
        )
    }

    fn lookup_client_peer_addr(&self, id: &PublicId) -> Option<&SocketAddr> {
        self.clients
            .iter()
            .find(|(_, client)| &client.public_id == id)
            .map(|(peer_addr, _)| peer_addr)
    }

    fn balance<K: balance::Key>(&self, key: &K) -> Option<Coins> {
        self.balances.get(key).map(|balance| balance.coins)
    }

    fn withdraw<K: balance::Key>(&mut self, key: &K, amount: Coins) -> Result<(), NdError> {
        let (public_key, mut balance) = self
            .balances
            .get_key_value(key)
            .ok_or(NdError::NoSuchBalance)?;
        balance.coins = balance
            .coins
            .checked_sub(amount)
            .ok_or(NdError::InsufficientBalance)?;
        self.put_balance(&public_key, &balance)
    }

    fn deposit<K: balance::Key>(&mut self, key: &K, amount: Coins) -> Result<(), NdError> {
        let (public_key, mut balance) = self
            .balances
            .get_key_value(key)
            .ok_or_else(|| NdError::from("No such balance"))?;
        balance.coins = balance
            .coins
            .checked_add(amount)
            .ok_or(NdError::ExcessiveValue)?;

        self.put_balance(&public_key, &balance)
    }

    fn put_balance(&mut self, public_key: &PublicKey, balance: &Balance) -> Result<(), NdError> {
        self.balances.put(public_key, balance).map_err(|error| {
            error!(
                "{}: Failed to update balance of {}: {}",
                self, public_key, error
            );

            NdError::from("Failed to update balance")
        })
    }

    // Pays cost of a request.
    fn pay(
        &mut self,
        requester_id: &PublicId,
        requester_key: &PublicKey,
        request: &Request,
        message_id: MessageId,
        cost: Coins,
    ) -> Option<()> {
        match self.withdraw(requester_key, cost) {
            Ok(()) => Some(()),
            Err(NdError::InsufficientBalance) | Err(NdError::NoSuchBalance) => {
                // Note: in phase 1, we proceed even if there are insufficient funds.
                trace!(
                    "{}: Insufficient balance to withdraw {} coins (but allowing the request anyway)",
                    self,
                    cost,
                );
                Some(())
            }
            Err(error) => {
                self.send_response_to_client(
                    requester_id,
                    message_id,
                    utils::to_error_response(&request, error),
                );
                None
            }
        }
    }

    fn handle_create_login_packet_client_req(
        &mut self,
        client_id: &PublicId,
        login_packet: LoginPacket,
        message_id: MessageId,
    ) -> Option<Action> {
        if !login_packet.size_is_valid() {
            self.send_response_to_client(
                client_id,
                message_id,
                Response::Mutation(Err(NdError::ExceededSize)),
            );
            return None;
        }

        let request = Request::CreateLoginPacket(login_packet);
        self.pay(
            client_id,
            utils::client(client_id)?.public_key(),
            &request,
            message_id,
            *COST_OF_PUT,
        )?;

        Some(Action::ForwardClientRequest(Rpc::Request {
            requester: client_id.clone(),
            request,
            message_id,
        }))
    }

    fn handle_create_login_packet_vault_req(
        &mut self,
        requester: PublicId,
        login_packet: &LoginPacket,
        message_id: MessageId,
    ) -> Option<Action> {
        let result = if self.login_packets.has(login_packet.destination()) {
            Err(NdError::LoginPacketExists)
        } else {
            self.login_packets
                .put(login_packet)
                .map_err(|error| error.to_string().into())
        };
        Some(Action::RespondToSrcElders {
            sender: *login_packet.destination(),
            message: Rpc::Response {
                response: Response::Mutation(result),
                requester,
                message_id,
            },
        })
    }

    fn handle_update_login_packet_req(
        &mut self,
        client_id: &PublicId,
        updated_login_packet: &LoginPacket,
        message_id: MessageId,
    ) -> Option<Action> {
        let result = self
            .login_packet(
                utils::own_key(client_id)?,
                updated_login_packet.destination(),
            )
            .and_then(|_existing_login_packet| {
                if !updated_login_packet.size_is_valid() {
                    return Err(NdError::ExceededSize);
                }
                self.login_packets
                    .put(updated_login_packet)
                    .map_err(|err| err.to_string().into())
            });
        self.send_response_to_client(client_id, message_id, Response::Mutation(result));
        None
    }

    fn handle_get_login_packet_req(
        &mut self,
        client_id: &PublicId,
        address: &XorName,
        message_id: MessageId,
    ) -> Option<Action> {
        let result = self
            .login_packet(utils::own_key(client_id)?, address)
            .map(|login_packet| {
                // TODO - Fix before committing.  Use new function from safe-nd rather than cloning
                (
                    login_packet.data().to_vec(),
                    login_packet.signature().clone(),
                )
            });
        self.send_response_to_client(client_id, message_id, Response::GetLoginPacket(result));
        None
    }

    fn login_packet(
        &self,
        requester_pub_key: &PublicKey,
        packet_name: &XorName,
    ) -> NdResult<LoginPacket> {
        self.login_packets
            .get(packet_name)
            .map_err(|e| match e {
                ChunkStoreError::NoSuchChunk => NdError::NoSuchLoginPacket,
                error => error.to_string().into(),
            })
            .and_then(|login_packet| {
                if login_packet.authorised_getter() == requester_pub_key {
                    Ok(login_packet)
                } else {
                    Err(NdError::AccessDenied)
                }
            })
    }

    fn handle_list_auth_keys_and_version(
        &mut self,
        client: &ClientInfo,
        message_id: MessageId,
    ) -> Option<Action> {
        let result = Ok(self
            .accounts
            .list_auth_keys_and_version(utils::client(&client.public_id)?));

        self.send_response_to_client(
            &client.public_id,
            message_id,
            Response::ListAuthKeysAndVersion(result),
        );
        None
    }

    fn handle_ins_auth_key(
        &mut self,
        client: &ClientInfo,
        key: PublicKey,
        new_version: u64,
        permissions: AppPermissions,
        message_id: MessageId,
    ) -> Option<Action> {
        let result = self.accounts.ins_auth_key(
            utils::client(&client.public_id)?,
            key,
            new_version,
            permissions,
        );
        self.send_response_to_client(&client.public_id, message_id, Response::Mutation(result));
        None
    }

    fn handle_del_auth_key(
        &mut self,
        client: &ClientInfo,
        key: PublicKey,
        new_version: u64,
        message_id: MessageId,
    ) -> Option<Action> {
        let result =
            self.accounts
                .del_auth_key(utils::client(&client.public_id)?, key, new_version);
        self.send_response_to_client(&client.public_id, message_id, Response::Mutation(result));
        None
    }
}

impl Display for SourceElder {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "Node({})", self.id.name())
    }
}
