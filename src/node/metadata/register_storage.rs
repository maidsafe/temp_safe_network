// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{build_client_error_response, build_client_query_response};
use crate::dbs::{EventStore, UsedSpace};
use crate::node::{error::convert_to_error_message, node_ops::NodeDuty, Error, Result};
use crate::types::{
    register::{Action, Address, Register, User},
    PublicKey,
};
use crate::{
    messaging::{
        client::{
            CmdError, QueryResponse, RegisterCmd, RegisterDataExchange, RegisterRead, RegisterWrite,
        },
        EndUser, MessageId,
    },
    types::DataAddress,
};
use dashmap::DashMap;
use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    path::{Path, PathBuf},
};
use tracing::info;
use xor_name::{Prefix, XorName};

/// Operations over the data type Register.
pub(super) struct RegisterStorage {
    path: PathBuf,
    used_space: UsedSpace,
    registers: DashMap<XorName, Option<StateEntry>>,
}

struct StateEntry {
    state: Register,
    db: EventStore<RegisterCmd>,
}

impl RegisterStorage {
    pub(super) fn new(path: &Path, used_space: UsedSpace) -> Self {
        used_space.add_dir(path);
        Self {
            path: path.to_path_buf(),
            used_space,
            registers: DashMap::new(),
        }
    }

    /// --- Synching ---

    /// Used for replication of data to new Elders.
    pub(super) async fn get_data_of(&self, prefix: Prefix) -> Result<RegisterDataExchange> {
        let mut the_data = BTreeMap::default();

        for entry in self.registers.iter() {
            let (key, cache) = entry.pair();
            if let Some(entry) = cache {
                if prefix.matches(entry.state.name()) {
                    let _ = the_data.insert(*key, entry.db.get_all());
                }
            } else {
            }
        }

        Ok(RegisterDataExchange(the_data))
    }

    /// On receiving data from Elders when promoted.
    pub async fn update(&self, reg_data: RegisterDataExchange) -> Result<()> {
        debug!("Updating Register store");

        let RegisterDataExchange(data) = reg_data;

        // todo: make outer loop parallel
        for (_, history) in data {
            for op in history {
                let _ = self.apply(op).await?;
            }
        }

        // let instance = Arc::new(self);

        // let handles = data.iter().map(|(_, history)| {
        //     let store = instance.clone();
        //     tokio::task::spawn(async {
        //         for op in history {
        //             let _ = store.apply(*op).await?;
        //             //let _ = self.apply(*op).await?;
        //         }
        //         Ok::<_, Error>(())
        //     })
        // });

        // join_all(handles)
        //     .await
        //     .iter()
        //     .flatten()
        //     .for_each(|e| error!("{:?}", e));

        Ok(())
    }

    /// --- Writing ---

    pub(super) async fn write(&self, op: RegisterCmd) -> Result<NodeDuty> {
        let required_space = std::mem::size_of::<RegisterCmd>() as u64;
        if !self.used_space.can_consume(required_space).await {
            return Err(Error::Database(crate::dbs::Error::NotEnoughSpace));
        }
        let msg_id = op.msg_id;
        let origin = op.origin;
        let write_result = self.apply(op).await;
        self.ok_or_error(write_result, msg_id, origin).await
    }

    async fn apply(&self, op: RegisterCmd) -> Result<()> {
        let RegisterCmd {
            write,
            msg_id,
            client_sig,
            ..
        } = op.clone();

        let address = *write.address();
        let key = to_id(&address)?;

        use RegisterWrite::*;
        match write {
            New(map) => {
                if self.registers.contains_key(&key) {
                    return Err(Error::DataExists);
                }
                let mut db = load_db(key, self.path.as_path()).await?;
                let _ = db.append(op)?;
                let _ = self
                    .registers
                    .insert(key, Some(StateEntry { state: map, db }));
                Ok(())
            }
            Delete(_) => {
                let result = match self.registers.get_mut(&key) {
                    None => {
                        if let Ok(db) = load_db(key, self.path.as_path()).await {
                            info!("Deleting Register");
                            db.as_deletable().delete().await.map_err(Error::from)
                        } else {
                            Ok(())
                        }
                    }
                    Some(mut entry) => {
                        let (_, cache) = entry.pair_mut();
                        if let Some(entry) = cache {
                            if entry.state.address().is_public() {
                                return Err(Error::InvalidMessage(
                                    msg_id,
                                    "Cannot delete public Register".to_string(),
                                ));
                            }
                            // TODO - Register::check_permission() doesn't support Delete yet in safe-nd
                            // register.check_permission(action, Some(client_sig.public_key))?;
                            if client_sig.public_key != entry.state.owner() {
                                Err(Error::InvalidOwner(client_sig.public_key))
                            } else {
                                info!("Deleting Register");
                                entry.db.as_deletable().delete().await.map_err(Error::from)
                            }
                        } else if let Ok(db) = load_db(key, self.path.as_path()).await {
                            info!("Deleting Register");
                            db.as_deletable().delete().await.map_err(Error::from)
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
                    // read from disk
                    let db = load_db(key, self.path.as_path()).await?;
                    let mut reg = None;
                    // apply all ops
                    for op in db.get_all() {
                        // first op shall be New
                        if let New(register) = op.write {
                            reg = Some(register);
                        } else if let Some(register) = &mut reg {
                            if let Edit(reg_op) = op.write {
                                register.apply_op(reg_op).map_err(Error::NetworkData)?;
                            }
                        }
                    }

                    let new_entry = reg
                        .take()
                        .ok_or(Error::NoSuchData(DataAddress::Register(address)))
                        .map(|state| StateEntry { state, db })?;

                    let _ = cache.replace(new_entry);

                    if let Some(entry) = cache.as_mut() {
                        entry
                    } else {
                        return Err(Error::NoSuchData(DataAddress::Register(address)));
                    }
                };

                info!("Editing Register");
                entry
                    .state
                    .check_permissions(Action::Write, Some(client_sig.public_key))?;
                let result = entry.state.apply_op(reg_op).map_err(Error::NetworkData);

                if result.is_ok() {
                    entry.db.append(op)?;
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
            .get(&to_id(address)?)
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
}

fn to_id(address: &Address) -> Result<XorName> {
    Ok(XorName::from_content(&[address
        .encode_to_zbase32()?
        .as_bytes()]))
}

async fn load_db(id: XorName, path: &Path) -> Result<EventStore<RegisterCmd>> {
    let db_dir = path.join("db").join("register".to_string());
    EventStore::new(id, db_dir.as_path())
        .await
        .map_err(Error::from)
}

impl Display for RegisterStorage {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "RegisterStorage")
    }
}
