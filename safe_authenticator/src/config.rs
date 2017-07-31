// Copyright 2017 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement.  This, along with the Licenses can be
// found in the root directory of this project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.
//
// Please review the Licences for the specific language governing permissions and limitations
// relating to use of the SAFE Network Software.

use super::{AuthError, AuthFuture};
use futures::Future;
use futures::future::{self, Either};
use maidsafe_utilities::serialisation::{deserialise, serialise};
use routing::{ClientError, EntryActions};
use safe_core::{Client, CoreError, FutureExt, recovery};
use safe_core::ipc::IpcError;
use safe_core::ipc::req::AppExchangeInfo;
use safe_core::ipc::resp::AppKeys;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::{HashMap, VecDeque};
use tiny_keccak::sha3_256;

// TODO: remove the `allow(unused)` attributes.

/// App data stored in the authenticator configuration.
///
/// We need to store it even for revoked apps because we need to
/// preserve the app keys. An app can encrypt data and create mutable data
/// instances on its own, so we need to make sure that the app can
/// access the encrypted data in future, even if the app was revoked
/// at some point.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppInfo {
    /// Application info (id, name, vendor, etc.)
    pub info: AppExchangeInfo,
    /// Application keys
    pub keys: AppKeys,
}

/// Config file key under which the list of registered apps is stored.
pub const KEY_APPS: &'static [u8] = b"apps";

/// Key under which the access container info is stored.
pub const KEY_ACCESS_CONTAINER: &'static [u8] = b"access-container";

/// Config file key under which the revocation queue is stored.
pub const KEY_REVOCATION_QUEUE: &'static [u8] = b"revocation-queue";

/// Maps from a SHA-3 hash of an app ID to app info
pub type Apps = HashMap<[u8; 32], AppInfo>;
/// Contains a queue of revocations that are currently running or have failed
/// String refers to `app_id`.
pub type RevocationQueue = VecDeque<String>;

/// Retrieves apps registered with the authenticator
pub fn list_apps(client: &Client<()>) -> Box<AuthFuture<(u64, Apps)>> {
    get_entry(client, KEY_APPS)
}

/// Retrieves an app info by the given app ID.
pub fn get_app(client: &Client<()>, app_id: &str) -> Box<AuthFuture<AppInfo>> {
    let app_id_hash = sha3_256(app_id.as_bytes());
    list_apps(client)
        .and_then(move |(_, config)| {
            config.get(&app_id_hash).cloned().ok_or_else(|| {
                AuthError::IpcError(IpcError::UnknownApp)
            })
        })
        .into_box()
}

/// Updates the list of apps registered with authenticator.
pub fn update_apps(client: &Client<()>, apps: &Apps, version: u64) -> Box<AuthFuture<()>> {
    update_entry(client, KEY_APPS, apps, version)
}

/// Register the given app with authenticator.
pub fn insert_app(client: &Client<()>, app: AppInfo) -> Box<AuthFuture<()>> {
    let c2 = client.clone();

    list_apps(client)
        .and_then(move |(version, mut apps)| {
            // Add app info to the authenticator config
            let hash = sha3_256(app.info.id.as_bytes());
            let _ = apps.insert(hash, app);
            update_apps(&c2, &apps, version + 1)
        })
        .into_box()
}


/// Push a new operation to the revocation queue.
pub fn push_to_revocation_queue(client: &Client<()>, app_id: String) -> Box<AuthFuture<()>> {
    let client = client.clone();
    get_revocation_queue(&client)
        .and_then(move |res| {
            let (version, mut queue) = res.map(|(version, queue)| (version + 1, queue))
                .unwrap_or_else(|| (0, Default::default()));
            queue.push_back(app_id);
            update_revocation_queue(&client, &queue, version)
        })
        .into_box()
}

/// Pop from the revocation queue.
/// Returns the removed entry & the revocation queue with the removed entry in a tuple.
/// If there are no entries left in the queue, returns just `None` along with the revocation queue.
/// If no revocation queue is found in the config, returns `None`
#[cfg_attr(feature = "cargo-clippy", allow(type_complexity))]
pub fn pop_from_revocation_queue(
    client: &Client<()>,
) -> Box<AuthFuture<Option<(Option<String>, RevocationQueue)>>> {
    let client = client.clone();
    get_revocation_queue(&client)
        .and_then(move |res| if let Some((version, mut queue)) = res {
            let op = queue.pop_front();
            Either::B(update_revocation_queue(&client, &queue, version + 1).map(
                move |_| Some((op, queue)),
            ))
        } else {
            return Either::A(future::ok(None));
        })
        .into_box()
}

/// Get authenticator's revocation queue.
/// Returns version and the revocation queue in a tuple.
/// If the queue is not found on the config file, returns `None`.
pub fn get_revocation_queue(
    client: &Client<()>,
) -> Box<AuthFuture<Option<(u64, RevocationQueue)>>> {
    get_entry(client, KEY_REVOCATION_QUEUE)
        .then(move |res| match res {
            Ok(value) => ok!(Some(value)),
            Err(AuthError::CoreError(CoreError::RoutingClientError(ClientError::NoSuchEntry))) => {
                ok!(None)
            }
            Err(error) => return err!(error),
        })
        .into_box()
}

/// Update authenticator's operation queue.
pub fn update_revocation_queue(
    client: &Client<()>,
    queue: &RevocationQueue,
    version: u64,
) -> Box<AuthFuture<()>> {
    update_entry(client, KEY_REVOCATION_QUEUE, queue, version)
}

fn get_entry<T>(client: &Client<()>, key: &[u8]) -> Box<AuthFuture<(u64, T)>>
where
    T: Default + DeserializeOwned + Serialize + 'static,
{
    let parent = fry!(client.config_root_dir());
    let key = fry!(parent.enc_entry_key(key));

    client
        .get_mdata_value(parent.name, parent.type_tag, key)
        .and_then(move |value| {
            let decoded = parent.decrypt(&value.content)?;
            let decoded = if !decoded.is_empty() {
                deserialise(&decoded)?
            } else {
                Default::default()
            };

            Ok((value.entry_version, decoded))
        })
        .map_err(From::from)
        .into_box()
}

fn update_entry<T>(
    client: &Client<()>,
    key: &[u8],
    content: &T,
    version: u64,
) -> Box<AuthFuture<()>>
where
    T: Serialize,
{
    let parent = fry!(client.config_root_dir());

    let key = fry!(parent.enc_entry_key(key));
    let encoded = fry!(serialise(content));
    let encoded = fry!(parent.enc_entry_value(&encoded));

    let actions = if version == 0 {
        EntryActions::new().ins(key, encoded, 0)
    } else {
        EntryActions::new().update(key, encoded, version)
    };

    recovery::mutate_mdata_entries(client, parent.name, parent.type_tag, actions.into())
        .map_err(From::from)
        .into_box()
}
