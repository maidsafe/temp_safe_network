// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{
    chunk_store::{error::Error as ChunkStoreError, AccountChunkStore},
    cmd::NodeCmd,
    node::Init,
    utils, Config, Result,
};
use safe_nd::{
    Account, AccountRead, AccountWrite, Error as NdError, MessageId, NodePublicId, PublicId,
    PublicKey, Result as NdResult, XorName, QueryResponse, Message, MsgEnvelope, MsgSender,
    Signature, ElderDuty, Duty,
};
use std::{
    cell::Cell,
    fmt::{self, Display, Formatter},
    rc::Rc,
};

pub(super) struct AccountStorage {
    id: NodePublicId,
    chunks: AccountChunkStore,
}

impl AccountStorage {
    pub fn new(
        id: NodePublicId,
        config: &Config,
        total_used_space: &Rc<Cell<u64>>,
        init_mode: Init,
    ) -> Result<Self> {
        let root_dir = config.root_dir()?;
        let max_capacity = config.max_capacity();
        let chunks = AccountChunkStore::new(
            root_dir,
            max_capacity,
            Rc::clone(total_used_space),
            init_mode,
        )?;
        Ok(Self { id, chunks })
    }

    pub(super) fn read(
        &self,
        read: &AccountRead,
        msg_id: MessageId,
        origin: MsgSender,
    ) -> Option<NodeCmd> {
        use AccountRead::*;
        match read {
            Get(ref address) => self.get(address, msg_id, origin),
        }
    }

    fn get(
        &self,
        address: &XorName,
        msg_id: MessageId,
        origin: MsgSender,
    ) -> Option<NodeCmd> {
        let result = self
            .account(origin.id(), address)
            .map(Account::into_data_and_signature);
        let message = Message::QueryResponse {
            id: MessageId::new(),
            response: QueryResponse::GetAccount(result),
            correlation_id: msg_id,
            query_origin: origin,
        };
        self.wrap(message)
    }

    pub(super) fn write(
        &mut self,
        write: AccountWrite,
        msg_id: MessageId,
        origin: MsgSender,
    ) -> Option<NodeCmd> {
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
    ) -> Option<NodeCmd> {
        let result = if self.chunks.has(account.address()) {
            Err(NdError::LoginPacketExists)
        } else if account.owner != origin.id() {
            Err(NdError::InvalidOwners)
        } else { // also check the signature
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
    ) -> Option<NodeCmd> {
        let result = self
            .account(origin.id(), updated_account.address())
            .and_then(|existing_account| {
                if !updated_account.size_is_valid() {
                    Err(NdError::ExceededSize)
                } else if updated_account.owner != existing_account.owner {
                    Err(NdError::InvalidOwners)
                } else { // also check the signature
                    self.chunks
                        .put(&updated_account)
                        //.map_err(|err| err.to_string().into())
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

    fn ok_or_error(&self, result: Result<()>, msg_id: MessageId, origin: MsgSender) -> Option<NodeCmd> {
        let error = match result {
            Ok(()) => return None,
            Err(error) => error,
        };
        let message = Message::CmdError {
            id: MessageId::new(),
            error: CmdError::Data(error),
            correlation_id: msg_id,
            cmd_origin: origin,
        };
        self.wrap(message)
    }

    fn wrap(&self, message: Message) -> Option<NodeCmd> {
        let msg = MsgEnvelope {
            message,
            origin: self.sign(message),
            proxies: Default::default(),
        };
        Some(NodeCmd::SendToSection(msg))
    }

    fn sign(&self, message: Message) -> MsgSender {
        let signature = &utils::sign(self.routing.borrow(), &utils::serialise(&message));
        MsgSender::Node {
            id: self.public_key(),
            duty: Duty::Elder(ElderDuty::Metadata),
            signature,
        }
    }
    
    fn public_key(&self) -> PublicKey {
        PublicKey::Bls(self.id.public_id().bls_public_key())
    }
}

impl Display for AccountStorage {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.id.name())
    }
}
