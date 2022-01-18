// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::routing::NetworkConfig;

use ed25519_dalek::Keypair;
use std::{
    collections::BTreeSet,
    net::{Ipv4Addr, SocketAddr},
};

/// Routing configuration.
#[derive(Debug)]
pub struct Config {
    /// If true, configures the node to start a new network
    /// instead of joining an existing one.
    pub first: bool,
    /// The `Keypair` of the node or `None` for randomly generated one.
    pub keypair: Option<Keypair>,
    /// The local address to bind to.
    pub local_addr: SocketAddr,
    /// Initial network contacts.
    pub bootstrap_nodes: BTreeSet<SocketAddr>,
    /// Network's genesis key if joining an existing network.
    pub genesis_key: Option<String>,
    /// Configuration for the underlying network transport.
    pub network_config: NetworkConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            first: false,
            keypair: None,
            local_addr: SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)),
            bootstrap_nodes: BTreeSet::new(),
            genesis_key: None,
            network_config: NetworkConfig::default(),
        }
    }
}
