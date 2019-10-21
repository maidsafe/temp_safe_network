// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use safe_api::{AuthReq, SafeAuthenticator};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

#[derive(Clone, Debug)]
pub struct IncomingAuthReq {
    pub auth_req: AuthReq,
    pub tx: mpsc::Sender<bool>,
}

// List of authorisation requests indexed by their request id
pub type AuthReqsList = BTreeMap<u32, IncomingAuthReq>;

// A thread-safe queue to keep the list of authorisation requests
pub type SharedAuthReqsHandle = Arc<Mutex<AuthReqsList>>;

// A thread-safe handle to keep the SafeAuthenticator instance
pub type SharedSafeAuthenticatorHandle = Arc<Mutex<SafeAuthenticator>>;

// A thread-safe handle to keep the list of notifications subscriptors' endpoints
pub type SharedNotifEndpointsHandle = Arc<Mutex<BTreeSet<String>>>;

pub fn lock_safe_authenticator<F, R>(
    safe_auth_handle: SharedSafeAuthenticatorHandle,
    mut f: F,
) -> Result<R, String>
where
    F: FnMut(&mut SafeAuthenticator) -> Result<R, String>,
{
    match safe_auth_handle.lock() {
        Err(err) => Err(format!(
            "Unexpectedly failed to obtain lock of the authenticator lib instance: {}",
            err
        )),
        Ok(mut locked_auth) => {
            let safe_authenticator: &mut SafeAuthenticator = &mut *(locked_auth);
            f(safe_authenticator)
        }
    }
}

pub fn lock_auth_reqs_list<F, R>(
    auth_reqs_handle: SharedAuthReqsHandle,
    mut f: F,
) -> Result<R, String>
where
    F: FnMut(&mut AuthReqsList) -> Result<R, String>,
{
    match auth_reqs_handle.lock() {
        Err(err) => Err(format!(
            "Unexpectedly failed to obtain lock of pending auth reqs list: {}",
            err
        )),
        Ok(mut locked_list) => {
            let auth_reqs_list: &mut AuthReqsList = &mut *(locked_list);
            f(auth_reqs_list)
        }
    }
}

pub fn lock_notif_endpoints_list<F, R>(
    notif_endpoints_handle: SharedNotifEndpointsHandle,
    mut f: F,
) -> Result<R, String>
where
    F: FnMut(&mut BTreeSet<String>) -> Result<R, String>,
{
    match notif_endpoints_handle.lock() {
        Err(err) => Err(format!(
            "Unexpectedly failed to obtain lock of list of notif subscriptors: {}",
            err
        )),
        Ok(mut locked_list) => {
            let notif_endpoints_list: &mut BTreeSet<String> = &mut *(locked_list);
            f(notif_endpoints_list)
        }
    }
}
