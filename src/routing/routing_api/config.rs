// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::routing::TransportConfig;
use ed25519_dalek::Keypair;

/// Routing configuration.
#[derive(Debug)]
pub struct Config {
    /// If true, configures the node to start a new network
    /// instead of joining an existing one.
    pub first: bool,
    /// The `Keypair` of the node or `None` for randomly generated one.
    pub keypair: Option<Keypair>,
    /// Configuration for the underlying network transport.
    pub transport_config: TransportConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            first: false,
            keypair: None,
            transport_config: TransportConfig::default(),
        }
    }
}
