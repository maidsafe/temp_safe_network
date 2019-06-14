// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use futures::future::Future;

use crate::lib_helpers::{decode_ipc_msg, xorurl_to_xorname2};
use crate::scl_mock::{PublicKeyMock, SafeApp as SafeAppMock, SecretKeyMock};
use log::{debug, info, warn};
use rand::{OsRng, Rng};
use rand_core::RngCore;
use safe_app::{run, App};

#[cfg(feature = "fake-auth")]
use safe_app::AppError;
//#[cfg(feature = "fake-auth")]
use safe_app::test_utils::create_app;
use safe_core::client::Client;
use safe_nd::mutable_data::{
    Action, MutableData, PermissionSet, SeqEntryAction, SeqMutableData, Value,
};
use safe_nd::{Error, PublicKey, XorName};

use std::collections::BTreeMap;
use threshold_crypto::SecretKey;
use unwrap::unwrap;
use uuid::Uuid;

// Type tag used for the Wallet container
static WALLET_TYPE_TAG: u64 = 10_000;

const APP_NOT_CONNECTED: &str = "Application is not connected to the network";

fn from_slice(bytes: &[u8]) -> [u8; 32] {
    let mut array = [0; 32];
    let bytes = &bytes[..array.len()]; // panics if not enough data
    array.copy_from_slice(bytes);
    array
}

pub struct SafeApp {
    safe_conn: Option<App>,
    scl_mock: SafeAppMock,
}

impl SafeApp {
    pub fn new() -> Self {
        Self {
            safe_conn: if cfg!(test) { Some(create_app()) } else { None },
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

    pub fn put_seq_mutable_data(
        &self,
        name: Option<XorName>,
        tag: Option<u64>,
        // _data: Option<String>,
        _permissions: Option<String>,
    ) -> Result<(), String> {
        let safe_app: &App = match &self.safe_conn {
            Some(app) => &app,
            None => return Err(APP_NOT_CONNECTED.to_string()),
        };

        unwrap!(run(safe_app, |client, _app_context| {
            let owners = match client.owner_key() {
                Some(PublicKey::Bls(pk)) => pk,
                other => panic!("Couldn't get account's owner pk"),
            };

            let mut rng = unwrap!(OsRng::new());
            //let name: XorName = rng.gen();
            let mut random_bytes = [0u8; 32];
            rng.fill_bytes(&mut random_bytes);
            let xorname = XorName(random_bytes);

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
                WALLET_TYPE_TAG,
                BTreeMap::new(),
                permission_map,
                owners,
            );
            client
                .put_seq_mutable_data(mdata)
                .map_err(|e| panic!("{:?}", e))
                .map(move |_| xorname)
        }));

        Ok(())
    }

    // TODO: we shouldn't need to expose this function, function like list_seq_mdata_entries should be exposed
    #[allow(dead_code)]
    fn get_seq_mdata(&self, xorurl: &str, type_tag: u64) -> Result<SeqMutableData, String> {
        let safe_app: &App = match &self.safe_conn {
            Some(app) => &app,
            None => return Err(APP_NOT_CONNECTED.to_string()),
        };

        let xorurl_string: String = xorurl.to_string();
        let md = unwrap!(run(safe_app, move |client, _app_context| {
            let xorname = unwrap!(xorurl_to_xorname2(&xorurl_string));
            client
                .get_seq_mdata(XorName(from_slice(&xorname)), type_tag)
                .map_err(|e| panic!("Failed to get MD: {:?}", e))
        }));
        Ok(md)
    }

    pub fn list_seq_mdata_entries(
        &self,
        xorurl: &str,
        type_tag: u64,
    ) -> Result<BTreeMap<Vec<u8>, Value>, String> {
        let xorurl_string: String = xorurl.to_string();
        let safe_app: &App = match &self.safe_conn {
            Some(app) => &app,
            None => return Err(APP_NOT_CONNECTED.to_string()),
        };

        let entries = unwrap!(run(safe_app, move |client, _app_context| {
            let xorname = unwrap!(xorurl_to_xorname2(&xorurl_string));
            client
                .list_seq_mdata_entries(XorName(from_slice(&xorname)), type_tag)
                .map_err(|e| panic!("Failed to get MD: {:?}", e))
        }));
        Ok(entries)
    }

    pub fn seq_mutable_data_insert(
        &self,
        safe_app: &App,
        xorurl: &str,
        type_tag: u64,
        key: &[u8],
        value: &[u8],
    ) -> Result<(), String> {
        let mut entry_actions: BTreeMap<Vec<u8>, SeqEntryAction> = Default::default();
        let _ = entry_actions.insert(
            key.to_vec(),
            SeqEntryAction::Ins(Value::new(value.to_vec(), 0)),
        );

        let xorurl_string: String = xorurl.to_string();
        unwrap!(run(safe_app, move |client, _app_context| {
            let xorname = unwrap!(xorurl_to_xorname2(&xorurl_string));

            client
                .mutate_seq_mdata_entries(
                    XorName(from_slice(&xorname)),
                    type_tag,
                    entry_actions.clone(),
                )
                .map_err(|e| panic!("Failed to insert to MD: {:?}", e))
        }));

        Ok(())
    }

    pub fn seq_mutable_data_update(
        safe_app: &App,
        xorurl: &str,
        type_tag: u64,
        key: &[u8],
        value: &[u8],
        version: u64,
    ) -> Result<(), String> {
        let mut entry_actions: BTreeMap<Vec<u8>, SeqEntryAction> = Default::default();
        let _ = entry_actions.insert(
            key.to_vec(),
            SeqEntryAction::Update(Value::new(value.to_vec(), version)),
        );

        let xorurl_string: String = xorurl.to_string();
        unwrap!(run(safe_app, move |client, _app_context| {
            let xorname = unwrap!(xorurl_to_xorname2(&xorurl_string));

            client
                .mutate_seq_mdata_entries(
                    XorName(from_slice(&xorname)),
                    type_tag,
                    entry_actions.clone(),
                )
                .map_err(|e| panic!("Failed to update MD: {:?}", e))
        }));

        Ok(())
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

    // TODO: replace with actual code for calling SCL
    pub fn keys_fetch_pk(&self, xorname: &XorName) -> Result<PublicKeyMock, &str> {
        self.scl_mock.keys_fetch_pk(xorname)
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
}
