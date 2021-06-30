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
    store: BTreeMap<XorName, (Register, EventStore<RegisterCmd>)>,
}

impl RegisterStorage {
    pub(super) fn new(path: &Path, _max_capacity: u64) -> Self {
        Self {
            path: path.to_path_buf(),
            store: BTreeMap::new(),
        }
    }

    /// --- Synching ---

    /// Used for replication of data to new Elders.
    pub(super) async fn get_data_of(&self, prefix: Prefix) -> Result<RegisterDataExchange> {
        let mut the_data = BTreeMap::default();

        for (key, (_, history)) in self
            .store
            .iter()
            .filter(|(_, (map, _))| prefix.matches(map.name()))
        {
            let _ = the_data.insert(*key, history.get_all());
        }

        Ok(RegisterDataExchange(the_data))
    }

    /// On receiving data from Elders when promoted.
    pub async fn update(&mut self, reg_data: RegisterDataExchange) -> Result<()> {
        debug!("Updating Register store");

        let RegisterDataExchange(data) = reg_data;

        // todo: make outer loop parallel
        for (_, history) in data {
            for op in history {
                let _ = self.apply(op).await?;
            }
        }
        Ok(())
    }

    /// --- Writing ---

    pub(super) async fn write(&mut self, op: RegisterCmd) -> Result<NodeDuty> {
        let msg_id = op.msg_id;
        let origin = op.origin;
        let write_result = self.apply(op).await;
        self.ok_or_error(write_result, msg_id, origin).await
    }

    async fn apply(&mut self, op: RegisterCmd) -> Result<()> {
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
                    Some((register, store)) => {
                        if register.address().is_public() {
                            return Err(Error::InvalidMessage(
                                msg_id,
                                "Cannot delete public Register".to_string(),
                            ));
                        }

                        // TODO - Register::check_permission() doesn't support Delete yet in safe-nd
                        // register.check_permission(action, Some(client_sig.public_key))?;

                        if client_sig.public_key != register.owner() {
                            Err(Error::InvalidOwner(client_sig.public_key))
                        } else {
                            info!("Deleting Register");
                            store.as_deletable().delete()
                        }
                    }
                    None => Ok(()),
                };

                if result.is_ok() {
                    let _ = self.store.remove(&key);
                }

                result
            }
            Edit(reg_op) => {
                let (register, store) = match self.store.get_mut(&key) {
                    Some(entry) => entry,
                    None => return Err(Error::NoSuchData(DataAddress::Register(address))),
                };

                info!("Editing Register");
                register.check_permissions(Action::Write, Some(client_sig.public_key))?;
                let result = register.apply_op(reg_op).map_err(Error::NetworkData);

                if result.is_ok() {
                    store.append(op)?;
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
            Ok(register) => Ok(register.clone()),
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
    ) -> Result<&Register> {
        match self.store.get(&to_id(address)?) {
            Some((register, _)) => {
                let _ = register
                    .check_permissions(action, Some(requester))
                    .map_err(Error::from)?;
                Ok(register)
            }
            None => Err(Error::NoSuchData(DataAddress::Register(*address))),
        }
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

fn new_store(id: XorName, path: &Path) -> Result<EventStore<RegisterCmd>> {
    let db_dir = path.join("register".to_string());
    EventStore::new(id, db_dir.as_path())
}

impl Display for RegisterStorage {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "RegisterStorage")
    }
}
