// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::dbs::{convert_to_error_msg, Error, FileStore, RegisterLog, Result};

use sn_interface::{
    messaging::{
        data::{
            CreateRegister, EditRegister, OperationId, RegisterCmd, RegisterQuery,
            RegisterStoreExport, ReplicatedRegisterLog, SignedRegisterCreate, SignedRegisterEdit,
        },
        system::NodeQueryResponse,
        SectionAuth, ServiceAuth, VerifyAuthority,
    },
    types::{
        register::{Action, EntryHash, Permissions, Policy, Register, User},
        Keypair, PublicKey, RegisterAddress, ReplicatedDataAddress as DataAddress,
        SPENTBOOK_TYPE_TAG,
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

const REGISTER_DB_NAME: &str = "register";

/// Operations over the data type Register.
#[derive(Debug, Clone)]
pub(super) struct RegisterStorage {
    file_store: FileStore,
}

#[derive(Clone, Debug)]
struct StoredRegister {
    state: Register,
    ops_log: RegisterLog,
    ops_log_path: PathBuf,
    section_auth: SectionAuth,
}

impl RegisterStorage {
    /// Create new `RegisterStorage`
    pub(crate) fn new(path: &Path, used_space: UsedSpace) -> Result<Self> {
        let file_store = FileStore::new(path.join(REGISTER_DB_NAME), used_space)?;

        Ok(Self { file_store })
    }

    #[allow(dead_code)]
    pub(crate) async fn remove_register(&mut self, address: &RegisterAddress) -> Result<()> {
        trace!("Removing register, {:?}", address);

        self.file_store
            .delete_data(&DataAddress::Register(*address))
            .await?;

        Ok(())
    }

    pub(crate) async fn addrs(&self) -> Vec<RegisterAddress> {
        self.file_store.list_all_reg_addrs().await
    }

    /// Used for replication of data to new Adults.
    pub(crate) async fn get_register_replica(
        &self,
        address: &RegisterAddress,
    ) -> Result<ReplicatedRegisterLog> {
        let stored_reg = match self.try_load_stored_register(address).await {
            Ok(stored_reg) => stored_reg,
            Err(Error::KeyNotFound(_key)) => {
                return Err(Error::NoSuchData(DataAddress::Register(*address)))
            }
            Err(e) => return Err(e),
        };

        self.create_replica(stored_reg)
    }

    fn create_replica(&self, stored_reg: StoredRegister) -> Result<ReplicatedRegisterLog> {
        let mut address = None;
        let op_log = stored_reg
            .ops_log
            .into_iter()
            .filter_map(|(_, stored_cmd)| {
                // only spread signed data
                match &stored_cmd {
                    RegisterCmd::Create { cmd, .. } => {
                        // TODO 1: in higher layers we must verify that the section_auth is from a proper section..!
                        // TODO 2: Enable this check once we have section signature over the container key.
                        // if section_auth.verify_authority(key).is_err() {
                        //     warn!("Invalid section auth on register container: {}", key);
                        //     return None;
                        // }
                        address = Some(cmd.dst_address());
                    }
                    RegisterCmd::Edit(SignedRegisterEdit { op, auth }) => {
                        let verification = auth.clone().verify_authority(serialize(op).ok()?);
                        if verification.is_err() {
                            error!(
                                "Invalid signature found for a cmd stored in db: {:?}",
                                stored_cmd
                            );
                            return None;
                        }
                    }
                }

                Some(stored_cmd)
            })
            .collect();

        Ok(ReplicatedRegisterLog {
            address: address.ok_or(Error::InvalidStore)?,
            section_auth: stored_reg.section_auth,
            op_log,
        })
    }

    /// Used for replication of data to new Adults.
    #[cfg(test)]
    pub(crate) async fn get_data_of(&mut self, prefix: Prefix) -> Result<RegisterStoreExport> {
        let mut the_data = vec![];

        let all_addrs = self.addrs().await;

        // TODO: make this concurrent
        for addr in all_addrs {
            match self.try_load_stored_register(&addr).await {
                Ok(stored_reg) => {
                    let read_only = stored_reg.state.clone();
                    if prefix.matches(read_only.name()) {
                        the_data.push(self.create_replica(stored_reg.clone())?);
                    }
                }
                Err(Error::KeyNotFound(_)) => return Err(Error::InvalidStore),
                Err(e) => return Err(e),
            }
        }

        Ok(RegisterStoreExport(the_data))
    }

    /// On receiving data from Elders when promoted.
    pub(crate) async fn update(&mut self, store_data: RegisterStoreExport) -> Result<()> {
        debug!("Updating Register store");

        let RegisterStoreExport(registers) = store_data;

        // nested loops, slow..
        for data in registers {
            let key = data.address.id()?;
            for replicated_cmd in data.op_log {
                if replicated_cmd.dst_address() != data.address {
                    warn!(
                        "Corrupt ReplicatedRegisterLog, op log contains foreign ops: {}",
                        key
                    );
                    continue;
                }
                self.apply(replicated_cmd).await?;
            }
        }

        Ok(())
    }

    /// --- Writing ---

    pub(crate) async fn write(&mut self, cmd: RegisterCmd) -> Result<()> {
        info!("Writing register cmd: {:?}", cmd);

        // rough estimate of the RegisterCmd
        let required_space = std::mem::size_of::<RegisterCmd>();
        if !self.file_store.can_add(required_space) {
            return Err(Error::NotEnoughSpace);
        }
        self.apply(cmd).await
    }

    // Append a new command and write to disk
    async fn append(
        &mut self,
        cmd: RegisterCmd,
        mut ops_log: RegisterLog,
        path: &Path,
    ) -> Result<()> {
        let reg_id = cmd.register_operation_id()?;
        if ops_log.get(&reg_id).is_some() {
            return Err(Error::DataExists);
        }

        let _old_cmd = ops_log.insert(reg_id, cmd);

        self.file_store.write_log_to_disk(&ops_log, path).await?;

        Ok(())
    }

    async fn apply(&mut self, cmd: RegisterCmd) -> Result<()> {
        let address = cmd.dst_address();
        match cmd.clone() {
            RegisterCmd::Create {
                cmd: SignedRegisterCreate { op, auth },
                ..
            } => {
                debug!("Creating Register....");
                // TODO 1: in higher layers we must verify that the section_auth is from a proper section..!
                // TODO 2: Enable this check once we have section signature over the container key.
                // let public_key = section_auth.sig.public_key;
                // let _ = section_auth.verify_authority(key).or(Err(Error::InvalidSignature(PublicKey::Bls(public_key))))?;

                let public_key = auth.public_key;
                let _ = auth
                    .verify_authority(serialize(&op)?)
                    .or(Err(Error::InvalidSignature(public_key)))?;

                if self
                    .file_store
                    .data_file_exists(&DataAddress::Register(address))?
                {
                    return Err(Error::DataExists);
                }

                // init store first, to allow append to happen asap after key insert
                // could be races, but edge case for later todos.
                let (ops_log, path) = self.file_store.open_reg_log_from_disk(&address).await?;

                trace!("Creating new register");

                // insert the op to the cmds log
                self.append(cmd, ops_log, &path).await?;

                Ok(())
            }
            RegisterCmd::Edit(SignedRegisterEdit { op, auth }) => {
                let public_key = auth.public_key;
                let _ = auth
                    .verify_authority(serialize(&op)?)
                    .or(Err(Error::InvalidSignature(public_key)))?;

                let EditRegister { edit, .. } = op;

                let mut stored_reg = self.try_load_stored_register(&address).await?;

                info!("Editing Register");
                stored_reg
                    .state
                    .check_permissions(Action::Write, Some(User::Key(public_key)))?;
                let result = stored_reg.state.apply_op(edit).map_err(Error::NetworkData);

                match result {
                    Ok(()) => {
                        self.append(cmd, stored_reg.ops_log, &stored_reg.ops_log_path)
                            .await?;

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

        match self.write(cmd).await {
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
        let stored_reg = match self.try_load_stored_register(address).await {
            Ok(stored_reg) => stored_reg,
            Err(Error::KeyNotFound(_key)) => {
                return Err(Error::NoSuchData(DataAddress::Register(*address)))
            }
            Err(e) => return Err(e),
        };

        let read_only = stored_reg.state;
        read_only
            .check_permissions(action, Some(requester))
            .map_err(Error::from)?;

        Ok(read_only)
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

    // gets stored register from disk
    async fn try_load_stored_register(&self, addr: &RegisterAddress) -> Result<StoredRegister> {
        // read from disk
        let (ops_log, ops_log_path) = self.file_store.open_reg_log_from_disk(addr).await?;

        // RegisterEdit ops could be read after the RegisterCreate op, so we should keep them
        // till we read the RegisterCreate op from disk, and then apply all queued ops on top.
        let mut queued = Vec::new();

        // apply all ops
        let mut hydrated_register = None;
        for (_, stored_cmd) in ops_log.clone().into_iter() {
            match stored_cmd {
                RegisterCmd::Create {
                    cmd:
                        SignedRegisterCreate {
                            op: CreateRegister { name, tag, policy },
                            ..
                        },
                    section_auth,
                } => {
                    // TODO: if we already have read a RegisterCreate op, check if there
                    // is any difference this other one, ... and perhaps log a warning if so?
                    hydrated_register = hydrated_register.or_else(|| {
                        Some((
                            Register::new(*policy.owner(), name, tag, policy),
                            section_auth,
                        ))
                    });
                }
                RegisterCmd::Edit(SignedRegisterEdit {
                    op: EditRegister { edit, .. },
                    ..
                }) => {
                    if let Some((state, _)) = &mut hydrated_register {
                        state.apply_op(edit).map_err(Error::NetworkData)?;
                    } else {
                        // we'll apply it after we read the RegisterCreate op from disk
                        queued.push(edit);
                    }
                }
            }
        }

        match hydrated_register {
            None => Err(Error::KeyNotFound(addr.id()?.to_string())),
            Some((mut state, section_auth)) => {
                // apply any queued RegisterEdit op
                for op in queued {
                    state.apply_op(op).map_err(Error::NetworkData)?;
                }

                Ok(StoredRegister {
                    state,
                    ops_log,
                    ops_log_path,
                    section_auth,
                })
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
    use super::{create_reg_w_policy, RegisterStorage};

    use crate::node::{Error, Result};
    use crate::UsedSpace;

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

    use rand::Rng;
    use tempfile::tempdir;
    use xor_name::Prefix;

    #[tokio::test]
    async fn test_register_write() -> Result<()> {
        // setup store
        let mut store = new_store()?;

        // create register
        let (cmd, authority) = create_register()?;
        store.write(cmd.clone()).await?;

        // get register

        let address = cmd.dst_address();
        let res = store.read(&RegisterQuery::Get(address), authority).await;
        match res {
            NodeQueryResponse::GetRegister((Ok(reg), _)) => {
                assert_eq!(reg.address(), &address, "Should have same address!");
                assert_eq!(reg.owner(), authority, "Should have same owner!");
            }
            e => panic!("Could not read! {:?}", e),
        }

        // try to create the register again
        // (should fail)

        let res = store.write(cmd).await;

        assert_eq!(
            res.err().unwrap().to_string(),
            Error::DataExists.to_string(),
            "Should not be able to create twice!"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_register_export() -> Result<()> {
        // setup store
        let mut store = new_store()?;

        // create register
        let (cmd, authority) = create_register()?;
        store.write(cmd.clone()).await?;

        // export db
        // get all data in db
        let prefix = Prefix::new(0, cmd.name());
        let for_update = store.get_data_of(prefix).await?;

        // create new db and update it with the data from first db
        let mut new_store = new_store()?;

        new_store.update(for_update).await?;
        let address = cmd.dst_address();
        // assert the same tests hold as for the first db

        // should fail to write same register again, also on this new store
        let res = new_store.write(cmd).await;

        assert_eq!(
            res.err().unwrap().to_string(),
            Error::DataExists.to_string(),
            "Should not be able to create twice!"
        );

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
        store.write(cmd.clone()).await?;

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
        store.write(cmd.clone()).await?;

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
