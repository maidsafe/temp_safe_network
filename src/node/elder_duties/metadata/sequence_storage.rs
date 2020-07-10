// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{
    chunk_store::{error::Error as ChunkStoreError, SequenceChunkStore},
    cmd::{NodeCmd, MetadataCmd},
    msg::Message,
    node::Init,
    utils, Config, Result,
};

use safe_nd::{
    Error as NdError, MessageId, NodePublicId, PublicId, Result as NdResult, SData,
    SDataAction, SDataAddress, SDataEntry, SDataIndex, SDataOwner, SDataPermissions,
    SDataPrivPermissions, SDataPubPermissions, SDataUser, SDataWriteOp, SequenceRead,
    SequenceWrite,
};

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
        requester: PublicId,
        read: &SequenceRead,
        message_id: MessageId,
    ) -> Option<NodeCmd> {
        use SequenceRead::*;
        match read {
            Get(address) => self.get(requester, *address, message_id),
            GetRange { address, range } => self.get_range(requester, *address, *range, message_id),
            GetLastEntry(address) => self.get_last_entry(requester, *address, message_id),
            GetOwner(address) => self.get_owner(requester, *address, message_id),
            GetUserPermissions { address, user } => {
                self.get_user_permissions(requester, *address, *user, message_id)
            }
            GetPermissions(address) => self.get_permissions(requester, *address, message_id),
        }
    }

    pub(super) fn write(
        &mut self,
        requester: PublicId,
        write: SequenceWrite,
        message_id: MessageId,
    ) -> Option<NodeCmd> {
        use SequenceWrite::*;
        match write {
            New(data) => self.store(requester, &data, message_id),
            Edit(operation) => self.edit(&requester, operation, message_id),
            Delete(address) => self.delete(requester, address, message_id),
            SetOwner(operation) => self.set_owner(&requester, operation, message_id),
            SetPubPermissions(operation) => {
                self.set_public_permissions(&requester, operation, message_id)
            }
            SetPrivPermissions(operation) => {
                self.set_private_permissions(&requester, operation, message_id)
            }
        }
    }

    fn store(
        &mut self,
        requester: PublicId,
        data: &SData,
        message_id: MessageId,
    ) -> Option<NodeCmd> {
        let result = if self.chunks.has(data.address()) {
            Err(NdError::DataExists)
        } else {
            self.chunks
                .put(&data)
                .map_err(|error| error.to_string().into())
        };

        wrap(MetadataCmd::RespondToGateway {
            sender: *data.name(),
            msg: Message::Response {
                requester,
                response: Response::Write(result),
                message_id,
                proof: None,
            },
        })
    }

    fn get(
        &self,
        requester: PublicId,
        address: SDataAddress,
        message_id: MessageId,
    ) -> Option<NodeCmd> {
        let result = self.get_chunk(&requester, address, SDataAction::Read);

        wrap(MetadataCmd::RespondToGateway {
            sender: *address.name(),
            msg: Message::Response {
                requester,
                response: Response::GetSData(result),
                message_id,
                proof: None,
            },
        })
    }

    fn get_chunk(
        &self,
        requester: &PublicId,
        address: SDataAddress,
        action: SDataAction,
    ) -> Result<SData, NdError> {
        let requester_key = utils::own_key(requester).ok_or(NdError::AccessDenied)?;
        let data = self.chunks.get(&address).map_err(|error| match error {
            ChunkStoreError::NoSuchChunk => NdError::NoSuchData,
            _ => error.to_string().into(),
        })?;

        data.check_permission(action, *requester_key)?;
        Ok(data)
    }

    fn delete(
        &mut self,
        requester: PublicId,
        address: SDataAddress,
        message_id: MessageId,
    ) -> Option<NodeCmd> {
        let requester_pk = *utils::own_key(&requester)?;
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
                    sdata.check_is_last_owner(requester_pk)
                }
            })
            .and_then(|_| {
                self.chunks
                    .delete(&address)
                    .map_err(|error| error.to_string().into())
            });

        wrap(MetadataCmd::RespondToGateway {
            sender: *address.name(),
            msg: Message::Response {
                requester,
                response: Response::Write(result),
                message_id,
                proof: None,
            },
        })
    }

    fn get_range(
        &self,
        requester: PublicId,
        address: SDataAddress,
        range: (SDataIndex, SDataIndex),
        message_id: MessageId,
    ) -> Option<NodeCmd> {
        let result = self
            .get_chunk(&requester, address, SDataAction::Read)
            .and_then(|sdata| sdata.in_range(range.0, range.1).ok_or(NdError::NoSuchEntry));

        wrap(MetadataCmd::RespondToGateway {
            sender: *address.name(),
            msg: Message::Response {
                requester,
                response: Response::GetSDataRange(result),
                message_id,
                proof: None,
            },
        })
    }

    fn get_last_entry(
        &self,
        requester: PublicId,
        address: SDataAddress,
        message_id: MessageId,
    ) -> Option<NodeCmd> {
        let result = self
            .get_chunk(&requester, address, SDataAction::Read)
            .and_then(|sdata| match sdata.last_entry() {
                Some(entry) => Ok((sdata.entries_index() - 1, entry.to_vec())),
                None => Err(NdError::NoSuchEntry),
            });

        wrap(MetadataCmd::RespondToGateway {
            sender: *address.name(),
            msg: Message::Response {
                requester,
                response: Response::GetSDataLastEntry(result),
                message_id,
                proof: None,
            },
        })
    }

    fn get_owner(
        &self,
        requester: PublicId,
        address: SDataAddress,
        message_id: MessageId,
    ) -> Option<NodeCmd> {
        let result = self
            .get_chunk(&requester, address, SDataAction::Read)
            .and_then(|sdata| {
                let index = sdata.owners_index() - 1;
                sdata.owner(index).cloned().ok_or(NdError::InvalidOwners)
            });

        wrap(MetadataCmd::RespondToGateway {
            sender: *address.name(),
            msg: Message::Response {
                requester,
                response: Response::GetSDataOwner(result),
                message_id,
                proof: None,
            },
        })
    }

    fn get_user_permissions(
        &self,
        requester: PublicId,
        address: SDataAddress,
        user: SDataUser,
        message_id: MessageId,
    ) -> Option<NodeCmd> {
        let result = self
            .get_chunk(&requester, address, SDataAction::Read)
            .and_then(|sdata| {
                let index = sdata.permissions_index() - 1;
                sdata.user_permissions(user, index)
            });

        wrap(MetadataCmd::RespondToGateway {
            sender: *address.name(),
            msg: Message::Response {
                requester,
                response: Response::GetSDataUserPermissions(result),
                message_id,
                proof: None,
            },
        })
    }

    fn get_permissions(
        &self,
        requester: PublicId,
        address: SDataAddress,
        message_id: MessageId,
    ) -> Option<NodeCmd> {
        let response = {
            let result = self
                .get_chunk(&requester, address, SDataAction::Read)
                .and_then(|sdata| {
                    let index = sdata.permissions_index() - 1;
                    let res = if sdata.is_pub() {
                        SDataPermissions::from(sdata.pub_permissions(index)?.clone())
                    } else {
                        SDataPermissions::from(sdata.priv_permissions(index)?.clone())
                    };

                    Ok(res)
                });
            Response::GetSDataPermissions(result)
        };

        wrap(MetadataCmd::RespondToGateway {
            sender: *address.name(),
            msg: Message::Response {
                requester,
                response,
                message_id,
                proof: None,
            },
        })
    }

    fn set_public_permissions(
        &mut self,
        requester: &PublicId,
        mutation_op: SDataWriteOp<SDataPubPermissions>,
        message_id: MessageId,
    ) -> Option<NodeCmd> {
        let address = mutation_op.address;
        self.edit_chunk(
            &requester,
            address,
            SDataAction::ManagePermissions,
            message_id,
            move |mut sdata| {
                sdata.apply_crdt_pub_perms_op(mutation_op.crdt_op)?;
                Ok(sdata)
            },
        )
    }

    fn set_private_permissions(
        &mut self,
        requester: &PublicId,
        mutation_op: SDataWriteOp<SDataPrivPermissions>,
        message_id: MessageId,
    ) -> Option<NodeCmd> {
        let address = mutation_op.address;
        self.edit_chunk(
            &requester,
            address,
            SDataAction::ManagePermissions,
            message_id,
            move |mut sdata| {
                sdata.apply_crdt_priv_perms_op(mutation_op.crdt_op)?;
                Ok(sdata)
            },
        )
    }

    fn set_owner(
        &mut self,
        requester: &PublicId,
        mutation_op: SDataWriteOp<SDataOwner>,
        message_id: MessageId,
    ) -> Option<NodeCmd> {
        let address = mutation_op.address;
        self.edit_chunk(
            &requester,
            address,
            SDataAction::ManagePermissions,
            message_id,
            move |mut sdata| {
                sdata.apply_crdt_owner_op(mutation_op.crdt_op);
                Ok(sdata)
            },
        )
    }

    fn edit(
        &mut self,
        requester: &PublicId,
        mutation_op: SDataWriteOp<SDataEntry>,
        message_id: MessageId,
    ) -> Option<NodeCmd> {
        let address = mutation_op.address;
        self.edit_chunk(
            &requester,
            address,
            SDataAction::Append,
            message_id,
            move |mut sdata| {
                sdata.apply_crdt_op(mutation_op.crdt_op);
                Ok(sdata)
            },
        )
    }

    fn edit_chunk<F>(
        &mut self,
        requester: &PublicId,
        address: SDataAddress,
        action: SDataAction,
        message_id: MessageId,
        mutation_fn: F,
    ) -> Option<NodeCmd>
    where
        F: FnOnce(SData) -> NdResult<SData>,
    {
        let result = self
            .get_chunk(requester, address, action)
            .and_then(mutation_fn)
            .and_then(move |sdata| {
                self.chunks
                    .put(&sdata)
                    .map_err(|error| error.to_string().into())
            });

        wrap(MetadataCmd::RespondToGateway {
            sender: *address.name(),
            msg: Message::Response {
                requester: requester.clone(),
                response: Response::Write(result),
                message_id,
                proof: None,
            },
        })
    }
}

fn wrap(cmd: MetadataCmd) -> Option<NodeCmd> {
    Some(NodeCmd::Metadata(cmd))
}

impl Display for SequenceStorage {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.id.name())
    }
}
