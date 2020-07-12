// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{
    chunk_store::{error::Error as ChunkStoreError, AccountChunkStore},
    cmd::OutboundMsg,
    node::msg_decisions::ElderMsgDecisions,
    node::Init,
    Config, Result,
};
use safe_nd::{
    Account, AccountRead, AccountWrite, CmdError, Error as NdError, Message, MessageId, MsgSender,
    PublicKey, QueryResponse, Result as NdResult, XorName,
};
use std::{
    cell::Cell,
    fmt::{self, Display, Formatter},
    rc::Rc,
};

pub(super) struct AccountStorage {
    chunks: AccountChunkStore,
    decisions: ElderMsgDecisions,
}

impl AccountStorage {
    pub fn new(
        config: &Config,
        total_used_space: &Rc<Cell<u64>>,
        init_mode: Init,
        decisions: ElderMsgDecisions,
    ) -> Result<Self> {
        let root_dir = config.root_dir()?;
        let max_capacity = config.max_capacity();
        let chunks = AccountChunkStore::new(
            root_dir,
            max_capacity,
            Rc::clone(total_used_space),
            init_mode,
        )?;
        Ok(Self { chunks, decisions })
    }

    pub(super) fn read(
        &self,
        read: &AccountRead,
        msg_id: MessageId,
        origin: MsgSender,
    ) -> Option<OutboundMsg> {
        use AccountRead::*;
        match read {
            Get(ref address) => self.get(address, msg_id, origin),
        }
    }

    fn get(&self, address: &XorName, msg_id: MessageId, origin: MsgSender) -> Option<OutboundMsg> {
        let result = self
            .account(origin.id(), address)
            .map(Account::into_data_and_signature);
        self.decisions.send(Message::QueryResponse {
            id: MessageId::new(),
            response: QueryResponse::GetAccount(result),
            correlation_id: msg_id,
            query_origin: origin.address(),
        })
    }

    pub(super) fn write(
        &mut self,
        write: AccountWrite,
        msg_id: MessageId,
        origin: MsgSender,
    ) -> Option<OutboundMsg> {
        use AccountWrite::*;
        match write {
            New(ref account) => self.create(account, msg_id, origin),
            Update(updated_account) => self.update(&updated_account, msg_id, origin),
        }
    }

    fn create(
        &mut self,
        account: &Account,
        msg_id: MessageId,
        origin: MsgSender,
    ) -> Option<OutboundMsg> {
        let result = if self.chunks.has(account.address()) {
            Err(NdError::LoginPacketExists)
        } else if account.owner() != origin.id() {
            Err(NdError::InvalidOwners)
        } else {
            // also check the signature
            self.chunks
                .put(account)
                .map_err(|error| error.to_string().into())
        };
        self.ok_or_error(result, msg_id, origin)
    }

    fn update(
        &mut self,
        updated_account: &Account,
        msg_id: MessageId,
        origin: MsgSender,
    ) -> Option<OutboundMsg> {
        let result = self
            .account(origin.id(), updated_account.address())
            .and_then(|existing_account| {
                if !updated_account.size_is_valid() {
                    Err(NdError::ExceededSize)
                } else if updated_account.owner() != existing_account.owner() {
                    Err(NdError::InvalidOwners)
                } else {
                    // also check the signature
                    self.chunks
                        .put(&updated_account)
                        .map_err(|err| err.to_string().into())
                }
            });
        self.ok_or_error(result, msg_id, origin)
    }

    fn account(&self, requester_pub_key: &PublicKey, address: &XorName) -> NdResult<Account> {
        self.chunks
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

    fn ok_or_error(
        &self,
        result: NdResult<()>,
        msg_id: MessageId,
        origin: MsgSender,
    ) -> Option<OutboundMsg> {
        if let Err(error) = result {
            return self.decisions.send(Message::CmdError {
                id: MessageId::new(),
                error: CmdError::Data(error),
                correlation_id: msg_id,
                cmd_origin: origin.address(),
            });
        }
        None
    }
}

impl Display for AccountStorage {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", "AccountStorage")
    }
}
