// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::types::{ReplicatedData, ReplicatedDataAddress};
use itertools::Itertools;
use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Index;
use tracing_subscriber::registry::Data;
use xor_name::XorName;

/// Keeps track of all the data replication tasks.
pub(crate) struct DataReplicator {
    // Nodes and the replicas we need to provide them
    to_be_transmitted: BTreeMap<XorName, BTreeMap<ReplicatedDataAddress, ReplicatedData>>, // Target -> Set<Data> mapping
}

impl DataReplicator {
    pub(crate) fn new() -> Self {
        DataReplicator {
            to_be_transmitted: BTreeMap::new(),
        }
    }

    // Keep track of nodes and the replicas we need to provide them
    pub(crate) fn add_to_transmitter(
        &mut self,
        data: &ReplicatedData,
        targets: &BTreeSet<XorName>,
    ) {
        for target in targets {
            let entry = self.to_be_transmitted.entry(*target);
            let data_address = data.address();
            info!("Storing {data_address:?} in replicator");
            match entry {
                Entry::Occupied(mut present_entries) => {
                    let addresses = present_entries
                        .get()
                        .keys()
                        .collect::<Vec<&ReplicatedDataAddress>>();
                    info!("We already need to provide {addresses:?} for node {target}");
                    present_entries.get_mut().insert(data_address, data.clone());
                }
                Entry::Vacant(e) => {
                    let mut map = BTreeMap::new();
                    map.insert(data_address, data.clone());
                    e.insert(map);
                }
            }
        }
    }

    pub(crate) fn get_for_replication(
        &mut self,
        data_address: ReplicatedDataAddress,
        target: XorName,
    ) -> Option<ReplicatedData> {
        let data_collection = self.to_be_transmitted.get_mut(&target)?;

        let data = data_collection.remove(&data_address);

        // Clean up of there is not data left for the target
        if data_collection.is_empty() {
            self.to_be_transmitted.remove(&target);
        }

        data
    }
}
