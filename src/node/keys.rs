// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::utils;
use safe_nd::{Keypair, PublicKey, Signature};
use serde::Serialize;
use std::{cell::RefCell, rc::Rc};

#[derive(Clone)]
pub struct NodeKeys {
    keys: Rc<RefCell<Keypair>>,
}

impl NodeKeys {
    pub fn new(keys: Rc<RefCell<Keypair>>) -> Self {
        Self { keys }
    }

    pub fn public_key(&self) -> PublicKey {
        self.keys.borrow().public_key()
    }

    pub fn sign<T: Serialize>(&self, data: &T) -> Signature {
        self.keys.borrow().sign(&utils::serialise(data))
    }
}
