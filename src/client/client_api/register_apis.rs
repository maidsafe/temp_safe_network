// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::Client;
use crate::client::Error;
use crate::messaging::client::{DataCmd, DataQuery, QueryResponse, RegisterRead, RegisterWrite};
use crate::types::{
    register::{
        Address, Entry, EntryHash, Permissions, Policy, PrivatePermissions, PrivatePolicy,
        PublicPermissions, PublicPolicy, Register, User,
    },
    PublicKey,
};
use std::collections::{BTreeMap, BTreeSet};
use tracing::{debug, trace};
use xor_name::XorName;

impl Client {
    //----------------------
    // Write Operations
    //---------------------

    /// Create a Private Register onto the Network
    ///
    /// Creates a private Register on the network which can then be written to.
    /// Private data can be removed from the network at a later date.
    ///
    /// A tag must be supplied.
    /// A xorname must be supplied, this can be random or deterministic as per your apps needs.
    pub async fn store_private_register(
        &self,
        name: XorName,
        tag: u64,
        owner: PublicKey,
        permissions: BTreeMap<PublicKey, PrivatePermissions>,
    ) -> Result<Address, Error> {
        trace!("Store Private Register data {:?}", name);
        let pk = self.public_key();
        let policy = PrivatePolicy { owner, permissions };
        let priv_register = Register::new_private(pk, name, tag, Some(policy));
        let address = *priv_register.address();

        self.pay_and_write_register_to_network(priv_register)
            .await?;

        Ok(address)
    }

    /// Create a Public Register onto the Network
    ///
    /// Creates a public Register on the network which can then be written to.
    /// Public data _can not_ be removed from the network at a later date.
    ///
    /// A tag must be supplied.
    /// A xorname must be supplied, this can be random or deterministic as per your apps needs.
    pub async fn store_public_register(
        &self,
        name: XorName,
        tag: u64,
        owner: PublicKey,
        permissions: BTreeMap<User, PublicPermissions>,
    ) -> Result<Address, Error> {
        trace!("Store Public Register data {:?}", name);
        let pk = self.public_key();
        let policy = PublicPolicy { owner, permissions };
        let pub_register = Register::new_public(pk, name, tag, Some(policy));
        let address = *pub_register.address();

        self.pay_and_write_register_to_network(pub_register).await?;

        Ok(address)
    }

    /// Delete Register
    ///
    /// You're only able to delete a PrivateRegister. Public data can no be removed from the network.
    pub async fn delete_register(&self, address: Address) -> Result<(), Error> {
        let cmd = DataCmd::Register(RegisterWrite::Delete(address));
        self.send_cmd(cmd).await
    }

    /// Write to Register
    ///
    /// Public or private isn't important for writing, though the data you write will
    /// be Public or Private according to the type of the targeted Register.
    pub async fn write_to_register(
        &self,
        address: Address,
        entry: Entry,
        parents: BTreeSet<EntryHash>,
    ) -> Result<EntryHash, Error> {
        // First we fetch it so we can get the causality info,
        // either from local CRDT replica or from the network if not found
        let mut register = self.get_register(address).await?;

        // We can now write the entry to the Register
        let (hash, mut op) = register.write(entry, parents)?;
        let bytes = bincode::serialize(&op.crdt_op)?;
        let signature = self.keypair.sign(&bytes);
        op.signature = Some(signature);

        // Finally we can send the mutation to the network's replicas
        let cmd = DataCmd::Register(RegisterWrite::Edit(op));

        self.pay_and_send_data_command(cmd).await?;

        Ok(hash)
    }

    /// Store a new Register data object
    /// Wraps msg_contents for payment validation and mutation
    pub(crate) async fn pay_and_write_register_to_network(
        &self,
        data: Register,
    ) -> Result<(), Error> {
        debug!("Attempting to pay and write a Register to the network");
        let cmd = DataCmd::Register(RegisterWrite::New(data));

        self.pay_and_send_data_command(cmd).await
    }

    //----------------------
    // Get Register
    //---------------------

    /// Get a Register from the Network
    pub async fn get_register(&self, address: Address) -> Result<Register, Error> {
        trace!("Get Register data at {:?}", address.name());
        // Let's fetch the Register from the network
        let query = DataQuery::Register(RegisterRead::Get(address));
        let query_result = self.send_query(query).await?;
        let msg_id = query_result.msg_id;
        match query_result.response {
            QueryResponse::GetRegister(res) => res.map_err(|err| Error::from((err, msg_id))),
            _ => Err(Error::ReceivedUnexpectedEvent),
        }
    }

    /// Get the last data entry from a Register data.
    pub async fn read_register(
        &self,
        address: Address,
    ) -> Result<BTreeSet<(EntryHash, Entry)>, Error> {
        trace!(
            "Get last entry/ies from Register data at {:?}",
            address.name()
        );

        let register = self.get_register(address).await?;
        let last = register.read(None)?;

        Ok(last)
    }

    /// Get an entry from a Register on the Network by its hash
    pub async fn get_register_entry(
        &self,
        address: Address,
        hash: EntryHash,
    ) -> Result<Entry, Error> {
        trace!(
            "Get entry with hash {:?} from Register data {:?}",
            hash,
            address.name()
        );

        let register = self.get_register(address).await?;
        let entry = register
            .get(hash, None)?
            .ok_or_else(|| Error::from(crate::types::Error::NoSuchEntry))?;

        Ok(entry.to_vec())
    }

    //----------------------
    // Ownership
    //---------------------

    /// Get the owner of a Register.
    pub async fn get_register_owner(&self, address: Address) -> Result<PublicKey, Error> {
        trace!("Get owner of the Register data at {:?}", address.name());

        let register = self.get_register(address).await?;
        let owner = register.owner();

        Ok(owner)
    }

    //----------------------
    // Permissions
    //---------------------

    /// Get the set of Permissions in a Register for a specific user.
    pub async fn get_register_permissions_for_user(
        &self,
        address: Address,
        user: PublicKey,
    ) -> Result<Permissions, Error> {
        trace!(
            "Get permissions from Public Register data at {:?}",
            address.name()
        );

        let register = self.get_register(address).await?;
        let perms = register.permissions(User::Key(user), None)?;

        Ok(perms)
    }

    /// Get the Policy of a Register.
    pub async fn get_register_policy(&self, address: Address) -> Result<Policy, Error> {
        trace!("Get Policy from Register data at {:?}", address.name());

        let register = self.get_register(address).await?;
        let policy = register.policy(None)?;

        Ok(policy.clone())
    }
}

#[cfg(test)]
mod tests {
    use crate::client::{
        utils::test_utils::{create_test_client, gen_ed_keypair, run_w_backoff},
        Error,
    };
    use crate::messaging::client::Error as ErrorMessage;
    use crate::retry_loop_for_pattern;
    use crate::types::{
        register::{Action, EntryHash, Permissions, PrivatePermissions, PublicPermissions, User},
        Error as DtError, PublicKey,
    };
    use anyhow::{anyhow, bail, Result};
    use std::{
        collections::{BTreeMap, BTreeSet},
        time::Instant,
    };
    use tokio::time::Duration;
    use xor_name::XorName;

    #[tokio::test]
    #[ignore = "too heavy for CI"]
    async fn measure_upload_times() -> Result<()> {
        let mut total = 0;

        let name = XorName(rand::random());
        let tag = 10;
        let client = create_test_client(None).await?;

        let owner = client.public_key();
        let mut perms = BTreeMap::<User, PublicPermissions>::new();
        let _ = perms.insert(User::Key(owner), PublicPermissions::new(true));

        let address = client
            .store_public_register(name, tag, owner, perms)
            .await?;

        for i in 0..1000_usize {
            let now = Instant::now();

            // write to the register
            let _value1_hash = run_w_backoff(
                || client.write_to_register(address, b"VALUE1".to_vec(), BTreeSet::new()),
                10,
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

    #[tokio::test]
    async fn register_basics() -> Result<()> {
        let client = create_test_client(None).await?;

        let name = XorName(rand::random());
        let tag = 15000;
        let owner = client.public_key();

        // store a Private Register
        let mut perms = BTreeMap::<PublicKey, PrivatePermissions>::new();
        let _ = perms.insert(owner, PrivatePermissions::new(true, true));
        let address = client
            .store_private_register(name, tag, owner, perms)
            .await?;

        let register = run_w_backoff(|| client.get_register(address), 10).await?;

        assert!(register.is_private());
        assert_eq!(*register.name(), name);
        assert_eq!(register.tag(), tag);
        assert_eq!(register.size(None)?, 0);
        assert_eq!(register.owner(), owner);

        // store a Public Register
        let mut perms = BTreeMap::<User, PublicPermissions>::new();
        let _ = perms.insert(User::Anyone, PublicPermissions::new(true));
        let address = client
            .store_public_register(name, tag, owner, perms)
            .await?;

        let register = run_w_backoff(|| client.get_register(address), 10).await?;

        assert!(register.is_public());
        assert_eq!(*register.name(), name);
        assert_eq!(register.tag(), tag);
        assert_eq!(register.size(None)?, 0);
        assert_eq!(register.owner(), owner);

        Ok(())
    }

    #[tokio::test]
    async fn register_private_permissions() -> Result<()> {
        let client = create_test_client(None).await?;
        let name = XorName(rand::random());
        let tag = 15000;
        let owner = client.public_key();
        let mut perms = BTreeMap::<PublicKey, PrivatePermissions>::new();
        let _ = perms.insert(owner, PrivatePermissions::new(true, true));
        let address = client
            .store_private_register(name, tag, owner, perms)
            .await?;

        let register = run_w_backoff(|| client.get_register(address), 10).await?;

        assert_eq!(register.size(None)?, 0);

        let permissions = run_w_backoff(
            || client.get_register_permissions_for_user(address, owner),
            10,
        )
        .await?;

        match permissions {
            Permissions::Private(user_perms) => {
                assert!(user_perms.is_allowed(Action::Read));
                assert!(user_perms.is_allowed(Action::Write));
            }
            Permissions::Public(_) => return Err(Error::IncorrectPermissions.into()),
        }

        let other_user = gen_ed_keypair().public_key();

        match client
            .get_register_permissions_for_user(address, other_user)
            .await
        {
            Err(Error::NetworkDataError(DtError::NoSuchEntry)) => Ok(()),
            other => bail!("Unexpected result when querying permissions: {:?}", other),
        }
    }

    #[tokio::test]
    async fn register_public_permissions() -> Result<()> {
        let client = create_test_client(None).await?;

        let name = XorName(rand::random());
        let tag = 15000;
        let owner = client.public_key();
        let mut perms = BTreeMap::<User, PublicPermissions>::new();
        let _ = perms.insert(User::Key(owner), PublicPermissions::new(None));
        let address = client
            .store_public_register(name, tag, owner, perms)
            .await?;

        let permissions = run_w_backoff(
            || client.get_register_permissions_for_user(address, owner),
            10,
        )
        .await?;

        match permissions {
            Permissions::Public(user_perms) => {
                assert_eq!(Some(true), user_perms.is_allowed(Action::Read));
                assert_eq!(None, user_perms.is_allowed(Action::Write));
            }
            Permissions::Private(_) => {
                return Err(anyhow!("Unexpectedly obtained incorrect user permissions",));
            }
        }

        let other_user = gen_ed_keypair().public_key();

        match client
            .get_register_permissions_for_user(address, other_user)
            .await
        {
            Err(Error::NetworkDataError(DtError::NoSuchEntry)) => Ok(()),
            other => bail!("Unexpected result when querying permissions: {:?}", other),
        }
    }

    #[tokio::test]
    async fn register_write() -> Result<()> {
        let name = XorName(rand::random());
        let tag = 10;
        let client = create_test_client(None).await?;

        let owner = client.public_key();
        let mut perms = BTreeMap::<User, PublicPermissions>::new();
        let _ = perms.insert(User::Key(owner), PublicPermissions::new(true));

        let address = client
            .store_public_register(name, tag, owner, perms)
            .await?;

        // write to the register
        let value1_hash = run_w_backoff(
            || client.write_to_register(address, b"VALUE1".to_vec(), BTreeSet::new()),
            10,
        )
        .await?;

        // now check last entry
        let hashes = retry_loop_for_pattern!(client.read_register(address), Ok(hashes) if !hashes.is_empty())?;

        assert_eq!(1, hashes.len());
        let current = hashes.iter().next();
        assert_eq!(current, Some(&(value1_hash, b"VALUE1".to_vec())));

        // write to the register
        let value2_hash = run_w_backoff(
            || client.write_to_register(address, b"VALUE2".to_vec(), BTreeSet::new()),
            10,
        )
        .await?;

        // and then lets check last entry
        let hashes =
            retry_loop_for_pattern!(client.read_register(address), Ok(hashes) if hashes.len() > 1)?;

        assert_eq!(2, hashes.len());
        let current = hashes.iter().next();
        assert_eq!(current, Some(&(value2_hash, b"VALUE2".to_vec())));

        // get_register_entry
        let value1 = client.get_register_entry(address, value1_hash).await?;
        assert_eq!(std::str::from_utf8(&value1)?, "VALUE1");

        let value2 = client.get_register_entry(address, value2_hash).await?;
        assert_eq!(std::str::from_utf8(&value2)?, "VALUE2");

        // Requesting a hash which desn't exist throws an error
        match client
            .get_register_entry(address, EntryHash::default())
            .await
        {
            Err(_) => Ok(()),
            Ok(_data) => Err(anyhow!(
                "Unexpectedly retrieved a register entry at index that's too high!",
            )),
        }
    }

    #[tokio::test]
    async fn register_owner() -> Result<()> {
        let name = XorName(rand::random());
        let tag = 10;
        let client = create_test_client(None).await?;

        let owner = client.public_key();
        let mut perms = BTreeMap::<PublicKey, PrivatePermissions>::new();
        let _ = perms.insert(owner, PrivatePermissions::new(true, true));
        let address = client
            .store_private_register(name, tag, owner, perms)
            .await?;

        // Assert that the data is stored.
        let current_owner = run_w_backoff(|| client.get_register_owner(address), 10).await?;

        assert_eq!(owner, current_owner);

        Ok(())
    }

    #[tokio::test]
    async fn register_can_delete_private() -> Result<()> {
        let mut client = create_test_client(None).await?;
        let name = XorName(rand::random());
        let tag = 15000;
        let owner = client.public_key();

        // store a Private Register
        let mut perms = BTreeMap::<PublicKey, PrivatePermissions>::new();
        let _ = perms.insert(owner, PrivatePermissions::new(true, true));
        let address = client
            .store_private_register(name, tag, owner, perms)
            .await?;

        let register = run_w_backoff(|| client.get_register(address), 10).await?;

        assert!(register.is_private());

        client.delete_register(address).await?;

        client.query_timeout = Duration::from_secs(5); // override with a short timeout
        let mut res = client.get_register(address).await;
        while res.is_ok() {
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            res = client.get_register(address).await;
        }

        match res {
            Err(Error::NoResponse) => Ok(()),
            Err(err) => Err(anyhow!(
                "Unexpected error returned when deleting a nonexisting Private Register: {}",
                err
            )),
            Ok(_data) => Err(anyhow!(
                "Unexpectedly retrieved a deleted Private Register!",
            )),
        }
    }

    #[tokio::test]
    async fn register_cannot_delete_public() -> Result<()> {
        let client = create_test_client(None).await?;

        let name = XorName(rand::random());
        let tag = 15000;
        let owner = client.public_key();

        // store a Public Register
        let mut perms = BTreeMap::<User, PublicPermissions>::new();
        let _ = perms.insert(User::Anyone, PublicPermissions::new(true));
        let address = client
            .store_public_register(name, tag, owner, perms)
            .await?;

        let register = run_w_backoff(|| client.get_register(address), 10).await?;
        assert!(register.is_public());

        match client.delete_register(address).await {
            Err(Error::ErrorMessage {
                source: ErrorMessage::InvalidOperation(_),
                ..
            }) => {}
            Err(err) => bail!(
                "Unexpected error returned when attempting to delete a Public Register: {}",
                err
            ),
            Ok(()) => {}
        }

        // Check that our data still exists.
        let register = client.get_register(address).await?;
        assert!(register.is_public());

        Ok(())
    }
}
