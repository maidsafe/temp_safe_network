// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{build_client_error_response, build_client_query_response};
use crate::dbs::EventStore;
use crate::node::{error::convert_to_error_message, node_ops::NodeDuty, Error, Result};
use crate::routing::Prefix;
use crate::types::{
    Error as DtError, PublicKey, Sequence, SequenceAction as Action, SequenceAddress as Address,
    SequenceIndex, SequenceUser,
};
use crate::{
    messaging::{
        data::{
            CmdError, DataCmd, QueryResponse, SequenceCmd, SequenceDataExchange, SequenceRead,
            SequenceWrite,
        },
        ClientAuthority, EndUser, MessageId,
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

/// Operations over the data type Sequence.
pub(super) struct SequenceStorage {
    path: PathBuf,
    store: BTreeMap<XorName, (Sequence, EventStore<SequenceCmd>)>,
}

impl SequenceStorage {
    pub(super) fn new(path: &Path, _max_capacity: u64) -> Self {
        Self {
            path: path.to_path_buf(),
            store: BTreeMap::new(),
        }
    }

    /// --- Synching ---

    /// Used for replication of data to new Elders.
    pub(super) async fn get_data_of(&self, prefix: Prefix) -> Result<SequenceDataExchange> {
        let mut the_data = BTreeMap::default();

        for (key, (_, history)) in self
            .store
            .iter()
            .filter(|(_, (map, _))| prefix.matches(map.name()))
        {
            let _ = the_data.insert(*key, history.get_all());
        }

        Ok(SequenceDataExchange(the_data))
    }

    /// On receiving data from Elders when promoted.
    pub(super) async fn update(&mut self, seq_data: SequenceDataExchange) -> Result<()> {
        debug!("Updating Sequence store");

        let SequenceDataExchange(data) = seq_data;

        // todo: make outer loop parallel
        for (_, history) in data {
            for op in history {
                let client_auth =
                    super::verify_op(op.client_sig.clone(), DataCmd::Sequence(op.write.clone()))?;
                let _ = self.apply(op, client_auth).await?;
            }
        }
        Ok(())
    }

    /// --- Writing ---

    pub(super) async fn write(
        &mut self,
        msg_id: MessageId,
        origin: EndUser,
        write: SequenceWrite,
        client_auth: ClientAuthority,
    ) -> Result<NodeDuty> {
        let op = SequenceCmd {
            write,
            client_sig: client_auth.to_signed(),
        };
        let write_result = self.apply(op, client_auth).await;
        self.ok_or_error(write_result, msg_id, origin).await
    }

    async fn apply(&mut self, op: SequenceCmd, client_auth: ClientAuthority) -> Result<()> {
        let SequenceCmd { write, .. } = op.clone();

        let address = *write.address();
        let key = to_id(&address)?;

        use SequenceWrite::*;
        match write {
            New(map) => {
                if self.store.contains_key(&key) {
                    return Err(Error::DataExists);
                }
                let mut store = new_store(key, self.path.as_path()).await?;
                let _ = store.append(op)?;
                let _ = self.store.insert(key, (map, store));
                Ok(())
            }
            Delete(_) => {
                let result = match self.store.get(&key) {
                    Some((sequence, store)) => {
                        if sequence.address().is_public() {
                            return Err(Error::InvalidOperation(
                                "Cannot delete public Sequence".to_string(),
                            ));
                        }

                        // TODO - Sequence::check_permission() doesn't support Delete yet in safe-nd
                        // sequence.check_permission(action, Some(client_sig.public_key))?;

                        let client_pk = *client_auth.public_key();
                        let policy = sequence.private_policy(Some(client_pk))?;
                        if client_pk != policy.owner {
                            Err(Error::InvalidOwner(client_pk))
                        } else {
                            info!("Deleting Sequence");
                            store.as_deletable().delete().await.map_err(Error::from)
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
                let (sequence, store) = match self.store.get_mut(&key) {
                    Some(entry) => entry,
                    None => return Err(Error::NoSuchData(DataAddress::Sequence(address))),
                };

                info!("Editing Sequence");
                sequence.check_permissions(Action::Append, Some(*client_auth.public_key()))?;
                let result = sequence.apply_op(reg_op).map_err(Error::NetworkData);

                if result.is_ok() {
                    store.append(op)?;
                    info!("Editing Sequence SUCCESSFUL!");
                } else {
                    info!("Editing Sequence FAILED!");
                }

                result
            }
        }
    }

    /// --- Reading ---

    pub(super) async fn read(
        &self,
        read: &SequenceRead,
        msg_id: MessageId,
        requester: PublicKey,
        origin: EndUser,
    ) -> Result<NodeDuty> {
        use SequenceRead::*;
        match read {
            Get(address) => self.get(*address, msg_id, requester, origin).await,
            GetRange { address, range } => {
                self.get_range(*address, *range, msg_id, requester, origin)
                    .await
            }
            GetLastEntry(address) => {
                self.get_last_entry(*address, msg_id, requester, origin)
                    .await
            }
            GetUserPermissions { address, user } => {
                self.get_user_permissions(*address, *user, msg_id, requester, origin)
                    .await
            }
            GetPublicPolicy(address) => {
                self.get_public_policy(*address, msg_id, requester, origin)
                    .await
            }
            GetPrivatePolicy(address) => {
                self.get_private_policy(*address, msg_id, requester, origin)
                    .await
            }
        }
    }

    /// Get entire Sequence.
    async fn get(
        &self,
        address: Address,
        msg_id: MessageId,
        requester: PublicKey,
        origin: EndUser,
    ) -> Result<NodeDuty> {
        let result = match self.get_sequence(&address, Action::Read, requester).await {
            Ok(register) => Ok(register.clone()),
            Err(Error::NoSuchData(addr)) => return Err(Error::NoSuchData(addr)),
            Err(error) => Err(convert_to_error_message(error)),
        };

        Ok(NodeDuty::Send(build_client_query_response(
            QueryResponse::GetSequence(result),
            msg_id,
            origin,
        )))
    }

    /// Get `Sequence` from the store and check permissions.
    async fn get_sequence(
        &self,
        address: &Address,
        action: Action,
        requester: PublicKey,
    ) -> Result<&Sequence> {
        match self.store.get(&to_id(address)?) {
            Some((sequence, _)) => {
                let _ = sequence
                    .check_permissions(action, Some(requester))
                    .map_err(Error::from)?;
                Ok(sequence)
            }
            None => Err(Error::NoSuchData(DataAddress::Sequence(*address))),
        }
    }

    async fn get_range(
        &self,
        address: Address,
        range: (SequenceIndex, SequenceIndex),
        msg_id: MessageId,
        requester: PublicKey,
        origin: EndUser,
    ) -> Result<NodeDuty> {
        let result = match self
            .get_sequence(&address, Action::Read, requester)
            .await
            .and_then(|sequence| {
                sequence
                    .in_range(range.0, range.1, Some(requester))?
                    .ok_or(Error::NetworkData(DtError::NoSuchEntry))
            }) {
            Ok(res) => Ok(res),
            Err(Error::NoSuchData(addr)) => return Err(Error::NoSuchData(addr)),
            Err(error) => Err(convert_to_error_message(error)),
        };

        Ok(NodeDuty::Send(build_client_query_response(
            QueryResponse::GetSequenceRange(result),
            msg_id,
            origin,
        )))
    }

    async fn get_last_entry(
        &self,
        address: Address,
        msg_id: MessageId,
        requester: PublicKey,
        origin: EndUser,
    ) -> Result<NodeDuty> {
        let result = match self
            .get_sequence(&address, Action::Read, requester)
            .await
            .and_then(|sequence| match sequence.last_entry(Some(requester))? {
                Some(entry) => Ok((sequence.len(Some(requester))? - 1, entry.to_vec())),
                None => Err(Error::NetworkData(DtError::NoSuchEntry)),
            }) {
            Ok(res) => Ok(res),
            Err(Error::NoSuchData(addr)) => return Err(Error::NoSuchData(addr)),
            Err(error) => Err(convert_to_error_message(error)),
        };

        Ok(NodeDuty::Send(build_client_query_response(
            QueryResponse::GetSequenceLastEntry(result),
            msg_id,
            origin,
        )))
    }

    async fn get_user_permissions(
        &self,
        address: Address,
        user: SequenceUser,
        msg_id: MessageId,
        requester: PublicKey,
        origin: EndUser,
    ) -> Result<NodeDuty> {
        let result = match self
            .get_sequence(&address, Action::Read, requester)
            .await
            .and_then(|sequence| {
                sequence
                    .permissions(user, Some(requester))
                    .map_err(|e| e.into())
            }) {
            Ok(res) => Ok(res),
            Err(Error::NoSuchData(addr)) => return Err(Error::NoSuchData(addr)),
            Err(error) => Err(convert_to_error_message(error)),
        };

        Ok(NodeDuty::Send(build_client_query_response(
            QueryResponse::GetSequenceUserPermissions(result),
            msg_id,
            origin,
        )))
    }

    async fn get_public_policy(
        &self,
        address: Address,
        msg_id: MessageId,
        requester: PublicKey,
        origin: EndUser,
    ) -> Result<NodeDuty> {
        let result = match self
            .get_sequence(&address, Action::Read, requester)
            .await
            .and_then(|sequence| {
                let res = if sequence.is_public() {
                    let policy = sequence.public_policy()?;
                    policy.clone()
                } else {
                    return Err(Error::NetworkData(DtError::CrdtUnexpectedState));
                };
                Ok(res)
            }) {
            Ok(res) => Ok(res),
            Err(Error::NoSuchData(addr)) => return Err(Error::NoSuchData(addr)),
            Err(error) => Err(convert_to_error_message(error)),
        };

        Ok(NodeDuty::Send(build_client_query_response(
            QueryResponse::GetSequencePublicPolicy(result),
            msg_id,
            origin,
        )))
    }

    async fn get_private_policy(
        &self,
        address: Address,
        msg_id: MessageId,
        requester: PublicKey,
        origin: EndUser,
    ) -> Result<NodeDuty> {
        let result = match self
            .get_sequence(&address, Action::Read, requester)
            .await
            .and_then(|sequence| {
                let res = if !sequence.is_public() {
                    let policy = sequence.private_policy(Some(requester))?;
                    policy.clone()
                } else {
                    return Err(Error::NetworkData(DtError::CrdtUnexpectedState));
                };
                Ok(res)
            }) {
            Ok(res) => Ok(res),
            Err(Error::NoSuchData(addr)) => return Err(Error::NoSuchData(addr)),
            Err(error) => Err(convert_to_error_message(error)),
        };

        Ok(NodeDuty::Send(build_client_query_response(
            QueryResponse::GetSequencePrivatePolicy(result),
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
                info!("Error on writing Sequence! {:?}", error);
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

async fn new_store(id: XorName, path: &Path) -> Result<EventStore<SequenceCmd>> {
    let db_dir = path.join("db").join("sequence".to_string());
    EventStore::new(id, db_dir.as_path())
        .await
        .map_err(Error::from)
}

impl Display for SequenceStorage {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "SequenceStorage")
    }
}
