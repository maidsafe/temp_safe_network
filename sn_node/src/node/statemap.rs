// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! Instrumentation for statemaps: <https://github.com/TritonDataCenter/statemap>

use super::core::MyNode;
use sn_interface::statemap::State;

impl MyNode {
    pub(crate) fn statemap_log_metadata(&self) {
        #[cfg(feature = "statemap")]
        sn_interface::statemap::log_metadata()
    }

    pub(crate) fn statemap_log_state(&self, #[allow(unused)] state: State) {
        #[cfg(feature = "statemap")]
        sn_interface::statemap::log_state(self.name().to_string(), state)
    }
}
