// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::helpers::{decode_ipc_msg, xorname_from_pk, KeyPair};
use super::xorurl::{create_random_xorname, XorUrlEncoder};
use futures::future::Future;
use log::{debug, warn};
use rand::rngs::OsRng;
use rand_core::RngCore;
use safe_app::{run, App};

#[cfg(feature = "fake-auth")]
use safe_app::test_utils::create_app;
use safe_core::client::Client;
use safe_nd::{
    AData, ADataAddress, ADataAppend, ADataIndex, ADataOwner, ADataPubPermissionSet,
    ADataPubPermissions, ADataUser, AppendOnlyData, ImmutableData, MDataAction, MDataPermissionSet,
    MDataSeqEntryAction, MDataValue, PubSeqAppendOnlyData, PublicKey as SafeNdPublicKey,
    SeqMutableData, XorName,
};

pub use threshold_crypto::{PublicKey, SecretKey};

use std::collections::BTreeMap;
use unwrap::unwrap;
use uuid::Uuid;

const APP_NOT_CONNECTED: &str = "Application is not connected to the network";

// Temporary untill SCL allows to pass a SeqEntryActions to mutate_seq_mdata_entries
type SeqEntryActions = BTreeMap<Vec<u8>, MDataSeqEntryAction>;

#[derive(Default)]
pub struct SafeApp {
    safe_conn: Option<App>,
}

impl SafeApp {
    pub fn new() -> Self {
        Self { safe_conn: None }
    }

    #[cfg(feature = "fake-auth")]
    pub fn connect(&mut self, _app_id: &str, _auth_credentials: &str) -> Result<(), String> {
        warn!("Using fake authorisation for testing...");
        self.safe_conn = Some(create_app());
        Ok(())
    }

    // Connect to the SAFE Network using the provided app id and auth credentials
    #[cfg(not(feature = "fake-auth"))]
    pub fn connect(&mut self, app_id: &str, auth_credentials: &str) -> Result<(), String> {
        debug!("Connecting to SAFE Network...");

        let disconnect_cb = || {
            warn!("Connection with the SAFE Network was lost");
        };

        match decode_ipc_msg(auth_credentials) {
            Ok(auth_granted) => {
                match App::registered(app_id.to_string(), auth_granted, disconnect_cb) {
                    Ok(app) => {
                        self.safe_conn = Some(app);
                        debug!("Successfully connected to the Network!!!");
                        Ok(())
                    }
                    Err(e) => Err(format!("Failed to connect to the SAFE Network: {:?}", e)),
                }
            }
            Err(e) => Err(e),
        }
    }

    // TODO: replace with actual code for calling SCL
    pub fn create_balance(
        &mut self,
        _from_pk: &PublicKey,
        _from_sk: &SecretKey,
        _new_balance_owner: &PublicKey,
        _amount: &str,
    ) -> Result<XorName, &str> {
        //self.scl_mock
        //    .create_balance(from_pk, from_sk, new_balance_owner, amount)
        Ok(create_random_xorname())
    }

    // TODO: replace with code
    pub fn allocate_test_coins(&mut self, _to_pk: &PublicKey, _amount: &str) -> XorName {
        // self.scl_mock.allocate_test_coins(to_pk, amount)
        create_random_xorname()
    }

    pub fn get_balance_from_pk(&self, pk: &PublicKey, sk: &SecretKey) -> Result<String, &str> {
        let xorname = xorname_from_pk(pk);
        self.get_balance_from_xorname(&xorname, sk)
    }

    // TODO: replace with actual code
    // some exisits but: https://github.com/maidsafe/safe_client_libs/blob/experimental/safe_core/src/client/mod.rs#L299 is missing SK for arbitrary / anon coin balance
    pub fn get_balance_from_xorname(
        &self,
        _xorname: &XorName,
        _sk: &SecretKey,
    ) -> Result<String, &str> {
        //let safe_app: &App = self.safe_app.ok_or_else(|| APP_NOT_CONNECTED)?;
        //let safe_app: &App = match &self.safe_app {
        //    Some(app) => &app,
        //    None => return Err(APP_NOT_CONNECTED.to_string()),
        //};

        // TODO: Make this work with SCL.

        // let balance = unwrap!(run(safe_app, |client, _app_context| {
        //     let owner_wallet = XorName(sha3_256(&unwrap!(client.owner_key()).0));
        //
        //     client.get_balance(owner_wallet)
        // 		.map_err(|e| panic!("Failed to get balance: {:?}", e))
        //
        // 	// .then(move |res| {
        //     //     match res {
        //     //         Err(/*CoreError::NewRoutingClientError(Error::AccessDenied)*/ _) => {
        //     //             println!("No permissions to access owner's wallet");
        //     //             ()
        //     //         }
        //     //         res => panic!("Unexpected result: {:?}", res),
        //     //     }
        // 	//
        //     //     Ok::<_, AppError>(())
        //     // })
        // }));

        // Ok(balance.to_string())

        // self.scl_mock.get_balance_from_xorname(xorname, sk)
        Ok("0".to_string())
    }

    // TODO: replace with actual code for calling SCL
    pub fn fetch_pk_from_xorname(&self, _xorname: &XorName) -> Result<PublicKey, &str> {
        // self.scl_mock.fetch_pk_from_xorname(xorname)
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
    ) -> Result<Uuid, &str> {
        // self.scl_mock
        //    .safecoin_transfer(from_pk, from_sk, to_pk, tx_id, amount)
        Ok(Uuid::new_v4())
    }

    // TODO: Replace with SCL calling code
    #[allow(dead_code)]
    pub fn get_transaction(&self, _tx_id: &Uuid, _pk: &PublicKey, _sk: &SecretKey) -> String {
        // self.scl_mock.get_transaction(tx_id, pk, sk)
        "Success(0)".to_string()
    }

    pub fn files_put_published_immutable(&mut self, data: &[u8]) -> Result<XorName, String> {
        let safe_app: &App = match &self.safe_conn {
            Some(app) => &app,
            None => return Err(APP_NOT_CONNECTED.to_string()),
        };

        let the_idata = ImmutableData::new(data.to_vec());
        let return_idata = the_idata.clone();
        unwrap!(run(safe_app, move |client, _app_context| {
            client
                .put_idata(the_idata)
                .map_err(|e| panic!("Failed to PUT Published ImmutableData: {:?}", e))
        }));

        Ok(*return_idata.name())
    }

    pub fn files_get_published_immutable(&self, xorname: XorName) -> Result<Vec<u8>, String> {
        let safe_app: &App = match &self.safe_conn {
            Some(app) => &app,
            None => return Err(APP_NOT_CONNECTED.to_string()),
        };

        let data = unwrap!(run(safe_app, move |client, _app_context| {
            client
                .get_idata(xorname)
                .map_err(|e| panic!("Failed to GET Published ImmutableData: {:?}", e))
        }));

        Ok(data.value().to_vec())
    }

    pub fn put_seq_appendable_data(
        &mut self,
        the_data: Vec<(Vec<u8>, Vec<u8>)>,
        name: Option<XorName>,
        tag: u64,
        _permissions: Option<String>,
    ) -> Result<XorName, String> {
        let safe_app: &App = match &self.safe_conn {
            Some(app) => &app,
            None => return Err(APP_NOT_CONNECTED.to_string()),
        };

        let xorname = name.unwrap_or_else(create_random_xorname);

        unwrap!(run(safe_app, move |client, _app_context| {
            let appendable_data_address = ADataAddress::new_pub_seq(xorname, tag);
            let append_client = client.clone();

            let mut data = PubSeqAppendOnlyData::new(xorname, tag);

            // TODO: setup permissions from props
            let mut perms = BTreeMap::<ADataUser, ADataPubPermissionSet>::new();
            let set = ADataPubPermissionSet::new(true, true);
            let usr = ADataUser::Key(SafeNdPublicKey::Bls(unwrap!(client.public_bls_key())));
            let _ = perms.insert(usr, set);
            unwrap!(data.append_permissions(ADataPubPermissions {
                permissions: perms,
                data_index: 0,
                owner_entry_index: 0,
            }));

            let append = ADataAppend {
                address: appendable_data_address,
                values: the_data,
            };

            let owner = ADataOwner {
                public_key: SafeNdPublicKey::Bls(unwrap!(client.public_bls_key())),
                data_index: 0,
                permissions_index: 1,
            };
            unwrap!(data.append_owner(owner));

            client
                .put_adata(AData::PubSeq(data.clone()))
                .and_then(move |_| append_client.append_seq_adata(append, 0))
                .map_err(|e| panic!("Failed to PUT Sequenced Appendable Data: {:?}", e))
                .map(move |_| xorname)
        }));
        Ok(xorname)
    }

    pub fn append_seq_appendable_data(
        &mut self,
        the_data: Vec<(Vec<u8>, Vec<u8>)>,
        new_version: u64,
        xorname: XorName,
        tag: u64,
    ) -> Result<u64, String> {
        let safe_app: &App = match &self.safe_conn {
            Some(app) => &app,
            None => return Err(APP_NOT_CONNECTED.to_string()),
        };

        unwrap!(run(safe_app, move |client, _app_context| {
            let appendable_data_address = ADataAddress::new_pub_seq(xorname, tag);

            let append = ADataAppend {
                address: appendable_data_address,
                values: the_data,
            };

            client
                .append_seq_adata(append, new_version)
                .map_err(|e| panic!("Failed to UPDATE Sequenced Appendable Data: {:?}", e))
                .map(move |_| xorname)
        }));
        Ok(new_version)
    }

    pub fn get_latest_seq_appendable_data(
        &self,
        xorname: XorName,
        tag: u64,
    ) -> Result<(u64, (Vec<u8>, Vec<u8>)), &str> {
        let safe_app: &App = match &self.safe_conn {
            Some(app) => &app,
            None => return Err(APP_NOT_CONNECTED),
        };

        let appendable_data_address = ADataAddress::new_pub_seq(xorname, tag);

        let data_length = self
            .get_current_seq_appendable_data_version(xorname, tag)
            .unwrap();

        let data = unwrap!(run(safe_app, move |client, _app_context| {
            client
                .get_adata_last_entry(appendable_data_address)
                .map_err(|e| panic!("Failed to get Sequenced Appendable Data: {:?}", e))
        }));

        Ok((data_length, data))
    }

    pub fn get_current_seq_appendable_data_version(
        &self,
        name: XorName,
        tag: u64,
    ) -> Result<u64, &str> {
        debug!("Getting seq appendable data, length for: {:?}", name);

        let safe_app: &App = match &self.safe_conn {
            Some(app) => &app,
            None => return Err(APP_NOT_CONNECTED),
        };

        let appendable_data_address = ADataAddress::new_pub_seq(name, tag);

        let data_length = unwrap!(run(safe_app, move |client, _app_context| {
            client
                .get_adata_indices(appendable_data_address)
                .map_err(|e| panic!("Failed to get Sequenced Appendable Data indices: {:?}", e))
        }))
        .data_index();

        debug!("AD length is, \"{:?}\"", data_length);

        Ok(data_length)
    }

    #[allow(dead_code)]
    pub fn get_seq_appendable_data(
        &self,
        name: XorName,
        tag: u64,
        version: u64,
    ) -> Result<(Vec<u8>, Vec<u8>), String> {
        debug!(
            "Getting seq appendable data, version: {:?}, from: {:?}",
            version, name
        );

        let safe_app: &App = match &self.safe_conn {
            Some(app) => &app,
            None => return Err(APP_NOT_CONNECTED.to_string()),
        };
        let appendable_data_address = ADataAddress::new_pub_seq(name, tag);

        let data_length = self
            .get_current_seq_appendable_data_version(name, tag)
            .unwrap();

        let start = ADataIndex::FromStart(version);
        let end = ADataIndex::FromStart(version + 1);

        if version >= data_length {
            return Err(format!(
                "The version, \"{:?}\" of \"{:?}\" does not exist",
                version, name
            ));
        }

        if version == data_length {
            let (_version, data) = self.get_latest_seq_appendable_data(name, tag).unwrap();
            return Ok(data);
        }

        let data = unwrap!(run(safe_app, move |client, _app_context| {
            client
                .get_adata_range(appendable_data_address, (start, end))
                .map_err(|e| panic!("Failed to get Sequenced Appendable Data: {:?}", e))
        }));

        let this_version = data[0].clone();
        Ok(this_version)
    }

    pub fn put_seq_mutable_data(
        &self,
        name: Option<XorName>,
        tag: u64,
        // _data: Option<String>,
        _permissions: Option<String>,
    ) -> Result<XorName, String> {
        let safe_app: &App = match &self.safe_conn {
            Some(app) => &app,
            None => return Err(APP_NOT_CONNECTED.to_string()),
        };

        let xorname = unwrap!(run(safe_app, move |client, _app_context| {
            let owners = match client.owner_key() {
                Some(SafeNdPublicKey::Bls(pk)) => pk,
                _ => panic!("Couldn't get account's owner pk"), // FIXME: return error instead of panic
            };

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
                .map_err(|e| panic!("{:?}", e))
                .map(move |_| xorname)
        }));

        Ok(xorname)
    }

    // TODO: we shouldn't need to expose this function, function like list_seq_mdata_entries should be exposed
    #[allow(dead_code)]
    fn get_seq_mdata(&self, xorurl: &str, tag: u64) -> Result<SeqMutableData, String> {
        let safe_app: &App = match &self.safe_conn {
            Some(app) => &app,
            None => return Err(APP_NOT_CONNECTED.to_string()),
        };

        let xorname = XorUrlEncoder::from_url(xorurl)?.xorname();
        let md = unwrap!(run(safe_app, move |client, _app_context| {
            client
                .get_seq_mdata(xorname, tag)
                .map_err(|e| panic!("Failed to get MD: {:?}", e))
        }));
        Ok(md)
    }

    pub fn seq_mutable_data_insert(
        &self,
        xorurl: &str,
        tag: u64,
        key: Vec<u8>,
        value: &[u8],
    ) -> Result<(), String> {
        let mut entry_actions: SeqEntryActions = Default::default();
        let _ = entry_actions.insert(
            key.to_vec(),
            MDataSeqEntryAction::Ins(MDataValue::new(value.to_vec(), 0)),
        );

        self.mutate_seq_mdata_entries(xorurl, tag, entry_actions, "Failed to insert to MD")
    }

    // TODO: Replace with real scl calling code
    #[allow(dead_code)]
    pub fn mutable_data_delete(&mut self, _xorname: &XorName, _tag: u64, _key: &[u8]) {
        // self.scl_mock.mutable_data_delete(xorname, tag, key)
    }

    pub fn seq_mutable_data_get_value(
        &mut self,
        xorurl: &str,
        tag: u64,
        key: Vec<u8>,
    ) -> Result<MDataValue, String> {
        let safe_app: &App = match &self.safe_conn {
            Some(app) => &app,
            None => return Err(APP_NOT_CONNECTED.to_string()),
        };

        let xorname = XorUrlEncoder::from_url(xorurl)?.xorname();
        let data = unwrap!(run(safe_app, move |client, _app_context| {
            client
                .get_seq_mdata_value(xorname, tag, key.to_vec())
                .map_err(|e| panic!("Failed to retrieve key. {:?}", e)) // FIXME: return error instead of panic
        }));

        Ok(data)
    }

    pub fn list_seq_mdata_entries(
        &self,
        xorurl: &str,
        tag: u64,
    ) -> Result<BTreeMap<Vec<u8>, MDataValue>, &str> {
        let safe_app: &App = match &self.safe_conn {
            Some(app) => &app,
            None => return Err(APP_NOT_CONNECTED),
        };

        let xorname = XorUrlEncoder::from_url(xorurl)
            .map_err(|_| "InvalidXorUrl")?
            .xorname();
        let entries = unwrap!(run(safe_app, move |client, _app_context| {
            client
                .list_seq_mdata_entries(xorname, tag)
                .map_err(|e| panic!("Failed to get MD: {:?}", e)) // FIXME: return error instead of panic
        }));
        Ok(entries)
    }

    #[allow(dead_code)]
    pub fn seq_mutable_data_update(
        &self,
        xorurl: &str,
        tag: u64,
        key: &[u8],
        value: &[u8],
        version: u64,
    ) -> Result<(), String> {
        let mut entry_actions: SeqEntryActions = Default::default();
        let _ = entry_actions.insert(
            key.to_vec(),
            MDataSeqEntryAction::Update(MDataValue::new(value.to_vec(), version)),
        );

        self.mutate_seq_mdata_entries(xorurl, tag, entry_actions, "Failed to update MD")
    }

    // private helper method
    fn mutate_seq_mdata_entries(
        &self,
        xorurl: &str,
        tag: u64,
        entry_actions: SeqEntryActions,
        error_msg: &str,
    ) -> Result<(), String> {
        let safe_app: &App = match &self.safe_conn {
            Some(app) => &app,
            None => return Err(APP_NOT_CONNECTED.to_string()),
        };
        let xorname = XorUrlEncoder::from_url(xorurl)?.xorname();
        let message = error_msg.to_string();
        unwrap!(run(safe_app, move |client, _app_context| {
            client
                .mutate_seq_mdata_entries(xorname, tag, entry_actions)
                .map_err(move |err| panic!(format!("{}: {}", message, err))) // FIXME: return error instead of panic
        }));

        Ok(())
    }
}

#[test]
fn test_put_and_get_immutable_data() {
    use super::Safe;
    let mut safe = Safe::new("base32z".to_string());
    safe.connect("", "").unwrap();

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
fn test_put_get_update_seq_appendable_data() {
    use super::Safe;
    let mut safe = Safe::new("base32z".to_string());
    safe.connect("", "").unwrap();

    let key1 = b"KEY1".to_vec();
    let val1 = b"VALUE1".to_vec();
    let data1 = [(key1, val1)].to_vec();

    let TYPE = 12322;
    let xorname = safe
        .safe_app
        .put_seq_appendable_data(data1, None, TYPE, None)
        .unwrap();

    let (_this_version, data) = safe
        .safe_app
        .get_latest_seq_appendable_data(xorname, TYPE)
        .unwrap();

    //TODO: Properly unwrap data so this is clear (0 being version, 1 being data)
    assert_eq!(std::str::from_utf8(data.0.as_slice()).unwrap(), "KEY1");
    assert_eq!(std::str::from_utf8(data.1.as_slice()).unwrap(), "VALUE1");

    let key2 = b"KEY2".to_vec();
    let val2 = b"VALUE2".to_vec();
    let data2 = [(key2, val2)].to_vec();
    let new_version = 1;

    let updated_version = safe
        .safe_app
        .append_seq_appendable_data(data2, new_version, xorname, TYPE)
        .unwrap();
    let (_v_updated, data_updated) = safe
        .safe_app
        .get_latest_seq_appendable_data(xorname, TYPE)
        .unwrap();

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
        .get_seq_appendable_data(xorname, TYPE, first_version)
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
        .get_seq_appendable_data(xorname, TYPE, second_version)
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
        .get_seq_appendable_data(xorname, TYPE, nonexistant_version)
    {
        Ok(data) => panic!("No error thrown for a version that does not exist"),

        Err(err) => assert!(true),
    }
}
