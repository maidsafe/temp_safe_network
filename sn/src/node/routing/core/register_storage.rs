// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::dbs::{
    convert_to_error_message, Error, EventStore, LruCache, Result, UsedSpace, SLED_FLUSH_TIME_MS,
};
use crate::types::{
    register::{Action, Register, User},
    PublicKey, RegisterAddress as Address,
};
use crate::{
    messaging::{
        data::{
            DataCmd, OperationId, QueryResponse, RegisterCmd, RegisterDataExchange, RegisterRead,
            RegisterWrite, ServiceMsg,
        },
        AuthorityProof, ServiceAuth, WireMsg,
    },
    types::DataAddress,
};

use bincode::serialize;
use rayon::prelude::*;
use sled::Db;
use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    path::Path,
    sync::Arc,
};
use tokio::sync::RwLock;
use tracing::info;
use xor_name::{Prefix, XorName, XOR_NAME_LEN};

const REG_DB_NAME: &str = "register";
const KEY_DB_NAME: &str = "addresses";
const CACHE_SIZE: u16 = 100;

type RegOpStore = EventStore<RegisterCmd>;
type Cache = LruCache<CacheEntry>;

/// Operations over the data type Register.
// TODO: dont expose this
#[derive(Clone, Debug)]
pub(crate) struct RegisterStorage {
    used_space: UsedSpace,
    cache: Cache,
    key_db: Db,
    reg_db: Db,
}

#[derive(Clone, Debug)]
struct CacheEntry {
    state: Arc<RwLock<Register>>,
    store: RegOpStore,
}

impl RegisterStorage {
    /// Create new RegisterStorage
    pub(crate) fn new(path: &Path, used_space: UsedSpace) -> Result<Self> {
        let create_path = |name: &str| path.join("db").join(name.to_string());
        let create_db = |db_dir| {
            sled::Config::default()
                .path(&db_dir)
                .flush_every_ms(SLED_FLUSH_TIME_MS)
                .open()
                .map_err(Error::from)
        };

        Ok(Self {
            used_space,
            cache: Cache::new(CACHE_SIZE),
            key_db: create_db(&create_path(KEY_DB_NAME))?,
            reg_db: create_db(&create_path(REG_DB_NAME))?,
        })
    }

    /// --- Synching ---

    /// Used for replication of data to new Elders.
    pub(crate) async fn get_data_of(&self, prefix: Prefix) -> Result<RegisterDataExchange> {
        type KeyResults = Vec<Result<XorName>>;
        let mut the_data = BTreeMap::default();

        // parse keys in parallel
        let (ok, err): (KeyResults, KeyResults) = self
            .key_db
            .export()
            .into_iter()
            .map(|(_, _, pairs)| pairs)
            .flatten()
            .par_bridge()
            .map(|pair| {
                let src_key = &pair[0];
                // we expect xornames as keys
                if src_key.len() != XOR_NAME_LEN {
                    return Err(Error::CouldNotParseDbKey(src_key.to_vec()));
                }
                let mut dst_key: [u8; 32] = Default::default();
                dst_key.copy_from_slice(src_key);

                Ok(XorName(dst_key))
            })
            .partition(|r| r.is_ok());

        if !err.is_empty() {
            for e in err {
                error!("{:?}", e);
            }
            return Err(Error::CouldNotConvertDbKey);
        }

        // TODO: make this concurrent
        for key in ok.iter().flatten() {
            match self.try_load_cache_entry(key).await {
                Ok(entry) => {
                    if prefix.matches(entry.state.read().await.name()) {
                        let _prev = the_data.insert(*key, entry.store.get_all()?);
                    }
                }
                Err(Error::KeyNotFound(_)) => return Err(Error::InvalidStore),
                Err(e) => return Err(e),
            }
        }

        Ok(RegisterDataExchange(the_data))
    }

    /// On receiving data from Elders when promoted.
    pub(crate) async fn update(&self, reg_data: RegisterDataExchange) -> Result<()> {
        debug!("Updating Register store");

        let RegisterDataExchange(data) = reg_data;

        // nested loops, slow..
        for (_, history) in data {
            for cmd in history {
                let verification = WireMsg::verify_sig(
                    cmd.auth.clone(),
                    ServiceMsg::Cmd(DataCmd::Register(cmd.write.clone())),
                );
                if verification.is_ok() {
                    let _ = self.apply(cmd).await?;
                } else {
                    error!("Invalid signature found for an op in history: {:?}", cmd);
                    return Err(Error::InvalidStore); // TODO: Custom error
                }
            }
        }

        Ok(())
    }

    /// --- Writing ---

    pub(crate) async fn write(
        &self,
        write: RegisterWrite,
        auth: AuthorityProof<ServiceAuth>,
    ) -> Result<()> {
        // rough estimate ignoring the extra space used by sled
        let required_space = std::mem::size_of::<RegisterCmd>();
        if !self.used_space.can_add(required_space) {
            return Err(Error::NotEnoughSpace);
        }
        let op = RegisterCmd {
            write,
            auth: auth.into_inner(),
        };
        self.apply(op).await
    }

    async fn apply(&self, op: RegisterCmd) -> Result<()> {
        // rough estimate ignoring the extra space used by sled
        let required_space = std::mem::size_of::<RegisterCmd>();

        let RegisterCmd { write, auth } = op.clone();

        let address = *write.address();
        let key = to_reg_key(&address)?;

        use RegisterWrite::*;
        match write {
            New(_reg) => {
                // inserts default size for now (not yet used)
                let mut old_value = Some(serialize(&u16::MAX)?);
                let new_value = old_value.take(); // leaves `None` in old_value, type inference shenanigans..

                // init store first, to allow append to happen asap after key insert
                // could be races, but edge case for later todos.
                let store = self.get_or_create_store(&key)?;

                // only inserts if no value existed - which is denoted by passing in `None` as old_value
                match self.key_db.compare_and_swap(key, old_value, new_value)? {
                    Ok(()) => trace!("Creating new register"),
                    Err(sled::CompareAndSwapError { .. }) => return Err(Error::DataExists),
                }

                // insert the op to the event log
                let _ = store.append(op)?;
                self.used_space.increase(required_space);

                Ok(())
            }
            Delete(address) => {
                if address.is_public() {
                    return Err(Error::CannotDeletePublicData(DataAddress::Register(
                        address,
                    )));
                }
                let result = match self.cache.get(&key).await {
                    None => {
                        trace!("Attempting to delete register if it exists");
                        self.drop_register_key(key).await?;
                        Ok(())
                    }
                    Some(entry) => {
                        let read_only = entry.state.read().await;
                        // TODO - Register::check_permission() doesn't support Delete yet in safe-nd
                        // register.check_permission(action, Some(auth.public_key))?;
                        if auth.public_key != read_only.owner() {
                            Err(Error::InvalidOwner(auth.public_key))
                        } else {
                            info!("Deleting Register");
                            self.drop_register_key(key).await?;
                            Ok(())
                        }
                    }
                };

                result
            }
            Edit(reg_op) => {
                let entry = self.try_load_cache_entry(&key).await?;

                info!("Editing Register");
                entry
                    .state
                    .read()
                    .await
                    .check_permissions(Action::Write, Some(auth.public_key))?;
                let result = entry
                    .state
                    .write()
                    .await
                    .apply_op(reg_op)
                    .map_err(Error::NetworkData);

                if result.is_ok() {
                    entry.store.append(op)?;
                    self.used_space.increase(required_space);
                    trace!("Editing Register success!");
                } else {
                    trace!("Editing Register failed!");
                }

                result
            }
        }
    }

    /// --- Reading ---

    pub(crate) async fn read(
        &self,
        read: &RegisterRead,
        requester_pk: PublicKey,
    ) -> Result<QueryResponse> {
        trace!("Reading register {:?}", read.dst_address());
        let operation_id = read.operation_id().map_err(|_| Error::NoOperationId)?;
        trace!("Operation of register read: {:?}", operation_id);
        use RegisterRead::*;
        match read {
            Get(address) => self.get(*address, requester_pk, operation_id).await,
            Read(address) => {
                self.read_register(*address, requester_pk, operation_id)
                    .await
            }
            GetOwner(address) => self.get_owner(*address, requester_pk, operation_id).await,
            GetUserPermissions { address, user } => {
                self.get_user_permissions(*address, *user, requester_pk, operation_id)
                    .await
            }
            GetPolicy(address) => self.get_policy(*address, requester_pk, operation_id).await,
        }
    }

    /// Get entire Register.
    async fn get(
        &self,
        address: Address,
        requester_pk: PublicKey,
        operation_id: OperationId,
    ) -> Result<QueryResponse> {
        let result = match self
            .get_register(&address, Action::Read, requester_pk)
            .await
        {
            Ok(register) => Ok(register),
            Err(error) => Err(convert_to_error_message(error)),
        };

        Ok(QueryResponse::GetRegister((result, operation_id)))
    }

    /// Get `Register` from the store and check permissions.
    async fn get_register(
        &self,
        address: &Address,
        action: Action,
        requester_pk: PublicKey,
    ) -> Result<Register> {
        let entry = match self.try_load_cache_entry(&to_reg_key(address)?).await {
            Ok(entry) => entry,
            Err(Error::KeyNotFound(_key)) => {
                return Err(Error::NoSuchData(DataAddress::Register(*address)))
            }
            Err(e) => return Err(e),
        };

        let read_only = entry.state.read().await;
        read_only
            .check_permissions(action, Some(requester_pk))
            .map_err(Error::from)?;

        Ok(read_only.clone())
    }

    async fn read_register(
        &self,
        address: Address,
        requester_pk: PublicKey,
        operation_id: OperationId,
    ) -> Result<QueryResponse> {
        let result = match self
            .get_register(&address, Action::Read, requester_pk)
            .await
        {
            Ok(register) => register.read(Some(requester_pk)).map_err(Error::from),
            Err(Error::NoSuchData(addr)) => return Err(Error::NoSuchData(addr)),
            Err(error) => Err(error),
        };

        Ok(QueryResponse::ReadRegister((
            result.map_err(convert_to_error_message),
            operation_id,
        )))
    }

    async fn get_owner(
        &self,
        address: Address,
        requester_pk: PublicKey,
        operation_id: OperationId,
    ) -> Result<QueryResponse> {
        let result = match self
            .get_register(&address, Action::Read, requester_pk)
            .await
        {
            Ok(res) => Ok(res.owner()),
            Err(Error::NoSuchData(addr)) => return Err(Error::NoSuchData(addr)),
            Err(error) => Err(convert_to_error_message(error)),
        };

        Ok(QueryResponse::GetRegisterOwner((result, operation_id)))
    }

    async fn get_user_permissions(
        &self,
        address: Address,
        user: User,
        requester_pk: PublicKey,
        operation_id: OperationId,
    ) -> Result<QueryResponse> {
        let result = match self
            .get_register(&address, Action::Read, requester_pk)
            .await
            .and_then(|register| {
                register
                    .permissions(user, Some(requester_pk))
                    .map_err(Error::from)
            }) {
            Ok(res) => Ok(res),
            Err(Error::NoSuchData(addr)) => return Err(Error::NoSuchData(addr)),
            Err(error) => Err(convert_to_error_message(error)),
        };

        Ok(QueryResponse::GetRegisterUserPermissions((
            result,
            operation_id,
        )))
    }

    async fn get_policy(
        &self,
        address: Address,
        requester_pk: PublicKey,
        operation_id: OperationId,
    ) -> Result<QueryResponse> {
        let result = match self
            .get_register(&address, Action::Read, requester_pk)
            .await
            .and_then(|register| {
                register
                    .policy(Some(requester_pk))
                    .map(|p| p.clone())
                    .map_err(Error::from)
            }) {
            Ok(res) => Ok(res),
            Err(Error::NoSuchData(addr)) => return Err(Error::NoSuchData(addr)),
            Err(error) => Err(convert_to_error_message(error)),
        };

        Ok(QueryResponse::GetRegisterPolicy((result, operation_id)))
    }

    /// Helpers

    // get or create a register op store
    fn get_or_create_store(&self, id: &XorName) -> Result<RegOpStore> {
        RegOpStore::new(id, self.reg_db.clone()).map_err(Error::from)
    }

    // helper that drops the sled tree for a given register
    // decreases the used space by a rough estimate of the size before deletion
    // as with addition this estimate ignores the extra space used by sled
    // (that estimate can fall victim to a race condition if someone writes to a register that is being deleted)
    async fn drop_register_key(&self, key: XorName) -> Result<()> {
        let regcmd_size = std::mem::size_of::<RegisterCmd>();
        let reg_tree = self.reg_db.open_tree(key)?;
        let len = reg_tree.len();
        let key_used_space = len * regcmd_size;

        let _removed = self.key_db.remove(key)?;
        let _removed = self.reg_db.drop_tree(key)?;

        self.cache.remove(&key).await;
        self.used_space.decrease(key_used_space);

        Ok(())
    }

    // gets entry from the cache, or populates cache from disk if expired
    async fn try_load_cache_entry(&self, key: &XorName) -> Result<Arc<CacheEntry>> {
        let entry = self.cache.get(key).await;

        // return early on cache hit
        if let Some(entry) = entry {
            return Ok(entry);
        }

        // read from disk
        let store = self.get_or_create_store(key)?;
        let mut register = None;
        // apply all ops
        use RegisterWrite::*;
        for op in store.get_all()? {
            // first op shall be New
            if let New(reg) = op.write {
                register = Some(reg);
            } else if let Some(reg) = &mut register {
                if let Edit(reg_op) = op.write {
                    reg.apply_op(reg_op).map_err(Error::NetworkData)?;
                }
            }
        }

        match register {
            None => Err(Error::KeyNotFound(key.to_string())), // nothing found on disk
            Some(reg) => {
                let entry = Arc::new(CacheEntry {
                    state: Arc::new(RwLock::new(reg)),
                    store,
                });
                // populate cache
                self.cache.insert(key, entry.clone()).await;
                Ok(entry)
            }
        }
    }
}

/// This also encodes the Public | Private scope,
/// as well as the tag of the Address.
fn to_reg_key(address: &Address) -> Result<XorName> {
    Ok(XorName::from_content(
        DataAddress::Register(*address)
            .encode_to_zbase32()?
            .as_bytes(),
    ))
}

impl Display for RegisterStorage {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "RegisterStorage")
    }
}

#[cfg(test)]
mod test {
    use super::{RegOpStore, RegisterStorage};

    use crate::node::{Error, Result};
    use crate::types::DataAddress;
    use crate::types::{
        register::{PublicPermissions, PublicPolicy, Register, User},
        Keypair,
    };
    use crate::UsedSpace;
    use crate::{
        messaging::{
            data::{DataCmd, QueryResponse, RegisterCmd, RegisterRead, RegisterWrite, ServiceMsg},
            AuthorityProof, ServiceAuth, WireMsg,
        },
        node::routing::core::register_storage::to_reg_key,
    };

    use rand::rngs::OsRng;
    use std::{collections::BTreeMap, path::Path};
    use tempfile::tempdir;
    use xor_name::{Prefix, XorName};

    fn new_store(path: &Path) -> Result<RegisterStorage> {
        let used_space = UsedSpace::new(usize::MAX);
        let store = RegisterStorage::new(path, used_space)?;
        Ok(store)
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_basic_read_and_write() -> Result<()> {
        let tmp_dir = tempdir()?;
        let first = tmp_dir.path().join("first");
        let store = new_store(first.as_path())?;

        let mut rng = OsRng;
        let keypair = Keypair::new_ed25519(&mut rng);
        let random_name = XorName::random();

        let register = Register::new_public(keypair.public_key(), random_name, 1, None);
        let address = register.address();

        let serialised_cmd = {
            let msg = ServiceMsg::Cmd(DataCmd::Register(RegisterWrite::New(register.clone())));
            WireMsg::serialize_msg_payload(&msg)?
        };
        let signature = keypair.sign(&serialised_cmd);
        let auth = ServiceAuth {
            public_key: keypair.public_key(),
            signature,
        };

        let write = RegisterWrite::New(register.clone());
        let proof = AuthorityProof::verify(auth, serialised_cmd)?;

        let _ = store.write(write.clone(), proof.clone()).await?;
        let res = store
            .read(&RegisterRead::Get(*address), keypair.public_key())
            .await?;

        match res {
            QueryResponse::GetRegister((Ok(reg), _)) => {
                assert_eq!(
                    reg.address(),
                    register.address(),
                    "Should have same address!"
                );
                assert_eq!(reg.owner(), register.owner(), "Should have same owner!");
            }
            e => panic!("Could not read! {:?}", e),
        }

        // should fail to write same register again
        let res = store.write(write.clone(), proof.clone()).await;

        assert_eq!(
            res.err().unwrap().to_string(),
            Error::DataExists.to_string(),
            "Should not be able to create twice!"
        );

        // get all data in db
        let prefix = Prefix::new(0, random_name);
        let for_update = store.get_data_of(prefix).await?;

        // create new db and update it with the data from first db
        let second = tmp_dir.path().join("second");
        let new_store = new_store(second.as_path())?;
        let _ = new_store.update(for_update).await?;

        // assert the same tests hold as for the first db

        // should fail to write same register again, also on this new store
        let res = new_store.write(write, proof).await;

        assert_eq!(
            res.err().unwrap().to_string(),
            Error::DataExists.to_string(),
            "Should not be able to create twice!"
        );

        // should be able to read the same value from this new store also
        let res = new_store
            .read(&RegisterRead::Get(*address), keypair.public_key())
            .await?;

        match res {
            QueryResponse::GetRegister((Ok(reg), _)) => {
                assert_eq!(
                    reg.address(),
                    register.address(),
                    "Should have same address!"
                );
                assert_eq!(reg.owner(), register.owner(), "Should have same owner!");
            }
            e => panic!("Could not read! {:?}", e),
        }

        let _ = store.drop_register_key(to_reg_key(address)?).await?;

        // should not get the removed register
        let res = store
            .read(&RegisterRead::Get(*address), keypair.public_key())
            .await?;

        use crate::messaging::data::Error as MsgError;

        match res {
            QueryResponse::GetRegister((Ok(_), _)) => panic!("Was not removed!"),
            QueryResponse::GetRegister((Err(MsgError::DataNotFound(addr)), _)) => {
                assert_eq!(addr, DataAddress::Register(*register.address()))
            }
            e => panic!("Unexpected response! {:?}", e),
        }

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn appends_and_reads_from_store() -> Result<()> {
        let id = xor_name::XorName::random();
        let tmp_dir = tempdir()?;
        let db_dir = tmp_dir.path().join(Path::new(&"db".to_string()));
        let db = sled::open(db_dir).map_err(|error| {
            trace!("Sled Error: {:?}", error);
            Error::Sled(error)
        })?;
        let store = RegOpStore::new(&id, db)?;

        let authority_keypair1 = Keypair::new_ed25519(&mut OsRng);
        let pk = authority_keypair1.public_key();

        let register_name: XorName = rand::random();
        let register_tag = 43_000u64;

        let mut permissions = BTreeMap::default();
        let user_perms = PublicPermissions::new(true);
        let _prev = permissions.insert(User::Key(pk), user_perms);

        let replica1 = Register::new_public(
            pk,
            register_name,
            register_tag,
            Some(PublicPolicy {
                owner: pk,
                permissions,
            }),
        );

        let write = RegisterWrite::New(replica1);

        let auth = ServiceAuth {
            public_key: pk,
            signature: authority_keypair1.sign(b""),
        };

        let cmd = RegisterCmd { write, auth };

        store.append(cmd.clone())?;

        let events = store.get_all()?;
        assert_eq!(events.len(), 1);

        match events.get(0) {
            Some(found_cmd) => assert_eq!(found_cmd, &cmd),
            None => unreachable!(),
        }

        Ok(())
    }
}
