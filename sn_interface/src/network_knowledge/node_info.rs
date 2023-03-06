// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::types::{utils::calc_age, NodeId, PublicKey};
use ed25519_dalek::Keypair;
use std::{
    fmt::{self, Display, Formatter},
    net::SocketAddr,
    sync::Arc,
};
use xor_name::XorName;

/// Information and state of our node
#[derive(Clone, custom_debug::Debug)]
pub struct MyNodeInfo {
    // Keep the secret key in Arc to allow Clone while also preventing multiple copies to exist in
    // memory which might be insecure.
    #[debug(skip)]
    pub keypair: Arc<Keypair>,
    pub addr: SocketAddr,
}

impl MyNodeInfo {
    pub fn new(keypair: Keypair, addr: SocketAddr) -> Self {
        Self {
            keypair: Arc::new(keypair),
            addr,
        }
    }

    pub fn id(&self) -> NodeId {
        NodeId::new(self.name(), self.addr)
    }

    pub fn name(&self) -> XorName {
        XorName::from(PublicKey::from(self.keypair.public))
    }

    pub fn public_key(&self) -> PublicKey {
        PublicKey::from(self.keypair.public)
    }

    // Last byte of the name represents the age.
    pub fn age(&self) -> u8 {
        calc_age(&self.name())
    }
}

impl Display for MyNodeInfo {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
