// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::Client;
use crate::client::Error;
use crate::messaging::client::{DataCmd, DataQuery, QueryResponse, SequenceRead, SequenceWrite};
use crate::types::{
    PublicKey, Sequence, SequenceAddress, SequenceEntries, SequenceEntry, SequenceIndex,
    SequencePermissions, SequencePrivatePermissions, SequencePrivatePolicy,
    SequencePublicPermissions, SequencePublicPolicy, SequenceUser,
};
use std::collections::BTreeMap;
use tracing::{debug, trace};
use xor_name::XorName;

impl Client {
    //----------------------
    // Write Operations
    //---------------------

    /// Create Private Sequence Data on to the Network
    ///
    /// Creates a private sequence on the network which can be appended to.
    /// Private data can be removed from the network at a later date.
    ///
    /// A tag must be supplied.
    /// A xorname must be supplied, this can be random or deterministic as per your apps needs.
    ///
    /// # Examples
    ///
    /// TODO: update once data types are crdt compliant
    ///
    pub async fn store_private_sequence(
        &self,
        sequence: Option<SequenceEntries>,
        name: XorName,
        tag: u64,
        owner: PublicKey,
        permissions: BTreeMap<PublicKey, SequencePrivatePermissions>,
    ) -> Result<SequenceAddress, Error> {
        trace!("Store Private Sequence Data {:?}", name);
        let pk = self.public_key();
        let policy = SequencePrivatePolicy { owner, permissions };
        let mut data = Sequence::new_private(pk, pk.to_string(), name, tag, Some(policy));
        let address = *data.address();

        if let Some(entries) = sequence {
            for entry in entries {
                let mut op = data.create_unsigned_append_op(entry)?;
                let bytes = bincode::serialize(&op.crdt_op)?;
                let signature = self.keypair.sign(&bytes);
                op.signature = Some(signature);
                data.apply_op(op.clone())?;
            }
        }

        self.pay_and_write_sequence_to_network(data.clone()).await?;

        Ok(address)
    }

    /// Create Public Sequence Data into the Network
    ///
    /// Creates a public sequence on the network which can be appended to.
    /// Public data can _not_ be removed from the network at a later date.
    ///
    /// A tag must be supplied.
    /// A xorname must be supplied, this can be random or deterministic as per your apps needs.
    ///
    /// # Examples
    ///
    /// TODO: update once data types are crdt compliant
    ///
    pub async fn store_public_sequence(
        &self,
        sequence: Option<SequenceEntries>,
        name: XorName,
        tag: u64,
        owner: PublicKey,
        permissions: BTreeMap<SequenceUser, SequencePublicPermissions>,
    ) -> Result<SequenceAddress, Error> {
        trace!("Store Public Sequence Data {:?}", name);
        let pk = self.public_key();
        let policy = SequencePublicPolicy { owner, permissions };
        let mut data = Sequence::new_public(pk, pk.to_string(), name, tag, Some(policy));
        let address = *data.address();

        if let Some(entries) = sequence {
            for entry in entries {
                let mut op = data.create_unsigned_append_op(entry)?;
                let bytes = bincode::serialize(&op.crdt_op)?;
                let signature = self.keypair.sign(&bytes);
                op.signature = Some(signature);
                data.apply_op(op.clone())?;
            }
        }

        self.pay_and_write_sequence_to_network(data.clone()).await?;

        Ok(address)
    }

    /// Delete sequence
    ///
    /// You're only able to delete a PrivateSequence. Public data can no be removed from the network.
    ///
    /// # Examples
    ///
    /// TODO: update once data types are crdt compliant
    ///
    pub async fn delete_sequence(&self, address: SequenceAddress) -> Result<(), Error> {
        let cmd = DataCmd::Sequence(SequenceWrite::Delete(address));
        self.send_cmd(cmd).await
    }

    /// Append to Sequence
    ///
    /// Public or private isn't important for append. You can append to either (though the data you append will be Public or Private).
    ///
    /// # Examples
    ///
    /// TODO: update once data types are crdt compliant
    ///
    pub async fn append_to_sequence(
        &self,
        address: SequenceAddress,
        entry: SequenceEntry,
    ) -> Result<(), Error> {
        // First we fetch it so we can get the causality info,
        // either from local CRDT replica or from the network if not found
        let mut sequence = self.get_sequence(address).await?;

        // We can now append the entry to the Sequence
        let mut op = sequence.create_unsigned_append_op(entry)?;
        let bytes = bincode::serialize(&op.crdt_op)?;
        let signature = self.keypair.sign(&bytes);
        op.signature = Some(signature);
        sequence.apply_op(op.clone())?;

        // Finally we can send the mutation to the network's replicas
        let cmd = DataCmd::Sequence(SequenceWrite::Edit(op));

        self.pay_and_send_data_command(cmd).await
    }

    /// Store a new public sequenced data object
    /// Wraps msg_contents for payment validation and mutation
    pub(crate) async fn pay_and_write_sequence_to_network(
        &self,
        data: Sequence,
    ) -> Result<(), Error> {
        debug!("Attempting to pay and write data to network");
        let cmd = DataCmd::Sequence(SequenceWrite::New(data));

        self.pay_and_send_data_command(cmd).await
    }

    //----------------------
    // Get Sequence
    //---------------------

    /// Get Sequence Data from the Network
    ///
    /// # Examples
    ///
    /// TODO: update once data types are crdt compliant
    ///
    pub async fn get_sequence(&self, address: SequenceAddress) -> Result<Sequence, Error> {
        trace!("Get Sequence Data at {:?}", address.name());
        // Let's fetch the Sequence from the network
        let query_result = self
            .send_query(DataQuery::Sequence(SequenceRead::Get(address)))
            .await?;
        let msg_id = query_result.msg_id;
        match query_result.response {
            QueryResponse::GetSequence(res) => res.map_err(|err| Error::from((err, msg_id))),
            _ => Err(Error::ReceivedUnexpectedEvent),
        }
    }

    /// Get the last data entry from a Sequence Data.
    ///
    /// # Examples
    ///
    /// TODO: update once data types are crdt compliant
    ///
    pub async fn get_sequence_last_entry(
        &self,
        address: SequenceAddress,
    ) -> Result<(u64, SequenceEntry), Error> {
        trace!(
            "Get latest entry from Sequence Data at {:?}",
            address.name()
        );

        let sequence = self.get_sequence(address).await?;
        // TODO: do we need to query with some specific PK?
        match sequence.last_entry(None)? {
            Some(entry) => Ok((sequence.len(None)? - 1, entry.to_vec())),
            None => Err(Error::from(crate::types::Error::NoSuchEntry)),
        }
    }

    /// Get Sequence Data from the Network at a specific version
    ///
    /// # Examples
    ///
    /// TODO: update once data types are crdt compliant
    ///
    pub async fn get_sequence_entry(
        &self,
        address: SequenceAddress,
        index_from_start: u64,
    ) -> Result<SequenceEntry, Error> {
        trace!(
            "Get entry at index {:?} from Sequence Data {:?}",
            index_from_start,
            address.name()
        );

        let sequence = self.get_sequence(address).await?;
        let index = SequenceIndex::FromStart(index_from_start);
        match sequence.get(index, None)? {
            Some(entry) => Ok(entry.to_vec()),
            None => Err(Error::from(crate::types::Error::NoSuchEntry)),
        }
    }

    /// Get a set of Entries for the requested range from a Sequence.
    ///
    /// # Examples
    ///
    /// TODO: update once data types are crdt compliant
    ///
    pub async fn get_sequence_range(
        &self,
        address: SequenceAddress,
        range: (SequenceIndex, SequenceIndex),
    ) -> Result<SequenceEntries, Error> {
        trace!(
            "Get range of entries from Sequence Data at {:?}",
            address.name()
        );

        let sequence = self.get_sequence(address).await?;
        // TODO: do we need to query with some specific PK?
        sequence
            .in_range(range.0, range.1, None)?
            .ok_or_else(|| Error::from(crate::types::Error::NoSuchEntry))
    }

    //----------------------
    // Ownership
    //---------------------

    /// Get the owner of a Sequence.
    ///
    /// # Examples
    ///
    /// TODO: update once data types are crdt compliant
    ///
    pub async fn get_sequence_owner(&self, address: SequenceAddress) -> Result<PublicKey, Error> {
        trace!("Get owner of the Sequence Data at {:?}", address.name());

        // TODO: perhaps we want to grab it directly from the network and update local replica
        let sequence = self.get_sequence(address).await?;

        let owner = if sequence.is_public() {
            sequence.public_policy()?.owner
        } else {
            // TODO: do we need to query with some specific PK?
            sequence.private_policy(None)?.owner
        };

        Ok(owner)
    }

    //----------------------
    // Permissions
    //---------------------

    /// Get the set of Permissions of a Public Sequence.
    ///
    /// # Examples
    ///
    /// TODO: update once data types are crdt compliant
    ///
    pub async fn get_sequence_public_permissions_for_user(
        &self,
        address: SequenceAddress,
        user: PublicKey,
    ) -> Result<SequencePublicPermissions, Error> {
        trace!(
            "Get permissions from Public Sequence Data at {:?}",
            address.name()
        );

        // TODO: perhaps we want to grab it directly from
        // the network and update local replica
        let sequence = self.get_sequence(address).await?;
        // TODO: do we need to query with some specific PK?
        let perms = match sequence
            .permissions(SequenceUser::Key(user), None)
            .map_err(Error::from)?
        {
            SequencePermissions::Public(perms) => perms,
            _ => return Err(Error::NotPublicPermissions),
        };

        Ok(perms)
    }

    /// Get the set of Permissions of a Private Sequence.
    ///
    /// # Examples
    ///
    /// TODO: update once data types are crdt compliant
    ///
    pub async fn get_sequence_private_permissions_for_user(
        &self,
        address: SequenceAddress,
        user: PublicKey,
    ) -> Result<SequencePrivatePermissions, Error> {
        trace!(
            "Get permissions from Private Sequence Data at {:?}",
            address.name()
        );
        let sequence = self.get_sequence(address).await?;

        // TODO: do we need to query with some specific PK?
        let perms = match sequence
            .permissions(SequenceUser::Key(user), None)
            .map_err(Error::from)?
        {
            SequencePermissions::Private(perms) => perms,
            _ => return Err(Error::NotPrivatePermissions),
        };

        Ok(perms)
    }

    /// Get the set of Permissions for a specific user in a Sequence.
    ///
    /// # Examples
    ///
    /// TODO: update once data types are crdt compliant
    ///
    pub async fn get_sequence_permissions(
        &self,
        address: SequenceAddress,
        user: SequenceUser,
    ) -> Result<SequencePermissions, Error> {
        trace!(
            "Get permissions for user {:?} from Sequence Data at {:?}",
            user,
            address.name()
        );

        // TODO: perhaps we want to grab it directly from
        // the network and update local replica
        let sequence = self.get_sequence(address).await?;
        // TODO: do we need to query with some specific PK?
        let perms = sequence.permissions(user, None).map_err(Error::from)?;

        Ok(perms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::utils::test_utils::{create_test_client, gen_ed_keypair, run_w_backoff};
    use crate::messaging::client::Error as ErrorMessage;
    use crate::types::{Error as DtError, SequenceAction, SequencePrivatePermissions};
    use anyhow::{anyhow, bail, Result};
    use tokio::time::Duration;
    use xor_name::XorName;

    /// Sequence data tests ///

    #[tokio::test]
    async fn sequence_basics() -> Result<()> {
        let client = create_test_client(None).await?;

        let name = XorName(rand::random());
        let tag = 15000;
        let owner = client.public_key();

        // store a Private Sequence
        let mut perms = BTreeMap::<PublicKey, SequencePrivatePermissions>::new();
        let _ = perms.insert(owner, SequencePrivatePermissions::new(true, true));
        let address = client
            .store_private_sequence(None, name, tag, owner, perms)
            .await?;

        let sequence = run_w_backoff(|| client.get_sequence(address), 10).await?;

        assert!(sequence.is_private());
        assert_eq!(*sequence.name(), name);
        assert_eq!(sequence.tag(), tag);
        assert_eq!(sequence.len(None)?, 0);

        // store a Public Sequence
        let mut perms = BTreeMap::<SequenceUser, SequencePublicPermissions>::new();
        let _ = perms.insert(SequenceUser::Anyone, SequencePublicPermissions::new(true));
        let address = client
            .store_public_sequence(None, name, tag, owner, perms)
            .await?;

        let sequence = run_w_backoff(|| client.get_sequence(address), 10).await?;

        assert!(sequence.is_public());
        assert_eq!(*sequence.name(), name);
        assert_eq!(sequence.tag(), tag);
        assert_eq!(sequence.len(None)?, 0);

        Ok(())
    }

    #[tokio::test]
    async fn sequence_private_permissions() -> Result<()> {
        let client = create_test_client(None).await?;
        let name = XorName(rand::random());
        let tag = 15000;
        let owner = client.public_key();
        let mut perms = BTreeMap::<PublicKey, SequencePrivatePermissions>::new();
        let _ = perms.insert(owner, SequencePrivatePermissions::new(true, true));
        let address = client
            .store_private_sequence(None, name, tag, owner, perms)
            .await?;

        let data = run_w_backoff(|| client.get_sequence(address), 10).await?;

        assert_eq!(data.len(None)?, 0);

        let user_perms = run_w_backoff(
            || client.get_sequence_private_permissions_for_user(address, owner),
            10,
        )
        .await?;

        assert!(user_perms.is_allowed(SequenceAction::Read));
        assert!(user_perms.is_allowed(SequenceAction::Append));

        let mut seq_res = client
            .get_sequence_permissions(address, SequenceUser::Key(owner))
            .await;

        while seq_res.is_err() {
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            seq_res = client
                .get_sequence_permissions(address, SequenceUser::Key(owner))
                .await;
        }

        match seq_res? {
            SequencePermissions::Private(user_perms) => {
                assert!(user_perms.is_allowed(SequenceAction::Read));
                assert!(user_perms.is_allowed(SequenceAction::Append));
            }
            SequencePermissions::Public(_) => return Err(Error::IncorrectPermissions.into()),
        }

        let other_user = gen_ed_keypair().public_key();

        match client
            .get_sequence_private_permissions_for_user(address, other_user)
            .await
        {
            Err(Error::NetworkDataError(DtError::NoSuchEntry)) => {}
            other => bail!(
                "Unexpected result when querying private permissions: {:?}",
                other
            ),
        }

        match client
            .get_sequence_permissions(address, SequenceUser::Key(other_user))
            .await
        {
            Err(Error::NetworkDataError(DtError::NoSuchEntry)) => Ok(()),
            other => Err(anyhow!(
                "Unexpected result when querying permissions: {:?}",
                other
            )),
        }
    }

    #[tokio::test]
    async fn sequence_public_permissions() -> Result<()> {
        let client = create_test_client(None).await?;

        let name = XorName(rand::random());
        let tag = 15000;
        let owner = client.public_key();
        let mut perms = BTreeMap::<SequenceUser, SequencePublicPermissions>::new();
        let _ = perms.insert(
            SequenceUser::Key(owner),
            SequencePublicPermissions::new(None),
        );
        let address = client
            .store_public_sequence(None, name, tag, owner, perms)
            .await?;

        let user_perms = run_w_backoff(
            || client.get_sequence_public_permissions_for_user(address, owner),
            10,
        )
        .await?;

        assert_eq!(Some(true), user_perms.is_allowed(SequenceAction::Read));
        assert_eq!(None, user_perms.is_allowed(SequenceAction::Append));

        let mut seq_res = client
            .get_sequence_permissions(address, SequenceUser::Key(owner))
            .await;

        while seq_res.is_err() {
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            seq_res = client
                .get_sequence_permissions(address, SequenceUser::Key(owner))
                .await;
        }

        match seq_res? {
            SequencePermissions::Public(user_perms) => {
                assert_eq!(Some(true), user_perms.is_allowed(SequenceAction::Read));
                assert_eq!(None, user_perms.is_allowed(SequenceAction::Append));
            }
            SequencePermissions::Private(_) => {
                return Err(anyhow!("Unexpectedly obtained incorrect user permissions",));
            }
        }

        let other_user = gen_ed_keypair().public_key();

        match client
            .get_sequence_public_permissions_for_user(address, other_user)
            .await
        {
            Err(Error::NetworkDataError(DtError::NoSuchEntry)) => {}
            other => bail!(
                "Unexpected result when querying private permissions: {:?}",
                other
            ),
        }

        match client
            .get_sequence_permissions(address, SequenceUser::Key(other_user))
            .await
        {
            Err(Error::NetworkDataError(DtError::NoSuchEntry)) => Ok(()),
            other => Err(anyhow!(
                "Unexpected result when querying permissions: {:?}",
                other
            )),
        }
    }

    #[tokio::test]
    async fn append_to_sequence() -> Result<()> {
        let name = XorName(rand::random());
        let tag = 10;
        let client = create_test_client(None).await?;

        let owner = client.public_key();
        let mut perms = BTreeMap::<SequenceUser, SequencePublicPermissions>::new();
        let _ = perms.insert(
            SequenceUser::Key(owner),
            SequencePublicPermissions::new(true),
        );

        let address = client
            .store_public_sequence(None, name, tag, owner, perms)
            .await?;

        // append to the data the data
        let _ = run_w_backoff(
            || client.append_to_sequence(address, b"VALUE1".to_vec()),
            10,
        )
        .await?;
        // now check last entry
        let (index, data) = run_w_backoff(|| client.get_sequence_last_entry(address), 10).await?;

        assert_eq!(0, index);
        assert_eq!(std::str::from_utf8(&data)?, "VALUE1");

        // append to the data the data
        let _ = run_w_backoff(
            || client.append_to_sequence(address, b"VALUE2".to_vec()),
            10,
        )
        .await?;

        // and then lets check last entry
        let (mut index, mut data) =
            run_w_backoff(|| client.get_sequence_last_entry(address), 10).await?;

        // we might still be getting old data here
        while index == 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            let (i, d) = client.get_sequence_last_entry(address).await?;
            index = i;
            data = d;
        }

        assert_eq!(1, index);
        assert_eq!(std::str::from_utf8(&data)?, "VALUE2");

        let data = run_w_backoff(
            || {
                client.get_sequence_range(
                    address,
                    (SequenceIndex::FromStart(0), SequenceIndex::FromEnd(0)),
                )
            },
            10,
        )
        .await?;

        assert_eq!(std::str::from_utf8(&data[0])?, "VALUE1");
        assert_eq!(std::str::from_utf8(&data[1])?, "VALUE2");

        // get_sequence_entry

        let data0 = client.get_sequence_entry(address, 0).await?;
        assert_eq!(std::str::from_utf8(&data0)?, "VALUE1");

        let data1 = client.get_sequence_entry(address, 1).await?;
        assert_eq!(std::str::from_utf8(&data1)?, "VALUE2");

        // Requesting a version that's too high throws an error
        let res = client.get_sequence_entry(address, 2).await;
        match res {
            Err(_) => Ok(()),
            Ok(_data) => Err(anyhow!(
                "Unexpectedly retrieved a sequence entry at index that's too high!",
            )),
        }
    }

    #[tokio::test]
    async fn sequence_owner() -> Result<()> {
        let name = XorName(rand::random());
        let tag = 10;
        let client = create_test_client(None).await?;

        let owner = client.public_key();
        let mut perms = BTreeMap::<PublicKey, SequencePrivatePermissions>::new();
        let _ = perms.insert(owner, SequencePrivatePermissions::new(true, true));
        let address = client
            .store_private_sequence(None, name, tag, owner, perms)
            .await?;

        // Assert that the data is stored.
        let current_owner = run_w_backoff(|| client.get_sequence_owner(address), 10).await?;

        assert_eq!(owner, current_owner);

        Ok(())
    }

    #[tokio::test]
    async fn sequence_can_delete_private() -> Result<()> {
        let mut client = create_test_client(None).await?;
        let name = XorName(rand::random());
        let tag = 15000;
        let owner = client.public_key();

        // store a Private Sequence
        let mut perms = BTreeMap::<PublicKey, SequencePrivatePermissions>::new();
        let _ = perms.insert(owner, SequencePrivatePermissions::new(true, true));
        let address = client
            .store_private_sequence(None, name, tag, owner, perms)
            .await?;

        let sequence = run_w_backoff(|| client.get_sequence(address), 10).await?;

        assert!(sequence.is_private());

        client.delete_sequence(address).await?;

        client.query_timeout = Duration::from_secs(5); // override with a short timeout

        let mut res = client.get_sequence(address).await;

        while res.is_ok() {
            tokio::time::sleep(Duration::from_millis(200)).await;
            res = client.get_sequence(address).await;
        }

        match res {
            Err(Error::NoResponse) => Ok(()),
            Err(err) => Err(anyhow!(
                "Unexpected error returned when deleting a nonexisting Private Sequence: {}",
                err
            )),
            Ok(_data) => Err(anyhow!(
                "Unexpectedly retrieved a deleted Private Sequence!",
            )),
        }
    }

    #[tokio::test]
    async fn sequence_cannot_delete_public() -> Result<()> {
        let mut client = create_test_client(None).await?;

        let name = XorName(rand::random());
        let tag = 15000;
        let owner = client.public_key();

        // store a Public Sequence
        let mut perms = BTreeMap::<SequenceUser, SequencePublicPermissions>::new();
        let _ = perms.insert(SequenceUser::Anyone, SequencePublicPermissions::new(true));
        let address = client
            .store_public_sequence(None, name, tag, owner, perms)
            .await?;

        let mut sequence = client.get_sequence(address).await;

        while sequence.is_err() {
            sequence = client.get_sequence(address).await;
        }

        assert!(sequence?.is_public());

        client.delete_sequence(address).await?;

        client.query_timeout = Duration::from_secs(5); // override with a short timeout

        // Check that our data still exists.
        match client.get_sequence(address).await {
            Err(Error::ErrorMessage {
                source: ErrorMessage::InvalidOperation(_),
                ..
            }) => Ok(()),
            Err(err) => Err(anyhow!(
                "Unexpected error returned when attempting to get a Public Sequence: {}",
                err
            )),
            Ok(_data) => Ok(()),
        }
    }
}
