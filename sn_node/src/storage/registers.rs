// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{
    errors::convert_to_error_msg,
    register_store::{RegisterLog, RegisterStore},
    Error, Result,
};

use sn_interface::{
    messaging::{
        data::{
            CreateRegister, EditRegister, OperationId, RegisterCmd, RegisterQuery,
            SignedRegisterCreate, SignedRegisterEdit,
        },
        system::NodeQueryResponse,
        SectionAuth, ServiceAuth, VerifyAuthority,
    },
    types::{
        register::{Action, EntryHash, Permissions, Policy, Register, User},
        Keypair, PublicKey, RegisterAddress, ReplicatedRegisterLog, SPENTBOOK_TYPE_TAG,
    },
};

use crate::UsedSpace;
use bincode::serialize;
use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    path::{Path, PathBuf},
};
use tracing::info;
#[cfg(test)]
use xor_name::Prefix;
use xor_name::XorName;

const REGISTER_STORE_DIR_NAME: &str = "register";

/// Operations over the data type Register.
#[derive(Debug, Clone)]
pub(super) struct RegisterStorage {
    file_store: RegisterStore,
}

impl RegisterStorage {
    /// Create new `RegisterStorage`
    pub(crate) fn new(path: &Path, used_space: UsedSpace) -> Result<Self> {
        let file_store = RegisterStore::new(path.join(REGISTER_STORE_DIR_NAME), used_space)?;

        Ok(Self { file_store })
    }

    #[allow(dead_code)]
    pub(crate) async fn remove_register(&mut self, address: &RegisterAddress) -> Result<()> {
        trace!("Removing register, {:?}", address);
        self.file_store.delete_data(address).await
    }

    pub(crate) async fn addrs(&self) -> Vec<RegisterAddress> {
        self.file_store.list_all_reg_addrs().await
    }

    /// Used for replication of data to new Adults.
    pub(crate) async fn get_register_replica(
        &self,
        address: &RegisterAddress,
    ) -> Result<ReplicatedRegisterLog> {
        let (register, section_auth, op_log, _) = self.try_load_stored_register(address).await?;

        // Build the replicaed register log assuming ops stored are all valid and correctly
        // signed since we performed such validations before storing them.
        Ok(ReplicatedRegisterLog {
            address: *register.address(),
            section_auth,
            op_log,
        })
    }

    // TODO: Use it for replication of data to new Adults.
    #[cfg(test)]
    async fn get_data_of(&mut self, prefix: Prefix) -> Result<Vec<ReplicatedRegisterLog>> {
        let mut the_data = vec![];

        let all_addrs = self.addrs().await;

        // TODO: make this concurrent
        for addr in all_addrs {
            match self.try_load_stored_register(&addr).await {
                Ok((register, section_auth, op_log, _)) => {
                    if prefix.matches(register.name()) {
                        let replica = ReplicatedRegisterLog {
                            address: *register.address(),
                            section_auth,
                            op_log,
                        };
                        the_data.push(replica);
                    }
                }
                Err(Error::RegisterNotFound { addr, .. }) => {
                    return Err(Error::InvalidRegisterStore(addr))
                }
                Err(e) => return Err(e),
            }
        }

        Ok(the_data)
    }

    /// On receiving data from other nodes.
    pub(crate) async fn update(&mut self, data: &ReplicatedRegisterLog) -> Result<()> {
        debug!("Updating Register store");
        let stored_reg = self
            .file_store
            .open_reg_log_from_disk(&data.address)
            .await?;

        let mut log_to_write = Vec::new();
        let register = match stored_reg.state {
            Some((register, _)) => Some(register),
            None => data.op_log.iter().find_map(|op| match op {
                RegisterCmd::Create {
                    cmd:
                        SignedRegisterCreate {
                            op: CreateRegister { name, tag, policy },
                            ..
                        },
                    section_auth: _,
                } => {
                    log_to_write.push(op.clone());
                    Some(Register::new(*policy.owner(), *name, *tag, policy.clone()))
                }
                _ => None,
            }),
        };

        let log_to_write = if let Some(mut reg) = register {
            for replicated_cmd in data.op_log.iter() {
                if let Err(err) = self.apply(replicated_cmd, &mut reg).await {
                    warn!(
                        "Discarding ReplicatedRegisterLog cmd {:?}: {:?}",
                        replicated_cmd, err
                    );
                } else {
                    log_to_write.push(replicated_cmd.clone());
                }
            }
            &log_to_write
        } else {
            &data.op_log
        };

        // write the new cmds all to disk
        self.file_store
            .write_log_to_disk(log_to_write, &stored_reg.ops_log_path)
            .await?;

        Ok(())
    }

    /// --- Writing ---

    pub(crate) async fn write(&mut self, cmd: &RegisterCmd) -> Result<()> {
        info!("Writing register cmd: {:?}", cmd);
        let log_path = match self.try_load_stored_register(&cmd.dst_address()).await {
            Ok((mut register, _, _, log_path)) => {
                self.apply(cmd, &mut register).await?;
                log_path
            }
            Err(Error::RegisterNotFound { path, .. }) => {
                // we still store the op
                if let RegisterCmd::Create {
                    cmd: SignedRegisterCreate { op, auth },
                    ..
                } = cmd
                {
                    debug!("Creating Register....");
                    // TODO 1: in higher layers we must verify that the section_auth is from a proper section..!
                    // TODO 2: Enable this check once we have section signature over the container key.
                    // let public_key = section_auth.sig.public_key;
                    // let _ = section_auth.verify_authority(key).or(Err(Error::InvalidSignature(PublicKey::Bls(public_key))))?;
                    let public_key = auth.public_key;
                    let _ = auth
                        .clone()
                        .verify_authority(serialize(op)?)
                        .or(Err(Error::InvalidSignature(public_key)))?;

                    trace!("Creating new register");
                }
                path
            }
            Err(other) => return Err(other),
        };

        // write the (single cmd) log to disk
        self.file_store
            .write_log_to_disk(&vec![cmd.clone()], &log_path)
            .await
    }

    // Try to apply the provided cmd to the register log and state, performing all vaidations
    async fn apply(&mut self, cmd: &RegisterCmd, register: &mut Register) -> Result<()> {
        // rough estimate of the RegisterCmd
        let required_space = std::mem::size_of::<RegisterCmd>();
        if !self.file_store.can_add(required_space) {
            return Err(Error::NotEnoughSpace);
        }

        let cmd_dst_addr = cmd.dst_address();
        if &cmd_dst_addr != register.address() {
            return Err(Error::RegisterAddrMismatch {
                cmd_dst_addr,
                reg_addr: *register.address(),
            });
        }

        match cmd {
            RegisterCmd::Create { .. } => Err(Error::DataExists),
            RegisterCmd::Edit(SignedRegisterEdit { op, auth }) => {
                let public_key = auth.public_key;
                let _ = auth
                    .clone()
                    .verify_authority(serialize(op)?)
                    .or(Err(Error::InvalidSignature(public_key)))?;

                info!("Editing Register");
                register.check_permissions(Action::Write, Some(User::Key(public_key)))?;
                let result = register
                    .apply_op(op.edit.clone())
                    .map_err(Error::NetworkData);

                match result {
                    Ok(()) => {
                        trace!("Editing Register success!");
                        Ok(())
                    }
                    Err(err) => {
                        trace!("Editing Register failed!: {:?}", err);
                        Err(err)
                    }
                }
            }
        }
    }

    /// Temporary helper function which makes sure there exists a Register for the spentbook,
    /// this shouldn't be required once we have a Spentbook data type.
    pub(crate) async fn create_spentbook_register(
        &mut self,
        address: &RegisterAddress,
        pk: PublicKey,
        keypair: Keypair,
    ) -> Result<()> {
        trace!("Creating new spentbook register: {:?}", address);

        let mut permissions = BTreeMap::new();
        let _ = permissions.insert(User::Anyone, Permissions::new(true));
        let owner = User::Key(pk);
        let policy = Policy { owner, permissions };

        let cmd = create_reg_w_policy(*address.name(), SPENTBOOK_TYPE_TAG, policy, keypair)?;

        match self.write(&cmd).await {
            Ok(()) | Err(Error::DataExists) => Ok(()),
            other => other,
        }
    }

    /// --- Reading ---

    pub(crate) async fn read(&self, read: &RegisterQuery, requester: User) -> NodeQueryResponse {
        trace!("Reading register {:?}", read.dst_address());
        let operation_id = match read.operation_id() {
            Ok(id) => id,
            Err(_e) => {
                return NodeQueryResponse::FailedToCreateOperationId;
            }
        };
        trace!("Operation of register read: {:?}", operation_id);
        use RegisterQuery::*;
        match read {
            Get(address) => self.get(*address, requester, operation_id).await,
            Read(address) => self.read_register(*address, requester, operation_id).await,
            GetOwner(address) => self.get_owner(*address, requester, operation_id).await,
            GetEntry { address, hash } => {
                self.get_entry(*address, *hash, requester, operation_id)
                    .await
            }
            GetPolicy(address) => self.get_policy(*address, requester, operation_id).await,
            GetUserPermissions { address, user } => {
                self.get_user_permissions(*address, *user, requester, operation_id)
                    .await
            }
        }
    }

    /// Get `Register` from the store and check permissions.
    async fn get_register(
        &self,
        address: &RegisterAddress,
        action: Action,
        requester: User,
    ) -> Result<Register> {
        let (register, _, _, _) = self.try_load_stored_register(address).await?;
        register
            .check_permissions(action, Some(requester))
            .map_err(Error::from)?;

        Ok(register)
    }

    /// Get entire Register.
    async fn get(
        &self,
        address: RegisterAddress,
        requester: User,
        operation_id: OperationId,
    ) -> NodeQueryResponse {
        let result = match self.get_register(&address, Action::Read, requester).await {
            Ok(register) => Ok(register),
            Err(error) => {
                error!("Error reading register from disk {error:?}");
                Err(convert_to_error_msg(error))
            }
        };

        NodeQueryResponse::GetRegister((result, operation_id))
    }

    async fn read_register(
        &self,
        address: RegisterAddress,
        requester: User,
        operation_id: OperationId,
    ) -> NodeQueryResponse {
        let result = match self.get_register(&address, Action::Read, requester).await {
            Ok(register) => Ok(register.read()),
            Err(error) => Err(error),
        };

        NodeQueryResponse::ReadRegister((result.map_err(convert_to_error_msg), operation_id))
    }

    async fn get_owner(
        &self,
        address: RegisterAddress,
        requester: User,
        operation_id: OperationId,
    ) -> NodeQueryResponse {
        let result = match self.get_register(&address, Action::Read, requester).await {
            Ok(res) => Ok(res.owner()),
            Err(error) => Err(convert_to_error_msg(error)),
        };

        NodeQueryResponse::GetRegisterOwner((result, operation_id))
    }

    async fn get_entry(
        &self,
        address: RegisterAddress,
        hash: EntryHash,
        requester: User,
        operation_id: OperationId,
    ) -> NodeQueryResponse {
        let result = match self
            .get_register(&address, Action::Read, requester)
            .await
            .and_then(|register| register.get(hash).map(|c| c.clone()).map_err(Error::from))
        {
            Ok(res) => Ok(res),
            Err(error) => Err(convert_to_error_msg(error)),
        };

        NodeQueryResponse::GetRegisterEntry((result, operation_id))
    }

    async fn get_user_permissions(
        &self,
        address: RegisterAddress,
        user: User,
        requester: User,
        operation_id: OperationId,
    ) -> NodeQueryResponse {
        let result = match self
            .get_register(&address, Action::Read, requester)
            .await
            .and_then(|register| register.permissions(user).map_err(Error::from))
        {
            Ok(res) => Ok(res),
            Err(error) => Err(convert_to_error_msg(error)),
        };

        NodeQueryResponse::GetRegisterUserPermissions((result, operation_id))
    }

    async fn get_policy(
        &self,
        address: RegisterAddress,
        requester_pk: User,
        operation_id: OperationId,
    ) -> NodeQueryResponse {
        let result = match self
            .get_register(&address, Action::Read, requester_pk)
            .await
            .map(|register| register.policy().clone())
        {
            Ok(res) => Ok(res),
            Err(error) => Err(convert_to_error_msg(error)),
        };

        NodeQueryResponse::GetRegisterPolicy((result, operation_id))
    }

    // ========================================================================
    // =========================== Helpers ====================================
    // ========================================================================

    // Gets stored register log from disk, trying to reconstruct the Register
    async fn try_load_stored_register(
        &self,
        addr: &RegisterAddress,
    ) -> Result<(Register, SectionAuth, RegisterLog, PathBuf)> {
        let stored_reg = self.file_store.open_reg_log_from_disk(addr).await?;
        // if we have the Register creation cmd, apply all ops to reconstruct the Register
        match stored_reg.state {
            None => Err(Error::RegisterNotFound {
                addr: *addr,
                path: stored_reg.ops_log_path,
            }),
            Some((mut register, section_auth)) => {
                for edit_op in stored_reg.ops_log.iter() {
                    if let RegisterCmd::Edit(SignedRegisterEdit {
                        op: EditRegister { edit, .. },
                        ..
                    }) = edit_op
                    {
                        register
                            .apply_op(edit.clone())
                            .map_err(Error::NetworkData)?;
                    }
                }

                Ok((
                    register,
                    section_auth,
                    stored_reg.ops_log,
                    stored_reg.ops_log_path,
                ))
            }
        }
    }
}

impl Display for RegisterStorage {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "RegisterStorage")
    }
}

// Helper functions temporarily used for spentbook logic, but also used for tests.
// This shouldn't be required outside of tests once we have a Spentbook data type.
fn create_reg_w_policy(
    name: XorName,
    tag: u64,
    policy: Policy,
    keypair: Keypair,
) -> Result<RegisterCmd> {
    let op = CreateRegister { name, tag, policy };
    let signature = keypair.sign(&serialize(&op)?);

    let auth = ServiceAuth {
        public_key: keypair.public_key(),
        signature,
    };

    Ok(RegisterCmd::Create {
        cmd: SignedRegisterCreate { op, auth },
        section_auth: section_auth(),
    })
}

fn section_auth() -> SectionAuth {
    use sn_interface::messaging::system::KeyedSig;

    let sk = bls::SecretKey::random();
    let public_key = sk.public_key();
    let data = "TODO-spentbook".to_string();
    let signature = sk.sign(&data);
    let sig = KeyedSig {
        public_key,
        signature,
    };
    SectionAuth {
        src_name: sn_interface::types::PublicKey::Bls(public_key).into(),
        sig,
    }
}

#[cfg(test)]
mod test {
    use super::{create_reg_w_policy, Error, RegisterStorage, UsedSpace};
    use sn_interface::{
        messaging::{
            data::{RegisterCmd, RegisterQuery},
            system::NodeQueryResponse,
        },
        types::{
            register::{EntryHash, Policy, User},
            Keypair,
        },
    };

    use eyre::{bail, eyre, Result};
    use rand::Rng;
    use tempfile::tempdir;
    use xor_name::Prefix;

    #[tokio::test]
    async fn test_register_write() -> Result<()> {
        // setup store
        let mut store = new_store()?;

        // create register
        let (cmd, authority) = create_register()?;
        store.write(&cmd).await?;

        // get register
        let address = cmd.dst_address();
        match store.read(&RegisterQuery::Get(address), authority).await {
            NodeQueryResponse::GetRegister((Ok(reg), _)) => {
                assert_eq!(reg.address(), &address, "Should have same address!");
                assert_eq!(reg.owner(), authority, "Should have same owner!");
            }
            e => bail!("Could not read register! {:?}", e),
        }

        match store.write(&cmd).await {
            Ok(()) => Err(eyre!("An error should occur for this test case")),
            Err(error) => match error {
                Error::DataExists => {
                    assert_eq!(error.to_string(), "Data already exists at this node");
                    Ok(())
                }
                _ => Err(eyre!("A Error::DataExists variant was expected")),
            },
        }
    }

    #[tokio::test]
    async fn test_register_export() -> Result<()> {
        // setup store
        let mut store = new_store()?;

        // create register
        let (cmd, authority) = create_register()?;
        store.write(&cmd).await?;

        // export db
        // get all data in db
        let prefix = Prefix::new(0, cmd.name());
        let for_update = store.get_data_of(prefix).await?;

        // create new db and update it with the data from first db
        let mut new_store = new_store()?;

        for log in for_update {
            new_store.update(&log).await?;
        }
        let address = cmd.dst_address();
        // assert the same tests hold as for the first db

        // should fail to write same register again, also on this new store
        match new_store.write(&cmd).await {
            Ok(()) => {
                return Err(eyre!("An error should occur for this test case"));
            }
            Err(error) => match error {
                Error::DataExists => {
                    assert_eq!(error.to_string(), "Data already exists at this node");
                }
                _ => {
                    return Err(eyre!("A Error::DataExists variant was expected"));
                }
            },
        }

        // should be able to read the same value from this new store also
        let res = new_store
            .read(&RegisterQuery::Get(address), authority)
            .await;

        match res {
            NodeQueryResponse::GetRegister((Ok(reg), _)) => {
                assert_eq!(reg.address(), &address, "Should have same address!");
                assert_eq!(reg.owner(), authority, "Should have same owner!");
            }
            e => panic!("Could not read! {:?}", e),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_register_non_existing_entry() -> Result<()> {
        // setup store
        let mut store = new_store()?;

        // create register
        let (cmd, authority) = create_register()?;
        store.write(&cmd).await?;

        let hash = EntryHash(rand::thread_rng().gen::<[u8; 32]>());

        // try get permissions of random user
        let address = cmd.dst_address();
        let res = store
            .read(&RegisterQuery::GetEntry { address, hash }, authority)
            .await;
        match res {
            NodeQueryResponse::GetRegisterEntry((Err(e), _)) => {
                assert_eq!(e, sn_interface::messaging::data::Error::NoSuchEntry)
            }
            NodeQueryResponse::GetRegisterEntry((Ok(entry), _)) => {
                panic!("Should not exist any entry for random hash! {:?}", entry)
            }
            e => panic!("Could not read! {:?}", e),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_register_non_existing_permissions() -> Result<()> {
        // setup store
        let mut store = new_store()?;

        // create register
        let (cmd, authority) = create_register()?;
        store.write(&cmd).await?;

        let (user, _) = random_user();

        // try get permissions of random user
        let address = cmd.dst_address();
        let res = store
            .read(
                &RegisterQuery::GetUserPermissions { address, user },
                authority,
            )
            .await;
        match res {
            NodeQueryResponse::GetRegisterUserPermissions((Err(e), _)) => {
                assert_eq!(e, sn_interface::messaging::data::Error::NoSuchEntry)
            }
            NodeQueryResponse::GetRegisterUserPermissions((Ok(perms), _)) => panic!(
                "Should not exist any permissions for random user! {:?}",
                perms
            ),
            e => panic!("Could not read! {:?}", e),
        }

        Ok(())
    }

    fn new_store() -> Result<RegisterStorage> {
        let tmp_dir = tempdir()?;
        let path = tmp_dir.path();
        let used_space = UsedSpace::new(usize::MAX);
        let store = RegisterStorage::new(path, used_space)?;
        Ok(store)
    }

    fn random_user() -> (User, Keypair) {
        let keypair = Keypair::new_ed25519();
        let authority = User::Key(keypair.public_key());
        (authority, keypair)
    }

    fn create_register() -> Result<(RegisterCmd, User)> {
        let (authority, keypair) = random_user();
        let policy = Policy {
            owner: authority,
            permissions: Default::default(),
        };
        Ok((
            create_reg_w_policy(xor_name::rand::random(), 0, policy, keypair)?,
            authority,
        ))
    }
}
