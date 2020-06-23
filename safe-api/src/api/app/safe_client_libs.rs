// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

#[cfg(not(feature = "fake-auth"))]
use super::helpers::{decode_ipc_msg, AuthResponseType};
use super::{
    fetch::Range,
    helpers::{xorname_from_pk, xorname_to_hex},
    SafeApp,
};
use crate::{Error, Result};
use async_trait::async_trait;
use log::{debug, info, warn};
#[cfg(feature = "fake-auth")]
use safe_app::test_utils::create_app;
use safe_app::App;
use safe_core::{client::test_create_balance, immutable_data, Client, CoreError as SafeCoreError};
use safe_nd::{
    ClientFullId, Coins, Error as SafeNdError, IDataAddress, MDataAction, MDataPermissionSet,
    MDataSeqEntryActions, MDataSeqValue, PublicKey as SafeNdPublicKey, SDataAddress, SDataIndex,
    SDataPubUserPermissions, SDataUser, SeqMutableData, Transaction, TransactionId, XorName,
};
use std::collections::BTreeMap;

pub use threshold_crypto::{PublicKey, SecretKey};

const APP_NOT_CONNECTED: &str = "Application is not connected to the network";

#[derive(Default)]
pub struct SafeAppScl {
    safe_conn: Option<App>,
}

impl SafeAppScl {
    // Private helper to obtain the App instance
    fn get_safe_app(&self) -> Result<&App> {
        match &self.safe_conn {
            Some(app) => Ok(app),
            None => Err(Error::ConnectionError(APP_NOT_CONNECTED.to_string())),
        }
    }

    async fn mutate_seq_mdata_entries(
        &self,
        name: XorName,
        tag: u64,
        entry_actions: MDataSeqEntryActions,
        error_msg: &str,
    ) -> Result<()> {
        let client = &self.get_safe_app()?.client;
        let message = error_msg.to_string();
        client
            .mutate_seq_mdata_entries(name, tag, entry_actions)
            .await
            .map_err(|err| {
                if let SafeCoreError::DataError(SafeNdError::InvalidEntryActions(_)) = err {
                    Error::EntryExists(format!("{}: {}", message, err))
                } else {
                    Error::NetDataError(format!("{}: {}", message, err))
                }
            })
    }
}

#[async_trait]
impl SafeApp for SafeAppScl {
    fn new() -> Self {
        Self { safe_conn: None }
    }

    #[cfg(feature = "fake-auth")]
    async fn connect(&mut self, _app_id: &str, _auth_credentials: Option<&str>) -> Result<()> {
        warn!("Using fake authorisation for testing...");
        self.safe_conn = Some(create_app().await);
        Ok(())
    }

    // Connect to the SAFE Network using the provided app id and auth credentials
    #[cfg(not(feature = "fake-auth"))]
    async fn connect(&mut self, app_id: &str, auth_credentials: Option<&str>) -> Result<()> {
        debug!("Connecting to SAFE Network...");

        let disconnect_cb = || {
            warn!("Connection with the SAFE Network was lost");
        };

        let app = match auth_credentials {
            Some(auth_credentials) => {
                let auth_granted = decode_ipc_msg(auth_credentials)?;
                match auth_granted {
                    AuthResponseType::Registered(authgranted) => {
                        App::registered(app_id.to_string(), authgranted, disconnect_cb).await
                    }
                    AuthResponseType::Unregistered(config) => {
                        App::unregistered(disconnect_cb, Some(config)).await
                    }
                }
            }
            None => App::unregistered(disconnect_cb, None).await,
        }
        .map_err(|err| {
            Error::ConnectionError(format!("Failed to connect to the SAFE Network: {:?}", err))
        })?;

        self.safe_conn = Some(app);
        debug!("Successfully connected to the Network!!!");
        Ok(())
    }

    // === Coins operations ===
    async fn create_balance(
        &mut self,
        from_sk: Option<SecretKey>,
        new_balance_owner: PublicKey,
        amount: Coins,
    ) -> Result<XorName> {
        let client = &self.get_safe_app()?.client;
        let from_fullid = from_sk.map(ClientFullId::from);
        client
            .create_balance(
                from_fullid.as_ref(),
                SafeNdPublicKey::Bls(new_balance_owner),
                amount,
                None,
            )
            .await
            .map_err(|err| {
                if let SafeCoreError::DataError(SafeNdError::InsufficientBalance) = err {
                    Error::NotEnoughBalance(amount.to_string())
                } else {
                    Error::NetDataError(format!("Failed to create a SafeKey: {:?}", err))
                }
            })?;

        let xorname = xorname_from_pk(new_balance_owner);
        Ok(xorname)
    }

    async fn allocate_test_coins(&mut self, owner_sk: SecretKey, amount: Coins) -> Result<XorName> {
        info!("Creating test SafeKey with {} test coins", amount);
        let xorname = xorname_from_pk(owner_sk.public_key());
        test_create_balance(&ClientFullId::from(owner_sk), amount)
            .await
            .map_err(|e| Error::NetDataError(format!("Failed to allocate test coins: {:?}", e)))?;

        Ok(xorname)
    }

    async fn get_balance_from_sk(&self, sk: SecretKey) -> Result<Coins> {
        let client = &self.get_safe_app()?.client;
        let coins = client
            .get_balance(Some(&ClientFullId::from(sk)))
            .await
            .map_err(|e| Error::NetDataError(format!("Failed to retrieve balance: {:?}", e)))?;

        Ok(coins)
    }

    async fn safecoin_transfer_to_xorname(
        &mut self,
        from_sk: Option<SecretKey>,
        to_xorname: XorName,
        tx_id: TransactionId,
        amount: Coins,
    ) -> Result<Transaction> {
        let client = &self.get_safe_app()?.client;
        let from_fullid = from_sk.map(ClientFullId::from);
        let tx = client
            .transfer_coins(from_fullid.as_ref(), to_xorname, amount, Some(tx_id))
            .await
            .map_err(|err| match err {
                SafeCoreError::DataError(SafeNdError::ExcessiveValue)
                | SafeCoreError::DataError(SafeNdError::InsufficientBalance) => {
                    Error::NotEnoughBalance(amount.to_string())
                }
                SafeCoreError::DataError(SafeNdError::InvalidOperation) => {
                    Error::InvalidAmount(amount.to_string())
                }
                other => Error::NetDataError(format!("Failed to transfer coins: {:?}", other)),
            })?;

        Ok(tx)
    }

    async fn safecoin_transfer_to_pk(
        &mut self,
        from_sk: Option<SecretKey>,
        to_pk: PublicKey,
        tx_id: TransactionId,
        amount: Coins,
    ) -> Result<Transaction> {
        let to_xorname = xorname_from_pk(to_pk);
        self.safecoin_transfer_to_xorname(from_sk, to_xorname, tx_id, amount)
            .await
    }

    // === ImmutableData operations ===
    async fn put_public_immutable(&mut self, data: &[u8], dry_run: bool) -> Result<XorName> {
        // TODO: allow this operation to work without a connection when it's a dry run
        let client = &self.get_safe_app()?.client;

        let data_vec = data.to_vec();
        let data_map = if dry_run {
            immutable_data::gen_data_map(
                client, &data_vec, /*public:*/ true, /*encryption_key:*/ None,
            )
            .await
        } else {
            immutable_data::create(
                client, &data_vec, /*public:*/ true, /*encryption_key:*/ None,
            )
            .await
        }
        .map_err(|e| {
            Error::NetDataError(format!(
                "Failed to create data map for Public ImmutableData: {:?}",
                e
            ))
        })?;

        let xorname = *data_map.address().name();

        if !dry_run {
            client.put_idata(data_map).await.map_err(|e| {
                Error::NetDataError(format!("Failed to PUT Public ImmutableData: {:?}", e))
            })?;
        }

        Ok(xorname)
    }

    async fn get_public_immutable(&self, xorname: XorName, range: Range) -> Result<Vec<u8>> {
        debug!("Fetching immutable data: {:?}", &xorname);

        let client = &self.get_safe_app()?.client;
        let immd_data_addr = IDataAddress::Pub(xorname);
        let data = if let Some((start, end)) = range {
            let len = if let Some(end_index) = end {
                Some(end_index - start.unwrap_or_else(|| 0))
            } else {
                None
            };

            immutable_data::get_value(
                client,
                immd_data_addr,
                start,
                len,
                /*decryption_key:*/ None,
            )
            .await
        } else {
            immutable_data::get_value(
                client,
                immd_data_addr,
                None,
                None,
                /*decryption_key:*/ None,
            )
            .await
        }
        .map_err(|e| Error::NetDataError(format!("Failed to GET Public ImmutableData: {:?}", e)))?;

        debug!(
            "Public ImmutableData data successfully retrieved from: {:?}",
            &xorname
        );

        Ok(data)
    }

    // === MutableData operations ===
    async fn put_mdata(
        &mut self,
        name: Option<XorName>,
        tag: u64,
        // _data: Option<String>,
        _permissions: Option<String>,
    ) -> Result<XorName> {
        let safe_app = self.get_safe_app()?;
        let client = &safe_app.client;
        let owner_key_option = client.owner_key().await;
        let owners = if let SafeNdPublicKey::Bls(owners) = owner_key_option {
            owners
        } else {
            return Err(Error::Unexpected(
                "Failed to retrieve public key.".to_string(),
            ));
        };

        let xorname = name.unwrap_or_else(rand::random);

        let permission_set = MDataPermissionSet::new()
            .allow(MDataAction::Read)
            .allow(MDataAction::Insert)
            .allow(MDataAction::Update)
            .allow(MDataAction::Delete)
            .allow(MDataAction::ManagePermissions);

        let mut permission_map = BTreeMap::new();
        let sign_pk = get_public_bls_key(safe_app).await?;
        let app_pk = SafeNdPublicKey::Bls(sign_pk);
        permission_map.insert(app_pk, permission_set);

        let mdata = SeqMutableData::new_with_data(
            xorname,
            tag,
            BTreeMap::new(),
            permission_map,
            SafeNdPublicKey::Bls(owners),
        );

        client
            .put_seq_mutable_data(mdata)
            .await
            .map_err(|err| Error::NetDataError(format!("Failed to put mutable data: {}", err)))?;

        Ok(xorname)
    }

    async fn get_mdata(&self, name: XorName, tag: u64) -> Result<SeqMutableData> {
        let client = &self.get_safe_app()?.client;
        client
            .get_seq_mdata(name, tag)
            .await
            .map_err(|e| Error::NetDataError(format!("Failed to get MD: {:?}", e)))
    }

    async fn mdata_insert(
        &mut self,
        name: XorName,
        tag: u64,
        key: &[u8],
        value: &[u8],
    ) -> Result<()> {
        let entry_actions = MDataSeqEntryActions::new();
        let entry_actions = entry_actions.ins(key.to_vec(), value.to_vec(), 0);
        self.mutate_seq_mdata_entries(name, tag, entry_actions, "Failed to insert to SeqMD")
            .await
    }

    async fn mdata_get_value(&self, name: XorName, tag: u64, key: &[u8]) -> Result<MDataSeqValue> {
        let client = &self.get_safe_app()?.client;
        let key_vec = key.to_vec();
        client
            .get_seq_mdata_value(name, tag, key_vec)
            .await
            .map_err(|err| match err {
                SafeCoreError::DataError(SafeNdError::AccessDenied) => {
                    Error::AccessDenied(format!("Failed to retrieve a key: {:?}", key))
                }
                SafeCoreError::DataError(SafeNdError::NoSuchData) => {
                    Error::ContentNotFound(format!(
                        "Sequenced MutableData not found at Xor name: {}",
                        xorname_to_hex(&name)
                    ))
                }
                SafeCoreError::DataError(SafeNdError::NoSuchEntry) => {
                    Error::EntryNotFound(format!(
                        "Entry not found in Sequenced MutableData found at Xor name: {}",
                        xorname_to_hex(&name)
                    ))
                }
                err => Error::NetDataError(format!("Failed to retrieve a key. {:?}", err)),
            })
    }

    async fn mdata_list_entries(
        &self,
        name: XorName,
        tag: u64,
    ) -> Result<BTreeMap<Vec<u8>, MDataSeqValue>> {
        let client = &self.get_safe_app()?.client;
        client
            .list_seq_mdata_entries(name, tag)
            .await
            .map_err(|err| match err {
                SafeCoreError::DataError(SafeNdError::AccessDenied) => {
                    Error::AccessDenied(format!(
                        "Failed to get Sequenced MutableData at: {:?} (type tag: {})",
                        name, tag
                    ))
                }
                SafeCoreError::DataError(SafeNdError::NoSuchData) => {
                    Error::ContentNotFound(format!(
                        "Sequenced MutableData not found at Xor name: {} (type tag: {})",
                        xorname_to_hex(&name),
                        tag
                    ))
                }
                SafeCoreError::DataError(SafeNdError::NoSuchEntry) => {
                    Error::EntryNotFound(format!(
                    "Entry not found in Sequenced MutableData found at Xor name: {} (type tag: {})",
                    xorname_to_hex(&name),
                    tag
                ))
                }
                err => {
                    Error::NetDataError(format!("Failed to get Sequenced MutableData. {:?}", err))
                }
            })
    }

    async fn mdata_update(
        &mut self,
        name: XorName,
        tag: u64,
        key: &[u8],
        value: &[u8],
        version: u64,
    ) -> Result<()> {
        let entry_actions = MDataSeqEntryActions::new();
        let entry_actions = entry_actions.update(key.to_vec(), value.to_vec(), version);
        self.mutate_seq_mdata_entries(name, tag, entry_actions, "Failed to update SeqMD")
            .await
    }

    // === Sequence data operations ===
    async fn store_sequence_data(
        &mut self,
        data: &[u8],
        name: Option<XorName>,
        tag: u64,
        _permissions: Option<String>,
    ) -> Result<XorName> {
        debug!(
            "Storing Sequence data with tag type: {:?}, xorname: {:?}",
            tag, name
        );

        let safe_app: &App = self.get_safe_app()?;
        let xorname = name.unwrap_or_else(rand::random);
        info!("Xorname for storage: {:?}", &xorname);

        // Set permissions for append and manage perms to this application
        let app_public_key = SafeNdPublicKey::Bls(get_public_bls_key(safe_app).await?);
        let user_app = SDataUser::Key(app_public_key);
        let mut perms = BTreeMap::<SDataUser, SDataPubUserPermissions>::new();
        let _ = perms.insert(user_app, SDataPubUserPermissions::new(true, true));

        // The Sequence's owner will be the user
        let user_acc_owner = safe_app.client.owner_key().await;

        // Store the Sequence on the network
        let address = safe_app
            .client
            .store_pub_sdata(xorname, tag, user_acc_owner, perms)
            .await
            .map_err(|e| Error::NetDataError(format!("Failed to store Sequence data: {:?}", e)))?;

        let _op = safe_app
            .client
            .sdata_append(address, data.to_vec())
            .await
            .map_err(|e| {
                Error::NetDataError(format!("Failed to append data to the Sequence: {:?}", e))
            })?;

        Ok(xorname)
    }

    async fn sequence_get_last_entry(&self, name: XorName, tag: u64) -> Result<(u64, Vec<u8>)> {
        debug!(
            "Fetching Sequence data w/ type: {:?}, xorname: {:?}",
            tag, name
        );

        let safe_app: &App = self.get_safe_app()?;

        let sequence_address = SDataAddress::Public { name, tag };
        safe_app
            .client
            .get_sdata_last_entry(sequence_address)
            .await
            .map_err(|err| {
                if let SafeCoreError::DataError(SafeNdError::NoSuchEntry) = err {
                    Error::EmptyContent(format!("Empty Sequence found at XoR name {}", name))
                } else {
                    Error::NetDataError(format!(
                        "Failed to retrieve last entry from Sequence data: {:?}",
                        err
                    ))
                }
            })
    }

    async fn sequence_get_entry(&self, name: XorName, tag: u64, index: u64) -> Result<Vec<u8>> {
        debug!(
            "Fetching Sequence data w/ type: {:?}, xorname: {:?}",
            tag, name
        );

        let safe_app: &App = self.get_safe_app()?;

        let sequence_address = SDataAddress::Public { name, tag };
        let start = SDataIndex::FromStart(index);
        let end = SDataIndex::FromStart(index + 1);
        let res = safe_app
            .client
            .get_sdata_range(sequence_address, (start, end))
            .await
            .map_err(|err| {
                if let SafeCoreError::DataError(SafeNdError::NoSuchEntry) = err {
                    Error::VersionNotFound(format!(
                        "Invalid version ({}) for Sequence found at XoR name {}",
                        index, name
                    ))
                } else {
                    Error::NetDataError(format!(
                        "Failed to retrieve entry at index {} from Sequence data: {:?}",
                        index, err
                    ))
                }
            })?;

        let entry = res.get(0).ok_or_else(|| {
            Error::EmptyContent(format!(
                "Empty Sequence found at Xor name {}",
                xorname_to_hex(&name)
            ))
        })?;

        Ok(entry.to_vec())
    }

    async fn sequence_append(&mut self, data: &[u8], name: XorName, tag: u64) -> Result<()> {
        debug!(
            "Appending to Sequence data w/ type: {:?}, xorname: {:?}",
            tag, name
        );

        let safe_app: &App = self.get_safe_app()?;

        let sdata_address = SDataAddress::Public { name, tag };
        safe_app
            .client
            .sdata_append(sdata_address, data.to_vec())
            .await
            .map_err(|e| Error::NetDataError(format!("Failed to append to Sequence: {:?}", e)))
    }
}

// Helpers

async fn get_public_bls_key(safe_app: &App) -> Result<PublicKey> {
    let pk = safe_app
        .client
        .public_key()
        .await
        .bls()
        .ok_or_else(|| Error::Unexpected("Client's key is not a BLS Public Key".to_string()))?;

    Ok(pk)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::app::test_helpers::new_safe_instance;

    // Helper function to obtain utf8 string from bytes slice
    fn utf8_str_from_slice(slice: &[u8]) -> Result<String> {
        let utf8_str = std::str::from_utf8(slice).map_err(|err| {
            Error::Unexpected(format!("Failed to read data as an utf8 string: {}", err))
        })?;
        Ok(utf8_str.to_string())
    }

    #[tokio::test]
    async fn test_put_and_get_immutable_data() -> Result<()> {
        let mut safe = new_safe_instance().await?;

        let id1 = b"HELLLOOOOOOO".to_vec();

        let xorname = safe.safe_app.put_public_immutable(&id1, false).await?;
        let data = safe.safe_app.get_public_immutable(xorname, None).await?;
        let text = utf8_str_from_slice(data.as_slice())?;
        assert_eq!(text, "HELLLOOOOOOO");
        Ok(())
    }

    #[tokio::test]
    async fn test_put_get_update_sequence() -> Result<()> {
        let mut safe = new_safe_instance().await?;

        let entry1 = b"VALUE1";
        let type_tag = 12322;
        let xorname = safe
            .safe_app
            .store_sequence_data(entry1, None, type_tag, None)
            .await?;

        let (this_version, entry) = safe
            .safe_app
            .sequence_get_last_entry(xorname, type_tag)
            .await?;

        assert_eq!(this_version, 0);
        assert_eq!(&utf8_str_from_slice(entry.as_slice())?, "VALUE1");

        let entry2 = b"VALUE2";
        safe.safe_app
            .sequence_append(entry2, xorname, type_tag)
            .await?;
        let (the_latest_version, data_updated) = safe
            .safe_app
            .sequence_get_last_entry(xorname, type_tag)
            .await?;

        assert_eq!(the_latest_version, 1);
        assert_eq!(&utf8_str_from_slice(data_updated.as_slice())?, "VALUE2");

        let first_version = 0;
        let first_data = safe
            .safe_app
            .sequence_get_entry(xorname, type_tag, first_version)
            .await?;

        assert_eq!(&utf8_str_from_slice(first_data.as_slice())?, "VALUE1");

        let second_version = 1;
        let second_data = safe
            .safe_app
            .sequence_get_entry(xorname, type_tag, second_version)
            .await?;

        assert_eq!(&utf8_str_from_slice(second_data.as_slice())?, "VALUE2");

        // test checking for versions that dont exist
        let nonexistant_version = 2;
        match safe
            .safe_app
            .sequence_get_entry(xorname, type_tag, nonexistant_version)
            .await
        {
            Ok(_) => Err(Error::Unexpected(
                "No error thrown when passing an outdated new version".to_string(),
            )),
            Err(Error::VersionNotFound(msg)) => {
                assert!(msg.contains(&format!(
                    "Invalid version ({}) for Sequence found at XoR name {}",
                    nonexistant_version, xorname
                )));
                Ok(())
            }
            err => Err(Error::Unexpected(format!(
                "Error returned is not the expected one: {:?}",
                err
            ))),
        }
    }
}
