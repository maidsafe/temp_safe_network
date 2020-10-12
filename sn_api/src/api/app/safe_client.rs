// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{fetch::Range, helpers::xorname_to_hex};
use crate::{Error, Result};

use log::{debug, info, warn};

use sn_client::{Client, ClientError as SafeClientError};
use sn_data_types::{
    Blob, BlobAddress, Error as SafeNdError, Map, MapAction, MapAddress, MapEntryActions,
    MapPermissionSet, MapSeqEntryActions, MapSeqValue, MapValue, Money, PublicBlob,
    PublicKey as SafeNdPublicKey, SeqMap, SequenceAddress, SequenceIndex,
    SequencePrivatePermissions, SequencePublicPermissions, SequenceUser,
};
use std::collections::BTreeMap;
use xor_name::XorName;

pub use threshold_crypto::{PublicKey, SecretKey};

const APP_NOT_CONNECTED: &str = "Application is not connected to the network";

#[derive(Default, Clone)]
pub struct SafeAppClient {
    safe_client: Option<Client>,
}

impl SafeAppClient {
    // Private helper to obtain the App instance
    fn get_safe_client(&mut self) -> Result<Client> {
        match &self.safe_client {
            Some(app) => Ok(app.clone()),
            None => Err(Error::ConnectionError(APP_NOT_CONNECTED.to_string())),
        }
    }

    pub fn new() -> Self {
        Self { safe_client: None }
    }

    // Connect to the SAFE Network using the provided app id and auth credentials
    pub async fn connect(&mut self, _app_id: &str, _auth_credentials: Option<&str>) -> Result<()> {
        debug!("Connecting to SAFE Network...");

        let _disconnect_cb = || {
            warn!("Connection with the SAFE Network was lost");
        };

        let app = Client::new(None).await?;

        // let app = match auth_credentials {
        //     Some(auth_credentials) => {
        //         let auth_granted = decode_ipc_msg(auth_credentials)?;
        //         match auth_granted {
        //             AuthResponseType::Registered(authgranted) => {
        //                 // TODO: This needs an existing SK now.
        //                 Client::new(None).await
        //                 // Client::new(app_id.to_string(), authgranted, disconnect_cb).await
        //             }
        //             // unregistered type used for returning bootstrap config for client
        //             // TODO: rename?
        //             AuthResponseType::Unregistered(config) => {
        //                 // TODO: what to do with config...
        //                 Client::new(None).await
        //             }
        //         }
        //     }
        //     None => Client::new(None).await,
        // }
        // .map_err(|err| {
        //     Error::ConnectionError(format!("Failed to connect to the SAFE Network: {:?}", err))
        // })?;

        self.safe_client = Some(app);
        debug!("Successfully connected to the Network!!!");
        Ok(())
    }

    // === Money operations ===
    pub async fn read_balance_from_sk(&mut self, sk: SecretKey) -> Result<Money> {
        let mut temp_client = Client::new(Some(sk)).await?;
        let coins = temp_client
            .get_balance()
            .await
            .map_err(|e| Error::NetDataError(format!("Failed to retrieve balance: {:?}", e)))?;

        Ok(coins)
    }

    #[cfg(feature = "simulated-payouts")]
    pub async fn trigger_simulated_farming_payout(&mut self, amount: Money) -> Result<()> {
        let mut client = self.get_safe_client()?;

        client.trigger_simulated_farming_payout(amount).await?;

        Ok(())
    }

    pub async fn safecoin_transfer_to_xorname(
        &mut self,
        from_sk: Option<SecretKey>,
        to_xorname: XorName,
        amount: Money,
    ) -> Result<()> {
        let client = match from_sk {
            Some(sk) => Client::new(Some(sk)).await?,
            None => self.get_safe_client()?,
        };

        unimplemented!();

        // TODO: attempt to get wallet pk from xorname

        // let to_pk = self.fetch(format!("safe://{:?}", to_xorname));
        // let to_url  = xorurl::SafeUrl::from::<XorName>(to_xorname);

        // let from_fullid = from_sk.map(ClientFullId::from);
        // let transfer_id = client
        //     .send_money( to_xorname, amount)
        //     .await
        //     .map_err(|err| match err {
        //         SafeClientError::DataError(SafeNdError::ExcessiveValue)
        //         | SafeClientError::DataError(SafeNdError::InsufficientBalance) => {
        //             Error::NotEnoughBalance(amount.to_string())
        //         }
        //         SafeClientError::DataError(SafeNdError::InvalidOperation) => {
        //             Error::InvalidAmount(amount.to_string())
        //         }
        //         other => Error::NetDataError(format!("Failed to transfer coins: {:?}", other)),
        //     })?;

        // Ok(transfer_id)
    }

    pub async fn safecoin_transfer_to_pk(
        &mut self,
        from_sk: Option<SecretKey>,
        to_pk: PublicKey,
        amount: Money,
    ) -> Result<(u64, SafeNdPublicKey)> {
        let mut client = match from_sk {
            Some(sk) => Client::new(Some(sk)).await?,
            None => self.get_safe_client()?,
        };

        let transfer_id = client
            .send_money(SafeNdPublicKey::Bls(to_pk), amount)
            .await?;

        Ok(transfer_id)
    }

    // // === Blob operations ===
    pub async fn store_public_blob(&mut self, data: &[u8], dry_run: bool) -> Result<XorName> {
        // TODO: allow this operation to work without a connection when it's a dry run
        let mut client = self.get_safe_client()?;

        let data_vec = data.to_vec();
        let blob_for_storage = Blob::Public(PublicBlob::new(data_vec));
        let xorname = blob_for_storage.address().name().clone();

        let _data_map = client
            .generate_data_map(&blob_for_storage)
            .await
            .map_err(|e| {
                Error::NetDataError(format!(
                    "Failed to create data map for Public Blob: {:?}",
                    e
                ))
            })?;

        if !dry_run {
            client
                .store_blob(blob_for_storage)
                .await
                .map_err(|e| Error::NetDataError(format!("Failed to PUT Public Blob: {:?}", e)))?;
        }

        Ok(xorname)
    }

    pub async fn get_public_blob(&mut self, xorname: XorName, range: Range) -> Result<Vec<u8>> {
        debug!("Fetching immutable data: {:?}", &xorname);

        let mut client = self.get_safe_client()?;
        let blob_address = BlobAddress::Public(xorname);
        let data = if let Some((start, end)) = range {
            let len = if let Some(end_index) = end {
                Some(end_index - start.unwrap_or_else(|| 0))
            } else {
                None
            };
            client.get_blob(blob_address, start, len).await
        } else {
            client.get_blob(blob_address, None, None).await
        }
        .map_err(|e| Error::NetDataError(format!("Failed to GET Public Blob: {:?}", e)))?;

        debug!(
            "Public Blob data successfully retrieved from: {:?}",
            &xorname
        );

        Ok(data.value().clone())
    }

    // === Map operations ===
    pub async fn store_map(
        &mut self,
        name: Option<XorName>,
        tag: u64,
        // _data: Option<String>,
        _permissions: Option<String>,
    ) -> Result<XorName> {
        let mut safe_client = self.get_safe_client()?;
        let client = &safe_client;
        let owner_key_option = client.public_key().await;
        let owners = if let SafeNdPublicKey::Bls(owners) = owner_key_option {
            owners
        } else {
            return Err(Error::Unexpected(
                "Failed to retrieve public key.".to_string(),
            ));
        };

        let xorname = name.unwrap_or_else(rand::random);

        let permission_set = MapPermissionSet::new()
            .allow(MapAction::Read)
            .allow(MapAction::Insert)
            .allow(MapAction::Update)
            .allow(MapAction::Delete)
            .allow(MapAction::ManagePermissions);

        let mut permission_map = BTreeMap::new();
        let sign_pk = get_public_bls_key(&safe_client).await?;
        let app_pk = SafeNdPublicKey::Bls(sign_pk);
        permission_map.insert(app_pk, permission_set);

        let map = Map::Seq(SeqMap::new_with_data(
            xorname,
            tag,
            BTreeMap::new(),
            permission_map,
            SafeNdPublicKey::Bls(owners),
        ));

        safe_client
            .store_map(map)
            .await
            .map_err(|err| Error::NetDataError(format!("Failed to put mutable data: {}", err)))?;

        Ok(xorname)
    }

    pub async fn get_map(&mut self, name: XorName, tag: u64) -> Result<Map> {
        let mut client = self.get_safe_client()?;
        let address = MapAddress::Seq { name, tag };

        client
            .get_map(address)
            .await
            .map_err(|e| Error::NetDataError(format!("Failed to get MD: {:?}", e)))
    }

    pub async fn map_insert(
        &mut self,
        name: XorName,
        tag: u64,
        key: &[u8],
        value: &[u8],
    ) -> Result<()> {
        let entry_actions = MapSeqEntryActions::new();
        let entry_actions = entry_actions.ins(key.to_vec(), value.to_vec(), 0);
        self.edit_map_entries(name, tag, entry_actions, "Failed to insert to SeqMD")
            .await
    }

    pub async fn map_get_value(&mut self, name: XorName, tag: u64, key: &[u8]) -> Result<MapValue> {
        let mut client = self.get_safe_client()?;
        let key_vec = key.to_vec();
        let address = MapAddress::Seq { name, tag };

        client
            .get_map_value(address, key_vec)
            .await
            .map_err(|err| match err {
                SafeClientError::DataError(SafeNdError::AccessDenied) => {
                    Error::AccessDenied(format!("Failed to retrieve a key: {:?}", key))
                }
                SafeClientError::DataError(SafeNdError::NoSuchData) => {
                    Error::ContentNotFound(format!(
                        "Sequenced Map not found at Xor name: {}",
                        xorname_to_hex(&name)
                    ))
                }
                SafeClientError::DataError(SafeNdError::NoSuchEntry) => {
                    Error::EntryNotFound(format!(
                        "Entry not found in Sequenced Map found at Xor name: {}",
                        xorname_to_hex(&name)
                    ))
                }
                err => Error::NetDataError(format!("Failed to retrieve a key. {:?}", err)),
            })
    }

    pub async fn list_map_entries(
        &mut self,
        name: XorName,
        tag: u64,
    ) -> Result<BTreeMap<Vec<u8>, MapSeqValue>> {
        let mut client = self.get_safe_client()?;
        client
            .list_seq_map_entries(name, tag)
            .await
            .map_err(|err| match err {
                SafeClientError::DataError(SafeNdError::AccessDenied) => {
                    Error::AccessDenied(format!(
                        "Failed to get Sequenced Map at: {:?} (type tag: {})",
                        name, tag
                    ))
                }
                SafeClientError::DataError(SafeNdError::NoSuchData) => {
                    Error::ContentNotFound(format!(
                        "Sequenced Map not found at Xor name: {} (type tag: {})",
                        xorname_to_hex(&name),
                        tag
                    ))
                }
                SafeClientError::DataError(SafeNdError::NoSuchEntry) => {
                    Error::EntryNotFound(format!(
                        "Entry not found in Sequenced Map found at Xor name: {} (type tag: {})",
                        xorname_to_hex(&name),
                        tag
                    ))
                }
                err => Error::NetDataError(format!("Failed to get Sequenced Map. {:?}", err)),
            })
    }

    async fn edit_map_entries(
        &mut self,
        name: XorName,
        tag: u64,
        entry_actions: MapSeqEntryActions,
        error_msg: &str,
    ) -> Result<()> {
        let mut client = self.get_safe_client()?;
        let message = error_msg.to_string();
        let address = MapAddress::Seq { name, tag };
        client
            .edit_map_entries(address, MapEntryActions::Seq(entry_actions))
            .await
            .map_err(|err| {
                if let SafeClientError::DataError(SafeNdError::InvalidEntryActions(_)) = err {
                    Error::EntryExists(format!("{}: {}", message, err))
                } else {
                    Error::NetDataError(format!("{}: {}", message, err))
                }
            })
    }

    pub async fn update_map(
        &mut self,
        name: XorName,
        tag: u64,
        key: &[u8],
        value: &[u8],
        version: u64,
    ) -> Result<()> {
        let entry_actions = MapSeqEntryActions::new();
        let entry_actions = entry_actions.update(key.to_vec(), value.to_vec(), version);
        self.edit_map_entries(name, tag, entry_actions, "Failed to update SeqMD")
            .await
    }

    // === Sequence data operations ===
    pub async fn store_sequence(
        &mut self,
        data: &[u8],
        name: Option<XorName>,
        tag: u64,
        _permissions: Option<String>,
        private: bool,
    ) -> Result<XorName> {
        debug!(
            "Storing {} Sequence data with tag type: {:?}, xorname: {:?}",
            if private { "Private" } else { "Public" },
            tag,
            name
        );

        let mut safe_client = self.get_safe_client()?;
        let xorname = name.unwrap_or_else(rand::random);
        info!("Xorname for storage: {:?}", &xorname);

        let app_public_key = get_public_bls_key(&safe_client).await?;

        // The Sequence's owner will be the user
        let user_acc_owner = safe_client.public_key().await;

        // Store the Sequence on the network
        let address = if private {
            // Set permissions for append, delete, and manage perms to this application
            let mut perms = BTreeMap::default();
            let _ = perms.insert(
                SafeNdPublicKey::Bls(app_public_key),
                SequencePrivatePermissions::new(true, true, true),
            );

            safe_client
                .store_private_sequence(
                    Some(vec![data.to_vec()]),
                    xorname,
                    tag,
                    user_acc_owner,
                    perms,
                )
                .await
                .map_err(|e| {
                    Error::NetDataError(format!("Failed to store Private Sequence data: {:?}", e))
                })?
        } else {
            // Set permissions for append and manage perms to this application
            let user_app = SequenceUser::Key(SafeNdPublicKey::Bls(app_public_key));
            let mut perms = BTreeMap::default();
            let _ = perms.insert(user_app, SequencePublicPermissions::new(true, true));

            safe_client
                .store_public_sequence(
                    Some(vec![data.to_vec()]),
                    xorname,
                    tag,
                    user_acc_owner,
                    perms,
                )
                .await
                .map_err(|e| {
                    Error::NetDataError(format!("Failed to store Public Sequence data: {:?}", e))
                })?
        };

        let _op = safe_client
            .append_to_sequence(address, data.to_vec())
            .await
            .map_err(|e| {
                Error::NetDataError(format!("Failed to append data to the Sequence: {:?}", e))
            })?;

        Ok(xorname)
    }

    pub async fn sequence_get_last_entry(
        &mut self,
        name: XorName,
        tag: u64,
        private: bool,
    ) -> Result<(u64, Vec<u8>)> {
        debug!(
            "Fetching {} Sequence data w/ type: {:?}, xorname: {:?}",
            if private { "Private" } else { "Public" },
            tag,
            name
        );

        let mut safe_client = self.get_safe_client()?;

        let sequence_address = if private {
            SequenceAddress::Private { name, tag }
        } else {
            SequenceAddress::Public { name, tag }
        };
        safe_client
            .get_sequence_last_entry(sequence_address)
            .await
            .map_err(|err| {
                if let SafeClientError::DataError(SafeNdError::NoSuchEntry) = err {
                    Error::EmptyContent(format!("Empty Sequence found at XoR name {}", name))
                } else {
                    Error::NetDataError(format!(
                        "Failed to retrieve last entry from Sequence data: {:?}",
                        err
                    ))
                }
            })
    }

    pub async fn sequence_get_entry(
        &mut self,
        name: XorName,
        tag: u64,
        index: u64,
        private: bool,
    ) -> Result<Vec<u8>> {
        debug!(
            "Fetching {} Sequence data w/ type: {:?}, xorname: {:?}",
            if private { "Private" } else { "Public" },
            tag,
            name
        );

        let mut safe_client = self.get_safe_client()?;

        let sequence_address = if private {
            SequenceAddress::Private { name, tag }
        } else {
            SequenceAddress::Public { name, tag }
        };
        let start = SequenceIndex::FromStart(index);
        let end = SequenceIndex::FromStart(index + 1);
        let res = safe_client
            .get_sequence_range(sequence_address, (start, end))
            .await
            .map_err(|err| {
                if let SafeClientError::DataError(SafeNdError::NoSuchEntry) = err {
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

    pub async fn append_to_sequence(
        &mut self,
        data: &[u8],
        name: XorName,
        tag: u64,
        private: bool,
    ) -> Result<()> {
        debug!(
            "Appending to {} Sequence data w/ type: {:?}, xorname: {:?}",
            if private { "Private" } else { "Public" },
            tag,
            name
        );

        let mut safe_client = self.get_safe_client()?;

        let sequence_address = if private {
            SequenceAddress::Private { name, tag }
        } else {
            SequenceAddress::Public { name, tag }
        };
        safe_client
            .append_to_sequence(sequence_address, data.to_vec())
            .await
            .map_err(|e| Error::NetDataError(format!("Failed to append to Sequence: {:?}", e)))
    }
}

// Helpers

async fn get_public_bls_key(safe_client: &Client) -> Result<PublicKey> {
    let pk = safe_client
        .public_key()
        .await
        .bls()
        .ok_or_else(|| Error::Unexpected("Client's key is not a BLS Public Key".to_string()))?;

    Ok(pk)
}
