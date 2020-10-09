// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

#![allow(unsafe_code)]

mod auth;

pub use self::auth::AuthReq;

use serde::{Deserialize, Serialize};


/// IPC request.
// TODO: `TransOwnership` variant
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum IpcReq {
    /// Authentication request.
    Auth(AuthReq),
    // /// Unregistered client authenticator request.
    /// Takes arbitrary user data as `Vec<u8>`, returns bootstrap config.
    Unregistered(Vec<u8>),
}
