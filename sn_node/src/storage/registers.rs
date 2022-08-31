// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{
    errors::convert_to_error_msg,
    register_store::{RegisterLog, RegisterStore, StoredRegister},
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
        DataAddress, Keypair, PublicKey, RegisterAddress, ReplicatedRegisterLog,
        SPENTBOOK_TYPE_TAG,
    },
};

use crate::UsedSpace;
use bincode::serialize;
use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    path::Path,
};
use tracing::info;
use xor_name::XorName;

const REGISTER_STORE_DIR_NAME: &str = "register";

/// Operations over the data type Register.
#[derive(Debug, Clone)]
pub(super) struct RegisterStorage {
    file_store: RegisterStore,
}

impl RegisterStorage {
    /// Create new `RegisterStorage`
    pub(super) fn new(path: &Path, used_space: UsedSpace) -> Result<Self> {
        let file_store = RegisterStore::new(path.join(REGISTER_STORE_DIR_NAME), used_space)?;

        Ok(Self { file_store })
    }

    #[allow(dead_code)]
    pub(super) async fn remove_register(&mut self, address: &RegisterAddress) -> Result<()> {
        trace!("Removing register, {:?}", address);
        self.file_store.delete_data(address).await
    }

    pub(super) async fn addrs(&self) -> Vec<RegisterAddress> {
        self.file_store.list_all_reg_addrs().await
    }

    /// Used for replication of data to new Adults.
    pub(super) async fn get_register_replica(
        &self,
        address: &RegisterAddress,
    ) -> Result<ReplicatedRegisterLog> {
        let (register, section_auth, op_log) = self.try_load_stored_register(address).await?;

        // Build the replicaed register log assuming ops stored are all valid and correctly
        // signed since we performed such validations before storing them.
        Ok(ReplicatedRegisterLog {
            address: *register.address(),
            section_auth,
            op_log,
        })
    }

    /// Update our Register's replica on receiving data from other nodes.
    pub(super) async fn update(&mut self, data: &ReplicatedRegisterLog) -> Result<()> {
        debug!("Updating Register store: {:?}", data.address);
        let mut stored_reg = self
            .file_store
            .open_reg_log_from_disk(&data.address)
            .await?;

        let mut log_to_write = Vec::new();
        for replicated_cmd in data.op_log.iter() {
            if let Err(err) = self
                .try_to_apply_cmd_against_register_state(replicated_cmd, &mut stored_reg)
                .await
            {
                warn!(
                    "Discarding ReplicatedRegisterLog cmd {:?}: {:?}",
                    replicated_cmd, err
                );
            } else {
                log_to_write.push(replicated_cmd.clone());
            }
        }

        // Write the new cmds all to disk
        self.file_store
            .write_log_to_disk(&log_to_write, &stored_reg.op_log_path)
            .await
    }

    /// --- Writing ---

    pub(super) async fn write(&mut self, cmd: &RegisterCmd) -> Result<()> {
        info!("Writing register cmd: {:?}", cmd);
        // Let's first try to load and reconstruct the replica of targetted Register
        // we have in local storage, to then try to apply the new command onto it.
        let mut stored_reg = self
            .file_store
            .open_reg_log_from_disk(&cmd.dst_address())
            .await?;

        self.try_to_apply_cmd_against_register_state(cmd, &mut stored_reg)
            .await?;

        // Everything went fine, let's write the single cmd to disk
        self.file_store
            .write_log_to_disk(&vec![cmd.clone()], &stored_reg.op_log_path)
            .await
    }

    /// --- Reading ---

    pub(super) async fn read(&self, read: &RegisterQuery, requester: User) -> NodeQueryResponse {
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
        let (register, _, _) = self.try_load_stored_register(address).await?;
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

    // Temporary helper function which makes sure there exists a Register for the spentbook,
    // this shouldn't be required once we have a Spentbook data type.
    pub(super) async fn create_spentbook_register(
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
            Ok(()) | Err(Error::DataExists(_)) => Ok(()),
            other => other,
        }
    }

    // Private helper which does all verification and tries to apply given cmd to given Register
    // state. It accumulates the cmd, if valid, into the log so further calls can be made with
    // the same state and log for other commands to be applied, as used by the `update` function.
    async fn try_to_apply_cmd_against_register_state(
        &mut self,
        cmd: &RegisterCmd,
        stored_reg: &mut StoredRegister,
    ) -> Result<()> {
        // If we have the target Register, try to apply the cmd, otherwise let's store
        // the cmd in our storage anyway, whenever we receive the 'Register create' cmd
        // it can be reconstructed from all cmds we hold. If this is a 'Register create'
        // cmd let's verify is valid before accepting it, however 'Edits cmds' cannot be
        // verified now untill we have the `Register create` cmd.
        match (stored_reg.state.as_mut(), cmd) {
            (Some((ref mut register, _)), cmd) => self.apply(cmd, register).await?,
            (
                None,
                RegisterCmd::Create {
                    cmd: SignedRegisterCreate { op, auth },
                    section_auth,
                },
            ) => {
                // the target Register is not in our store or we don't have the 'Register create',
                // let's verify the create cmd we received is valid and try to apply stored cmds we may have.
                let public_key = auth.public_key;
                let _ = auth
                    .clone()
                    .verify_authority(serialize(op)?)
                    .or(Err(Error::InvalidSignature(public_key)))?;

                trace!("Creating new register: {:?}", cmd.dst_address());
                // let's do a final check, let's try to apply all cmds to it,
                // those which are new cmds were not validated yet, so let's do it now.
                let mut register =
                    Register::new(*op.policy.owner(), op.name, op.tag, op.policy.clone());

                for cmd in stored_reg.op_log.iter() {
                    self.apply(cmd, &mut register).await?;
                }

                stored_reg.state = Some((register, section_auth.clone()));
            }
            (None, edit_cmd) => {
                // we cannot validate it, but we'll store it
                stored_reg.op_log.push(edit_cmd.clone());
            }
        }

        Ok(())
    }

    // Try to apply the provided cmd to the register log and state, performing all vaidations
    async fn apply(&mut self, cmd: &RegisterCmd, register: &mut Register) -> Result<()> {
        let addr = cmd.dst_address();
        if &addr != register.address() {
            return Err(Error::RegisterAddrMismatch {
                cmd_dst_addr: addr,
                reg_addr: *register.address(),
            });
        }

        match cmd {
            RegisterCmd::Create { .. } => Err(Error::DataExists(DataAddress::Register(addr))),
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
                        trace!("Editing Register success: {:?}", addr);
                        Ok(())
                    }
                    Err(err) => {
                        trace!("Editing Register failed {:?}: {:?}", addr, err);
                        Err(err)
                    }
                }
            }
        }
    }

    // Gets stored register log from disk, trying to reconstruct the Register
    // Note this doesn't perform any cmd sig/perms validation, it's only used when the log
    // is read from disk which has already been validated before storing it.
    async fn try_load_stored_register(
        &self,
        addr: &RegisterAddress,
    ) -> Result<(Register, SectionAuth, RegisterLog)> {
        let stored_reg = self.file_store.open_reg_log_from_disk(addr).await?;
        // if we have the Register creation cmd, apply all ops to reconstruct the Register
        match stored_reg.state {
            None => Err(Error::RegisterNotFound(*addr)),
            Some((mut register, section_auth)) => {
                for op in stored_reg.op_log.iter() {
                    if let RegisterCmd::Edit(SignedRegisterEdit {
                        op: EditRegister { edit, .. },
                        ..
                    }) = op
                    {
                        register
                            .apply_op(edit.clone())
                            .map_err(Error::NetworkData)?;
                    }
                }

                Ok((register, section_auth, stored_reg.op_log))
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
            Keypair, ReplicatedRegisterLog,
        },
    };

    use eyre::{bail, eyre, Result};
    use rand::Rng;
    use tempfile::tempdir;

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
                Error::DataExists(addr) => {
                    assert_eq!(
                        error.to_string(),
                        format!("Data already exists at this node: {:?}", addr)
                    );
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
        let address = cmd.dst_address();
        store.write(&cmd).await?;

        // should fail to write same register again
        match store.write(&cmd).await {
            Ok(()) => {
                return Err(eyre!("An error should occur for this test case"));
            }
            Err(error) => match error {
                Error::DataExists(addr) => {
                    assert_eq!(
                        error.to_string(),
                        format!("Data already exists at this node: {:?}", addr)
                    );
                }
                _ => {
                    return Err(eyre!("A Error::DataExists variant was expected"));
                }
            },
        }

        // export Registers, get all data we held in storage
        let all_addrs = store.addrs().await;
        let mut for_update = Vec::new();
        for addr in all_addrs {
            let (register, section_auth, op_log) = store.try_load_stored_register(&addr).await?;
            let replica = ReplicatedRegisterLog {
                address: *register.address(),
                section_auth,
                op_log,
            };
            for_update.push(replica);
        }

        // create new store and update it with the data from first store
        let mut new_store = new_store()?;
        for log in for_update {
            new_store.update(&log).await?;
        }

        // assert the same tests hold as for the first store
        // should fail to write same register again, also on this new store
        match new_store.write(&cmd).await {
            Ok(()) => {
                return Err(eyre!("An error should occur for this test case"));
            }
            Err(error) => match error {
                Error::DataExists(addr) => {
                    assert_eq!(
                        error.to_string(),
                        format!("Data already exists at this node: {:?}", addr)
                    );
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
