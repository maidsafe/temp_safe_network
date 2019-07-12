// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::helpers::{xorname_from_pk, KeyPair};
use super::xorurl::{create_random_xorname, XorUrlEncoder};
use super::{Error, ResultReturn};
use futures::future::Future;
use log::{debug, info, warn};
use rand::rngs::OsRng;
use rand_core::RngCore;
use safe_app::AppError::CoreError;
use safe_app::{run, App};
use safe_core::CoreError as SafeCoreError;
use std::str::FromStr;

#[cfg(not(feature = "fake-auth"))]
use super::helpers::decode_ipc_msg;
#[cfg(feature = "fake-auth")]
use safe_app::test_utils::create_app;
use safe_core::client::Client;
use safe_nd::{
    AData, ADataAddress, ADataAppend, ADataIndex, ADataOwner, ADataPubPermissionSet,
    ADataPubPermissions, ADataUser, AppendOnlyData, Coins, MDataAction, MDataPermissionSet,
    MDataSeqEntryActions, MDataValue, PubImmutableData, PubSeqAppendOnlyData,
    PublicKey as SafeNdPublicKey, SeqMutableData, XorName,
};

pub use threshold_crypto::{PublicKey, SecretKey};

use std::collections::BTreeMap;
use unwrap::unwrap;
use uuid::Uuid;

const APP_NOT_CONNECTED: &str = "Application is not connected to the network";

type AppendOnlyDataRawData = (Vec<u8>, Vec<u8>);

#[derive(Default)]
pub struct SafeApp {
    safe_conn: Option<App>,
}

impl SafeApp {
    pub fn new() -> Self {
        Self { safe_conn: None }
    }

    #[allow(dead_code)]
    #[cfg(feature = "fake-auth")]
    pub fn connect(&mut self, _app_id: &str, _auth_credentials: Option<&str>) -> ResultReturn<()> {
        warn!("Using fake authorisation for testing...");
        self.safe_conn = Some(create_app());
        Ok(())
    }

    // Connect to the SAFE Network using the provided app id and auth credentials
    #[cfg(not(feature = "fake-auth"))]
    pub fn connect(&mut self, app_id: &str, auth_credentials: Option<&str>) -> ResultReturn<()> {
        debug!("Connecting to SAFE Network...");

        let disconnect_cb = || {
            warn!("Connection with the SAFE Network was lost");
        };

        let app = match auth_credentials {
            Some(auth_credentials) => {
                let auth_granted = decode_ipc_msg(auth_credentials)?;
                App::registered(app_id.to_string(), auth_granted, disconnect_cb).map_err(|err| {
                    Error::ConnectionError(format!(
                        "Failed to connect to the SAFE Network: {:?}",
                        err
                    ))
                })?
            }
            None => App::unregistered(disconnect_cb, None).map_err(|err| {
                Error::ConnectionError(format!("Failed to connect to the SAFE Network: {:?}", err))
            })?,
        };

        self.safe_conn = Some(app);
        debug!("Successfully connected to the Network!!!");
        Ok(())
    }

    pub fn create_balance(
        &mut self,
        _from_pk: &PublicKey,
        _from_sk: &SecretKey,
        new_balance_owner: &PublicKey,
        amount: &str,
    ) -> ResultReturn<XorName> {
        let safe_app: &App = match &self.safe_conn {
            Some(app) => &app,
            None => return Err(Error::ConnectionError(APP_NOT_CONNECTED.to_string())),
        };

        let bls_sk = threshold_crypto::SecretKey::random(); // FIXME: use provided from_sk instead of creating a random one
        let pk = bls_sk.public_key(); // FIXME: use provided new_balance_owner instead of creating a random one
        let coins_amount =
            Coins::from_str(amount).map_err(|err| Error::InvalidAmount(format!("{:?}", err)))?;

        run(safe_app, move |client, _app_context| {
            client
                .create_balance(Some(&bls_sk), SafeNdPublicKey::Bls(pk), coins_amount, None)
                .map_err(|e| CoreError(SafeCoreError::Unexpected(format!("{:?}", e))))
        })
        .map_err(|e| Error::NetDataError(format!("Failed to create a CoinBalance: {:?}", e)))?;

        let xorname = xorname_from_pk(new_balance_owner);
        Ok(xorname)
    }

    pub fn allocate_test_coins(
        &mut self,
        to_pk: &PublicKey,
        amount: &str,
    ) -> ResultReturn<XorName> {
        let safe_app: &App = match &self.safe_conn {
            Some(app) => &app,
            None => return Err(Error::ConnectionError(APP_NOT_CONNECTED.to_string())),
        };
        let xorname = xorname_from_pk(to_pk);
        let coins_amount =
            Coins::from_str(amount).map_err(|err| Error::InvalidAmount(format!("{:?}", err)))?;

        run(safe_app, move |client, _app_context| {
            client
                .allocate_test_coins(&xorname, coins_amount)
                .map_err(|_| {
                    CoreError(SafeCoreError::Unexpected(
                        "Couldn't get account's owner pk".to_string(),
                    ))
                })
        })
        .map_err(|e| Error::NetDataError(format!("Failed to allocate test coins: {:?}", e)))?;

        Ok(xorname)
    }

    pub fn get_balance_from_pk(&self, pk: &PublicKey, sk: &SecretKey) -> ResultReturn<String> {
        let xorname = xorname_from_pk(pk);
        self.get_balance_from_xorname(&xorname, sk)
    }

    pub fn get_balance_from_xorname(
        &self,
        _xorname: &XorName,
        _sk: &SecretKey,
    ) -> ResultReturn<String> {
        let safe_app: &App = match &self.safe_conn {
            Some(app) => &app,
            None => return Err(Error::ConnectionError(APP_NOT_CONNECTED.to_string())),
        };
        // FIXME: use provided sk instead of creating a random one
        let bls_sk = threshold_crypto::SecretKey::random();
        let coins_amount = run(safe_app, move |client, _app_context| {
            client
                .get_balance(Some(&bls_sk))
                .map_err(|e| CoreError(SafeCoreError::Unexpected(format!("{:?}", e))))
        })
        .map_err(|e| Error::NetDataError(format!("Failed to retrieve balance: {:?}", e)))?;

        Ok(coins_amount.to_string())
    }

    // TODO: replace with actual code for calling SCL
    pub fn fetch_pk_from_xorname(&self, _xorname: &XorName) -> ResultReturn<PublicKey> {
        Ok(KeyPair::random().pk)
    }

    // TODO: replace with actual code for calling SCL
    pub fn safecoin_transfer(
        &mut self,
        _from_pk: &PublicKey,
        _from_sk: &SecretKey,
        _to_pk: &PublicKey,
        _tx_id: &Uuid,
        _amount: &str,
    ) -> ResultReturn<Uuid> {
        Ok(Uuid::new_v4())
    }

    // TODO: Replace with SCL calling code
    #[allow(dead_code)]
    pub fn get_transaction(
        &self,
        _tx_id: &Uuid,
        _pk: &PublicKey,
        _sk: &SecretKey,
    ) -> ResultReturn<String> {
        Ok("Success(0)".to_string())
    }

    pub fn files_put_published_immutable(&mut self, data: &[u8]) -> ResultReturn<XorName> {
        let safe_app: &App = match &self.safe_conn {
            Some(app) => &app,
            None => return Err(Error::ConnectionError(APP_NOT_CONNECTED.to_string())),
        };

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

    pub fn files_get_published_immutable(&self, xorname: XorName) -> ResultReturn<Vec<u8>> {
        debug!("Fetching immutable data: {:?}", &xorname);

        let safe_app: &App = match &self.safe_conn {
            Some(app) => &app,
            None => return Err(Error::ConnectionError(APP_NOT_CONNECTED.to_string())),
        };

        let data = run(safe_app, move |client, _app_context| {
            client.get_idata(xorname).map_err(CoreError)
        })
        .map_err(|e| {
            Error::NetDataError(format!("Failed to GET Published ImmutableData: {:?}", e))
        })?;
        debug!("the_data: {:?}", &xorname);

        Ok(data.value().to_vec())
    }

    pub fn put_seq_append_only_data(
        &mut self,
        the_data: Vec<(Vec<u8>, Vec<u8>)>,
        name: Option<XorName>,
        tag: u64,
        _permissions: Option<String>,
    ) -> ResultReturn<XorName> {
        debug!(
            "Putting appendable data w/ type: {:?}, xorname: {:?}",
            &tag, &name
        );

        let safe_app: &App = match &self.safe_conn {
            Some(app) => &app,
            None => return Err(Error::ConnectionError(APP_NOT_CONNECTED.to_string())),
        };

        let xorname = name.unwrap_or_else(create_random_xorname);
        info!("Xorname for storage: {:?}", &xorname);

        run(safe_app, move |client, _app_context| {
            let appendable_data_address = ADataAddress::PubSeq { name: xorname, tag };
            let append_client = client.clone();

            let mut data = PubSeqAppendOnlyData::new(xorname, tag);

            // TODO: setup permissions from props
            let mut perms = BTreeMap::<ADataUser, ADataPubPermissionSet>::new();
            let set = ADataPubPermissionSet::new(true, true);
            let usr = ADataUser::Key(SafeNdPublicKey::Bls(unwrap!(client.public_bls_key())));
            let _ = perms.insert(usr, set);
            unwrap!(data.append_permissions(
                ADataPubPermissions {
                    permissions: perms,
                    data_index: 0,
                    owner_entry_index: 0,
                },
                0
            ));

            let append = ADataAppend {
                address: append_only_data_address,
                values: the_data,
            };

            let owner = ADataOwner {
                public_key: SafeNdPublicKey::Bls(unwrap!(client.public_bls_key())),
                data_index: 0,
                permissions_index: 1,
            };
            unwrap!(data.append_owner(owner, 0));

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

    pub fn append_seq_append_only_data(
        &mut self,
        the_data: Vec<(Vec<u8>, Vec<u8>)>,
        new_version: u64,
        xorname: XorName,
        tag: u64,
    ) -> ResultReturn<u64> {
        let safe_app: &App = match &self.safe_conn {
            Some(app) => &app,
            None => return Err(Error::ConnectionError(APP_NOT_CONNECTED.to_string())),
        };

        run(safe_app, move |client, _app_context| {
            let appendable_data_address = PubSeqAppendOnlyData::new(xorname, tag);

            let append = ADataAppend {
                address: *appendable_data_address.address(),
                values: the_data,
            };

            let target_index = new_version - 1;

            client
                .append_seq_adata(append, target_index)
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

    pub fn get_latest_seq_append_only_data(
        &self,
        xorname: XorName,
        tag: u64,
    ) -> ResultReturn<(u64, AppendOnlyDataRawData)> {
        debug!("Getting latest seq_append_only_data for: {:?}", &xorname);

        let safe_app: &App = match &self.safe_conn {
            Some(app) => &app,
            None => return Err(Error::ConnectionError(APP_NOT_CONNECTED.to_string())),
        };

        let appendable_data_address = PubSeqAppendOnlyData::new(xorname, tag);

        debug!(
            "Address for a_data : {:?}",
            *appendable_data_address.address()
        );

        let data_length = self
            .get_current_seq_append_only_data_version(xorname, tag)
            .unwrap();

        let data = run(safe_app, move |client, _app_context| {
            client
                .get_adata_last_entry(*appendable_data_address.address())
                .map_err(CoreError)
        })
        .map_err(|e| {
            Error::NetDataError(format!("Failed to get Sequenced Append Only Data: {:?}", e))
        })?;

        Ok((data_length, data))
    }

    pub fn get_current_seq_append_only_data_version(
        &self,
        name: XorName,
        tag: u64,
    ) -> ResultReturn<u64> {
        debug!("Getting seq appendable data, length for: {:?}", name);

        let safe_app: &App = match &self.safe_conn {
            Some(app) => &app,
            None => return Err(Error::ConnectionError(APP_NOT_CONNECTED.to_string())),
        };

        let appendable_data_address = PubSeqAppendOnlyData::new(name, tag);

        run(safe_app, move |client, _app_context| {
            client
                .get_adata_indices(*appendable_data_address.address())
                .map_err(CoreError)
        })
        .map_err(|e| {
            Error::NetDataError(format!(
                "Failed to get Sequenced Append Only Data indices: {:?}",
                e
            ))
        })
        .map(|data_returned| data_returned.data_index())
    }

    #[allow(dead_code)]
    pub fn get_seq_append_only_data(
        &self,
        name: XorName,
        tag: u64,
        version: u64,
    ) -> ResultReturn<AppendOnlyDataRawData> {
        debug!(
            "Getting seq appendable data, version: {:?}, from: {:?}",
            version, name
        );

        let safe_app: &App = match &self.safe_conn {
            Some(app) => &app,
            None => return Err(Error::ConnectionError(APP_NOT_CONNECTED.to_string())),
        };
        let appendable_data_address = PubSeqAppendOnlyData::new(name, tag);

        let data_length = self
            .get_current_seq_append_only_data_version(name, tag)
            .unwrap();

        let start = ADataIndex::FromStart(version);
        let end = ADataIndex::FromStart(version + 1);

        if version >= data_length {
            return Err(Error::NetDataError(format!(
                "The version, \"{:?}\" of \"{:?}\" does not exist",
                version, name
            )));
        }

        if version == data_length {
            let (_version, data) = self.get_latest_seq_append_only_data(name, tag).unwrap();
            return Ok(data);
        }

        let data = run(safe_app, move |client, _app_context| {
            client
                .get_adata_range(*appendable_data_address.address(), (start, end))
                .map_err(CoreError)
        })
        .map_err(|e| {
            Error::NetDataError(format!("Failed to get Sequenced Append Only Data: {:?}", e))
        })?;

        let this_version = data[0].clone();
        Ok(this_version)
    }

    pub fn put_seq_mutable_data(
        &self,
        name: Option<XorName>,
        tag: u64,
        // _data: Option<String>,
        _permissions: Option<String>,
    ) -> ResultReturn<XorName> {
        let safe_app: &App = match &self.safe_conn {
            Some(app) => &app,
            None => return Err(Error::ConnectionError(APP_NOT_CONNECTED.to_string())),
        };

        let owner_key_option = run(safe_app, move |client, _app_context| {
            let key = client.owner_key();

            Ok(key)
        })
        .map_err(|err| Error::Unexpected(format!("Failed to retrieve public key: {}", err)))?;

        let owners = match owner_key_option {
            Some(SafeNdPublicKey::Bls(pk)) => pk,
            _ => {
                return Err(Error::Unexpected(
                    "Failed to retrieve public key.".to_string(),
                ))
            }
        };

        run(safe_app, move |client, _app_context| {
            let xorname = name.unwrap_or_else(|| {
                let mut rng = unwrap!(OsRng::new());
                let mut xorname = XorName::default();
                rng.fill_bytes(&mut xorname.0);
                xorname
            });

            let permission_set = MDataPermissionSet::new()
                .allow(MDataAction::Read)
                .allow(MDataAction::Insert)
                .allow(MDataAction::Update)
                .allow(MDataAction::Delete)
                .allow(MDataAction::ManagePermissions);

            let mut permission_map = BTreeMap::new();
            let sign_pk = unwrap!(client.public_bls_key());
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
                .map_err(CoreError)
                .map(move |_| xorname)
        })
        .map_err(|err| Error::NetDataError(format!("Failed to put mutable data: {}", err)))
    }

    // TODO: we shouldn't need to expose this function, function like list_seq_mdata_entries should be exposed
    #[allow(dead_code)]
    fn get_seq_mdata(&self, xorurl: &str, tag: u64) -> ResultReturn<SeqMutableData> {
        let safe_app: &App = match &self.safe_conn {
            Some(app) => &app,
            None => return Err(Error::ConnectionError(APP_NOT_CONNECTED.to_string())),
        };

        let xorname = XorUrlEncoder::from_url(xorurl)?.xorname();
        run(safe_app, move |client, _app_context| {
            client.get_seq_mdata(xorname, tag).map_err(CoreError)
        })
        .map_err(|e| Error::NetDataError(format!("Failed to get MD: {:?}", e)))
    }

    pub fn seq_mutable_data_insert(
        &self,
        xorurl: &str,
        tag: u64,
        key: Vec<u8>,
        value: &[u8],
    ) -> ResultReturn<()> {
        let entry_actions = MDataSeqEntryActions::new();
        let entry_actions = entry_actions.ins(key.to_vec(), value.to_vec(), 0);
        self.mutate_seq_mdata_entries(xorurl, tag, entry_actions, "Failed to insert to MD")
    }

    // TODO: Replace with real scl calling code
    #[allow(dead_code)]
    pub fn mutable_data_delete(
        &mut self,
        _xorname: &XorName,
        _tag: u64,
        _key: &[u8],
    ) -> ResultReturn<()> {
        Ok(())
    }

    pub fn seq_mutable_data_get_value(
        &mut self,
        xorurl: &str,
        tag: u64,
        key: Vec<u8>,
    ) -> ResultReturn<MDataValue> {
        let safe_app: &App = match &self.safe_conn {
            Some(app) => &app,
            None => return Err(Error::ConnectionError(APP_NOT_CONNECTED.to_string())),
        };

        let xorname = XorUrlEncoder::from_url(xorurl)?.xorname();

        run(safe_app, move |client, _app_context| {
            client
                .get_seq_mdata_value(xorname, tag, key.to_vec())
                .map_err(CoreError)
        })
        .map_err(|e| Error::NetDataError(format!("Failed to retrieve key. {:?}", e)))
    }

    pub fn list_seq_mdata_entries(
        &self,
        xorurl: &str,
        tag: u64,
    ) -> ResultReturn<BTreeMap<Vec<u8>, MDataValue>> {
        let safe_app: &App = match &self.safe_conn {
            Some(app) => &app,
            None => return Err(Error::ConnectionError(APP_NOT_CONNECTED.to_string())),
        };

        let xorname = XorUrlEncoder::from_url(xorurl)?.xorname();

        run(safe_app, move |client, _app_context| {
            client
                .list_seq_mdata_entries(xorname, tag)
                .map_err(CoreError)
        })
        .map_err(|e| Error::NetDataError(format!("Failed to get MD: {:?}", e)))
    }

    #[allow(dead_code)]
    pub fn seq_mutable_data_update(
        &self,
        xorurl: &str,
        tag: u64,
        key: &[u8],
        value: &[u8],
        version: u64,
    ) -> ResultReturn<()> {
        let entry_actions = MDataSeqEntryActions::new();
        let entry_actions = entry_actions.ins(key.to_vec(), value.to_vec(), version);
        self.mutate_seq_mdata_entries(xorurl, tag, entry_actions, "Failed to update MD")
    }

    // private helper method
    fn mutate_seq_mdata_entries(
        &self,
        xorurl: &str,
        tag: u64,
        entry_actions: MDataSeqEntryActions,
        error_msg: &str,
    ) -> ResultReturn<()> {
        let safe_app: &App = match &self.safe_conn {
            Some(app) => &app,
            None => return Err(Error::ConnectionError(APP_NOT_CONNECTED.to_string())),
        };
        let xorname = XorUrlEncoder::from_url(xorurl)?.xorname();
        let message = error_msg.to_string();

        run(safe_app, move |client, _app_context| {
            client
                .mutate_seq_mdata_entries(xorname, tag, entry_actions)
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

#[test]
fn test_put_and_get_immutable_data() {
    use super::Safe;
    let mut safe = Safe::new("base32z".to_string());
    safe.connect("", Some("")).unwrap();

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
    safe.connect("", Some("")).unwrap();

    let key1 = b"KEY1".to_vec();
    let val1 = b"VALUE1".to_vec();
    let data1 = [(key1, val1)].to_vec();

    let type_tag = 12322;
    let xorname = safe
        .safe_app
        .put_seq_append_only_data(data1, None, type_tag, None)
        .unwrap();

    let (_this_version, data) = safe
        .safe_app
        .get_latest_seq_append_only_data(xorname, type_tag)
        .unwrap();

    assert_eq!(_this_version, 1);

    //TODO: Properly unwrap data so this is clear (0 being version, 1 being data)
    assert_eq!(std::str::from_utf8(data.0.as_slice()).unwrap(), "KEY1");
    assert_eq!(std::str::from_utf8(data.1.as_slice()).unwrap(), "VALUE1");

    let key2 = b"KEY2".to_vec();
    let val2 = b"VALUE2".to_vec();
    let data2 = [(key2, val2)].to_vec();
    let new_version = 2;

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

    let nonexistant_version = 2;
    // test cehcking for versions that dont exist
    match safe
        .safe_app
        .get_seq_append_only_data(xorname, type_tag, nonexistant_version)
    {
        Ok(_data) => panic!("No error thrown for a version that does not exist"),

        Err(_err) => assert!(true),
    }
}

// TODO: Enable once merged: https://github.com/maidsafe/safe_client_libs/issues/898
#[test]
#[ignore]
fn test_update_seq_append_only_data_error() {
    use super::Safe;
    let mut safe = Safe::new("base32z".to_string());
    safe.connect("", Some("")).unwrap();

    let key1 = b"KEY1".to_vec();
    let val1 = b"VALUE1".to_vec();
    let data1 = [(key1, val1)].to_vec();

    let type_tag = 12322;
    let xorname = safe
        .safe_app
        .put_seq_append_only_data(data1, None, type_tag, None)
        .unwrap();

    let (_this_version, data) = safe
        .safe_app
        .get_latest_seq_append_only_data(xorname, type_tag)
        .unwrap();

    assert_eq!(_this_version, 1);

    //TODO: Properly unwrap data so this is clear (0 being version, 1 being data)
    assert_eq!(std::str::from_utf8(data.0.as_slice()).unwrap(), "KEY1");
    assert_eq!(std::str::from_utf8(data.1.as_slice()).unwrap(), "VALUE1");

    let key2 = b"KEY2".to_vec();
    let val2 = b"VALUE2".to_vec();
    let data2 = [(key2, val2)].to_vec();
    let wrong_new_version = 1;

    match safe
        .safe_app
        .append_seq_append_only_data(data2, wrong_new_version, xorname, type_tag)
    {
        Ok(_) => panic!("No error thrown when passing an outdated new version"),
        Err(error) => assert!(format!("{}", error).contains("Something about the version")),
    }
}
