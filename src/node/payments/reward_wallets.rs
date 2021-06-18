// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::types::{NodeAge, PublicKey};
use dashmap::DashMap;
use log::debug;
use std::collections::BTreeMap;
use xor_name::{Prefix, XorName};

/// The accumulation and paying
/// out of rewards to nodes for
/// their work in the network.
#[derive(Clone)]
pub struct RewardWallets {
    node_wallets: DashMap<XorName, (NodeAge, PublicKey)>,
}

impl RewardWallets {
    pub fn new(node_wallets: BTreeMap<XorName, (NodeAge, PublicKey)>) -> Self {
        Self {
            node_wallets: node_wallets.into_iter().collect(),
        }
    }

    /// Returns the stage of a specific node.
    pub fn get(&self, node_name: &XorName) -> Option<(NodeAge, PublicKey)> {
        Some(*self.node_wallets.get(node_name)?)
    }

    // /// Returns the name and age of the node for a specific wallet key.
    // pub fn get_by_wallet(&self, wallet: PublicKey) -> Option<(XorName, NodeAge)> {
    //     for item in &self.node_wallets {
    //         let name = item.key();
    //         let (age, key) = *item;
    //         if key == wallet {
    //             return Some((*name, age));
    //         }
    //     }
    //     None
    // }

    /// Returns the node ids of all nodes.
    #[allow(unused)]
    pub fn all_nodes(&self) -> Vec<XorName> {
        self.node_wallets.iter().map(|r| *r.key()).collect()
    }

    ///
    pub fn node_wallets(&self) -> BTreeMap<XorName, (NodeAge, PublicKey)> {
        self.node_wallets
            .clone()
            .into_read_only()
            .iter()
            .map(|(node, (age, key))| (*node, (*age, *key)))
            .collect()
    }

    /// Removes a subset of the nodes,
    /// more specifically those no longer
    /// part of this section, after a split.
    pub fn keep_wallets_of(&self, prefix: Prefix) {
        // Removes keys that are no longer our section responsibility.
        let keys = self
            .node_wallets
            .iter()
            .map(|info| *info.key())
            .collect::<Vec<_>>();

        for key in keys {
            if !prefix.matches(&key) {
                if let Some((name, _)) = self.node_wallets.remove(&key) {
                    debug!("Removed node {} from rewards list.", name);
                }
            }
        }
    }

    /// A new node registers a wallet id for future reward payout.
    /// ... or, an active node updates its wallet.
    pub fn set_node_wallet(&self, node_name: XorName, age: NodeAge, wallet: PublicKey) {
        let _ = self.node_wallets.insert(node_name, (age, wallet));
    }

    /// When the section becomes aware that a node has left,
    /// its reward key is removed.
    pub fn remove_wallet(&self, node_name: XorName) {
        let _ = self.node_wallets.remove(&node_name);
    }
}
