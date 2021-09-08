// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::data::Batch;
use super::Client;
use crate::client::Error;
use crate::messaging::data::{DataCmd, DataQuery, QueryResponse, RegisterRead, RegisterWrite};
use crate::types::{
    register::{
        Entry, EntryHash, Permissions, Policy, PrivatePermissions, PrivatePolicy,
        PublicPermissions, PublicPolicy, Register, User,
    },
    PublicKey, RegisterAddress as Address,
};
use std::collections::{BTreeMap, BTreeSet};
use std::iter::FromIterator;
use xor_name::XorName;

/// Register Write Ahead Log
///
/// Batches up register write operation before publishing them up to the network, in order.
/// Can also be used as a way to implement dry runs:
/// nothing is uploaded to the network as long as the wal is not published.
/// Batches can be republished without duplication risks thanks to the CRDT nature of registers.
pub type RegisterWriteAheadLog = Vec<DataCmd>;

impl Client {
    //----------------------
    // Write Operations
    //---------------------

    /// Publish all register mutation operations in a WAL to the network
    /// Incrementing the WAL index as successful writes are sent out. Stops at the first error.
    /// Starts publishing from the index when called again with the same WAL.
    #[instrument(skip(self), level = "debug")]
    pub async fn publish_register_ops(&self, wal: RegisterWriteAheadLog) -> Result<(), Error> {
        for cmd in wal.iter() {
            self.send_cmd(cmd.clone()).await?;
        }
        Ok(())
    }

    /// Create a Private Register onto the Network
    ///
    /// Creates a private Register on the network which can then be written to.
    /// Private data can be removed from the network at a later date.
    ///
    /// Returns a write ahead log (WAL) of register operations, note that the changes are not uploaded to the
    /// network until the WAL is published with `publish_register_ops`
    ///
    /// A tag must be supplied.
    /// A xorname must be supplied, this can be random or deterministic as per your apps needs.
    #[instrument(skip(self), level = "debug")]
    pub async fn store_private_register(
        &self,
        name: XorName,
        tag: u64,
        owner: PublicKey,
        permissions: BTreeMap<PublicKey, PrivatePermissions>,
    ) -> Result<(Address, RegisterWriteAheadLog), Error> {
        let pk = self.public_key();
        let policy = PrivatePolicy { owner, permissions };
        let priv_register = Register::new_private(pk, name, tag, Some(policy));
        let address = *priv_register.address();

        let batch = self
            .batch_up_pay_write_register_to_network(priv_register)
            .await?;

        Ok((address, batch))
    }

    /// Create a Public Register onto the Network
    ///
    /// Creates a public Register on the network which can then be written to.
    /// Public data _can not_ be removed from the network at a later date.
    ///
    /// Returns a write ahead log (WAL) of register operations, note that the changes are not uploaded to the
    /// network until the WAL is published with `publish_register_ops`
    ///
    /// A tag must be supplied.
    /// A xorname must be supplied, this can be random or deterministic as per your apps needs.
    #[instrument(skip(self), level = "debug")]
    pub async fn store_public_register(
        &self,
        name: XorName,
        tag: u64,
        owner: PublicKey,
        permissions: BTreeMap<User, PublicPermissions>,
    ) -> Result<(Address, RegisterWriteAheadLog), Error> {
        let pk = self.public_key();
        let policy = PublicPolicy { owner, permissions };
        let pub_register = Register::new_public(pk, name, tag, Some(policy));
        let address = *pub_register.address();

        let batch = self
            .batch_up_pay_write_register_to_network(pub_register)
            .await?;

        Ok((address, batch))
    }

    /// Delete Register
    ///
    /// Returns a write ahead log (WAL) of register operations, note that the changes are not uploaded to the
    /// network until the WAL is published with `publish_register_ops`
    ///
    /// You're only able to delete a PrivateRegister. Public data can not be removed from the network.
    #[instrument(skip(self), level = "debug")]
    pub async fn delete_register(&self, address: Address) -> Result<RegisterWriteAheadLog, Error> {
        let cmd = DataCmd::Register(RegisterWrite::Delete(address));

        let batch = vec![cmd];

        Ok(batch)
    }

    /// Write to Register
    ///
    /// Returns a write ahead log (WAL) of register operations, note that the changes are not uploaded to the
    /// network until the WAL is published with `publish_register_ops`
    ///
    /// Public or private isn't important for writing, though the data you write will
    /// be Public or Private according to the type of the targeted Register.
    #[instrument(skip(self, children), level = "debug")]
    pub async fn write_to_register(
        &self,
        address: Address,
        entry: Entry,
        children: BTreeSet<EntryHash>,
    ) -> Result<(EntryHash, RegisterWriteAheadLog), Error> {
        // First we fetch it so we can get the causality info,
        // either from local CRDT replica or from the network if not found
        let mut register = self.get_register(address).await?;

        // We can now write the entry to the Register
        let (hash, mut op) = register.write(entry, children)?;
        let bytes = bincode::serialize(&op.crdt_op)?;
        let signature = self.keypair.sign(&bytes);
        op.signature = Some(signature);

        // let id = format!("{:?}", &hash);
        // // Finally we can send the mutation to the network's replicas
        // self.push_reg_op_to_batch(id, RegisterWrite::Edit(op));

        // Finally we can send the mutation to the network's replicas
        let cmd = DataCmd::Register(RegisterWrite::Edit(op));
        let batch = vec![cmd];
        Ok((hash, batch))
    }

    /// Store a new Register data object
    /// Wraps msg_contents for payment validation and mutation
    ///
    /// Returns a write ahead log (WAL) of register operations, note that the changes are not uploaded to the
    /// network until the WAL is published with `publish_register_ops`
    #[instrument(skip_all, level = "trace")]
    pub(crate) async fn batch_up_pay_write_register_to_network(
        &self,
        data: Register,
    ) -> Result<RegisterWriteAheadLog, Error> {
        // let id = data.address().encode_to_zbase32()?;
        // // Finally we can send the mutation to the network's replicas
        // self.push_reg_op_to_batch(id, RegisterWrite::New(data));

        let cmd = DataCmd::Register(RegisterWrite::New(data));

        let batch = vec![cmd];
        Ok(batch)
    }

    ///
    pub fn push_reg_op_to_batch(&self, id: String, reg_op: RegisterWrite) {
        self.push_reg_ops_to_batch(BTreeMap::from_iter(vec![(id, reg_op)]))
    }

    ///
    pub fn push_reg_ops_to_batch(&self, reg_ops: BTreeMap<String, RegisterWrite>) {
        self.push_batch(Batch {
            reg_ops,
            ..Default::default()
        })
    }

    //----------------------
    // Get Register
    //---------------------

    /// Get a Register from the Network
    #[instrument(skip(self), level = "debug")]
    pub async fn get_register(&self, address: Address) -> Result<Register, Error> {
        // Let's fetch the Register from the network
        let query = DataQuery::Register(RegisterRead::Get(address));
        let query_result = self.send_query(query).await?;
        match query_result.response {
            QueryResponse::GetRegister((res, op_id)) => {
                res.map_err(|err| Error::ErrorMessage { source: err, op_id })
            }
            _ => Err(Error::ReceivedUnexpectedEvent),
        }
    }

    /// Get the last data entry from a Register data.
    #[instrument(skip(self), level = "debug")]
    pub async fn read_register(
        &self,
        address: Address,
    ) -> Result<BTreeSet<(EntryHash, Entry)>, Error> {
        let register = self.get_register(address).await?;
        let last = register.read(None)?;

        Ok(last)
    }

    /// Get an entry from a Register on the Network by its hash
    #[instrument(skip(self), level = "debug")]
    pub async fn get_register_entry(
        &self,
        address: Address,
        hash: EntryHash,
    ) -> Result<Entry, Error> {
        let register = self.get_register(address).await?;
        let entry = register
            .get(hash, None)?
            .ok_or_else(|| Error::from(crate::types::Error::NoSuchEntry))?;

        Ok(entry.to_owned())
    }

    //----------------------
    // Ownership
    //---------------------

    /// Get the owner of a Register.
    #[instrument(skip(self), level = "debug")]
    pub async fn get_register_owner(&self, address: Address) -> Result<PublicKey, Error> {
        let register = self.get_register(address).await?;
        let owner = register.owner();

        Ok(owner)
    }

    //----------------------
    // Permissions
    //---------------------

    /// Get the set of Permissions in a Register for a specific user.
    #[instrument(skip(self), level = "debug")]
    pub async fn get_register_permissions_for_user(
        &self,
        address: Address,
        user: PublicKey,
    ) -> Result<Permissions, Error> {
        let register = self.get_register(address).await?;
        let perms = register.permissions(User::Key(user), None)?;

        Ok(perms)
    }

    /// Get the Policy of a Register.
    #[instrument(skip(self), level = "debug")]
    pub async fn get_register_policy(&self, address: Address) -> Result<Policy, Error> {
        let register = self.get_register(address).await?;
        let policy = register.policy(None)?;

        Ok(policy.clone())
    }
}

#[cfg(test)]
mod tests {
    use crate::client::utils::test_utils::create_test_client_with;
    use crate::client::{
        utils::test_utils::{
            create_test_client, gen_ed_keypair, init_test_logger, run_w_backoff_delayed,
        },
        Error,
    };
    use crate::messaging::data::Error as ErrorMessage;
    use crate::retry_loop_for_pattern;
    use crate::types::log_markers::LogMarker;
    use crate::types::{
        register::{Action, EntryHash, Permissions, PrivatePermissions, PublicPermissions, User},
        Error as DtError, PublicKey,
    };
    use eyre::{bail, eyre, Result};
    use std::{
        collections::{BTreeMap, BTreeSet},
        time::Instant,
    };
    use tokio::time::Duration;
    use tracing::Instrument;
    use xor_name::XorName;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_register_batching() -> Result<()> {
        init_test_logger();
        let _outer_span = tracing::info_span!("test__register_basics").entered();

        let client = create_test_client().await?;
        let one_sec = tokio::time::Duration::from_secs(1);
        let name = XorName(rand::random());
        let tag = 15000;
        let owner = client.public_key();

        // store a Private Register
        let mut perms = BTreeMap::<PublicKey, PrivatePermissions>::new();
        let _ = perms.insert(owner, PrivatePermissions::new(true, true));
        let (address, mut batch) = client
            .store_private_register(name, tag, owner, perms)
            .await?;

        // make sure private register was not created
        tokio::time::sleep(one_sec).await;
        let register = client.get_register(address).await;
        assert!(register.is_err());

        // store a Public Register
        let mut perms = BTreeMap::<User, PublicPermissions>::new();
        let _ = perms.insert(User::Anyone, PublicPermissions::new(true));
        let (address2, mut batch2) = client
            .store_public_register(name, tag, owner, perms)
            .await?;

        // make sure public register was not created
        tokio::time::sleep(one_sec).await;
        let register = client.get_register(address2).await;
        assert!(register.is_err());

        // batch them up
        batch.append(&mut batch2);

        // publish that batch to the network
        client.publish_register_ops(batch).await?;
        tokio::time::sleep(one_sec).await;

        // check they're both there
        let priv_register = client.get_register(address).await?;
        assert!(priv_register.is_private());
        assert_eq!(*priv_register.name(), name);
        assert_eq!(priv_register.tag(), tag);
        assert_eq!(priv_register.size(None)?, 0);
        assert_eq!(priv_register.owner(), owner);

        let pub_register = client.get_register(address2).await?;
        assert!(pub_register.is_public());
        assert_eq!(*pub_register.name(), name);
        assert_eq!(pub_register.tag(), tag);
        assert_eq!(pub_register.size(None)?, 0);
        assert_eq!(pub_register.owner(), owner);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "Testnet network_assert_ tests should be excluded from normal tests runs, they need to be run in sequence to ensure validity of checks"]
    async fn register_network_assert_expected_log_counts() -> Result<()> {
        init_test_logger();
        let _outer_span = tracing::info_span!("register_network_assert").entered();

        let mut the_logs = crate::testnet_grep::NetworkLogState::new()?;

        let network_assert_delay: u64 = std::env::var("NETWORK_ASSERT_DELAY")
            .unwrap_or_else(|_| "3".to_string())
            .parse()?;

        let client = create_test_client().await?;

        let delay = tokio::time::Duration::from_secs(network_assert_delay);
        debug!("Running network asserts with delay of {:?}", delay);

        let name = XorName(rand::random());
        let tag = 15000;
        let owner = client.public_key();

        // store a Private Register
        let mut perms = BTreeMap::<PublicKey, PrivatePermissions>::new();
        let _ = perms.insert(owner, PrivatePermissions::new(true, true));
        let (_address, batch) = client
            .store_private_register(name, tag, owner, perms)
            .await?;
        client.publish_register_ops(batch).await?;

        // small delay to ensure logs have written
        tokio::time::sleep(delay).await;

        // All elders should have been written to
        the_logs.assert_count(LogMarker::RegisterWrite, 7).await?;

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "too heavy for CI"]
    async fn measure_upload_times() -> Result<()> {
        init_test_logger();
        let _outer_span = tracing::info_span!("test__measure_upload_times").entered();

        let mut total = 0;
        let name = XorName(rand::random());
        let tag = 10;
        let client = create_test_client().await?;

        let owner = client.public_key();
        let mut perms = BTreeMap::<User, PublicPermissions>::new();
        let _ = perms.insert(User::Key(owner), PublicPermissions::new(true));

        let (address, batch) = client
            .store_public_register(name, tag, owner, perms)
            .await?;
        client.publish_register_ops(batch).await?;

        let value_1 = random_register_entry();

        for i in 0..1000_usize {
            let now = Instant::now();

            // write to the register
            let _value1_hash = run_w_backoff_delayed(
                || async {
                    let (hash, batch) = client
                        .write_to_register(address, value_1.clone(), BTreeSet::new())
                        .await?;
                    client.publish_register_ops(batch).await?;
                    Ok(hash)
                },
                10,
                1,
            )
            .await?;

            let elapsed = now.elapsed().as_millis();
            total += elapsed;
            println!("Iter # {}, elapsed: {}", i, elapsed);
        }

        println!("Total elapsed: {}", total);

        Ok(())
    }

    /**** Register data tests ****/

    #[tokio::test(flavor = "multi_thread")]
    async fn register_basics() -> Result<()> {
        init_test_logger();
        let _outer_span = tracing::info_span!("test__register_basics").entered();

        let client = create_test_client().await?;
        let name = XorName(rand::random());
        let tag = 15000;
        let owner = client.public_key();

        // store a Private Register
        let mut perms = BTreeMap::<PublicKey, PrivatePermissions>::new();
        let _ = perms.insert(owner, PrivatePermissions::new(true, true));
        let (address, batch) = client
            .store_private_register(name, tag, owner, perms)
            .await?;
        client.publish_register_ops(batch).await?;

        let delay = tokio::time::Duration::from_secs(1);
        tokio::time::sleep(delay).await;

        let register = client.get_register(address).await?;

        assert!(register.is_private());
        assert_eq!(*register.name(), name);
        assert_eq!(register.tag(), tag);
        assert_eq!(register.size(None)?, 0);
        assert_eq!(register.owner(), owner);

        // store a Public Register
        let mut perms = BTreeMap::<User, PublicPermissions>::new();
        let _ = perms.insert(User::Anyone, PublicPermissions::new(true));
        let (address, batch) = client
            .store_public_register(name, tag, owner, perms)
            .await?;
        client.publish_register_ops(batch).await?;

        tokio::time::sleep(delay).await;
        let register = client.get_register(address).await?;

        assert!(register.is_public());
        assert_eq!(*register.name(), name);
        assert_eq!(register.tag(), tag);
        assert_eq!(register.size(None)?, 0);
        assert_eq!(register.owner(), owner);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn register_private_permissions() -> Result<()> {
        init_test_logger();
        let _outer_span = tracing::info_span!("test__register_private_permissions").entered();

        let client = create_test_client().await?;
        let name = XorName(rand::random());
        let tag = 15000;
        let owner = client.public_key();
        let mut perms = BTreeMap::<PublicKey, PrivatePermissions>::new();
        let _ = perms.insert(owner, PrivatePermissions::new(true, true));
        let (address, batch) = client
            .store_private_register(name, tag, owner, perms)
            .await?;
        client.publish_register_ops(batch).await?;

        let delay = tokio::time::Duration::from_secs(1);
        tokio::time::sleep(delay).await;

        let register = client.get_register(address).await?;

        assert_eq!(register.size(None)?, 0);

        tokio::time::sleep(delay).await;
        let permissions = client
            .get_register_permissions_for_user(address, owner)
            .instrument(tracing::info_span!("first get perms for owner"))
            .await?;

        match permissions {
            Permissions::Private(user_perms) => {
                assert!(user_perms.is_allowed(Action::Read));
                assert!(user_perms.is_allowed(Action::Write));
            }
            Permissions::Public(_) => bail!(
                "Incorrect user permissions were returned: {:?}",
                permissions
            ),
        }

        let other_user = gen_ed_keypair().public_key();

        loop {
            match client
                .get_register_permissions_for_user(address, other_user)
                .instrument(tracing::info_span!("get other user perms"))
                .await
            {
                Ok(_) => bail!("Should not be able to retrive an entry for a random user"),
                Err(Error::NetworkDataError(DtError::NoSuchEntry)) => return Ok(()),
                _ => continue,
            }
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn register_public_permissions() -> Result<()> {
        init_test_logger();
        let _outer_span = tracing::info_span!("test__register_public_permissions").entered();

        let client = create_test_client().await?;

        let name = XorName(rand::random());
        let tag = 15000;
        let owner = client.public_key();
        let mut perms = BTreeMap::<User, PublicPermissions>::new();
        let _ = perms.insert(User::Key(owner), PublicPermissions::new(None));
        let (address, batch) = client
            .store_public_register(name, tag, owner, perms)
            .await?;
        client.publish_register_ops(batch).await?;

        let delay = tokio::time::Duration::from_secs(1);
        tokio::time::sleep(delay).await;

        // keep retrying until ok
        let permissions = client
            .get_register_permissions_for_user(address, owner)
            .instrument(tracing::info_span!("get owner perms"))
            .await?;

        match permissions {
            Permissions::Public(user_perms) => {
                assert_eq!(Some(true), user_perms.is_allowed(Action::Read));
                assert_eq!(None, user_perms.is_allowed(Action::Write));
            }
            Permissions::Private(_) => {
                return Err(eyre!("Unexpectedly obtained incorrect user permissions",));
            }
        }

        let other_user = gen_ed_keypair().public_key();

        loop {
            match client
                .get_register_permissions_for_user(address, other_user)
                .instrument(tracing::info_span!("get other user perms"))
                .await
            {
                Ok(_) => bail!("Should not be able to retrive an entry for a random user"),
                Err(Error::NetworkDataError(DtError::NoSuchEntry)) => return Ok(()),
                _ => continue,
            }
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn register_write() -> Result<()> {
        init_test_logger();
        let start_span = tracing::info_span!("test__register_write_start").entered();

        let tag = 10;
        let name = XorName(rand::random());
        let client = create_test_client().await?;

        let owner = client.public_key();
        let mut perms = BTreeMap::<User, PublicPermissions>::new();
        let _ = perms.insert(User::Key(owner), PublicPermissions::new(true));

        let (address, batch) = client
            .store_public_register(name, tag, owner, perms)
            .await?;
        client.publish_register_ops(batch).await?;

        let value_1 = random_register_entry();

        // write to the register
        let value1_hash = run_w_backoff_delayed(
            || async {
                let (hash, batch) = client
                    .write_to_register(address, value_1.clone(), BTreeSet::new())
                    .await?;
                client.publish_register_ops(batch).await?;
                Ok(hash)
            },
            10,
            1,
        )
        .await?;

        // now check last entry
        let hashes = retry_loop_for_pattern!(client.read_register(address), Ok(hashes) if !hashes.is_empty())?;

        assert_eq!(1, hashes.len());
        let current = hashes.iter().next();
        assert_eq!(current, Some(&(value1_hash, value_1.clone())));

        let value_2 = random_register_entry();

        drop(start_span);
        let _second_span = tracing::info_span!("test__register_write__second_write").entered();
        // write to the register
        let value2_hash = run_w_backoff_delayed(
            || async {
                let (hash, batch) = client
                    .write_to_register(address, value_2.clone(), BTreeSet::new())
                    .await?;
                client.publish_register_ops(batch).await?;
                Ok(hash)
            },
            10,
            1,
        )
        .await?;

        // and then lets check all entries are returned
        // NB: these will not be ordered according to insertion order, but according to the hashes of the values.
        let hashes =
            retry_loop_for_pattern!(client.read_register(address), Ok(hashes) if hashes.len() > 1)?;

        assert_eq!(2, hashes.len());

        let delay = tokio::time::Duration::from_secs(1);
        tokio::time::sleep(delay).await;
        // get_register_entry
        let retrieved_value_1 = client
            .get_register_entry(address, value1_hash)
            .instrument(tracing::info_span!("get_value_1"))
            .await?;
        assert_eq!(retrieved_value_1, value_1);

        tokio::time::sleep(delay).await;

        // loop here until we see the value set...
        // TODO: writes should be stable enoupgh that we can remove this...
        let retrieved_value_2 = client
            .get_register_entry(address, value2_hash)
            .instrument(tracing::info_span!("get_value_2"))
            .await?;
        assert_eq!(retrieved_value_2, value_2);

        // Requesting a hash which desn't exist throws an error
        match client
            .get_register_entry(address, EntryHash::default())
            .instrument(tracing::info_span!("final get"))
            .await
        {
            Err(_) => Ok(()),
            Ok(_data) => Err(eyre!(
                "Unexpectedly retrieved a register entry at index that's too high!",
            )),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn register_owner() -> Result<()> {
        init_test_logger();
        let _outer_span = tracing::info_span!("test__register_owner").entered();
        let tag = 10;

        let name = XorName(rand::random());
        let client = create_test_client().await?;

        let owner = client.public_key();
        let mut perms = BTreeMap::<PublicKey, PrivatePermissions>::new();
        let _ = perms.insert(owner, PrivatePermissions::new(true, true));
        let (address, batch) = client
            .store_private_register(name, tag, owner, perms)
            .await?;
        client.publish_register_ops(batch).await?;

        // Assert that the data is stored.
        let current_owner = client.get_register_owner(address).await?;

        assert_eq!(owner, current_owner);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn register_can_delete_private() -> Result<()> {
        init_test_logger();
        let _outer_span = tracing::info_span!("test__register_can_delete_private").entered();

        let mut client = create_test_client().await?;
        let name = XorName(rand::random());
        let tag = 15000;
        let owner = client.public_key();

        // store a Private Register
        let mut perms = BTreeMap::<PublicKey, PrivatePermissions>::new();
        let _ = perms.insert(owner, PrivatePermissions::new(true, true));
        let (address, batch) = client
            .store_private_register(name, tag, owner, perms)
            .await?;
        client.publish_register_ops(batch).await?;

        let delay = tokio::time::Duration::from_secs(1);
        tokio::time::sleep(delay).await;

        let register = client.get_register(address).await?;

        assert!(register.is_private());

        let batch2 = client.delete_register(address).await?;
        client.publish_register_ops(batch2).await?;

        client.query_timeout = Duration::from_secs(5); // override with a short timeout
        let mut res = client.get_register(address).await;
        while res.is_ok() {
            // attempt to delete register again (perhaps a message was dropped)
            let batch3 = client.delete_register(address).await?;
            client.publish_register_ops(batch3).await?;
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            res = client.get_register(address).await;
        }

        match res {
            Err(Error::NoResponse) => Ok(()),
            Err(err) => Err(eyre!(
                "Unexpected error returned when deleting a nonexisting Private Register: {:?}",
                err
            )),
            Ok(_data) => Err(eyre!("Unexpectedly retrieved a deleted Private Register!",)),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn register_cannot_delete_public() -> Result<()> {
        init_test_logger();
        let _outer_span = tracing::info_span!("test__register_cannot_delete_public").entered();

        let client = create_test_client().await?;

        let name = XorName(rand::random());
        let tag = 15000;
        let owner = client.public_key();

        // store a Public Register
        let mut perms = BTreeMap::<User, PublicPermissions>::new();
        let _ = perms.insert(User::Anyone, PublicPermissions::new(true));
        let (address, batch) = client
            .store_public_register(name, tag, owner, perms)
            .await?;
        client.publish_register_ops(batch).await?;

        let delay = tokio::time::Duration::from_secs(1);
        tokio::time::sleep(delay).await;

        let register = client.get_register(address).await?;
        assert!(register.is_public());

        let batch2 = client.delete_register(address).await?;
        match client.publish_register_ops(batch2).await {
            Err(Error::ErrorMessage {
                source: ErrorMessage::InvalidOperation(_),
                ..
            }) => {}
            Err(err) => bail!(
                "Unexpected error returned when attempting to delete a Public Register: {:?}",
                err
            ),
            Ok(()) => {}
        }

        // Check that our data still exists.
        let register = client.get_register(address).await?;
        assert!(register.is_public());

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn ae_checks_register_test() -> Result<()> {
        init_test_logger();
        let _outer_span = tracing::info_span!("ae_checks_register_test").entered();
        let client = create_test_client_with(None, None, false).await?;

        let name = XorName::random();

        // store a Public Register
        let mut perms = BTreeMap::<User, PublicPermissions>::new();
        let _ = perms.insert(User::Anyone, PublicPermissions::new(true));
        let (address, batch) = client
            .store_public_register(name, 15000, client.public_key(), perms)
            .await?;
        client.publish_register_ops(batch).await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let register = client.get_register(address).await?;
        assert!(register.is_public());

        Ok(())
    }

    fn random_register_entry() -> Vec<u8> {
        use rand::Rng;
        let random_bytes = rand::thread_rng().gen::<[u8; 32]>();
        random_bytes.to_vec()
    }
}
