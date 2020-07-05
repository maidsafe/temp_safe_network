// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{
    chunk_store::{error::Error as ChunkStoreError, AccountChunkStore},
    cmd::{ElderCmd, MetadataCmd},
    msg::Message,
    node::Init,
    utils, Config, Result,
};
use safe_nd::{
    Account, AccountRead, AccountWrite, Error as NdError, MessageId, NodePublicId, PublicId,
    PublicKey, Response, Result as NdResult, XorName,
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

fn wrap(cmd: MetadataCmd) -> Option<ElderCmd> {
    Some(ElderCmd::Metadata(cmd))
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
        client: PublicId,
        read: &AccountRead,
        message_id: MessageId,
    ) -> Option<ElderCmd> {
        use AccountRead::*;
        match read {
            Get(ref address) => self.get(client, address, message_id),
        }
    }

    fn get(
        &self,
        requester: PublicId,
        address: &XorName,
        message_id: MessageId,
    ) -> Option<ElderCmd> {
        let result = self
            .account(utils::own_key(&requester)?, address)
            .map(Account::into_data_and_signature);

        wrap(MetadataCmd::RespondToGateway {
            sender: *address,
            msg: Message::Response {
                requester,
                response: Response::GetLoginPacket(result),
                message_id,
                proof: None,
            },
        })
    }

    pub(super) fn write(
        &mut self,
        client: PublicId,
        write: AccountWrite,
        message_id: MessageId,
    ) -> Option<ElderCmd> {
        use AccountWrite::*;
        match write {
            New(ref account) => self.create(client, account, message_id),
            Update(updated_account) => self.update(client, &updated_account, message_id),
        }
    }

    fn create(
        &mut self,
        requester: PublicId,
        account: &Account,
        message_id: MessageId,
    ) -> Option<ElderCmd> {
        let result = if self.chunks.has(account.address()) {
            Err(NdError::LoginPacketExists)
        } else {
            self.chunks
                .put(account)
                .map_err(|error| error.to_string().into())
        };

        wrap(MetadataCmd::RespondToGateway {
            sender: *account.address(),
            msg: Message::Response {
                requester,
                response: Response::Write(result),
                message_id,
                proof: None,
            },
        })
    }

    fn update(
        &mut self,
        requester: PublicId,
        updated_account: &Account,
        message_id: MessageId,
    ) -> Option<ElderCmd> {
        let result = self
            .account(utils::own_key(&requester)?, updated_account.address())
            .and_then(|_existing_login_packet| {
                if !updated_account.size_is_valid() {
                    return Err(NdError::ExceededSize);
                }
                self.chunks
                    .put(&updated_account)
                    .map_err(|err| err.to_string().into())
            });

        wrap(MetadataCmd::RespondToGateway {
            sender: *updated_account.address(),
            msg: Message::Response {
                requester,
                response: Response::Write(result),
                message_id,
                proof: None,
            },
        })
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
}

impl Display for AccountStorage {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.id.name())
    }
}
