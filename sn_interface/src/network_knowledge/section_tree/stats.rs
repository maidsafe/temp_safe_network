// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::types::log_markers::LogMarker;

use std::fmt::{self, Display};

pub struct NetworkStats {
    pub(super) known_elders: u64,
    pub(super) total_elders: u64,
    pub(super) total_elders_exact: bool,
}

impl Display for NetworkStats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.total_elders_exact {
            write!(
                f,
                "*** Exact {}: {} ***",
                LogMarker::TotalNetworkElders,
                self.known_elders
            )
        } else {
            write!(
                f,
                "*** Known network elders: {}, Estimated total network elders: {} ***",
                self.known_elders, self.total_elders
            )
        }
    }
}
