// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::types::{utils::calc_age, NodeId, PublicKey, RewardNodeId};
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
    pub reward_node_id: RewardNodeId,
    // Keep the secret key in Arc to allow Clone while also preventing multiple copies to exist in
    // memory which might be insecure.
    #[debug(skip)]
    pub keypair: Arc<Keypair>,
}

impl MyNodeInfo {
    pub fn new(keypair: Keypair, reward_node_id: RewardNodeId) -> Self {
        Self {
            reward_node_id,
            keypair: Arc::new(keypair),
        }
    }

    pub fn id(&self) -> NodeId {
        self.reward_node_id.node_id()
    }

    pub fn reward_node_id(&self) -> RewardNodeId {
        self.reward_node_id
    }

    pub fn addr(&self) -> SocketAddr {
        self.reward_node_id.addr()
    }

    pub fn name(&self) -> XorName {
        self.reward_node_id.name()
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
