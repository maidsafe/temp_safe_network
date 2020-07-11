// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{
    chunk_store::{error::Error as ChunkStoreError, SequenceChunkStore},
    cmd::NodeCmd,
    node::Init,
    utils, Config, Result,
};
use safe_nd::{
    Error as NdError, Message, MessageId, MsgEnvelope, MsgSender, NodePublicId, PublicKey,
    QueryResponse, Result as NdResult, SData, SDataAction, SDataAddress, SDataEntry, SDataIndex,
    SDataOwner, SDataPermissions, SDataPrivPermissions, SDataPubPermissions, SDataUser,
    SDataWriteOp, SequenceRead, SequenceWrite,
};
use serde::Serialize;
use std::{
    cell::Cell,
    fmt::{self, Display, Formatter},
    rc::Rc,
};

pub(super) struct SequenceStorage {
    id: NodePublicId,
    chunks: SequenceChunkStore,
}

impl SequenceStorage {
    pub(super) fn new(
        id: NodePublicId,
        config: &Config,
        total_used_space: &Rc<Cell<u64>>,
        init_mode: Init,
    ) -> Result<Self> {
        let root_dir = config.root_dir()?;
        let max_capacity = config.max_capacity();
        let chunks = SequenceChunkStore::new(
            &root_dir,
            max_capacity,
            Rc::clone(total_used_space),
            init_mode,
        )?;
        Ok(Self { id, chunks })
    }

    pub(super) fn read(
        &self,
        read: &SequenceRead,
        msg_id: MessageId,
        origin: MsgSender,
    ) -> Option<NodeCmd> {
        use SequenceRead::*;
        match read {
            Get(address) => self.get(*address, msg_id, origin),
            GetRange { address, range } => self.get_range(*address, *range, msg_id, origin),
            GetLastEntry(address) => self.get_last_entry(*address, msg_id, origin),
            GetOwner(address) => self.get_owner(*address, msg_id, origin),
            GetUserPermissions { address, user } => {
                self.get_user_permissions(*address, *user, msg_id, origin)
            }
            GetPermissions(address) => self.get_permissions(*address, msg_id, origin),
        }
    }

    pub(super) fn write(
        &mut self,
        write: SequenceWrite,
        msg_id: MessageId,
        origin: MsgSender,
    ) -> Option<NodeCmd> {
        use SequenceWrite::*;
        match write {
            New(data) => self.store(&data, msg_id, origin),
            Edit(operation) => self.edit(operation, msg_id, origin),
            Delete(address) => self.delete(address, msg_id, origin),
            SetOwner(operation) => self.set_owner(operation, msg_id, origin),
            SetPubPermissions(operation) => self.set_public_permissions(operation, msg_id, origin),
            SetPrivPermissions(operation) => {
                self.set_private_permissions(operation, msg_id, origin)
            }
        }
    }

    fn store(&mut self, data: &SData, msg_id: MessageId, origin: MsgSender) -> Option<NodeCmd> {
        let result = if self.chunks.has(data.address()) {
            Err(NdError::DataExists)
        } else {
            self.chunks
                .put(&data)
                .map_err(|error| error.to_string().into())
        };
        self.ok_or_error(result, msg_id, origin)
    }

    fn get(&self, address: SDataAddress, msg_id: MessageId, origin: MsgSender) -> Option<NodeCmd> {
        let result = self.get_chunk(address, SDataAction::Read, msg_id, origin);
        self.wrap(Message::QueryResponse {
            response: QueryResponse::GetSData(result),
            id: MessageId::new(),
            query_origin: origin.address(),
            correlation_id: msg_id,
        })
    }

    fn get_chunk(
        &self,
        address: SDataAddress,
        action: SDataAction,
        origin: MsgSender,
    ) -> Result<SData, NdError> {
        //let requester_key = utils::own_key(requester).ok_or(NdError::AccessDenied)?;
        let data = self.chunks.get(&address).map_err(|error| match error {
            ChunkStoreError::NoSuchChunk => NdError::NoSuchData,
            _ => error.to_string().into(),
        })?;
        data.check_permission(action, origin.id())?;
        Ok(data)
    }

    fn delete(
        &mut self,
        address: SDataAddress,
        msg_id: MessageId,
        origin: MsgSender,
    ) -> Option<NodeCmd> {
        //let requester_pk = *utils::own_key(&requester)?;
        let result = self
            .chunks
            .get(&address)
            .map_err(|error| match error {
                ChunkStoreError::NoSuchChunk => NdError::NoSuchData,
                error => error.to_string().into(),
            })
            .and_then(|sdata| {
                // TODO - SData::check_permission() doesn't support Delete yet in safe-nd
                if sdata.address().is_pub() {
                    Err(NdError::InvalidOperation)
                } else {
                    sdata.check_is_last_owner(origin.id())
                }
            })
            .and_then(|_| {
                self.chunks
                    .delete(&address)
                    .map_err(|error| error.to_string().into())
            });

        self.ok_or_error(result, msg_id, origin)
    }

    fn get_range(
        &self,
        address: SDataAddress,
        range: (SDataIndex, SDataIndex),
        msg_id: MessageId,
        origin: MsgSender,
    ) -> Option<NodeCmd> {
        let result = self
            .get_chunk(address, SDataAction::Read, msg_id, origin)
            .and_then(|sdata| sdata.in_range(range.0, range.1).ok_or(NdError::NoSuchEntry));
        self.wrap(Message::QueryResponse {
            response: QueryResponse::GetSDataRange(result),
            id: MessageId::new(),
            query_origin: origin.address(),
            correlation_id: msg_id,
        })
    }

    fn get_last_entry(
        &self,
        address: SDataAddress,
        msg_id: MessageId,
        origin: MsgSender,
    ) -> Option<NodeCmd> {
        let result = self
            .get_chunk(address, SDataAction::Read, msg_id, origin)
            .and_then(|sdata| match sdata.last_entry() {
                Some(entry) => Ok((sdata.entries_index() - 1, entry.to_vec())),
                None => Err(NdError::NoSuchEntry),
            });
        self.wrap(Message::QueryResponse {
            response: QueryResponse::GetSDataLastEntry(result),
            id: MessageId::new(),
            query_origin: origin.address(),
            correlation_id: msg_id,
        })
    }

    fn get_owner(
        &self,
        address: SDataAddress,
        msg_id: MessageId,
        origin: MsgSender,
    ) -> Option<NodeCmd> {
        let result = self
            .get_chunk(address, SDataAction::Read, msg_id, origin)
            .and_then(|sdata| {
                let index = sdata.owners_index() - 1;
                sdata.owner(index).cloned().ok_or(NdError::InvalidOwners)
            });
        self.wrap(Message::QueryResponse {
            response: QueryResponse::GetSDataOwner(result),
            id: MessageId::new(),
            query_origin: origin.address(),
            correlation_id: msg_id,
        })
    }

    fn get_user_permissions(
        &self,
        address: SDataAddress,
        user: SDataUser,
        msg_id: MessageId,
        origin: MsgSender,
    ) -> Option<NodeCmd> {
        let result = self
            .get_chunk(address, SDataAction::Read, msg_id, origin)
            .and_then(|sdata| {
                let index = sdata.permissions_index() - 1;
                sdata.user_permissions(user, index)
            });
        self.wrap(Message::QueryResponse {
            response: QueryResponse::GetSDataUserPermissions(result),
            id: MessageId::new(),
            query_origin: origin.address(),
            correlation_id: msg_id,
        })
    }

    fn get_permissions(
        &self,
        address: SDataAddress,
        msg_id: MessageId,
        origin: MsgSender,
    ) -> Option<NodeCmd> {
        let result = self
            .get_chunk(address, SDataAction::Read)
            .and_then(|sdata| {
                let index = sdata.permissions_index() - 1;
                let res = if sdata.is_pub() {
                    SDataPermissions::from(sdata.pub_permissions(index)?.clone())
                } else {
                    SDataPermissions::from(sdata.priv_permissions(index)?.clone())
                };
                Ok(res)
            });
        self.wrap(Message::QueryResponse {
            response: QueryResponse::GetSDataPermissions(result),
            id: MessageId::new(),
            query_origin: origin.address(),
            correlation_id: msg_id,
        })
    }

    fn set_public_permissions(
        &mut self,
        write_op: SDataWriteOp<SDataPubPermissions>,
        msg_id: MessageId,
        origin: MsgSender,
    ) -> Option<NodeCmd> {
        let address = write_op.address;
        let result = self.edit_chunk(
            address,
            SDataAction::ManagePermissions,
            origin,
            move |mut sdata| sdata.apply_crdt_pub_perms_op(write_op.crdt_op),
        );
        self.ok_or_error(result, msg_id, origin)
    }

    fn set_private_permissions(
        &mut self,
        write_op: SDataWriteOp<SDataPrivPermissions>,
        msg_id: MessageId,
        origin: MsgSender,
    ) -> Option<NodeCmd> {
        let address = write_op.address;
        let result = self.edit_chunk(
            address,
            SDataAction::ManagePermissions,
            origin,
            move |mut sdata| sdata.apply_crdt_priv_perms_op(write_op.crdt_op),
        );
        self.ok_or_error(result, msg_id, origin)
    }

    fn set_owner(
        &mut self,
        write_op: SDataWriteOp<SDataOwner>,
        msg_id: MessageId,
        origin: MsgSender,
    ) -> Option<NodeCmd> {
        let address = write_op.address;
        let result = self.edit_chunk(
            address,
            SDataAction::ManagePermissions,
            origin,
            move |mut sdata| {
                sdata.apply_crdt_owner_op(write_op.crdt_op);
                Ok(())
            },
        );
        self.ok_or_error(result, msg_id, origin)
    }

    fn edit(
        &mut self,
        write_op: SDataWriteOp<SDataEntry>,
        msg_id: MessageId,
        origin: MsgSender,
    ) -> Option<NodeCmd> {
        let address = write_op.address;
        let result = self.edit_chunk(address, SDataAction::Append, origin, move |mut sdata| {
            sdata.apply_crdt_op(write_op.crdt_op);
            Ok(())
        });
        self.ok_or_error(result, msg_id, origin)
    }

    fn edit_chunk<F>(
        &mut self,
        address: SDataAddress,
        action: SDataAction,
        origin: MsgSender,
        write_fn: F,
    ) -> NdResult<SData>
    where
        F: FnOnce(SData) -> NdResult<()>,
    {
        self.get_chunk(address, action, origin)
            .and_then(write_fn)
            .and_then(move |sdata| {
                self.chunks
                    .put(&sdata)
                    .map_err(|error| error.to_string().into())
            })
    }

    fn set_proxy(&self, msg: &mut MsgEnvelope) {
        // origin signs the message, while proxies sign the envelope
        msg.add_proxy(self.sign(msg))
    }

    fn ok_or_error(
        &self,
        result: Result<()>,
        msg_id: MessageId,
        origin: MsgSender,
    ) -> Option<NodeCmd> {
        let error = match result {
            Ok(()) => return None,
            Err(error) => error,
        };
        self.wrap(Message::CmdError {
            id: MessageId::new(),
            error: CmdError::Data(error),
            correlation_id: msg_id,
            cmd_origin: origin,
        })
    }

    fn wrap(&self, message: Message) -> Option<NodeCmd> {
        let msg = MsgEnvelope {
            message,
            origin: self.sign(message),
            proxies: Default::default(),
        };
        Some(NodeCmd::SendToSection(msg))
    }

    fn sign<T: Serialize>(&self, data: &T) -> MsgSender {
        let signature = &utils::sign(self.routing.borrow(), &utils::serialise(data));
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

impl Display for SequenceStorage {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.id.name())
    }
}
