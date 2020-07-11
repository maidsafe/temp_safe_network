// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::utils;
use safe_nd::{Signature, NodeFullId};
use serde::Serialize;
use std::{
    cell::RefCell,
    rc::Rc,
};

#[derive(Clone)]
pub(crate) struct NodeKeys {
    id: NodePublicId,
    keys: Rc<RefCell<NodeFullId>>,
}

impl NodeKeys {
    pub fn new(keys: Rc<RefCell<NodeFullId>>) -> Self {
        let id = keys.borrow().public_id().clone();
        Self {
            id,
            keys
        }
    }

    pub fn public_key(&self) -> PublicKey {
        if let Some(key) = self.id.public_id().bls_public_key() {
            PublicKey::Bls(key)
        } else {
            PublicKey::Ed25519(self.id.public_id().ed25519_public_key())
        }
    }

    pub fn sign<T: Serialize>(&self, data: &T) -> Signature {
        let data = utils::serialise(data);
        let bls = self.keys.borrow().sign_using_bls(&data);
        if let Some(signature) = bls {
            signature
        } else {
            self.keys.borrow().sign_using_ed25519(&data)
        }
    }
}
