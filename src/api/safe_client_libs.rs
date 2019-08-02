// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::helpers::xorname_from_pk;
use super::safe_net::AppendOnlyDataRawData;
use super::xorurl::create_random_xorname;
use super::{Error, ResultReturn, SafeApp};
use futures::future::Future;
use log::{debug, info, warn};
use rand::rngs::OsRng;
use rand_core::RngCore;
use safe_app::{run, App, AppError::CoreError};
use safe_core::{client::wallet_transfer_coins, CoreError as SafeCoreError};
use safe_nd::Error as SafeNdError;

#[cfg(not(feature = "fake-auth"))]
use super::helpers::decode_ipc_msg;
#[cfg(feature = "fake-auth")]
use safe_app::test_utils::create_app;
use safe_core::client::Client;
use safe_nd::{
    AData, ADataAddress, ADataAppend, ADataIndex, ADataOwner, ADataPubPermissionSet,
    ADataPubPermissions, ADataUser, AppendOnlyData, Coins, MDataAction, MDataAddress,
    MDataPermissionSet, MDataSeqEntryActions, MDataValue, PubImmutableData, PubSeqAppendOnlyData,
    PublicKey as SafeNdPublicKey, SeqMutableData, XorName,
};

pub use threshold_crypto::{PublicKey, SecretKey};

use std::collections::BTreeMap;

const APP_NOT_CONNECTED: &str = "Application is not connected to the network";

#[derive(Default)]
pub struct SafeAppScl {
    safe_conn: Option<App>,
}

impl SafeAppScl {
    // Private helper to obtain the App instance
    fn get_safe_app(&self) -> ResultReturn<&App> {
        match &self.safe_conn {
            Some(app) => Ok(app),
            None => Err(Error::ConnectionError(APP_NOT_CONNECTED.to_string())),
        }
    }

    fn mutate_seq_mdata_entries(
        &self,
        name: XorName,
        tag: u64,
        entry_actions: MDataSeqEntryActions,
        error_msg: &str,
    ) -> ResultReturn<()> {
        let safe_app = self.get_safe_app()?;
        let message = error_msg.to_string();
        run(safe_app, move |client, _app_context| {
            client
                .mutate_seq_mdata_entries(name, tag, entry_actions)
                .map_err(CoreError)
        })
        .map_err(|err| {
            Error::NetDataError(format!(
                "Failed to mutate seq mutable data entrues: {}: {}",
                message, err
            ))
        })
    }
}

impl SafeApp for SafeAppScl {
    fn new() -> Self {
        Self { safe_conn: None }
    }

    #[allow(dead_code)]
    #[cfg(feature = "fake-auth")]
    fn connect(&mut self, _app_id: &str, _auth_credentials: Option<&str>) -> ResultReturn<()> {
        warn!("Using fake authorisation for testing...");
        self.safe_conn = Some(create_app());
        Ok(())
    }

    // Connect to the SAFE Network using the provided app id and auth credentials
    #[cfg(not(feature = "fake-auth"))]
    fn connect(&mut self, app_id: &str, auth_credentials: Option<&str>) -> ResultReturn<()> {
        debug!("Connecting to SAFE Network...");

        let disconnect_cb = || {
            warn!("Connection with the SAFE Network was lost");
        };

        let app = match auth_credentials {
            Some(auth_credentials) => {
                let auth_granted = decode_ipc_msg(auth_credentials)?;
                App::registered(app_id.to_string(), auth_granted, disconnect_cb)
            }
            None => App::unregistered(disconnect_cb, None),
        }
        .map_err(|err| {
            Error::ConnectionError(format!("Failed to connect to the SAFE Network: {:?}", err))
        })?;

        self.safe_conn = Some(app);
        debug!("Successfully connected to the Network!!!");
        Ok(())
    }

    fn create_balance(
        &mut self,
        from_sk: Option<SecretKey>,
        new_balance_owner: PublicKey,
        amount: Coins,
    ) -> ResultReturn<XorName> {
        let safe_app: &App = self.get_safe_app()?;
        let from_sk = match from_sk {
            Some(sk) => sk,
            None => get_secret_bls_key(safe_app)?,
        };

        run(safe_app, move |client, _app_context| {
            client
                .create_balance(
                    Some(&from_sk),
                    SafeNdPublicKey::Bls(new_balance_owner),
                    amount,
                    None,
                )
                .map_err(|err| match err {
                    SafeCoreError::NewRoutingClientError(e) => {
                        CoreError(SafeCoreError::Unexpected(format!("{:?}", e)))
                    }
                    other => CoreError(SafeCoreError::Unexpected(format!("{:?}", other))),
                })
        })
        .map_err(|err| {
            if let CoreError(SafeCoreError::Unexpected(e)) = err {
                if e == "InsufficientBalance" {
                    Error::NotEnoughBalance(amount.to_string())
                } else {
                    Error::NetDataError(format!("Failed to create a CoinBalance: {:?}", e))
                }
            } else {
                Error::NetDataError(format!("Failed to create a CoinBalance: {:?}", err))
            }
        })?;

        let xorname = xorname_from_pk(&new_balance_owner);
        Ok(xorname)
    }

    fn allocate_test_coins(&mut self, to_pk: PublicKey, amount: Coins) -> ResultReturn<XorName> {
        info!("Creating test CoinBalance with {} test coins", amount);
        let safe_app: &App = self.get_safe_app()?;
        let xorname = xorname_from_pk(&to_pk);
        run(safe_app, move |client, _app_context| {
            client.test_create_balance(&xorname, amount, SafeNdPublicKey::Bls(to_pk));
            Ok(())
        })
        .map_err(|e| Error::NetDataError(format!("Failed to allocate test coins: {:?}", e)))?;

        Ok(xorname)
    }

    fn get_balance_from_sk(&self, sk: SecretKey) -> ResultReturn<Coins> {
        let safe_app: &App = self.get_safe_app()?;
        let coins = run(safe_app, move |client, _app_context| {
            client
                .get_balance(Some(&sk))
                .map_err(|e| CoreError(SafeCoreError::Unexpected(format!("{:?}", e))))
        })
        .map_err(|e| Error::NetDataError(format!("Failed to retrieve balance: {:?}", e)))?;

        Ok(coins)
    }

    fn safecoin_transfer_to_xorname(
        &mut self,
        from_sk: SecretKey,
        to_xorname: XorName,
        tx_id: u64,
        amount: Coins,
    ) -> ResultReturn<u64> {
        wallet_transfer_coins(&from_sk, to_xorname, amount, Some(tx_id)).map_err(
            |err| match err {
                SafeNdError::ExcessiveValue => Error::NotEnoughBalance(amount.to_string()),
                other => Error::NetDataError(format!("Failed to transfer coins: {:?}", other)),
            },
        )?;

        Ok(tx_id)
    }

    fn safecoin_transfer_to_pk(
        &mut self,
        from_sk: SecretKey,
        to_pk: PublicKey,
        tx_id: u64,
        amount: Coins,
    ) -> ResultReturn<u64> {
        let to_xorname = xorname_from_pk(&to_pk);
        self.safecoin_transfer_to_xorname(from_sk, to_xorname, tx_id, amount)
    }

    // TODO: Replace with SCL calling code
    fn get_transaction(&self, _tx_id: u64, _pk: PublicKey, _sk: SecretKey) -> ResultReturn<String> {
        Ok("Success(0)".to_string())
    }

    fn files_put_published_immutable(&mut self, data: &[u8]) -> ResultReturn<XorName> {
        let safe_app: &App = self.get_safe_app()?;

        let the_idata = PubImmutableData::new(data.to_vec());
        let return_idata = the_idata.clone();
        run(safe_app, move |client, _app_context| {
            client.put_idata(the_idata).map_err(CoreError)
        })
        .map_err(|e| {
            Error::NetDataError(format!("Failed to PUT Published ImmutableData: {:?}", e))
        })?;

        Ok(*return_idata.name())
    }

    fn files_get_published_immutable(&self, xorname: XorName) -> ResultReturn<Vec<u8>> {
        debug!("Fetching immutable data: {:?}", &xorname);

        let safe_app: &App = self.get_safe_app()?;

        let data = run(safe_app, move |client, _app_context| {
            client.get_idata(xorname).map_err(CoreError)
        })
        .map_err(|e| {
            Error::NetDataError(format!("Failed to GET Published ImmutableData: {:?}", e))
        })?;
        debug!("the_data: {:?}", &xorname);

        Ok(data.value().to_vec())
    }

    fn put_seq_append_only_data(
        &mut self,
        the_data: Vec<(Vec<u8>, Vec<u8>)>,
        name: Option<XorName>,
        tag: u64,
        _permissions: Option<String>,
    ) -> ResultReturn<XorName> {
        debug!(
            "Putting appendable data w/ type: {:?}, xorname: {:?}",
            tag, name
        );

        let safe_app: &App = self.get_safe_app()?;
        let xorname = name.unwrap_or_else(create_random_xorname);
        info!("Xorname for storage: {:?}", &xorname);

        let append_only_data_address = ADataAddress::PubSeq { name: xorname, tag };
        let mut data = PubSeqAppendOnlyData::new(xorname, tag);

        // TODO: setup permissions from props
        let mut perms = BTreeMap::<ADataUser, ADataPubPermissionSet>::new();
        let set = ADataPubPermissionSet::new(true, true);
        let usr_app = ADataUser::Key(SafeNdPublicKey::Bls(get_public_bls_key(safe_app)?));
        let _ = perms.insert(usr_app, set);
        data.append_permissions(
            ADataPubPermissions {
                permissions: perms,
                data_index: 0,
                owner_entry_index: 0,
            },
            0,
        )
        .map_err(|e| {
            Error::Unexpected(format!(
                "Failed to set permissions for the Sequenced Append Only Data: {:?}",
                e
            ))
        })?;

        let usr_acc_owner = get_owner_pk(safe_app)?;
        let owner = ADataOwner {
            public_key: usr_acc_owner,
            data_index: 0,
            permissions_index: 1,
        };
        data.append_owner(owner, 0).map_err(|e| {
            Error::Unexpected(format!(
                "Failed to set the owner to the Sequenced Append Only Data: {:?}",
                e
            ))
        })?;

        let append = ADataAppend {
            address: append_only_data_address,
            values: the_data,
        };

        run(safe_app, move |client, _app_context| {
            let append_client = client.clone();
            client
                .put_adata(AData::PubSeq(data.clone()))
                .and_then(move |_| append_client.append_seq_adata(append, 0))
                .map_err(CoreError)
                .map(move |_| xorname)
        })
        .map_err(|e| {
            Error::NetDataError(format!("Failed to PUT Sequenced Append Only Data: {:?}", e))
        })
    }

    fn append_seq_append_only_data(
        &mut self,
        the_data: Vec<(Vec<u8>, Vec<u8>)>,
        new_version: u64,
        name: XorName,
        tag: u64,
    ) -> ResultReturn<u64> {
        let safe_app: &App = self.get_safe_app()?;
        run(safe_app, move |client, _app_context| {
            let append_only_data_address = ADataAddress::PubSeq { name, tag };
            let append = ADataAppend {
                address: append_only_data_address,
                values: the_data,
            };

            client
                .append_seq_adata(append, new_version)
                .map_err(CoreError)
        })
        .map_err(|e| {
            Error::NetDataError(format!(
                "Failed to UPDATE Sequenced Append Only Data: {:?}",
                e
            ))
        })?;

        Ok(new_version)
    }

    fn get_latest_seq_append_only_data(
        &self,
        name: XorName,
        tag: u64,
    ) -> ResultReturn<(u64, AppendOnlyDataRawData)> {
        debug!("Getting latest seq_append_only_data for: {:?}", &name);

        let safe_app: &App = self.get_safe_app()?;
        let append_only_data_address = ADataAddress::PubSeq { name, tag };

        debug!("Address for a_data : {:?}", append_only_data_address);

        let data_length = self
            .get_current_seq_append_only_data_version(name, tag)
            .map_err(|e| {
                Error::NetDataError(format!("Failed to get Sequenced Append Only Data: {:?}", e))
            })?;

        let data = run(safe_app, move |client, _app_context| {
            client
                .get_adata_last_entry(append_only_data_address)
                .map_err(CoreError)
        })
        .map_err(|e| {
            Error::NetDataError(format!("Failed to get Sequenced Append Only Data: {:?}", e))
        })?;

        Ok((data_length, data))
    }

    fn get_current_seq_append_only_data_version(
        &self,
        name: XorName,
        tag: u64,
    ) -> ResultReturn<u64> {
        debug!("Getting seq appendable data, length for: {:?}", name);

        let safe_app: &App = self.get_safe_app()?;
        let append_only_data_address = ADataAddress::PubSeq { name, tag };

        run(safe_app, move |client, _app_context| {
            client
                .get_adata_indices(append_only_data_address)
                .map_err(CoreError)
        })
        .map_err(|e| {
            Error::NetDataError(format!(
                "Failed to get Sequenced Append Only Data indices: {:?}",
                e
            ))
        })
        .map(|data_returned| data_returned.data_index() - 1)
    }

    fn get_seq_append_only_data(
        &self,
        name: XorName,
        tag: u64,
        version: u64,
    ) -> ResultReturn<AppendOnlyDataRawData> {
        debug!(
            "Getting seq appendable data, version: {:?}, from: {:?}",
            version, name
        );

        let safe_app: &App = self.get_safe_app()?;
        let append_only_data_address = ADataAddress::PubSeq { name, tag };

        // START TEMP BLOCK
        // This is a temporary block to overcome an issue in SCL panicing when trying
        // to fetch invalid versions: https://github.com/maidsafe/safe-nd/issues/93
        let data_length = self
            .get_current_seq_append_only_data_version(name, tag)
            .unwrap();
        if version > data_length {
            return Err(Error::VersionNotFound(format!(
                "Invalid version ({}) for Sequential AppendOnlyData found at XoR name {}",
                version, name
            )));
        }
        // END TEMP BLOCK

        let start = ADataIndex::FromStart(version);
        let end = ADataIndex::FromStart(version + 1);
        let data = run(safe_app, move |client, _app_context| {
            client
                .get_adata_range(append_only_data_address, (start, end))
                .map_err(CoreError)
        })
        .map_err(|e| {
            Error::NetDataError(format!("Failed to get Sequenced Append Only Data: {:?}", e))
        })?;

        let this_version = data[0].clone();
        Ok(this_version)
    }

    fn put_seq_mutable_data(
        &mut self,
        name: Option<XorName>,
        tag: u64,
        // _data: Option<String>,
        _permissions: Option<String>,
    ) -> ResultReturn<XorName> {
        let safe_app: &App = self.get_safe_app()?;
        let owner_key_option = get_owner_pk(safe_app)?;
        let owners = if let SafeNdPublicKey::Bls(owners) = owner_key_option {
            owners
        } else {
            return Err(Error::Unexpected(
                "Failed to retrieve public key.".to_string(),
            ));
        };

        let xorname = match name {
            Some(xorname) => xorname,
            None => {
                let mut rng = OsRng::new().map_err(|err| {
                    Error::Unexpected(format!("Failed to generate a random XoR name: {}", err))
                })?;
                let mut xorname = XorName::default();
                rng.fill_bytes(&mut xorname.0);
                xorname
            }
        };

        let permission_set = MDataPermissionSet::new()
            .allow(MDataAction::Read)
            .allow(MDataAction::Insert)
            .allow(MDataAction::Update)
            .allow(MDataAction::Delete)
            .allow(MDataAction::ManagePermissions);

        let mut permission_map = BTreeMap::new();
        let sign_pk = get_public_bls_key(safe_app)?;
        let app_pk = SafeNdPublicKey::Bls(sign_pk);
        permission_map.insert(app_pk, permission_set);

        let mdata = SeqMutableData::new_with_data(
            xorname,
            tag,
            BTreeMap::new(),
            permission_map,
            SafeNdPublicKey::Bls(owners),
        );

        run(safe_app, move |client, _app_context| {
            client
                .put_seq_mutable_data(mdata)
                .map_err(CoreError)
                .map(move |_| xorname)
        })
        .map_err(|err| Error::NetDataError(format!("Failed to put mutable data: {}", err)))
    }

    fn get_seq_mdata(&self, name: XorName, tag: u64) -> ResultReturn<SeqMutableData> {
        let safe_app: &App = self.get_safe_app()?;
        run(safe_app, move |client, _app_context| {
            client.get_seq_mdata(name, tag).map_err(CoreError)
        })
        .map_err(|e| Error::NetDataError(format!("Failed to get MD: {:?}", e)))
    }

    fn seq_mutable_data_insert(
        &mut self,
        name: XorName,
        tag: u64,
        key: &[u8],
        value: &[u8],
    ) -> ResultReturn<()> {
        let entry_actions = MDataSeqEntryActions::new();
        let entry_actions = entry_actions.ins(key.to_vec(), value.to_vec(), 0);
        self.mutate_seq_mdata_entries(name, tag, entry_actions, "Failed to insert to MD")
    }

    fn mutable_data_delete(&mut self, name: XorName, tag: u64) -> ResultReturn<()> {
        let safe_app: &App = self.get_safe_app()?;
        run(safe_app, move |client, _app_context| {
            client
                .delete_mdata(MDataAddress::Seq { name: name, tag })
                .map_err(CoreError)
        })
        .map_err(|e| Error::NetDataError(format!("Failed to delete MD: {:?}", e)))
    }

    fn seq_mutable_data_get_value(
        &mut self,
        name: XorName,
        tag: u64,
        key: &[u8],
    ) -> ResultReturn<MDataValue> {
        let safe_app: &App = self.get_safe_app()?;
        let key_vec = key.to_vec();
        run(safe_app, move |client, _app_context| {
            client
                .get_seq_mdata_value(name, tag, key_vec)
                .map_err(CoreError)
        })
        .map_err(|e| Error::NetDataError(format!("Failed to retrieve key. {:?}", e)))
    }

    fn list_seq_mdata_entries(
        &self,
        name: XorName,
        tag: u64,
    ) -> ResultReturn<BTreeMap<Vec<u8>, MDataValue>> {
        let safe_app: &App = self.get_safe_app()?;
        run(safe_app, move |client, _app_context| {
            client.list_seq_mdata_entries(name, tag).map_err(CoreError)
        })
        .map_err(|e| Error::NetDataError(format!("Failed to get MD: {:?}", e)))
    }

    fn seq_mutable_data_update(
        &mut self,
        name: XorName,
        tag: u64,
        key: &[u8],
        value: &[u8],
        version: u64,
    ) -> ResultReturn<()> {
        let entry_actions = MDataSeqEntryActions::new();
        let entry_actions = entry_actions.ins(key.to_vec(), value.to_vec(), version);
        self.mutate_seq_mdata_entries(name, tag, entry_actions, "Failed to update MD")
    }
}

// Helpers

fn get_owner_pk(safe_app: &App) -> ResultReturn<SafeNdPublicKey> {
    run(safe_app, move |client, _app_context| Ok(client.owner_key()))
        .map_err(|err| {
            Error::Unexpected(format!("Failed to retrieve account's public key: {}", err))
        })?
        .ok_or_else(|| Error::Unexpected("Account doesn't have a public key".to_string()))
}

fn get_public_bls_key(safe_app: &App) -> ResultReturn<PublicKey> {
    run(safe_app, move |client, _app_context| {
        Ok(client.public_bls_key())
    })
    .map_err(|err| {
        Error::Unexpected(format!(
            "Failed to retrieve account's public BLS key: {}",
            err
        ))
    })?
    .ok_or_else(|| Error::Unexpected("Account doesn't have a public BLS key".to_string()))
}

fn get_secret_bls_key(safe_app: &App) -> ResultReturn<SecretKey> {
    run(safe_app, move |client, _app_context| {
        Ok(client.secret_bls_key())
    })
    .map_err(|err| {
        Error::Unexpected(format!(
            "Failed to retrieve account's secret BLS key: {}",
            err
        ))
    })?
    .ok_or_else(|| Error::Unexpected("Account doesn't have a secret BLS key".to_string()))
}

// Unit tests

#[test]
fn test_put_and_get_immutable_data() {
    use super::Safe;
    let mut safe = Safe::new("base32z".to_string());
    safe.connect("", Some("fake-credentials")).unwrap();

    let id1 = b"HELLLOOOOOOO".to_vec();

    let xorname = safe.safe_app.files_put_published_immutable(&id1).unwrap();
    let data = safe
        .safe_app
        .files_get_published_immutable(xorname)
        .unwrap();
    let text = std::str::from_utf8(data.as_slice()).unwrap();
    assert_eq!(text.to_string(), "HELLLOOOOOOO");
}

#[test]
fn test_put_get_update_seq_append_only_data() {
    use super::Safe;
    let mut safe = Safe::new("base32z".to_string());
    safe.connect("", Some("fake-credentials")).unwrap();

    let key1 = b"KEY1".to_vec();
    let val1 = b"VALUE1".to_vec();
    let data1 = [(key1, val1)].to_vec();

    let type_tag = 12322;
    let xorname = safe
        .safe_app
        .put_seq_append_only_data(data1, None, type_tag, None)
        .unwrap();

    let (this_version, data) = safe
        .safe_app
        .get_latest_seq_append_only_data(xorname, type_tag)
        .unwrap();

    assert_eq!(this_version, 0);

    //TODO: Properly unwrap data so this is clear (0 being version, 1 being data)
    assert_eq!(std::str::from_utf8(data.0.as_slice()).unwrap(), "KEY1");
    assert_eq!(std::str::from_utf8(data.1.as_slice()).unwrap(), "VALUE1");

    let key2 = b"KEY2".to_vec();
    let val2 = b"VALUE2".to_vec();
    let data2 = [(key2, val2)].to_vec();
    let new_version = 1;

    let updated_version = safe
        .safe_app
        .append_seq_append_only_data(data2, new_version, xorname, type_tag)
        .unwrap();
    let (the_latest_version, data_updated) = safe
        .safe_app
        .get_latest_seq_append_only_data(xorname, type_tag)
        .unwrap();

    assert_eq!(updated_version, the_latest_version);

    assert_eq!(
        std::str::from_utf8(data_updated.0.as_slice()).unwrap(),
        "KEY2"
    );
    assert_eq!(
        std::str::from_utf8(data_updated.1.as_slice()).unwrap(),
        "VALUE2"
    );

    let first_version = 0;

    let first_data = safe
        .safe_app
        .get_seq_append_only_data(xorname, type_tag, first_version)
        .unwrap();

    assert_eq!(
        std::str::from_utf8(first_data.0.as_slice()).unwrap(),
        "KEY1"
    );
    assert_eq!(
        std::str::from_utf8(first_data.1.as_slice()).unwrap(),
        "VALUE1"
    );

    let second_version = 1;
    let second_data = safe
        .safe_app
        .get_seq_append_only_data(xorname, type_tag, second_version)
        .unwrap();

    assert_eq!(
        std::str::from_utf8(second_data.0.as_slice()).unwrap(),
        "KEY2"
    );
    assert_eq!(
        std::str::from_utf8(second_data.1.as_slice()).unwrap(),
        "VALUE2"
    );

    // test checking for versions that dont exist
    let nonexistant_version = 2;
    match safe
        .safe_app
        .get_seq_append_only_data(xorname, type_tag, nonexistant_version)
    {
        Ok(_) => panic!("No error thrown when passing an outdated new version"),
        Err(Error::VersionNotFound(msg)) => assert!(msg.contains(&format!(
            "Invalid version ({}) for Sequential AppendOnlyData found at XoR name {}",
            nonexistant_version, xorname
        ))),
        _ => panic!("Error returned is not the expected one"),
    }
}

#[test]
fn test_update_seq_append_only_data_error() {
    use super::Safe;
    let mut safe = Safe::new("base32z".to_string());
    safe.connect("", Some("fake-credentials")).unwrap();

    let key1 = b"KEY1".to_vec();
    let val1 = b"VALUE1".to_vec();
    let data1 = [(key1, val1)].to_vec();

    let type_tag = 12322;
    let xorname = safe
        .safe_app
        .put_seq_append_only_data(data1, None, type_tag, None)
        .unwrap();

    let (this_version, data) = safe
        .safe_app
        .get_latest_seq_append_only_data(xorname, type_tag)
        .unwrap();

    assert_eq!(this_version, 0);

    //TODO: Properly unwrap data so this is clear (0 being version, 1 being data)
    assert_eq!(std::str::from_utf8(data.0.as_slice()).unwrap(), "KEY1");
    assert_eq!(std::str::from_utf8(data.1.as_slice()).unwrap(), "VALUE1");

    let key2 = b"KEY2".to_vec();
    let val2 = b"VALUE2".to_vec();
    let data2 = [(key2, val2)].to_vec();
    let wrong_new_version = 0;

    match safe
        .safe_app
        .append_seq_append_only_data(data2, wrong_new_version, xorname, type_tag)
    {
        Ok(_) => panic!("No error thrown when passing an outdated new version"),
        Err(Error::NetDataError(msg)) => assert!(msg.contains("Invalid data successor")),
        _ => panic!("Error returned is not the expected one"),
    }
}
