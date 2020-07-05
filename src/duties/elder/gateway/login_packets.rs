// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{
    chunk_store::{error::Error as ChunkStoreError, LoginPacketChunkStore},
    cmd::{ConsensusAction, GatewayCmd},
    utils,
};
use safe_nd::{
    Account, AccountRead, AccountWrite, DebitAgreementProof, Error as NdError, MessageId,
    NodePublicId, NodeRequest, PublicId, PublicKey, Request, Response, Result as NdResult, Write,
    XorName,
};
use std::fmt::{self, Display, Formatter};

pub(super) struct LoginPackets {
    id: NodePublicId,
    login_packets: LoginPacketChunkStore,
}

impl LoginPackets {
    pub fn new(id: NodePublicId, login_packets: LoginPacketChunkStore) -> Self {
        Self { id, login_packets }
    }

    // on client request
    pub fn read(
        &mut self,
        client: PublicId,
        read: AccountRead,
        message_id: MessageId,
    ) -> Option<GatewayCmd> {
        use AccountRead::*;
        match read {
            Get(ref address) => self.get(client, address, message_id),
        }
    }

    // on client request
    pub fn initiate_write(
        &mut self,
        client: PublicId,
        write: AccountWrite,
        message_id: MessageId,
        debit_proof: DebitAgreementProof,
    ) -> Option<GatewayCmd> {
        use AccountWrite::*;
        match write {
            New(login_packet) => {
                self.initiate_creation(client, login_packet, message_id, debit_proof)
            }
            Update(updated_login_packet) => {
                self.initiate_update(client, updated_login_packet, message_id, debit_proof)
            }
        }
    }

    // client query
    fn get(
        &mut self,
        client: PublicId,
        address: &XorName,
        message_id: MessageId,
    ) -> Option<GatewayCmd> {
        let result = self
            .account(utils::own_key(&client)?, address)
            .map(Account::into_data_and_signature);
        Some(GatewayCmd::RespondToClient {
            message_id,
            response: Response::GetLoginPacket(result),
        })
    }

    // on client request
    fn initiate_creation(
        &mut self,
        client_public_id: PublicId,
        account: Account,
        message_id: MessageId,
        _debit_proof: DebitAgreementProof,
    ) -> Option<GatewayCmd> {
        if !account.size_is_valid() {
            return Some(GatewayCmd::RespondToClient {
                message_id,
                response: Response::Write(Err(NdError::ExceededSize)),
            });
        }

        let request = Self::wrap(AccountWrite::New(account));

        Some(GatewayCmd::VoteFor(ConsensusAction::Forward {
            request,
            client_public_id,
            message_id,
            //debit_proof,
        }))
    }

    // on consensus
    pub fn finalise_write(
        &mut self,
        client: PublicId,
        write: AccountWrite,
        message_id: MessageId,
    ) -> Option<GatewayCmd> {
        use AccountWrite::*;
        match write {
            New(ref account) => self.finalise_creation(client, account, message_id),
            Update(updated_account) => self.finalise_update(client, &updated_account, message_id),
        }
    }

    // on consensus
    fn finalise_creation(
        &mut self,
        requester: PublicId,
        account: &Account,
        message_id: MessageId,
    ) -> Option<GatewayCmd> {
        let result = if self.login_packets.has(account.address()) {
            Err(NdError::LoginPacketExists)
        } else {
            self.login_packets
                .put(account)
                .map_err(|error| error.to_string().into())
        };

        Some(GatewayCmd::RespondToClient {
            message_id,
            response: Response::Write(result),
        })
    }

    // on client request
    fn initiate_update(
        &mut self,
        client_public_id: PublicId,
        updated_account: Account,
        message_id: MessageId,
        _debit_proof: DebitAgreementProof,
    ) -> Option<GatewayCmd> {
        Some(GatewayCmd::VoteFor(ConsensusAction::Forward {
            request: Self::wrap(AccountWrite::Update(updated_account)),
            client_public_id,
            message_id,
            //debit_proof,
        }))
    }

    // on consensus
    fn finalise_update(
        &mut self,
        requester: PublicId,
        updated_account: &Account,
        message_id: MessageId,
    ) -> Option<GatewayCmd> {
        let result = self
            .account(utils::own_key(&requester)?, updated_account.address())
            .and_then(|_existing_login_packet| {
                if !updated_account.size_is_valid() {
                    return Err(NdError::ExceededSize);
                }
                self.login_packets
                    .put(&updated_account)
                    .map_err(|err| err.to_string().into())
            });
        Some(GatewayCmd::RespondToClient {
            message_id,
            response: Response::Write(result),
        })
    }

    fn account(&self, requester_pub_key: &PublicKey, address: &XorName) -> NdResult<Account> {
        self.login_packets
            .get(address)
            .map_err(|e| match e {
                ChunkStoreError::NoSuchChunk => NdError::NoSuchLoginPacket,
                error => error.to_string().into(),
            })
            .and_then(|account| {
                if account.owner() == requester_pub_key {
                    Ok(account)
                } else {
                    Err(NdError::AccessDenied)
                }
            })
    }

    fn wrap(write: AccountWrite) -> Request {
        Request::Node(NodeRequest::Write(Write::Account(write)))
    }
}

impl Display for LoginPackets {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.id.name())
    }
}
