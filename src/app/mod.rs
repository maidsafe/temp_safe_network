// Copyright 2016 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net
// Commercial License, version 1.0 or later, or (2) The General Public License
// (GPL), version 3, depending on which licence you accepted on initial access
// to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project
// generally, you agree to be bound by the terms of the MaidSafe Contributor
// Agreement, version 1.0.
// This, along with the Licenses can be found in the root directory of this
// project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network
// Software distributed under the GPL Licence is distributed on an "AS IS"
// BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied.
//
// Please review the Licences for the specific language governing permissions
// and limitations relating to use of the SAFE Network Software.

pub mod low_level_api;

mod errors;
mod object_cache;
#[cfg(test)]
mod test_util;

use core::{Client, CoreMsg, CoreMsgTx};
use futures::Future;
use maidsafe_utilities::thread::Joiner;
use self::errors::AppError;
use self::object_cache::ObjectCache;
use std::sync::Mutex;

/// Handle to an application instance.
pub struct App {
    core_tx: Mutex<CoreMsgTx<ObjectCache>>,
    _core_joiner: Joiner,
}

impl App {
    /// Send a message to app's event loop
    pub fn send<F>(&self, f: F) -> Result<(), AppError>
        where F: FnOnce(&Client, &ObjectCache) -> Option<Box<Future<Item=(), Error=()>>>
                 + Send + 'static
    {
        let msg = CoreMsg::new(f);
        let mut core_tx = unwrap!(self.core_tx.lock());
        core_tx.send(msg).map_err(AppError::from)
    }
}
