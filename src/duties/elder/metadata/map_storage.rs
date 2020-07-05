// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{
    chunk_store::{error::Error as ChunkStoreError, MutableChunkStore},
    cmd::ElderCmd,
    msg::Message,
    node::Init,
    utils, Config, Result,
};
use log::error;

use safe_nd::{
    Error as NdError, MData, MDataAction, MDataAddress, MDataEntryActions, MDataPermissionSet,
    MDataValue, MapRead, MapWrite, MessageId, NodePublicId, PublicId, PublicKey, Response,
    Result as NdResult,
};

use std::{
    cell::Cell,
    fmt::{self, Display, Formatter},
    rc::Rc,
};

pub(super) struct MapStorage {
    id: NodePublicId,
    chunks: MutableChunkStore,
}

impl MapStorage {
    pub(super) fn new(
        id: NodePublicId,
        config: &Config,
        total_used_space: &Rc<Cell<u64>>,
        init_mode: Init,
    ) -> Result<Self> {
        let root_dir = config.root_dir()?;
        let max_capacity = config.max_capacity();
        let chunks = MutableChunkStore::new(
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
        read: &MapRead,
        message_id: MessageId,
    ) -> Option<ElderCmd> {
        use MapRead::*;
        match read {
            Get(address) => self.get(requester, *address, message_id),
            GetValue { address, ref key } => self.get_value(requester, *address, key, message_id),
            GetShell(address) => self.get_shell(requester, *address, message_id),
            GetVersion(address) => self.get_version(requester, *address, message_id),
            ListEntries(address) => self.list_entries(requester, *address, message_id),
            ListKeys(address) => self.list_keys(requester, *address, message_id),
            ListValues(address) => self.list_values(requester, *address, message_id),
            ListPermissions(address) => self.list_permissions(requester, *address, message_id),
            ListUserPermissions { address, user } => {
                self.list_user_permissions(requester, *address, *user, message_id)
            }
        }
    }

    pub(super) fn write(
        &mut self,
        requester: PublicId,
        write: MapWrite,
        message_id: MessageId,
    ) -> Option<ElderCmd> {
        use MapWrite::*;
        match write {
            New(data) => self.create(requester, &data, message_id),
            Delete(address) => self.delete(requester, address, message_id),
            SetUserPermissions {
                address,
                user,
                ref permissions,
                version,
            } => self.set_user_permissions(
                requester,
                address,
                user,
                permissions,
                version,
                message_id,
            ),
            DelUserPermissions {
                address,
                user,
                version,
            } => self.delete_user_permissions(requester, address, user, version, message_id),
            Edit { address, changes } => self.edit_entries(requester, address, changes, message_id),
        }
    }

    /// Get `MData` from the chunk store and check permissions.
    /// Returns `Some(Result<..>)` if the flow should be continued, returns
    /// `None` if there was a logic error encountered and the flow should be
    /// terminated.
    fn get_chunk(
        &self,
        address: &MDataAddress,
        requester: &PublicId,
        action: MDataAction,
    ) -> Option<NdResult<MData>> {
        let requester_pk = if let Some(pk) = utils::own_key(&requester) {
            pk
        } else {
            error!("Logic error: requester {:?} must not be Node", requester);
            return None;
        };

        Some(
            self.chunks
                .get(&address)
                .map_err(|e| match e {
                    ChunkStoreError::NoSuchChunk => NdError::NoSuchData,
                    error => error.to_string().into(),
                })
                .and_then(move |mdata| {
                    mdata
                        .check_permissions(action, *requester_pk)
                        .map(move |_| mdata)
                }),
        )
    }

    /// Get MData from the chunk store, update it, and overwrite the stored chunk.
    fn edit_chunk<F>(
        &mut self,
        address: &MDataAddress,
        requester: PublicId,
        message_id: MessageId,
        mutation_fn: F,
    ) -> Option<ElderCmd>
    where
        F: FnOnce(MData) -> NdResult<MData>,
    {
        let result = self
            .chunks
            .get(address)
            .map_err(|e| match e {
                ChunkStoreError::NoSuchChunk => NdError::NoSuchData,
                error => error.to_string().into(),
            })
            .and_then(mutation_fn)
            .and_then(move |mdata| {
                self.chunks
                    .put(&mdata)
                    .map_err(|error| error.to_string().into())
            });

        Some(ElderCmd::RespondToGateway {
            sender: *address.name(),
            msg: Message::Response {
                requester,
                response: Response::Write(result),
                message_id,
                proof: None,
            },
        })
    }

    /// Put MData.
    fn create(
        &mut self,
        requester: PublicId,
        data: &MData,
        message_id: MessageId,
    ) -> Option<ElderCmd> {
        let result = if self.chunks.has(data.address()) {
            Err(NdError::DataExists)
        } else {
            self.chunks
                .put(&data)
                .map_err(|error| error.to_string().into())
        };

        Some(ElderCmd::RespondToGateway {
            sender: *data.name(),
            msg: Message::Response {
                requester,
                response: Response::Write(result),
                message_id,
                proof: None,
            },
        })
    }

    fn delete(
        &mut self,
        requester: PublicId,
        address: MDataAddress,
        message_id: MessageId,
    ) -> Option<ElderCmd> {
        let requester_pk = *utils::own_key(&requester)?;

        let result = self
            .chunks
            .get(&address)
            .map_err(|e| match e {
                ChunkStoreError::NoSuchChunk => NdError::NoSuchData,
                error => error.to_string().into(),
            })
            .and_then(move |mdata| {
                mdata.check_is_owner(requester_pk)?;

                self.chunks
                    .delete(&address)
                    .map_err(|error| error.to_string().into())
            });

        Some(ElderCmd::RespondToGateway {
            sender: *address.name(),
            msg: Message::Response {
                requester,
                response: Response::Write(result),
                message_id,
                proof: None,
            },
        })
    }

    /// Set MData user permissions.
    fn set_user_permissions(
        &mut self,
        requester: PublicId,
        address: MDataAddress,
        user: PublicKey,
        permissions: &MDataPermissionSet,
        version: u64,
        message_id: MessageId,
    ) -> Option<ElderCmd> {
        let requester_pk = *utils::own_key(&requester)?;

        self.edit_chunk(&address, requester, message_id, move |mut data| {
            data.check_permissions(MDataAction::ManagePermissions, requester_pk)?;
            data.set_user_permissions(user, permissions.clone(), version)?;
            Ok(data)
        })
    }

    /// Delete MData user permissions.
    fn delete_user_permissions(
        &mut self,
        requester: PublicId,
        address: MDataAddress,
        user: PublicKey,
        version: u64,
        message_id: MessageId,
    ) -> Option<ElderCmd> {
        let requester_pk = *utils::own_key(&requester)?;

        self.edit_chunk(&address, requester, message_id, move |mut data| {
            data.check_permissions(MDataAction::ManagePermissions, requester_pk)?;
            data.del_user_permissions(user, version)?;
            Ok(data)
        })
    }

    /// Edit MData.
    fn edit_entries(
        &mut self,
        requester: PublicId,
        address: MDataAddress,
        actions: MDataEntryActions,
        message_id: MessageId,
    ) -> Option<ElderCmd> {
        let requester_pk = *utils::own_key(&requester)?;

        self.edit_chunk(&address, requester, message_id, move |mut data| {
            data.mutate_entries(actions, requester_pk)?;
            Ok(data)
        })
    }

    /// Get entire MData.
    fn get(
        &self,
        requester: PublicId,
        address: MDataAddress,
        message_id: MessageId,
    ) -> Option<ElderCmd> {
        let result = self.get_chunk(&address, &requester, MDataAction::Read)?;

        Some(ElderCmd::RespondToGateway {
            sender: *address.name(),
            msg: Message::Response {
                requester,
                response: Response::GetMData(result),
                message_id,
                proof: None,
            },
        })
    }

    /// Get MData shell.
    fn get_shell(
        &self,
        requester: PublicId,
        address: MDataAddress,
        message_id: MessageId,
    ) -> Option<ElderCmd> {
        let result = self
            .get_chunk(&address, &requester, MDataAction::Read)?
            .map(|data| data.shell());

        Some(ElderCmd::RespondToGateway {
            sender: *address.name(),
            msg: Message::Response {
                requester,
                response: Response::GetMDataShell(result),
                message_id,
                proof: None,
            },
        })
    }

    /// Get MData version.
    fn get_version(
        &self,
        requester: PublicId,
        address: MDataAddress,
        message_id: MessageId,
    ) -> Option<ElderCmd> {
        let result = self
            .get_chunk(&address, &requester, MDataAction::Read)?
            .map(|data| data.version());

        Some(ElderCmd::RespondToGateway {
            sender: *address.name(),
            msg: Message::Response {
                requester,
                response: Response::GetMDataVersion(result),
                message_id,
                proof: None,
            },
        })
    }

    /// Get MData value.
    fn get_value(
        &self,
        requester: PublicId,
        address: MDataAddress,
        key: &[u8],
        message_id: MessageId,
    ) -> Option<ElderCmd> {
        let res = self.get_chunk(&address, &requester, MDataAction::Read)?;

        let response = Response::GetMDataValue(res.and_then(|data| {
            match data {
                MData::Seq(md) => md
                    .get(key)
                    .cloned()
                    .map(MDataValue::from)
                    .ok_or_else(|| NdError::NoSuchEntry),
                MData::Unseq(md) => md
                    .get(key)
                    .cloned()
                    .map(MDataValue::from)
                    .ok_or_else(|| NdError::NoSuchEntry),
            }
        }));

        Some(ElderCmd::RespondToGateway {
            sender: *address.name(),
            msg: Message::Response {
                requester,
                response,
                message_id,
                proof: None,
            },
        })
    }

    /// Get MData keys.
    fn list_keys(
        &self,
        requester: PublicId,
        address: MDataAddress,
        message_id: MessageId,
    ) -> Option<ElderCmd> {
        let result = self
            .get_chunk(&address, &requester, MDataAction::Read)?
            .map(|data| data.keys());

        Some(ElderCmd::RespondToGateway {
            sender: *address.name(),
            msg: Message::Response {
                requester,
                response: Response::ListMDataKeys(result),
                message_id,
                proof: None,
            },
        })
    }

    /// Get MData values.
    fn list_values(
        &self,
        requester: PublicId,
        address: MDataAddress,
        message_id: MessageId,
    ) -> Option<ElderCmd> {
        let res = self.get_chunk(&address, &requester, MDataAction::Read)?;

        let response = Response::ListMDataValues(res.and_then(|data| match data {
            MData::Seq(md) => Ok(md.values().into()),
            MData::Unseq(md) => Ok(md.values().into()),
        }));

        Some(ElderCmd::RespondToGateway {
            sender: *address.name(),
            msg: Message::Response {
                requester,
                response,
                message_id,
                proof: None,
            },
        })
    }

    /// Get MData entries.
    fn list_entries(
        &self,
        requester: PublicId,
        address: MDataAddress,
        message_id: MessageId,
    ) -> Option<ElderCmd> {
        let res = self.get_chunk(&address, &requester, MDataAction::Read)?;

        let response = Response::ListMDataEntries(res.and_then(|data| match data {
            MData::Seq(md) => Ok(md.entries().clone().into()),
            MData::Unseq(md) => Ok(md.entries().clone().into()),
        }));

        Some(ElderCmd::RespondToGateway {
            sender: *address.name(),
            msg: Message::Response {
                requester,
                response,
                message_id,
                proof: None,
            },
        })
    }

    /// Get MData permissions.
    fn list_permissions(
        &self,
        requester: PublicId,
        address: MDataAddress,
        message_id: MessageId,
    ) -> Option<ElderCmd> {
        let result = self
            .get_chunk(&address, &requester, MDataAction::Read)?
            .map(|data| data.permissions());

        Some(ElderCmd::RespondToGateway {
            sender: *address.name(),
            msg: Message::Response {
                requester,
                response: Response::ListMDataPermissions(result),
                message_id,
                proof: None,
            },
        })
    }

    /// Get MData user permissions.
    fn list_user_permissions(
        &self,
        requester: PublicId,
        address: MDataAddress,
        user: PublicKey,
        message_id: MessageId,
    ) -> Option<ElderCmd> {
        let result = self
            .get_chunk(&address, &requester, MDataAction::Read)?
            .and_then(|data| data.user_permissions(user).map(MDataPermissionSet::clone));

        Some(ElderCmd::RespondToGateway {
            sender: *address.name(),
            msg: Message::Response {
                requester,
                response: Response::ListMDataUserPermissions(result),
                message_id,
                proof: None,
            },
        })
    }
}

impl Display for MapStorage {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.id.name())
    }
}
