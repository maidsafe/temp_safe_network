// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod auth;
mod auth_keys;
mod elder_data;
mod login_packets;
mod messaging;

use self::{
    auth::{Auth, ClientInfo},
    auth_keys::AuthKeysDb,
    elder_data::ElderData,
    login_packets::LoginPackets,
    messaging::Messaging,
};
use crate::{
    action::{Action, ConsensusAction},
    chunk_store::LoginPacketChunkStore,
    routing::Node,
    rpc::Rpc,
    vault::Init,
    Config, Result,
};
use bytes::Bytes;
use log::{error, trace};
use rand::{CryptoRng, Rng};
use safe_nd::{Coins, MessageId, NodePublicId, PublicId, Request, Response, Signature, XorName};
use std::{
    cell::{Cell, RefCell},
    fmt::{self, Display, Formatter},
    net::SocketAddr,
    rc::Rc,
};

/// The cost to Put a chunk to the network.
pub const COST_OF_PUT: Coins = Coins::from_nano(1);

pub(crate) struct ClientHandler {
    id: NodePublicId,
    messaging: Messaging,
    auth: Auth,
    login_packets: LoginPackets,
    data: ElderData,
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
        let auth_db = AuthKeysDb::new(root_dir, init_mode)?;
        let packet_db = LoginPacketChunkStore::new(
            root_dir,
            config.max_capacity(),
            Rc::clone(&total_used_space),
            init_mode,
        )?;

        let messaging = Messaging::new(id.clone(), routing_node);

        let auth = Auth::new(id.clone(), auth_db);
        let login_packets = LoginPackets::new(id.clone(), packet_db);
        let data = ElderData::new(id.clone());

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

    pub fn handle_vault_rpc(&mut self, src: XorName, rpc: Rpc) -> Option<Action> {
        match rpc {
            Rpc::Request {
                request,
                requester,
                message_id,
            } => self.finalize_client_request(src, requester, request, message_id),
            Rpc::Response {
                response,
                requester,
                message_id,
            } => self
                .messaging
                .relay_reponse_to_client(src, &requester, response, message_id),
        }
    }

    pub fn handle_consensused_action(&mut self, action: ConsensusAction) -> Option<Action> {
        use ConsensusAction::*;
        trace!("{}: Consensused {:?}", self, action,);
        match action {
            Forward {
                request,
                client_public_id,
                message_id,
                ..
            } => Some(Action::ForwardClientRequest(Rpc::Request {
                requester: client_public_id,
                request,
                message_id,
            })),
            Proxy {
                request,
                client_public_id,
                message_id,
                ..
            } => Some(Action::ProxyClientRequest(Rpc::Request {
                requester: client_public_id,
                request,
                message_id,
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
        match result {
            Some(result) => self.process_client_request(
                &result.client,
                result.request,
                result.message_id,
                result.signature,
            ),
            None => None,
        }
    }

    #[allow(clippy::cognitive_complexity)]
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
        if let Some(action) = self.auth.verify_consistent_address(&request, message_id) {
            return Some(action);
        }

        match request {
            //
            // ===== Immutable Data =====
            //
            PutIData(chunk) => self
                .data
                .idata
                .initiate_idata_creation(client, chunk, message_id),
            GetIData(address) => {
                // TODO: We don't check for the existence of a valid signature for published data,
                // since it's free for anyone to get.  However, as a means of spam prevention, we
                // could change this so that signatures are required, and the signatures would need
                // to match a pattern which becomes increasingly difficult as the client's
                // behaviour is deemed to become more "spammy". (e.g. the get requests include a
                // `seed: [u8; 32]`, and the client needs to form a sig matching a required pattern
                // by brute-force attempts with varying seeds)
                self.data.idata.get_idata(client, address, message_id)
            }
            DeleteUnpubIData(address) => self
                .data
                .idata
                .initiate_unpub_idata_deletion(client, address, message_id),
            //
            // ===== Mutable Data =====
            //
            PutMData(chunk) => self
                .data
                .mdata
                .initiate_mdata_creation(client, chunk, message_id),
            MutateMDataEntries { .. }
            | SetMDataUserPermissions { .. }
            | DelMDataUserPermissions { .. } => self
                .data
                .mdata
                .initiate_mdata_mutation(request, client, message_id),
            DeleteMData(..) => self
                .data
                .mdata
                .initiate_mdata_deletion(request, client, message_id),
            GetMData(..)
            | GetMDataVersion(..)
            | GetMDataShell(..)
            | GetMDataValue { .. }
            | ListMDataPermissions(..)
            | ListMDataUserPermissions { .. }
            | ListMDataEntries(..)
            | ListMDataKeys(..)
            | ListMDataValues(..) => self.data.mdata.get_mdata(request, client, message_id),
            //
            // ===== Append Only Data =====
            //
            PutAData(chunk) => self
                .data
                .adata
                .initiate_adata_creation(client, chunk, message_id),
            GetAData(_)
            | GetADataShell { .. }
            | GetADataRange { .. }
            | GetADataIndices(_)
            | GetADataLastEntry(_)
            | GetADataOwners { .. }
            | GetADataPermissions { .. }
            | GetPubADataUserPermissions { .. }
            | GetUnpubADataUserPermissions { .. }
            | GetADataValue { .. } => self.data.adata.get_adata(client, request, message_id),
            DeleteAData(address) => self
                .data
                .adata
                .initiate_adata_deletion(client, address, message_id),
            AddPubADataPermissions { .. }
            | AddUnpubADataPermissions { .. }
            | SetADataOwner { .. }
            | AppendSeq { .. }
            | AppendUnseq(..) => self
                .data
                .adata
                .initiate_adata_mutation(client, request, message_id),
            //
            // ===== Coins =====
            //
            TransferCoins { .. } | GetBalance | CreateBalance { .. } => unimplemented!(), // temporarily removed
            //
            // ===== Login packets =====
            //
            CreateLoginPacket(login_packet) => self.login_packets.initiate_login_packet_creation(
                &client.public_id,
                login_packet,
                message_id,
            ),
            CreateLoginPacketFor {
                new_owner,
                amount,
                new_login_packet,
            } => self.login_packets.initiate_proxied_login_packet_creation(
                &client.public_id,
                new_owner,
                amount,
                new_login_packet,
                message_id,
            ),
            UpdateLoginPacket(updated_login_packet) => {
                self.login_packets.initiate_login_packet_update(
                    client.public_id.clone(),
                    updated_login_packet,
                    message_id,
                )
            }
            GetLoginPacket(ref address) => {
                self.login_packets
                    .get_login_packet(&client.public_id, address, message_id)
            }
            //
            // ===== Client (Owner) to ClientHandlers =====
            //
            ListAuthKeysAndVersion => self.auth.list_auth_keys_and_version(client, message_id),
            InsAuthKey {
                key,
                version,
                permissions,
            } => {
                self.auth
                    .initiate_auth_key_insertion(client, key, version, permissions, message_id)
            }
            DelAuthKey { key, version } => self
                .auth
                .initiate_auth_key_deletion(client, key, version, message_id),
        }
    }

    // on consensus
    fn finalize_client_request(
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
            CreateLoginPacket(ref login_packet) => self
                .login_packets
                .finalize_login_packet_creation(requester, login_packet, message_id),
            CreateLoginPacketFor {
                new_owner,
                amount,
                new_login_packet,
            } => self.login_packets.finalize_proxied_login_packet_creation(
                src,
                requester,
                new_owner,
                amount,
                new_login_packet,
                message_id,
            ),
            CreateBalance { .. } | TransferCoins { .. } => unimplemented!(),
            UpdateLoginPacket(updated_login_packet) => self
                .login_packets
                .finalize_login_packet_update(requester, &updated_login_packet, message_id),
            InsAuthKey {
                key,
                version,
                permissions,
            } => self.auth.finalize_auth_key_insertion(
                requester,
                key,
                version,
                permissions,
                message_id,
            ),
            DelAuthKey { key, version } => self
                .auth
                .finalize_auth_key_deletion(requester, key, version, message_id),
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
            | MutateMDataEntries { .. }
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
            | GetBalance
            | ListAuthKeysAndVersion
            | GetLoginPacket(..) => {
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
