// Copyright 2016 MaidSafe.net limited.
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

use super::AccessContainerEntry;
use Authenticator;
use access_container::access_container_entry;
use app_auth;
use errors::AuthError;
use futures::{Future, IntoFuture};
use futures::future;
use ipc::decode_ipc_msg;
use routing::User;
use rust_sodium::crypto::sign;
use safe_core::{Client, CoreError, FutureExt, utils};
#[cfg(feature = "use-mock-routing")]
use safe_core::MockRouting;
use safe_core::ipc::{self, AppExchangeInfo, AuthGranted, AuthReq, IpcMsg, IpcReq, Permission};
use safe_core::ipc::req::ffi::convert_permission_set;
use std::collections::{BTreeSet, HashMap};
use std::sync::mpsc;

/// Creates a new random account for authenticator
pub fn create_authenticator() -> Authenticator {
    let locator = unwrap!(utils::generate_random_string(10));
    let password = unwrap!(utils::generate_random_string(10));
    let invitation = unwrap!(utils::generate_random_string(10));

    unwrap!(Authenticator::create_acc(
        locator,
        password,
        invitation,
        |_| (),
    ))
}

/// Create a random authenticator and login using the same credentials.
pub fn create_account_and_login() -> Authenticator {
    let locator = unwrap!(utils::generate_random_string(10));
    let password = unwrap!(utils::generate_random_string(10));
    let invitation = unwrap!(utils::generate_random_string(10));

    let _ = unwrap!(Authenticator::create_acc(
        locator.clone(),
        password.clone(),
        invitation,
        |_| (),
    ));
    unwrap!(Authenticator::login(locator, password, |_| ()))
}

/// Create a random authenticator and login using the same credentials.
/// Attaches a hook to the Routing to override responses.
#[cfg(all(any(test, feature = "testing"), feature = "use-mock-routing"))]
pub fn create_account_and_login_with_hook<F>(hook: F) -> Authenticator
where
    F: Fn(MockRouting) -> MockRouting + Send + 'static,
{
    let locator = unwrap!(utils::generate_random_string(10));
    let password = unwrap!(utils::generate_random_string(10));
    let invitation = unwrap!(utils::generate_random_string(10));

    let _ = unwrap!(Authenticator::create_acc(
        locator.clone(),
        password.clone(),
        invitation,
        |_| (),
    ));
    unwrap!(Authenticator::login_with_hook(
        locator,
        password,
        |_| (),
        hook,
    ))
}

/// Registers a mock application using a given `AuthReq`.
pub fn register_app(
    authenticator: &Authenticator,
    auth_req: &AuthReq,
) -> Result<AuthGranted, AuthError> {
    let req_id = ipc::gen_req_id();
    let msg = IpcMsg::Req {
        req_id: req_id,
        req: IpcReq::Auth(auth_req.clone()),
    };

    // Invoke `decode_ipc_msg` and expect to get AuthReq back.
    let ipc_req = run(authenticator, move |client| decode_ipc_msg(client, msg));
    match ipc_req {
        Ok(IpcMsg::Req { req: IpcReq::Auth(_), .. }) => (),
        x => return Err(AuthError::Unexpected(format!("Unexpected {:?}", x))),
    }

    let auth_req = auth_req.clone();
    try_run(authenticator, move |client| {
        app_auth::authenticate(client, auth_req)
    })
}

/// Run the given closure inside the event loop of the authenticator. The closure
/// should return a future which will then be driven to completion and its result
/// returned.
pub fn try_run<F, I, T>(authenticator: &Authenticator, f: F) -> Result<T, AuthError>
where
    F: FnOnce(&Client<()>) -> I + Send + 'static,
    I: IntoFuture<Item = T, Error = AuthError> + 'static,
    T: Send + 'static,
{
    let (tx, rx) = mpsc::channel();

    unwrap!(authenticator.send(move |client| {
        let future = f(client)
            .into_future()
            .then(move |result| {
                unwrap!(tx.send(result));
                Ok(())
            })
            .into_box();

        Some(future)
    }));

    unwrap!(rx.recv())
}

/// Like `try_run`, but expects success.
pub fn run<F, I, T>(authenticator: &Authenticator, f: F) -> T
where
    F: FnOnce(&Client<()>) -> I + Send + 'static,
    I: IntoFuture<Item = T, Error = AuthError> + 'static,
    T: Send + 'static,
{
    unwrap!(try_run(authenticator, f))
}

/// Creates a random `AppExchangeInfo`
pub fn rand_app() -> Result<AppExchangeInfo, CoreError> {
    Ok(AppExchangeInfo {
        id: utils::generate_random_string(10)?,
        scope: None,
        name: utils::generate_random_string(10)?,
        vendor: utils::generate_random_string(10)?,
    })
}

/// Fetch the access container entry for the app.
pub fn access_container<S: Into<String>>(
    authenticator: &Authenticator,
    app_id: S,
    auth_granted: AuthGranted,
) -> AccessContainerEntry {
    unwrap!(
        try_access_container(authenticator, app_id, auth_granted),
        "Access container entry is empty"
    )
}

/// Fetch the access container entry for the app.
pub fn try_access_container<S: Into<String>>(
    authenticator: &Authenticator,
    app_id: S,
    auth_granted: AuthGranted,
) -> Option<AccessContainerEntry> {
    let app_keys = auth_granted.app_keys;
    let ac_md_info = auth_granted.access_container.into_mdata_info(
        app_keys.enc_key.clone(),
    );
    let app_id = app_id.into();
    run(authenticator, move |client| {
        access_container_entry(client, &ac_md_info, &app_id, app_keys).map(move |(_, entry)| entry)
    })
}

/// Check that the given permission set is contained in the access container
pub fn compare_access_container_entries(
    authenticator: &Authenticator,
    app_sign_pk: sign::PublicKey,
    mut access_container: AccessContainerEntry,
    expected: HashMap<String, BTreeSet<Permission>>,
) {
    let results = run(authenticator, move |client| {
        let mut reqs = Vec::new();
        let user = User::Key(app_sign_pk);

        for (container, expected_perms) in expected {
            // Check the requested permissions in the access container.
            let expected_perm_set = convert_permission_set(&expected_perms);
            let (md_info, perms) = unwrap!(
                access_container.remove(&container),
                "No '{}' in access container {:?}",
                container,
                access_container
            );
            assert_eq!(perms, expected_perms);

            let fut = client
                .list_mdata_user_permissions(md_info.name, md_info.type_tag, user)
                .map(move |perms| (perms, expected_perm_set));

            reqs.push(fut);
        }

        future::join_all(reqs).map_err(AuthError::from)
    });

    // Check the permission on the the mutable data for each of the above directories.
    for (perms, expected) in results {
        assert_eq!(perms, expected);
    }
}
