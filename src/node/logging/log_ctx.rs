// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::network::Network;
use xor_name::Prefix;

#[derive(Debug)]
pub struct LogCtx {
    network: Network,
}

impl LogCtx {
    pub fn new(network: Network) -> Self {
        Self { network }
    }

    pub async fn prefix(&self) -> Prefix {
        self.network.our_prefix().await
    }
}
