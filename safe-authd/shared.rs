// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use futures::lock::Mutex;
use sn_api::{AuthReq, SafeAuthenticator};
use std::{collections::BTreeMap, sync::Arc, time::SystemTime};
use tokio::sync::mpsc;

#[derive(Clone, Debug)]
pub struct IncomingAuthReq {
    pub timestamp: SystemTime,
    pub auth_req: AuthReq,
    pub tx: mpsc::Sender<bool>,
    pub notified: bool,
}

// List of authorisation requests indexed by their request id
pub type AuthReqsList = BTreeMap<u32, IncomingAuthReq>;

// A thread-safe queue to keep the list of authorisation requests
pub type SharedAuthReqsHandle = Arc<Mutex<AuthReqsList>>;

// A thread-safe handle to keep the SafeAuthenticator instance
pub type SharedSafeAuthenticatorHandle = Arc<Mutex<SafeAuthenticator>>;

// A thread-safe handle to keep the list of notifications subscribers' endpoints,
// we also keep the certificates' base path which is needed to create the communication channel
pub type SharedNotifEndpointsHandle = Arc<Mutex<BTreeMap<String, String>>>;
