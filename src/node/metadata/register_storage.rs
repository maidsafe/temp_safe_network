// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{build_client_error_response, build_client_query_response};
use crate::dbs::{RegisterOpStore, UsedSpace};
use crate::node::{error::convert_to_error_message, node_ops::NodeDuty, Error, Result};
use crate::types::{
    register::{Action, Address, Register, User},
    PublicKey,
};
use crate::{
    messaging::{
        data::{
            CmdError, DataCmd, QueryResponse, RegisterCmd, RegisterDataExchange, RegisterRead,
            RegisterWrite,
        },
        Authority, DataSigned, EndUser, MessageId,
    },
    types::DataAddress,
};
use dashmap::DashMap;
use sled::Db;
use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    path::{Path, PathBuf},
};
use tracing::info;
use xor_name::{Prefix, XorName};

const DATABASE_NAME: &str = "register.db";

/// Operations over the data type Register.
#[derive(Clone)]
pub(super) struct RegisterStorage {
    path: PathBuf,
    used_space: UsedSpace,
    registers: DashMap<XorName, Option<StateEntry>>,
    db: Db,
}

#[derive(Clone)]
struct StateEntry {
    state: Register,
    store: RegisterOpStore,
}

impl RegisterStorage {
    pub(super) fn new(path: &Path, used_space: UsedSpace) -> Result<Self> {
        used_space.add_dir(path);
        let db_dir = path.join("db").join(DATABASE_NAME.to_string());

        let db = sled::open(db_dir).map_err(|error| {
            trace!("Sled Error: {:?}", error);
            Error::UnableToCreateRegisterDb
        })?;

        Ok(Self {
            path: path.to_path_buf(),
            used_space,
            registers: DashMap::new(),
            db,
        })
    }

    /// --- Synching ---

    /// Used for replication of data to new Elders.
    pub(super) async fn get_data_of(&self, prefix: Prefix) -> Result<RegisterDataExchange> {
        let mut the_data = BTreeMap::default();

        for entry in self.registers.iter() {
            let (key, cache) = entry.pair();
            if let Some(entry) = cache {
                if prefix.matches(entry.state.name()) {
                    let _ = the_data.insert(*key, entry.store.get_all()?);
                }
            } else {
                let entry = self.load_state(*key)?;
                if prefix.matches(entry.state.name()) {
                    let _ = the_data.insert(*key, entry.store.get_all()?);
                }
            }
        }

        Ok(RegisterDataExchange(the_data))
    }

    /// On receiving data from Elders when promoted.
    pub(super) async fn update(&self, reg_data: RegisterDataExchange) -> Result<()> {
        debug!("Updating Register store");

        let RegisterDataExchange(data) = reg_data;

        // todo: make outer loop parallel
        for (_, history) in data {
            for op in history {
                let data_auth =
                    super::verify_op(op.client_sig.clone(), DataCmd::Register(op.write.clone()))?;
                let _ = self.apply(op, data_auth).await?;
            }
        }

        Ok(())
    }

    /// --- Writing ---

    pub(super) async fn write(
        &self,
        msg_id: MessageId,
        origin: EndUser,
        write: RegisterWrite,
        data_auth: Authority<DataSigned>,
    ) -> Result<NodeDuty> {
        let required_space = std::mem::size_of::<RegisterCmd>() as u64;
        if !self.used_space.can_consume(required_space).await {
            return Err(Error::Database(crate::dbs::Error::NotEnoughSpace));
        }
        let op = RegisterCmd {
            write,
            client_sig: data_auth.clone().into_inner(),
        };
        let write_result = self.apply(op, data_auth).await;
        self.ok_or_error(write_result, msg_id, origin).await
    }

    async fn apply(&self, op: RegisterCmd, data_auth: Authority<DataSigned>) -> Result<()> {
        let RegisterCmd { write, .. } = op.clone();

        let address = *write.address();
        let key = to_reg_key(&address)?;

        use RegisterWrite::*;
        match write {
            New(map) => {
                if self.registers.contains_key(&key) {
                    return Err(Error::DataExists);
                }
                let mut store = self.load_store(key)?;
                let _ = store.append(op)?;
                let _ = self
                    .registers
                    .insert(key, Some(StateEntry { state: map, store }));
                Ok(())
            }
            Delete(_) => {
                let result = match self.registers.get_mut(&key) {
                    None => {
                        info!("Attempting to delete register if it exists");
                        let _ = self.db.drop_tree(key)?;
                        Ok(())
                    }
                    Some(mut entry) => {
                        let (_, cache) = entry.pair_mut();
                        if let Some(entry) = cache {
                            if entry.state.address().is_public() {
                                return Err(Error::InvalidOperation(
                                    "Cannot delete public Register".to_string(),
                                ));
                            }
                            // TODO - Register::check_permission() doesn't support Delete yet in safe-nd
                            // register.check_permission(action, Some(client_sig.public_key))?;
                            if data_auth.public_key != entry.state.owner() {
                                Err(Error::InvalidOwner(data_auth.public_key))
                            } else {
                                info!("Deleting Register");
                                let _ = self.db.drop_tree(key)?;
                                Ok(())
                            }
                        } else if self.load_store(key).is_ok() {
                            info!("Deleting Register");
                            let _ = self.db.drop_tree(key)?;
                            Ok(())
                        } else {
                            Ok(())
                        }
                    }
                };

                if result.is_ok() {
                    let _ = self.registers.remove(&key);
                }

                result
            }
            Edit(reg_op) => {
                let mut cache = self
                    .registers
                    .get_mut(&key)
                    .ok_or(Error::NoSuchData(DataAddress::Register(address)))?;
                let entry = if let Some(cached_entry) = cache.as_mut() {
                    cached_entry
                } else {
                    let fresh_entry = self.load_state(key)?;
                    let _ = cache.replace(fresh_entry);
                    if let Some(entry) = cache.as_mut() {
                        entry
                    } else {
                        return Err(Error::NoSuchData(DataAddress::Register(address)));
                    }
                };

                info!("Editing Register");
                entry
                    .state
                    .check_permissions(Action::Write, Some(data_auth.public_key))?;
                let result = entry.state.apply_op(reg_op).map_err(Error::NetworkData);

                if result.is_ok() {
                    entry.store.append(op)?;
                    info!("Editing Register SUCCESSFUL!");
                } else {
                    info!("Editing Register FAILED!");
                }

                result
            }
        }
    }

    /// --- Reading ---

    pub(super) async fn read(
        &self,
        read: &RegisterRead,
        msg_id: MessageId,
        requester: PublicKey,
        origin: EndUser,
    ) -> Result<NodeDuty> {
        use RegisterRead::*;
        match read {
            Get(address) => self.get(*address, msg_id, requester, origin).await,
            Read(address) => {
                self.read_register(*address, msg_id, requester, origin)
                    .await
            }
            GetOwner(address) => self.get_owner(*address, msg_id, requester, origin).await,
            GetUserPermissions { address, user } => {
                self.get_user_permissions(*address, *user, msg_id, requester, origin)
                    .await
            }
            GetPolicy(address) => self.get_policy(*address, msg_id, requester, origin).await,
        }
    }

    /// Get entire Register.
    async fn get(
        &self,
        address: Address,
        msg_id: MessageId,
        requester: PublicKey,
        origin: EndUser,
    ) -> Result<NodeDuty> {
        let result = match self.get_register(&address, Action::Read, requester).await {
            Ok(register) => Ok(register),
            Err(Error::NoSuchData(addr)) => return Err(Error::NoSuchData(addr)),
            Err(error) => Err(convert_to_error_message(error)),
        };

        Ok(NodeDuty::Send(build_client_query_response(
            QueryResponse::GetRegister(result),
            msg_id,
            origin,
        )))
    }

    /// Get `Register` from the store and check permissions.
    async fn get_register(
        &self,
        address: &Address,
        action: Action,
        requester: PublicKey,
    ) -> Result<Register> {
        let cache = self
            .registers
            .get(&to_reg_key(address)?)
            .ok_or_else(|| Error::NoSuchData(DataAddress::Register(*address)))?;
        let StateEntry { state, .. } = cache
            .as_ref()
            .ok_or_else(|| Error::NoSuchData(DataAddress::Register(*address)))?;
        state
            .check_permissions(action, Some(requester))
            .map_err(Error::from)?;

        Ok(state.clone())
    }

    async fn read_register(
        &self,
        address: Address,
        msg_id: MessageId,
        requester: PublicKey,
        origin: EndUser,
    ) -> Result<NodeDuty> {
        let result = match self.get_register(&address, Action::Read, requester).await {
            Ok(register) => register.read(Some(requester)).map_err(Error::from),
            Err(Error::NoSuchData(addr)) => return Err(Error::NoSuchData(addr)),
            Err(error) => Err(error),
        };

        Ok(NodeDuty::Send(build_client_query_response(
            QueryResponse::ReadRegister(result.map_err(convert_to_error_message)),
            msg_id,
            origin,
        )))
    }

    async fn get_owner(
        &self,
        address: Address,
        msg_id: MessageId,
        requester: PublicKey,
        origin: EndUser,
    ) -> Result<NodeDuty> {
        let result = match self.get_register(&address, Action::Read, requester).await {
            Ok(res) => Ok(res.owner()),
            Err(Error::NoSuchData(addr)) => return Err(Error::NoSuchData(addr)),
            Err(error) => Err(convert_to_error_message(error)),
        };

        Ok(NodeDuty::Send(build_client_query_response(
            QueryResponse::GetRegisterOwner(result),
            msg_id,
            origin,
        )))
    }

    async fn get_user_permissions(
        &self,
        address: Address,
        user: User,
        msg_id: MessageId,
        requester: PublicKey,
        origin: EndUser,
    ) -> Result<NodeDuty> {
        let result = match self
            .get_register(&address, Action::Read, requester)
            .await
            .and_then(|register| {
                register
                    .permissions(user, Some(requester))
                    .map_err(Error::from)
            }) {
            Ok(res) => Ok(res),
            Err(Error::NoSuchData(addr)) => return Err(Error::NoSuchData(addr)),
            Err(error) => Err(convert_to_error_message(error)),
        };

        Ok(NodeDuty::Send(build_client_query_response(
            QueryResponse::GetRegisterUserPermissions(result),
            msg_id,
            origin,
        )))
    }

    async fn get_policy(
        &self,
        address: Address,
        msg_id: MessageId,
        requester: PublicKey,
        origin: EndUser,
    ) -> Result<NodeDuty> {
        let result = match self
            .get_register(&address, Action::Read, requester)
            .await
            .and_then(|register| {
                register
                    .policy(Some(requester))
                    .map(|p| p.clone())
                    .map_err(Error::from)
            }) {
            Ok(res) => Ok(res),
            Err(Error::NoSuchData(addr)) => return Err(Error::NoSuchData(addr)),
            Err(error) => Err(convert_to_error_message(error)),
        };

        Ok(NodeDuty::Send(build_client_query_response(
            QueryResponse::GetRegisterPolicy(result),
            msg_id,
            origin,
        )))
    }

    async fn ok_or_error<T>(
        &self,
        result: Result<T>,
        msg_id: MessageId,
        origin: EndUser,
    ) -> Result<NodeDuty> {
        let error = match result {
            Ok(_) => return Ok(NodeDuty::NoOp),
            Err(error) => {
                info!("Error on writing Register! {:?}", error);
                convert_to_error_message(error)
            }
        };

        Ok(NodeDuty::Send(build_client_error_response(
            CmdError::Data(error),
            msg_id,
            origin,
        )))
    }

    /// Load a register op store
    fn load_store(&self, id: XorName) -> Result<RegisterOpStore> {
        RegisterOpStore::new(id, self.db.clone()).map_err(Error::from)
    }

    fn load_state(&self, key: XorName) -> Result<StateEntry> {
        // read from disk
        let store = self.load_store(key)?;
        let mut reg = None;
        // apply all ops
        use RegisterWrite::*;
        for op in store.get_all()? {
            // first op shall be New
            if let New(register) = op.write {
                reg = Some(register);
            } else if let Some(register) = &mut reg {
                if let Edit(reg_op) = op.write {
                    register.apply_op(reg_op).map_err(Error::NetworkData)?;
                }
            }
        }

        reg.take()
            .ok_or_else(|| {
                Error::Logic("A store was found, but its contents were invalid.".to_string())
            })
            .map(|state| StateEntry { state, store })
    }
}

/// This also encodes the Public | Private scope,
/// as well as the tag of the Address.
fn to_reg_key(address: &Address) -> Result<XorName> {
    Ok(XorName::from_content(&[address
        .encode_to_zbase32()?
        .as_bytes()]))
}

impl Display for RegisterStorage {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "RegisterStorage")
    }
}
