// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{build_client_error_response, build_client_query_response};
use crate::node::{
    error::convert_to_error_message, event_store::EventStore, node_ops::NodeDuty, Error, Result,
};
use crate::routing::Prefix;
use crate::types::{
    Error as DtError, Map, MapAction, MapAddress as Address, MapPermissionSet, MapValue, PublicKey,
};
use crate::{
    messaging::{
        client::{CmdError, MapCmd, MapDataExchange, MapRead, MapWrite, QueryResponse},
        EndUser, MessageId,
    },
    types::DataAddress,
};
use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    path::{Path, PathBuf},
};
use tracing::{debug, info};
use xor_name::XorName;

/// Operations over the data type Map.
pub(super) struct MapStorage {
    path: PathBuf,
    store: BTreeMap<XorName, (Map, EventStore<MapCmd>)>,
}

impl MapStorage {
    pub(super) fn new(path: &Path, _max_capacity: u64) -> Self {
        Self {
            path: path.to_path_buf(),
            store: BTreeMap::new(),
        }
    }

    /// --- Synching ---

    /// Used for replication of data to new Elders.
    pub(super) async fn get_data_of(&self, prefix: Prefix) -> Result<MapDataExchange> {
        let mut the_data = BTreeMap::default();

        for (key, (_, history)) in self
            .store
            .iter()
            .filter(|(_, (map, _))| prefix.matches(map.name()))
        {
            let _ = the_data.insert(*key, history.get_all());
        }

        Ok(MapDataExchange(the_data))
    }

    /// On receiving data from Elders when promoted.
    pub async fn update(&mut self, map_data: MapDataExchange) -> Result<()> {
        debug!("Updating Map DataStore");

        let MapDataExchange(data) = map_data;

        // todo: make outer loop parallel
        for (_, history) in data {
            for op in history {
                let _ = self.apply(op).await?;
            }
        }
        Ok(())
    }

    /// --- Writing ---

    pub(super) async fn write(&mut self, op: MapCmd) -> Result<NodeDuty> {
        let msg_id = op.msg_id;
        let origin = op.origin;
        let write_result = self.apply(op).await;
        self.ok_or_error(write_result, msg_id, origin).await
    }

    async fn apply(&mut self, op: MapCmd) -> Result<()> {
        let MapCmd {
            write, client_sig, ..
        } = op.clone();

        let address = *write.address();
        let key = to_id(&address)?;

        use MapWrite::*;
        match write {
            New(map) => {
                if self.store.contains_key(&key) {
                    return Err(Error::DataExists);
                }
                let mut store = new_store(key, self.path.as_path())?;
                let _ = store.append(op)?;
                let _ = self.store.insert(key, (map, store));
                Ok(())
            }
            Delete(_) => {
                let result = match self.store.get(&key) {
                    Some((map, store)) => match map.check_is_owner(&client_sig.public_key) {
                        Ok(()) => {
                            info!("Deleting Map");
                            store.as_deletable().delete()
                        }
                        Err(_e) => {
                            info!("Error: Delete Map called by non-owner");
                            return Err(Error::NetworkData(DtError::AccessDenied(
                                client_sig.public_key,
                            )));
                        }
                    },
                    None => Ok(()),
                };

                if result.is_ok() {
                    let _ = self.store.remove(&key);
                }

                result
            }
            SetUserPermissions {
                user,
                ref permissions,
                version,
                ..
            } => {
                let (map, store) = match self.store.get_mut(&key) {
                    Some(entry) => entry,
                    None => return Err(Error::NoSuchData(DataAddress::Map(address))),
                };
                map.check_permissions(MapAction::ManagePermissions, &client_sig.public_key)
                    .map_err(Error::from)?;
                map.set_user_permissions(user, permissions.clone(), version)
                    .map_err(Error::from)?;
                store.append(op)
            }
            DelUserPermissions { user, version, .. } => {
                let (map, store) = match self.store.get_mut(&key) {
                    Some(entry) => entry,
                    None => return Err(Error::NoSuchData(DataAddress::Map(address))),
                };
                map.check_permissions(MapAction::ManagePermissions, &client_sig.public_key)
                    .map_err(Error::from)?;
                map.del_user_permissions(user, version)
                    .map_err(Error::from)?;
                store.append(op)
            }
            Edit { changes, .. } => {
                let (map, store) = match self.store.get_mut(&key) {
                    Some(entry) => entry,
                    None => return Err(Error::NoSuchData(DataAddress::Map(address))),
                };
                map.mutate_entries(changes, &client_sig.public_key)
                    .map_err(Error::from)?;
                store.append(op)
            }
        }
    }

    /// --- Reading ---

    pub(super) async fn read(
        &self,
        read: &MapRead,
        msg_id: MessageId,
        requester: PublicKey,
        origin: EndUser,
    ) -> Result<NodeDuty> {
        use MapRead::*;
        match read {
            Get(address) => self.get(*address, msg_id, requester, origin).await,
            GetValue { address, ref key } => {
                self.get_value(*address, key, msg_id, requester, origin)
                    .await
            }
            GetShell(address) => self.get_shell(*address, msg_id, requester, origin).await,
            GetVersion(address) => self.get_version(*address, msg_id, requester, origin).await,
            ListEntries(address) => self.list_entries(*address, msg_id, requester, origin).await,
            ListKeys(address) => self.list_keys(*address, msg_id, requester, origin).await,
            ListValues(address) => self.list_values(*address, msg_id, requester, origin).await,
            ListPermissions(address) => {
                self.list_permissions(*address, msg_id, requester, origin)
                    .await
            }
            ListUserPermissions { address, user } => {
                self.list_user_permissions(*address, *user, msg_id, requester, origin)
                    .await
            }
        }
    }

    /// Get `Map` from the chunk store and check permissions.
    /// Returns `Some(Result<..>)` if the flow should be continued, returns
    /// `None` if there was a logic error encountered and the flow should be
    /// terminated.
    async fn get_map(
        &self,
        address: &Address,
        requester: PublicKey,
        action: MapAction,
    ) -> Result<&Map> {
        match self.store.get(&to_id(address)?) {
            Some((map, _)) => {
                let _ = map
                    .check_permissions(action, &requester)
                    .map_err(Error::from)?;
                Ok(map)
            }
            None => Err(Error::NoSuchData(DataAddress::Map(*address))),
        }
    }

    /// Get entire Map.
    async fn get(
        &self,
        address: Address,
        msg_id: MessageId,
        requester: PublicKey,
        origin: EndUser,
    ) -> Result<NodeDuty> {
        let result = match self.get_map(&address, requester, MapAction::Read).await {
            Ok(res) => Ok(res.clone()),
            Err(Error::NoSuchData(addr)) => return Err(Error::NoSuchData(addr)),
            Err(error) => Err(convert_to_error_message(error)),
        };

        Ok(NodeDuty::Send(build_client_query_response(
            QueryResponse::GetMap(result),
            msg_id,
            origin,
        )))
    }

    /// Get Map shell.
    async fn get_shell(
        &self,
        address: Address,
        msg_id: MessageId,
        requester: PublicKey,
        origin: EndUser,
    ) -> Result<NodeDuty> {
        let result = match self
            .get_map(&address, requester, MapAction::Read)
            .await
            .map(|data| data.shell())
        {
            Ok(res) => Ok(res),
            Err(Error::NoSuchData(addr)) => return Err(Error::NoSuchData(addr)),
            Err(error) => Err(convert_to_error_message(error)),
        };

        Ok(NodeDuty::Send(build_client_query_response(
            QueryResponse::GetMapShell(result),
            msg_id,
            origin,
        )))
    }

    /// Get Map version.
    async fn get_version(
        &self,
        address: Address,
        msg_id: MessageId,
        requester: PublicKey,
        origin: EndUser,
    ) -> Result<NodeDuty> {
        let result = match self
            .get_map(&address, requester, MapAction::Read)
            .await
            .map(|data| data.version())
        {
            Ok(res) => Ok(res),
            Err(Error::NoSuchData(addr)) => return Err(Error::NoSuchData(addr)),
            Err(error) => Err(convert_to_error_message(error)),
        };

        Ok(NodeDuty::Send(build_client_query_response(
            QueryResponse::GetMapVersion(result),
            msg_id,
            origin,
        )))
    }

    /// Get Map value.
    async fn get_value(
        &self,
        address: Address,
        key: &[u8],
        msg_id: MessageId,
        requester: PublicKey,
        origin: EndUser,
    ) -> Result<NodeDuty> {
        let res = self.get_map(&address, requester, MapAction::Read).await;
        let result = match res.and_then(|map| {
            map.get(key)
                .cloned()
                .map(MapValue::from)
                .ok_or(Error::NetworkData(DtError::NoSuchEntry))
        }) {
            Ok(res) => Ok(res),
            Err(Error::NoSuchData(addr)) => return Err(Error::NoSuchData(addr)),
            Err(error) => Err(convert_to_error_message(error)),
        };

        Ok(NodeDuty::Send(build_client_query_response(
            QueryResponse::GetMapValue(result),
            msg_id,
            origin,
        )))
    }

    /// Get Map keys.
    async fn list_keys(
        &self,
        address: Address,
        msg_id: MessageId,
        requester: PublicKey,
        origin: EndUser,
    ) -> Result<NodeDuty> {
        let result = match self
            .get_map(&address, requester, MapAction::Read)
            .await
            .map(|data| data.keys())
        {
            Ok(res) => Ok(res),
            Err(Error::NoSuchData(addr)) => return Err(Error::NoSuchData(addr)),
            Err(error) => Err(convert_to_error_message(error)),
        };

        Ok(NodeDuty::Send(build_client_query_response(
            QueryResponse::ListMapKeys(result),
            msg_id,
            origin,
        )))
    }

    /// Get Map values.
    async fn list_values(
        &self,
        address: Address,
        msg_id: MessageId,
        requester: PublicKey,
        origin: EndUser,
    ) -> Result<NodeDuty> {
        let res = self.get_map(&address, requester, MapAction::Read).await;
        let result = match res.map(|map| map.values()) {
            Ok(res) => Ok(res),
            Err(Error::NoSuchData(addr)) => return Err(Error::NoSuchData(addr)),
            Err(error) => Err(convert_to_error_message(error)),
        };

        Ok(NodeDuty::Send(build_client_query_response(
            QueryResponse::ListMapValues(result),
            msg_id,
            origin,
        )))
    }

    /// Get Map entries.
    async fn list_entries(
        &self,
        address: Address,
        msg_id: MessageId,
        requester: PublicKey,
        origin: EndUser,
    ) -> Result<NodeDuty> {
        let res = self.get_map(&address, requester, MapAction::Read).await;
        let result = match res.map(|map| map.entries().clone()) {
            Ok(res) => Ok(res),
            Err(Error::NoSuchData(addr)) => return Err(Error::NoSuchData(addr)),
            Err(error) => Err(convert_to_error_message(error)),
        };

        Ok(NodeDuty::Send(build_client_query_response(
            QueryResponse::ListMapEntries(result),
            msg_id,
            origin,
        )))
    }

    /// Get Map permissions.
    async fn list_permissions(
        &self,
        address: Address,
        msg_id: MessageId,
        requester: PublicKey,
        origin: EndUser,
    ) -> Result<NodeDuty> {
        let result = match self
            .get_map(&address, requester, MapAction::Read)
            .await
            .map(|data| data.permissions())
        {
            Ok(res) => Ok(res),
            Err(Error::NoSuchData(addr)) => return Err(Error::NoSuchData(addr)),
            Err(error) => Err(convert_to_error_message(error)),
        };

        Ok(NodeDuty::Send(build_client_query_response(
            QueryResponse::ListMapPermissions(result),
            msg_id,
            origin,
        )))
    }

    /// Get Map user permissions.
    async fn list_user_permissions(
        &self,
        address: Address,
        user: PublicKey,
        msg_id: MessageId,
        requester: PublicKey,
        origin: EndUser,
    ) -> Result<NodeDuty> {
        let result = match self
            .get_map(&address, requester, MapAction::Read)
            .await
            .and_then(|data| {
                data.user_permissions(&user)
                    .map_err(|e| e.into())
                    .map(MapPermissionSet::clone)
            }) {
            Ok(res) => Ok(res),
            Err(Error::NoSuchData(addr)) => return Err(Error::NoSuchData(addr)),
            Err(error) => Err(convert_to_error_message(error)),
        };

        Ok(NodeDuty::Send(build_client_query_response(
            QueryResponse::ListMapUserPermissions(result),
            msg_id,
            origin,
        )))
    }

    async fn ok_or_error(
        &self,
        result: Result<()>,
        msg_id: MessageId,
        origin: EndUser,
    ) -> Result<NodeDuty> {
        if let Err(error) = result {
            let error = convert_to_error_message(error);
            info!("MapStorage: Writing chunk FAILED!");

            Ok(NodeDuty::Send(build_client_error_response(
                CmdError::Data(error),
                msg_id,
                origin,
            )))
        } else {
            info!("MapStorage: Writing chunk PASSED!");
            Ok(NodeDuty::NoOp)
        }
    }
}

fn to_id(address: &Address) -> Result<XorName> {
    Ok(XorName::from_content(&[address
        .encode_to_zbase32()?
        .as_bytes()]))
}

fn new_store(id: XorName, path: &Path) -> Result<EventStore<MapCmd>> {
    let db_dir = path.join("map".to_string());
    EventStore::new(id, db_dir.as_path())
}

impl Display for MapStorage {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "MapStorage")
    }
}
