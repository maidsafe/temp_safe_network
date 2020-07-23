// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::utils;
use safe_nd::{
    BlsProof, BlsProofShare, Ed25519Proof, NodeKeypairs, NodePublicId, Proof, PublicKey, Signature,
};
use serde::Serialize;
use std::{cell::RefCell, rc::Rc};

#[derive(Clone)]
pub struct NodeKeys {
    keys: Rc<RefCell<NodeKeypairs>>,
}

impl NodeKeys {
    pub fn new(keys: Rc<RefCell<NodeKeypairs>>) -> Self {
        Self { keys }
    }

    pub fn public_id(&self) -> NodePublicId {
        self.keys.borrow().public_id().clone()
    }

    pub fn public_key(&self) -> PublicKey {
        self.keys.borrow().public_key()
    }

    pub fn sign<T: Serialize>(&self, data: &T) -> Signature {
        self.keys.borrow().sign(&utils::serialise(data))
    }

    pub fn produce_proof<T: Serialize>(&self, data: &T) -> Proof {
        match self.sign(data) {
            Signature::BlsShare(share) => Proof::BlsShare(BlsProofShare {
                index: share.index,
                signature_share: share.share,
                public_key_set: match self.keys.borrow().public_key_set() {
                    Some(key_set) => key_set.clone(),
                    None => unreachable!(), // this is admittedly not very elegant code..
                },
            }),
            Signature::Ed25519(signature) => Proof::Ed25519(Ed25519Proof {
                public_key: match self.public_key() {
                    PublicKey::Ed25519(key) => key,
                    _ => unreachable!(), // this is admittedly not very elegant code..
                },
                signature,
            }),
            Signature::Bls(signature) => Proof::Bls(BlsProof {
                public_key: match self.public_key() {
                    PublicKey::Bls(key) => key,
                    _ => unreachable!(), // this is admittedly not very elegant code..
                },
                signature,
            }),
        }
    }
}
