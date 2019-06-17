// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use futures::future::Future;

#[cfg(not(feature = "fake-auth"))]
use crate::lib_helpers::decode_ipc_msg;
use crate::lib_helpers::{xorname_from_pk, xorurl_to_xorname};

use crate::scl_mock::{PublicKeyMock, SafeApp as SafeAppMock, SecretKeyMock};
use log::{debug, warn};
use rand::rngs::OsRng;
use rand_core::RngCore;
use safe_app::{run, App};

//#[cfg(feature = "fake-auth")]
use safe_app::test_utils::create_app;
use safe_core::client::Client;
use safe_nd::mutable_data::{Action, PermissionSet, SeqEntryAction, SeqMutableData, Value};
use safe_nd::{PublicKey, XorName};

use std::collections::BTreeMap;
use unwrap::unwrap;
use uuid::Uuid;

const APP_NOT_CONNECTED: &str = "Application is not connected to the network";

//Temporary untill SCL allows to pass a SeqEntryActions to mutate_seq_mdata_entries
type SeqEntryActions = BTreeMap<Vec<u8>, SeqEntryAction>;

pub struct SafeApp {
    safe_conn: Option<App>,
    scl_mock: SafeAppMock, // TODO: this is temporary until we don't rely on our scl-mock anymore
}

impl SafeApp {
    pub fn new() -> Self {
        Self {
            safe_conn: Some(create_app()), // TODO: initialise with None once we don't rely on our scl-mock anymore
            scl_mock: SafeAppMock::new(),
        }
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
        from_pk: &PublicKeyMock,
        from_sk: &SecretKeyMock,
        new_balance_owner: &PublicKeyMock,
        amount: &str,
    ) -> Result<XorName, &str> {
        self.scl_mock
            .create_balance(from_pk, from_sk, new_balance_owner, amount)
    }

    // TODO: replace with code
    pub fn allocate_test_coins(&mut self, to_pk: &PublicKeyMock, amount: &str) -> XorName {
        self.scl_mock.allocate_test_coins(to_pk, amount)
    }

    pub fn get_balance_from_pk(
        &self,
        pk: &PublicKeyMock,
        sk: &SecretKeyMock,
    ) -> Result<String, &str> {
        let xorname = xorname_from_pk(pk);
        self.get_balance_from_xorname(&xorname, sk)
    }

    // TODO: replace with actual code
    // some exisits but: https://github.com/maidsafe/safe_client_libs/blob/experimental/safe_core/src/client/mod.rs#L299 is missing SK for arbitrary / anon coin balance
    pub fn get_balance_from_xorname(
        &self,
        xorname: &XorName,
        sk: &SecretKeyMock,
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

        self.scl_mock.get_balance_from_xorname(xorname, sk)
    }

    // TODO: replace with actual code for calling SCL
    pub fn fetch_pk_from_xorname(&self, xorname: &XorName) -> Result<PublicKeyMock, &str> {
        self.scl_mock.fetch_pk_from_xorname(xorname)
    }

    // TODO: replace with actual code for calling SCL
    pub fn safecoin_transfer(
        &mut self,
        from_pk: &PublicKeyMock,
        from_sk: &SecretKeyMock,
        to_pk: &PublicKeyMock,
        tx_id: &Uuid,
        amount: &str,
    ) -> Result<Uuid, &str> {
        self.scl_mock
            .safecoin_transfer(from_pk, from_sk, to_pk, tx_id, amount)
    }

    //TODO: Replace with SCL calling code
    #[allow(dead_code)]
    pub fn get_transaction(&self, tx_id: &Uuid, pk: &PublicKeyMock, sk: &SecretKeyMock) -> String {
        self.scl_mock.get_transaction(tx_id, pk, sk)
    }

    //TODO: Replace with SCL calling code
    #[allow(dead_code)]
    pub fn unpublished_append_only_put(
        &mut self,
        pk: &PublicKeyMock,
        sk: &SecretKeyMock,
        data: &[u8],
    ) -> XorName {
        self.scl_mock.unpublished_append_only_put(pk, sk, data)
    }

    //TODO: Replace with SCL calling code
    #[allow(dead_code)]
    pub fn unpublished_append_only_get(
        &self,
        pk: &PublicKeyMock,
        sk: &SecretKeyMock,
        version: Option<usize>,
    ) -> Vec<u8> {
        self.scl_mock.unpublished_append_only_get(pk, sk, version)
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
                Some(PublicKey::Bls(pk)) => pk,
                _ => panic!("Couldn't get account's owner pk"), // FIXME: return error instead of panic
            };

            let xorname = name.unwrap_or_else(|| {
                let mut rng = unwrap!(OsRng::new());
                let mut xorname = XorName::default();
                rng.fill_bytes(&mut xorname.0);
                xorname
            });

            let permission_set = PermissionSet::new()
                .allow(Action::Read)
                .allow(Action::Insert)
                .allow(Action::Update)
                .allow(Action::Delete)
                .allow(Action::ManagePermissions);

            let mut permission_map = BTreeMap::new();
            let sign_pk = unwrap!(client.public_bls_key());
            let app_pk = PublicKey::Bls(sign_pk);
            permission_map.insert(app_pk, permission_set);

            let mdata = SeqMutableData::new_with_data(
                xorname,
                tag.clone(),
                BTreeMap::new(),
                permission_map,
                owners,
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

        let xorname = xorurl_to_xorname(xorurl)?;
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
            SeqEntryAction::Ins(Value::new(value.to_vec(), 0)),
        );

        self.mutate_seq_mdata_entries(xorurl, tag, entry_actions, "Failed to insert to MD")
    }

    // TODO: Replace with real scl calling code
    #[allow(dead_code)]
    pub fn mutable_data_delete(&mut self, xorname: &XorName, tag: u64, key: &[u8]) {
        self.scl_mock.mutable_data_delete(xorname, tag, key)
    }

    pub fn seq_mutable_data_get_value(
        &mut self,
        xorurl: &str,
        tag: u64,
        key: Vec<u8>,
    ) -> Result<Value, String> {
        let safe_app: &App = match &self.safe_conn {
            Some(app) => &app,
            None => return Err(APP_NOT_CONNECTED.to_string()),
        };

        let xorname = xorurl_to_xorname(xorurl)?;
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
    ) -> Result<BTreeMap<Vec<u8>, Value>, String> {
        let safe_app: &App = match &self.safe_conn {
            Some(app) => &app,
            None => return Err(APP_NOT_CONNECTED.to_string()),
        };

        let xorname = xorurl_to_xorname(xorurl)?;
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
            SeqEntryAction::Update(Value::new(value.to_vec(), version)),
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
        let xorname = xorurl_to_xorname(xorurl)?;
        let message = error_msg.to_string();
        unwrap!(run(safe_app, move |client, _app_context| {
            client
                .mutate_seq_mdata_entries(xorname, tag, entry_actions)
                .map_err(move |err| panic!(format!("{}: {}", message, err))) // FIXME: return error instead of panic
        }));

        Ok(())
    }
}
