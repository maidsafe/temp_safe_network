// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod auth;
mod auth_keys;
mod balance;
mod elder_data;
mod login_packets;
mod messaging;

use self::{
    auth::{Auth, ClientInfo},
    auth_keys::AuthKeysDb,
    balance::{Balance, BalancesDb},
    elder_data::ElderData,
    login_packets::LoginPackets,
    messaging::Messaging,
};
use crate::{
    action::{Action, ConsensusAction},
    chunk_store::LoginPacketChunkStore,
    routing::Node,
    rpc::Rpc,
    utils,
    vault::Init,
    Config, Result,
};
use bytes::Bytes;
use log::{error, info, trace};
use rand::{CryptoRng, Rng};
use safe_nd::{
    Coins, Error as NdError, MessageId, NodePublicId, PublicId, PublicKey, Request, Response,
    Signature, Transaction, TransactionId, XorName,
};
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
    balances: BalancesDb,
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
        let balances = BalancesDb::new(root_dir, init_mode)?;
        let login_packets_db = LoginPacketChunkStore::new(
            root_dir,
            config.max_capacity(),
            Rc::clone(&total_used_space),
            init_mode,
        )?;

        let messaging = Messaging::new(id.clone(), routing_node);

        let auth = Auth::new(id.clone(), auth_keys_db);
        let login_packets = LoginPackets::new(id.clone(), login_packets_db);
        let data = ElderData::new(id.clone());

        let client_handler = Self {
            id,
            messaging,
            auth,
            login_packets,
            data,
            balances,
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
                refund,
            } => {
                if let Some(refund_amount) = refund {
                    if let Err(error) = self.deposit(requester.name(), refund_amount) {
                        error!(
                            "{}: Failed to refund {} coins for {:?}: {:?}",
                            self, refund_amount, requester, error,
                        )
                    };
                }
                self.messaging
                    .relay_reponse_to_client(src, &requester, response, message_id)
            }
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
                cost,
            } => {
                let owner = utils::owner(&client_public_id)?;
                self.pay(
                    &client_public_id,
                    owner.public_key(),
                    &request,
                    message_id,
                    cost,
                )?;

                Some(Action::ForwardClientRequest(Rpc::Request {
                    requester: client_public_id,
                    request,
                    message_id,
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
            })),
            PayAndProxy {
                request,
                client_public_id,
                message_id,
                cost,
            } => {
                let owner = utils::owner(&client_public_id)?;

                self.pay(
                    &client_public_id,
                    owner.public_key(),
                    &request,
                    message_id,
                    cost,
                )?;

                Some(Action::ProxyClientRequest(Rpc::Request {
                    requester: client_public_id,
                    request,
                    message_id,
                }))
            }
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
                &result.client,
                result.request,
                result.message_id,
                result.signature,
            )
        } else {
            None
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
            TransferCoins {
                destination,
                amount,
                transaction_id,
            } => self.handle_transfer_coins_client_req(
                &client.public_id,
                destination,
                amount,
                transaction_id,
                message_id,
            ),
            GetBalance => {
                let balance = self
                    .balance(client.public_id.name())
                    .ok_or(NdError::NoSuchBalance);
                let response = Response::GetBalance(balance);
                self.respond_to_client(message_id, response);
                None
            }
            CreateBalance {
                new_balance_owner,
                amount,
                transaction_id,
            } => self.handle_create_balance_client_req(
                &client.public_id,
                new_balance_owner,
                amount,
                transaction_id,
                message_id,
            ),
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
                transaction_id,
            } => self.login_packets.initiate_proxied_login_packet_creation(
                &client.public_id,
                new_owner,
                amount,
                transaction_id,
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
                transaction_id,
            } => {
                if &src == requester.name() {
                    // Step two - create balance and forward login_packet.
                    if let Err(error) = self.create_balance(&requester, new_owner, amount) {
                        // Refund amount (Including the cost of creating the balance)
                        let refund = Some(amount.checked_add(COST_OF_PUT)?);

                        Some(Action::RespondToClientHandlers {
                            sender: XorName::from(new_owner),
                            rpc: Rpc::Response {
                                response: Response::Transaction(Err(error)),
                                requester,
                                message_id,
                                refund,
                            },
                        })
                    } else {
                        Some(Action::ForwardClientRequest(Rpc::Request {
                            request: CreateLoginPacketFor {
                                new_owner,
                                amount,
                                new_login_packet,
                                transaction_id,
                            },
                            requester,
                            message_id,
                        }))
                    }
                } else {
                    self.login_packets.finalize_proxied_login_packet_creation(
                        requester,
                        amount,
                        transaction_id,
                        new_login_packet,
                        message_id,
                    )
                }
            }
            CreateBalance {
                new_balance_owner,
                amount,
                transaction_id,
            } => self.handle_create_balance_vault_req(
                requester,
                new_balance_owner,
                amount,
                transaction_id,
                message_id,
            ),
            TransferCoins {
                destination,
                amount,
                transaction_id,
            } => self.handle_transfer_coins_vault_req(
                requester,
                destination,
                amount,
                transaction_id,
                message_id,
            ),
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

    fn handle_create_balance_client_req(
        &mut self,
        requester: &PublicId,
        owner_key: PublicKey,
        amount: Coins,
        transaction_id: TransactionId,
        message_id: MessageId,
    ) -> Option<Action> {
        let request = Request::CreateBalance {
            new_balance_owner: owner_key,
            amount,
            transaction_id,
        };
        // For phases 1 & 2 we allow owners to create their own balance freely.
        let own_request = utils::own_key(requester)
            .map(|key| key == &owner_key)
            .unwrap_or(false);
        if own_request {
            return Some(Action::VoteFor(ConsensusAction::Forward {
                request,
                client_public_id: requester.clone(),
                message_id,
            }));
        }

        let total_amount = amount.checked_add(COST_OF_PUT)?;
        // When ClientA(owner/app with permissions) creates a balance for ClientB
        Some(Action::VoteFor(ConsensusAction::PayAndForward {
            request,
            client_public_id: requester.clone(),
            message_id,
            cost: total_amount,
        }))
    }

    fn handle_create_balance_vault_req(
        &mut self,
        requester: PublicId,
        owner_key: PublicKey,
        amount: Coins,
        transaction_id: TransactionId,
        message_id: MessageId,
    ) -> Option<Action> {
        let (result, refund) = match self.create_balance(&requester, owner_key, amount) {
            Ok(()) => {
                let destination = XorName::from(owner_key);
                let transaction = Transaction {
                    id: transaction_id,
                    amount,
                };
                self.messaging
                    .notify_destination_owners(&destination, transaction);
                (Ok(transaction), None)
            }
            Err(error) => {
                // Refund amount (Including the cost of creating a balance)
                let amount = amount.checked_add(COST_OF_PUT)?;
                (Err(error), Some(amount))
            }
        };

        Some(Action::RespondToClientHandlers {
            sender: *self.id.name(),
            rpc: Rpc::Response {
                response: Response::Transaction(result),
                requester,
                message_id,
                refund,
            },
        })
    }

    fn handle_transfer_coins_client_req(
        &mut self,
        requester: &PublicId,
        destination: XorName,
        amount: Coins,
        transaction_id: TransactionId,
        message_id: MessageId,
    ) -> Option<Action> {
        Some(Action::VoteFor(ConsensusAction::PayAndForward {
            request: Request::TransferCoins {
                destination,
                amount,
                transaction_id,
            },
            client_public_id: requester.clone(),
            message_id,
            cost: amount,
        }))
    }

    fn handle_transfer_coins_vault_req(
        &mut self,
        requester: PublicId,
        destination: XorName,
        amount: Coins,
        transaction_id: TransactionId,
        message_id: MessageId,
    ) -> Option<Action> {
        let (result, refund) = match self.deposit(&destination, amount) {
            Ok(()) => {
                let transaction = Transaction {
                    id: transaction_id,
                    amount,
                };

                self.messaging
                    .notify_destination_owners(&destination, transaction);
                (Ok(transaction), None)
            }
            Err(error) => (Err(error), Some(amount)),
        };

        Some(Action::RespondToClientHandlers {
            sender: *self.id.name(),
            rpc: Rpc::Response {
                response: Response::Transaction(result),
                requester,
                message_id,
                refund,
            },
        })
    }

    fn withdraw<K: balance::Key>(&mut self, key: &K, amount: Coins) -> Result<(), NdError> {
        if amount.as_nano() == 0 {
            return Err(NdError::InvalidOperation);
        }
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
            .ok_or_else(|| NdError::NoSuchBalance)?;
        balance.coins = balance
            .coins
            .checked_add(amount)
            .ok_or(NdError::ExcessiveValue)?;

        self.put_balance(&public_key, &balance)
    }

    fn put_balance(&mut self, public_key: &PublicKey, balance: &Balance) -> Result<(), NdError> {
        trace!(
            "{}: Setting balance to {} for {}",
            self,
            balance,
            public_key
        );
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
        trace!("{}: {} is paying {} coins", self, requester_id, cost);
        match self.withdraw(requester_key, cost) {
            Ok(()) => Some(()),
            Err(error) => {
                trace!("{}: Unable to withdraw {} coins: {}", self, cost, error);
                self.messaging
                    .respond_to_client(message_id, request.error_response(error));
                None
            }
        }
    }

    fn balance<K: balance::Key>(&self, key: &K) -> Option<Coins> {
        self.balances.get(key).map(|balance| balance.coins)
    }

    fn create_balance(
        &mut self,
        requester: &PublicId,
        owner_key: PublicKey,
        amount: Coins,
    ) -> Result<(), NdError> {
        let own_request = utils::own_key(requester)
            .map(|key| key == &owner_key)
            .unwrap_or(false);
        if !own_request && self.balances.exists(&owner_key) {
            info!(
                "{}: Failed to create balance for {:?}: already exists.",
                self, owner_key
            );

            Err(NdError::BalanceExists)
        } else {
            let balance = Balance { coins: amount };
            self.put_balance(&owner_key, &balance)?;
            Ok(())
        }
    }
}

impl Display for ClientHandler {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.id.name())
    }
}
