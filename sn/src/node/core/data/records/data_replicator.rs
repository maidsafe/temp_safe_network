// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::types::{DataAddress, ReplicatedData};
use std::collections::HashMap;
use xor_name::XorName;

/// Keeps track of all the data republishing tasks.
pub(crate) struct DataReplicator {
    // Keeps track of data that need to be provided to other replicas
    transmitter: Vec<HashMap<XorName, Vec<ReplicatedData>>>,
    // Keeps track of data that need to be pulled from other replicas
    receiver: HashMap<DataAddress, Vec<XorName>>,
}

impl DataReplicator {
    pub(crate) fn new() -> Self {
        DataReplicator {
            transmitter: Vec::new(),
            receiver: HashMap::new(),
        }
    }
}
