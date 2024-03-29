// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::register_store::{RegisterStore, StoredRegister};

use crate::protocol::{
    address::RegisterAddress,
    error::{Error, Result},
    messages::{
        EditRegister, QueryResponse, RegisterCmd, RegisterQuery, ReplicatedRegisterLog,
        SignedRegisterCreate, SignedRegisterEdit,
    },
    register::{Action, EntryHash, Register, User},
};

use bincode::serialize;

/// Operations over the Register data type and its storage.
#[derive(Clone, Default)]
pub(crate) struct RegisterStorage {
    register_store: RegisterStore,
}

impl RegisterStorage {
    /// Create new `RegisterStorage`
    pub(crate) fn new() -> Self {
        Self {
            register_store: RegisterStore::default(),
        }
    }

    /// --- Writing ---

    pub(crate) async fn write(&self, cmd: &RegisterCmd) -> Result<()> {
        info!("Writing register cmd: {cmd:?}");
        let addr = cmd.dst();
        // Let's first try to load and reconstruct the replica of targetted Register
        // we have in local storage, to then try to apply the new command onto it.
        let mut stored_reg = self.try_load_stored_register(&addr).await?;

        self.try_to_apply_cmd_against_register_state(cmd, &mut stored_reg)?;

        // Everything went fine, let's store the updated Register
        self.register_store
            .store_register_ops_log(&vec![cmd.clone()], stored_reg, addr)
            .await
    }

    /// Update our Register's replica on receiving data from other nodes.
    #[allow(dead_code)]
    pub(super) async fn update(&self, data: &ReplicatedRegisterLog) -> Result<()> {
        let addr = data.address;
        debug!("Updating Register store: {addr:?}");
        let mut stored_reg = self.try_load_stored_register(&addr).await?;

        let mut log_to_write = Vec::new();
        for replicated_cmd in &data.op_log {
            if let Err(err) =
                self.try_to_apply_cmd_against_register_state(replicated_cmd, &mut stored_reg)
            {
                warn!("Discarding ReplicatedRegisterLog cmd {replicated_cmd:?}: {err:?}",);
            } else {
                log_to_write.push(replicated_cmd.clone());
            }
        }

        // Write the new cmds all to disk
        self.register_store
            .store_register_ops_log(&log_to_write, stored_reg, addr)
            .await
    }

    /// --- Reading ---
    pub(crate) async fn read(&self, read: &RegisterQuery, requester: User) -> QueryResponse {
        trace!("Reading register: {:?}", read.dst());
        use RegisterQuery::*;
        match read {
            Get(address) => self.get(*address, requester).await,
            Read(address) => self.read_register(*address, requester).await,
            GetOwner(address) => self.get_owner(*address, requester).await,
            GetEntry { address, hash } => self.get_entry(*address, *hash, requester).await,
            GetPolicy(address) => self.get_policy(*address, requester).await,
            GetUserPermissions { address, user } => {
                self.get_user_permissions(*address, *user, requester).await
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
        let stored_reg = self.try_load_stored_register(address).await?;
        if let Some(register) = stored_reg.state {
            register
                .check_permissions(action, Some(requester))
                .map_err(Error::from)?;

            Ok(register)
        } else {
            Err(Error::RegisterNotFound(*address))
        }
    }

    /// Get entire Register.
    async fn get(&self, address: RegisterAddress, requester: User) -> QueryResponse {
        let result = self.get_register(&address, Action::Read, requester).await;

        QueryResponse::GetRegister(result)
    }

    async fn read_register(&self, address: RegisterAddress, requester: User) -> QueryResponse {
        let result = match self.get_register(&address, Action::Read, requester).await {
            Ok(register) => Ok(register.read()),
            Err(error) => Err(error),
        };

        QueryResponse::ReadRegister(result)
    }

    async fn get_owner(&self, address: RegisterAddress, requester: User) -> QueryResponse {
        let result = match self.get_register(&address, Action::Read, requester).await {
            Ok(res) => Ok(res.owner()),
            Err(error) => Err(error),
        };

        QueryResponse::GetRegisterOwner(result)
    }

    async fn get_entry(
        &self,
        address: RegisterAddress,
        hash: EntryHash,
        requester: User,
    ) -> QueryResponse {
        let result = self
            .get_register(&address, Action::Read, requester)
            .await
            .and_then(|register| register.get(hash).map(|c| c.clone()));

        QueryResponse::GetRegisterEntry(result)
    }

    async fn get_user_permissions(
        &self,
        address: RegisterAddress,
        user: User,
        requester: User,
    ) -> QueryResponse {
        let result = self
            .get_register(&address, Action::Read, requester)
            .await
            .and_then(|register| register.permissions(user));

        QueryResponse::GetRegisterUserPermissions(result)
    }

    async fn get_policy(&self, address: RegisterAddress, requester_pk: User) -> QueryResponse {
        let result = self
            .get_register(&address, Action::Read, requester_pk)
            .await
            .map(|register| register.policy().clone());

        QueryResponse::GetRegisterPolicy(result)
    }

    // ========================================================================
    // =========================== Helpers ====================================
    // ========================================================================

    // Private helper which does all verification and tries to apply given cmd to given Register
    // state. It accumulates the cmd, if valid, into the log so further calls can be made with
    // the same state and log, as used by the `update` function.
    // Note the cmd is always pushed to the log even if it's a duplicated cmd.
    fn try_to_apply_cmd_against_register_state(
        &self,
        cmd: &RegisterCmd,
        stored_reg: &mut StoredRegister,
    ) -> Result<()> {
        // If we have the target Register, try to apply the cmd, otherwise let's keep
        // the cmd in the log anyway, whenever we receive the 'Register create' cmd
        // it can be reconstructed from all cmds we hold in the log. If this is a 'Register create'
        // cmd let's verify it's valid before accepting it, however 'Edits cmds' cannot be
        // verified until we have the `Register create` cmd.
        match (stored_reg.state.as_mut(), cmd) {
            (Some(_), RegisterCmd::Create { .. }) => return Ok(()), // no op, since already created
            (Some(ref mut register), RegisterCmd::Edit(_)) => self.apply(cmd, register)?,
            (None, RegisterCmd::Create(cmd)) => {
                // the target Register is not in our store or we don't have the 'Register create',
                // let's verify the create cmd we received is valid and try to apply stored cmds we may have.
                let SignedRegisterCreate { op, auth } = cmd;
                auth.verify_authority(serialize(op).map_err(|e| Error::Bincode(e.to_string()))?)?;

                trace!("Creating new register: {:?}", cmd.dst());
                // let's do a final check, let's try to apply all cmds to it,
                // those which are new cmds were not validated yet, so let's do it now.
                let mut register =
                    Register::new(*op.policy.owner(), op.name, op.tag, op.policy.clone());

                for cmd in &stored_reg.op_log {
                    self.apply(cmd, &mut register)?;
                }

                stored_reg.state = Some(register);
            }
            (None, _edit_cmd) => { /* we cannot validate it right now, but we'll store it */ }
        }

        stored_reg.op_log.push(cmd.clone());
        Ok(())
    }

    // Try to apply the provided cmd to the register state, performing all op validations
    fn apply(&self, cmd: &RegisterCmd, register: &mut Register) -> Result<()> {
        let addr = cmd.dst();
        if &addr != register.address() {
            return Err(Error::RegisterAddrMismatch {
                cmd_dst_addr: addr,
                reg_addr: *register.address(),
            });
        }

        match cmd {
            RegisterCmd::Create { .. } => Ok(()),
            RegisterCmd::Edit(SignedRegisterEdit { op, auth }) => {
                auth.verify_authority(serialize(op).map_err(|e| Error::Bincode(e.to_string()))?)?;

                info!("Editing Register: {addr:?}");
                let public_key = auth.public_key;
                register.check_permissions(Action::Write, Some(User::Key(public_key)))?;
                let result = register.apply_op(op.edit.clone());

                match result {
                    Ok(()) => {
                        trace!("Editing Register success: {addr:?}");
                        Ok(())
                    }
                    Err(err) => {
                        trace!("Editing Register failed {addr:?}: {err:?}");
                        Err(err)
                    }
                }
            }
        }
    }

    // Gets stored register log from disk, trying to reconstruct the Register
    // Note this doesn't perform any cmd sig/perms validation, it's only used when the log
    // is read from disk which has already been validated before storing it.
    async fn try_load_stored_register(&self, addr: &RegisterAddress) -> Result<StoredRegister> {
        let mut stored_reg = self.register_store.get(addr).await;
        // if we have the Register creation cmd, apply all ops to reconstruct the Register
        if let Some(register) = &mut stored_reg.state {
            for cmd in &stored_reg.op_log {
                if let RegisterCmd::Edit(SignedRegisterEdit { op, .. }) = cmd {
                    let EditRegister { edit, .. } = op;
                    register.apply_op(edit.clone())?;
                }
            }
        }

        Ok(stored_reg)
    }

    #[cfg(test)]
    async fn addrs(&self) -> Vec<RegisterAddress> {
        self.register_store.addrs().await
    }

    /// Used for replication of data to new nodes.
    /// Currently only used by the tests.
    /// TODO: to be used by replication logic.
    #[cfg(test)]
    async fn get_register_replica(
        &self,
        address: &RegisterAddress,
    ) -> Result<ReplicatedRegisterLog> {
        let stored_reg = self.try_load_stored_register(address).await?;
        // Build the replicated register log assuming ops stored are all valid and correctly
        // signed since we performed such validations before storing them.
        Ok(ReplicatedRegisterLog {
            address: *address,
            op_log: stored_reg.op_log,
        })
    }
}

#[cfg(test)]
mod test {
    use super::RegisterStorage;
    use crate::protocol::{
        authority::DataAuthority,
        error::Error,
        messages::{
            CreateRegister, EditRegister, QueryResponse, RegisterCmd, RegisterQuery,
            SignedRegisterCreate, SignedRegisterEdit,
        },
        register::{EntryHash, Policy, Register, User},
    };

    use bincode::serialize;
    use bls::SecretKey;
    use eyre::{bail, Result};
    use rand::{distributions::Alphanumeric, Rng};
    use std::collections::BTreeSet;
    use xor_name::XorName;

    // Helper functions temporarily used for spentbook logic, but also used for tests.
    // This shouldn't be required outside of tests once we have a Spentbook data type.
    fn create_reg_w_policy(
        name: XorName,
        tag: u64,
        policy: Policy,
        sk: &SecretKey,
    ) -> Result<RegisterCmd> {
        let op = CreateRegister { name, tag, policy };
        let signature = sk.sign(serialize(&op)?);

        let auth = DataAuthority {
            public_key: sk.public_key(),
            signature,
        };

        Ok(RegisterCmd::Create(SignedRegisterCreate { op, auth }))
    }

    #[tokio::test]
    async fn test_register_try_load_stored() -> Result<()> {
        let store = RegisterStorage::default();

        let (cmd_create, _, sk, name, policy) = create_register()?;
        let addr = cmd_create.dst();
        let mut register = Register::new(*policy.owner(), name, 0, policy);

        let stored_reg = store.try_load_stored_register(&addr).await?;
        // it should *not* contain the create cmd
        assert!(stored_reg.state.is_none());
        assert!(stored_reg.op_log.is_empty());

        store.write(&cmd_create).await?;
        let stored_reg = store.try_load_stored_register(&addr).await?;
        // it should contain the create cmd
        assert_eq!(stored_reg.state.as_ref(), Some(&register));
        assert_eq!(stored_reg.op_log, vec![cmd_create.clone()]);
        assert_eq!(stored_reg.state.map(|reg| reg.size()), Some(0));

        // let's now edit the register
        let cmd_edit = edit_register(&mut register, &sk)?;
        store.write(&cmd_edit).await?;

        let stored_reg = store.try_load_stored_register(&addr).await?;
        // it should contain the create and edit cmds
        assert_eq!(stored_reg.state.as_ref(), Some(&register));
        assert_eq!(stored_reg.op_log.len(), 2);
        assert!(
            stored_reg
                .op_log
                .iter()
                .all(|op| [&cmd_create, &cmd_edit].contains(&op)),
            "Op log doesn't match"
        );
        assert_eq!(stored_reg.state.map(|reg| reg.size()), Some(1));

        Ok(())
    }

    #[tokio::test]
    async fn test_register_try_load_stored_inverted_cmds_order() -> Result<()> {
        let store = RegisterStorage::default();

        let (cmd_create, _, sk, name, policy) = create_register()?;
        let addr = cmd_create.dst();
        let mut register = Register::new(*policy.owner(), name, 0, policy);

        // let's first store an edit cmd for the register
        let cmd_edit = edit_register(&mut register, &sk)?;
        store.write(&cmd_edit).await?;

        let stored_reg = store.try_load_stored_register(&addr).await?;
        // it should contain the edit cmd only
        assert_eq!(stored_reg.state, None);
        assert_eq!(stored_reg.op_log, vec![cmd_edit.clone()]);

        // and now store the create cmd for the register
        store.write(&cmd_create).await?;

        let stored_reg = store.try_load_stored_register(&addr).await?;
        // it should contain the create and edit cmds
        assert_eq!(stored_reg.state.as_ref(), Some(&register));
        assert_eq!(stored_reg.op_log.len(), 2);
        assert!(
            stored_reg
                .op_log
                .iter()
                .all(|op| [&cmd_create, &cmd_edit].contains(&op)),
            "Op log doesn't match"
        );
        assert_eq!(stored_reg.state.map(|reg| reg.size()), Some(1));

        Ok(())
    }

    #[tokio::test]
    async fn test_register_apply_cmd_against_state() -> Result<()> {
        let store = RegisterStorage::default();

        let (cmd_create, _, sk, name, policy) = create_register()?;
        let addr = cmd_create.dst();
        let mut register = Register::new(*policy.owner(), name, 0, policy);
        let mut stored_reg = store.try_load_stored_register(&addr).await?;

        // apply the create cmd
        store.try_to_apply_cmd_against_register_state(&cmd_create, &mut stored_reg)?;
        // it should contain the create cmd
        assert_eq!(stored_reg.state.as_ref(), Some(&register));
        assert_eq!(stored_reg.op_log, vec![cmd_create.clone()]);
        assert_eq!(stored_reg.state.as_ref().map(|reg| reg.size()), Some(0));

        // apply the create cmd again should change nothing
        match store.try_to_apply_cmd_against_register_state(&cmd_create, &mut stored_reg) {
            Ok(()) => (),
            Err(err) => bail!(
                "An error should not occur when applying create cmd again: {:?}",
                err
            ),
        }

        // let's now apply an edit cmd
        let cmd_edit = edit_register(&mut register, &sk)?;
        store.try_to_apply_cmd_against_register_state(&cmd_edit, &mut stored_reg)?;
        // it should contain the create and edit cmds
        assert_eq!(stored_reg.state.as_ref(), Some(&register));
        assert_eq!(stored_reg.op_log.len(), 2);
        assert!(
            stored_reg
                .op_log
                .iter()
                .all(|op| [&cmd_create, &cmd_edit].contains(&op)),
            "Op log doesn't match"
        );
        assert_eq!(stored_reg.state.as_ref().map(|reg| reg.size()), Some(1));

        // applying the edit cmd again shouldn't fail or alter the register content,
        // although the log will contain the edit cmd duplicated
        store.try_to_apply_cmd_against_register_state(&cmd_edit, &mut stored_reg)?;
        assert_eq!(stored_reg.state.as_ref(), Some(&register));
        assert_eq!(stored_reg.op_log.len(), 3);
        assert!(
            stored_reg
                .op_log
                .iter()
                .all(|op| [&cmd_create, &cmd_edit].contains(&op)),
            "Op log doesn't match"
        );
        assert_eq!(stored_reg.state.map(|reg| reg.size()), Some(1));

        Ok(())
    }

    #[tokio::test]
    async fn test_register_apply_cmd_against_state_inverted_cmds_order() -> Result<()> {
        let store = RegisterStorage::default();

        let (cmd_create, _, sk, name, policy) = create_register()?;
        let addr = cmd_create.dst();
        let mut register = Register::new(*policy.owner(), name, 0, policy);
        let mut stored_reg = store.try_load_stored_register(&addr).await?;

        // apply an edit cmd first
        let cmd_edit = edit_register(&mut register, &sk)?;
        store.try_to_apply_cmd_against_register_state(&cmd_edit, &mut stored_reg)?;
        // it should contain the edit cmd
        assert_eq!(stored_reg.state, None);
        assert_eq!(stored_reg.op_log, vec![cmd_edit.clone()]);

        // applying the edit cmd again shouldn't fail,
        // although the log will contain the edit cmd duplicated
        store.try_to_apply_cmd_against_register_state(&cmd_edit, &mut stored_reg)?;
        assert_eq!(stored_reg.state, None);
        assert_eq!(stored_reg.op_log.len(), 2);
        assert!(
            stored_reg.op_log.iter().all(|op| op == &cmd_edit),
            "Op log doesn't match"
        );

        // let's apply the create cmd now
        store.try_to_apply_cmd_against_register_state(&cmd_create, &mut stored_reg)?;
        // it should contain the create and edit cmds
        assert_eq!(stored_reg.state.as_ref(), Some(&register));
        assert_eq!(stored_reg.op_log.len(), 3);
        assert!(
            stored_reg
                .op_log
                .iter()
                .all(|op| [&cmd_create, &cmd_edit].contains(&op)),
            "Op log doesn't match"
        );
        assert_eq!(stored_reg.state.as_ref().map(|reg| reg.size()), Some(1));

        // apply the create cmd again should change nothing
        match store.try_to_apply_cmd_against_register_state(&cmd_create, &mut stored_reg) {
            Ok(()) => (),
            Err(err) => bail!(
                "An error should not occur when applying create cmd again: {:?}",
                err
            ),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_register_write() -> Result<()> {
        // setup store
        let store = RegisterStorage::default();

        // create register
        let (cmd, authority, _, _, _) = create_register()?;
        store.write(&cmd).await?;

        // get register
        let address = cmd.dst();
        match store.read(&RegisterQuery::Get(address), authority).await {
            QueryResponse::GetRegister(Ok(reg)) => {
                assert_eq!(reg.address(), &address, "Should have same address!");
                assert_eq!(reg.owner(), authority, "Should have same owner!");
            }
            e => bail!("Could not read register! {:?}", e),
        }

        // apply the create cmd again should change nothing
        match store.write(&cmd).await {
            Ok(()) => Ok(()),
            Err(err) => bail!(
                "An error should not occur when applying create cmd again: {:?}",
                err
            ),
        }
    }

    #[tokio::test]
    async fn test_register_export() -> Result<()> {
        // setup store
        let store = RegisterStorage::default();

        let (cmd_create, authority, sk, name, policy) = create_register()?;
        let addr = cmd_create.dst();
        let mut register = Register::new(*policy.owner(), name, 0, policy);

        // store the register along with a few edit ops
        store.write(&cmd_create).await?;
        for _ in 0..10 {
            let cmd_edit = edit_register(&mut register, &sk)?;
            store.write(&cmd_edit).await?;
        }

        // create cmd should be idempotent
        match store.write(&cmd_create).await {
            Ok(()) => (),
            Err(err) => bail!(
                "An error should not occur when applying create cmd again: {:?}",
                err
            ),
        }

        // export Registers, get all data we held in storage
        let all_addrs = store.addrs().await;

        // create new store and update it with the data from first store
        let new_store = RegisterStorage::default();
        for addr in all_addrs {
            let replica = store.get_register_replica(&addr).await?;
            new_store.update(&replica).await?;
        }

        // assert the same tests hold as for the first store
        // create cmd should be idempotent, also on this new store
        match new_store.write(&cmd_create).await {
            Ok(()) => (),
            Err(err) => bail!(
                "An error should not occur when applying create cmd again: {:?}",
                err
            ),
        }

        // should be able to read the same value from this new store also
        let res = new_store.read(&RegisterQuery::Get(addr), authority).await;

        match res {
            QueryResponse::GetRegister(Ok(reg)) => {
                assert_eq!(reg.address(), &addr, "Should have same address!");
                assert_eq!(reg.owner(), authority, "Should have same owner!");
            }
            e => panic!("Could not read! {e:?}"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_register_non_existing_entry() -> Result<()> {
        // setup store
        let store = RegisterStorage::default();

        // create register
        let (cmd_create, authority, _, _, _) = create_register()?;
        store.write(&cmd_create).await?;

        let hash = EntryHash(rand::thread_rng().gen::<[u8; 32]>());

        // try get permissions of random user
        let address = cmd_create.dst();
        let res = store
            .read(&RegisterQuery::GetEntry { address, hash }, authority)
            .await;
        match res {
            QueryResponse::GetRegisterEntry(Err(e)) => {
                assert_eq!(e, Error::NoSuchEntry(hash))
            }
            QueryResponse::GetRegisterEntry(Ok(entry)) => {
                panic!("Should not exist any entry for random hash! {entry:?}")
            }
            e => panic!("Could not read! {e:?}"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_register_non_existing_permissions() -> Result<()> {
        // setup store
        let store = RegisterStorage::default();

        // create register
        let (cmd_create, authority, _, _, _) = create_register()?;
        store.write(&cmd_create).await?;

        let (user, _) = random_user();

        // try get permissions of random user
        let address = cmd_create.dst();
        let res = store
            .read(
                &RegisterQuery::GetUserPermissions { address, user },
                authority,
            )
            .await;
        match res {
            QueryResponse::GetRegisterUserPermissions(Err(e)) => {
                assert_eq!(e, Error::NoSuchUser(user))
            }
            QueryResponse::GetRegisterUserPermissions(Ok(perms)) => {
                panic!("Should not exist any permissions for random user! {perms:?}",)
            }
            e => panic!("Could not read! {e:?}"),
        }

        Ok(())
    }

    fn random_user() -> (User, SecretKey) {
        let sk = SecretKey::random();
        let authority = User::Key(sk.public_key());
        (authority, sk)
    }

    fn create_register() -> Result<(RegisterCmd, User, SecretKey, XorName, Policy)> {
        let (authority, sk) = random_user();
        let policy = Policy {
            owner: authority,
            permissions: Default::default(),
        };
        let xorname = xor_name::rand::random();
        let cmd = create_reg_w_policy(xorname, 0, policy.clone(), &sk)?;

        Ok((cmd, authority, sk, xorname, policy))
    }

    fn edit_register(register: &mut Register, sk: &SecretKey) -> Result<RegisterCmd> {
        let data = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(15)
            .collect();
        let (_, edit) = register.write(data, BTreeSet::default())?;
        let op = EditRegister {
            address: *register.address(),
            edit,
        };
        let signature = sk.sign(serialize(&op)?);

        Ok(RegisterCmd::Edit(SignedRegisterEdit {
            op,
            auth: DataAuthority {
                public_key: sk.public_key(),
                signature,
            },
        }))
    }
}
