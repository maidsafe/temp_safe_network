// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{
    chunk_store::{error::Error as ChunkStoreError, MutableChunkStore},
    cmd::{MetadataCmd, NodeCmd},
    msg::Message,
    node::Init,
    utils, Config, Result,
};
use log::error;

use safe_nd::{
    CmdError, Duty, ElderDuty, Error as NdError, MData, MDataAction, MDataAddress,
    MDataEntryActions, MDataPermissionSet, MDataValue, MapRead, MapWrite, Message, MessageId,
    MsgEnvelope, MsgSender, NodePublicId, PublicKey, QueryResponse, Result as NdResult,
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
    ) -> Option<NodeCmd> {
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

    pub(super) fn write(&mut self, write: MapWrite, msg: MsgEnvelope) -> Option<NodeCmd> {
        use MapWrite::*;
        match write {
            New(data) => self.create(&data, msg),
            Delete(address) => self.delete(address, msg),
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
        origin: MsgSender,
        action: MDataAction,
    ) -> Option<NdResult<MData>> {
        Some(
            self.chunks
                .get(&address)
                .map_err(|e| match e {
                    ChunkStoreError::NoSuchChunk => NdError::NoSuchData,
                    error => error.to_string().into(),
                })
                .and_then(move |mdata| {
                    mdata
                        .check_permissions(action, origin.id())
                        .map(move |_| mdata)
                }),
        )
    }

    /// Get MData from the chunk store, update it, and overwrite the stored chunk.
    fn edit_chunk<F>(
        &mut self,
        address: &MDataAddress,
        origin: MsgSender,
        msg_id: MessageId,
        mutation_fn: F,
    ) -> Option<NodeCmd>
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
        self.ok_or_error(result, msg_id, origin)
    }

    /// Put MData.
    fn create(&mut self, data: &MData, msg: MsgEnvelope) -> Option<NodeCmd> {
        let result = if self.chunks.has(data.address()) {
            Err(NdError::DataExists)
        } else {
            self.chunks
                .put(&data)
                .map_err(|error| error.to_string().into())
        };
        self.ok_or_error(result, msg.id(), msg.origin)
    }

    fn delete(&mut self, address: MDataAddress, msg: MsgEnvelope) -> Option<NodeCmd> {
        let result = self
            .chunks
            .get(&address)
            .map_err(|e| match e {
                ChunkStoreError::NoSuchChunk => NdError::NoSuchData,
                error => error.to_string().into(),
            })
            .and_then(move |mdata| {
                mdata.check_is_owner(msg.origin.id())?;
                self.chunks
                    .delete(&address)
                    .map_err(|error| error.to_string().into())
            });

        self.ok_or_error(result, msg.id(), msg.origin)
    }

    /// Set MData user permissions.
    fn set_user_permissions(
        &mut self,
        address: MDataAddress,
        user: PublicKey,
        permissions: &MDataPermissionSet,
        version: u64,
        msg: MsgEnvelope,
    ) -> Option<NodeCmd> {
        self.edit_chunk(&address, msg.origin, message_id, move |mut data| {
            data.check_permissions(MDataAction::ManagePermissions, msg.origin.id())?;
            data.set_user_permissions(user, permissions.clone(), version)?;
            Ok(data)
        })
    }

    /// Delete MData user permissions.
    fn delete_user_permissions(
        &mut self,
        address: MDataAddress,
        user: PublicKey,
        version: u64,
        msg: MsgEnvelope,
    ) -> Option<NodeCmd> {
        self.edit_chunk(&address, msg.origin, msg.id(), move |mut data| {
            data.check_permissions(MDataAction::ManagePermissions, msg.origin.id())?;
            data.del_user_permissions(user, version)?;
            Ok(data)
        })
    }

    /// Edit MData.
    fn edit_entries(
        &mut self,
        address: MDataAddress,
        actions: MDataEntryActions,
        msg: MsgEnvelope,
    ) -> Option<NodeCmd> {
        self.edit_chunk(&address, msg.origin, msg.id(), move |mut data| {
            data.mutate_entries(actions, msg.origin.id())?;
            Ok(data)
        })
    }

    /// Get entire MData.
    fn get(&self, address: MDataAddress, msg: MsgEnvelope) -> Option<NodeCmd> {
        let result = self.get_chunk(&address, &msg.origin.id(), MDataAction::Read)?;
        self.wrap(Message::QueryResponse {
            query: QueryResponse::GetMap(result),
            id: MessageId::new(),
            correlation_id: msg.id(),
            query_origin: msg.origin,
        })
    }

    /// Get MData shell.
    fn get_shell(&self, address: MDataAddress, msg: MsgEnvelope) -> Option<NodeCmd> {
        let result = self
            .get_chunk(&address, &msg.origin.id(), MDataAction::Read)?
            .map(|data| data.shell());
        self.wrap(Message::QueryResponse {
            query: QueryResponse::GetMapShell(result),
            id: MessageId::new(),
            correlation_id: msg.id(),
            query_origin: msg.origin,
        })
    }

    /// Get MData version.
    fn get_version(&self, address: MDataAddress, msg: MsgEnvelope) -> Option<NodeCmd> {
        let result = self
            .get_chunk(&address, &msg.origin.id(), MDataAction::Read)?
            .map(|data| data.version());
        self.wrap(Message::QueryResponse {
            query: QueryResponse::GetMapVersion(result),
            id: MessageId::new(),
            correlation_id: msg.id(),
            query_origin: msg.origin,
        })
    }

    /// Get MData value.
    fn get_value(&self, address: MDataAddress, key: &[u8], msg: MsgEnvelope) -> Option<NodeCmd> {
        let res = self.get_chunk(&address, &msg.origin.id(), MDataAction::Read)?;
        let result = res.and_then(|data| match data {
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
        });
        self.wrap(Message::QueryResponse {
            query: QueryResponse::GetMapValue(result),
            id: MessageId::new(),
            correlation_id: msg.id(),
            query_origin: msg.origin,
        })
    }

    /// Get MData keys.
    fn list_keys(&self, address: MDataAddress, msg: MsgEnvelope) -> Option<NodeCmd> {
        let result = self
            .get_chunk(&address, &msg.origin.id(), MDataAction::Read)?
            .map(|data| data.keys());
        self.wrap(Message::QueryResponse {
            query: QueryResponse::ListMapKeys(result),
            id: MessageId::new(),
            correlation_id: msg.id(),
            query_origin: msg.origin,
        })
    }

    /// Get MData values.
    fn list_values(&self, address: MDataAddress, msg: MsgEnvelope) -> Option<NodeCmd> {
        let res = self.get_chunk(&address, &msg.origin.id(), MDataAction::Read)?;
        let result = res.and_then(|data| match data {
            MData::Seq(md) => Ok(md.values().into()),
            MData::Unseq(md) => Ok(md.values().into()),
        });
        self.wrap(Message::QueryResponse {
            query: QueryResponse::ListMapValues(result),
            id: MessageId::new(),
            correlation_id: msg.id(),
            query_origin: msg.origin,
        })
    }

    /// Get MData entries.
    fn list_entries(&self, address: MDataAddress, msg: MsgEnvelope) -> Option<NodeCmd> {
        let res = self.get_chunk(&address, &msg.origin.id(), MDataAction::Read)?;
        let result = res.and_then(|data| match data {
            MData::Seq(md) => Ok(md.entries().clone().into()),
            MData::Unseq(md) => Ok(md.entries().clone().into()),
        });
        self.wrap(Message::QueryResponse {
            query: QueryResponse::ListMapEntries(result),
            id: MessageId::new(),
            correlation_id: msg.id(),
            query_origin: msg.origin,
        })
    }

    /// Get MData permissions.
    fn list_permissions(&self, address: MDataAddress, msg: MsgEnvelope) -> Option<NodeCmd> {
        let result = self
            .get_chunk(&address, &msg.origin.id(), MDataAction::Read)?
            .map(|data| data.permissions());
        self.wrap(Message::QueryResponse {
            query: QueryResponse::ListMapPermissions(result),
            id: MessageId::new(),
            correlation_id: msg.id(),
            query_origin: msg.origin,
        })
    }

    /// Get MData user permissions.
    fn list_user_permissions(&self, address: MDataAddress, msg: MsgEnvelope) -> Option<NodeCmd> {
        let result = self
            .get_chunk(&address, &origin.id(), MDataAction::Read)?
            .and_then(|data| {
                data.user_permissions(origin.id())
                    .map(MDataPermissionSet::clone)
            });
        self.wrap(Message::QueryResponse {
            query: QueryResponse::ListMapUserPermissions(result),
            id: MessageId::new(),
            correlation_id: msg.id(),
            query_origin: msg.origin,
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

impl Display for MapStorage {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.id.name())
    }
}
