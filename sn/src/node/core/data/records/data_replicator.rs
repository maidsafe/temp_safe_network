// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::types::ReplicatedDataAddress;
use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet};
use xor_name::XorName;

/// Keeps track of all the data that needs to be replicated.
pub(crate) struct DataReplicator {
    // Nodes and the replicas we need to provide them
    to_be_transmitted: BTreeMap<XorName, Vec<ReplicatedDataAddress>>, // Target -> Set<Data> mapping
}

impl DataReplicator {
    pub(crate) fn new() -> Self {
        DataReplicator {
            to_be_transmitted: BTreeMap::new(),
        }
    }

    /// Add target nodes and the replicas we need to provide them
    pub(crate) fn start_replication_for(
        &mut self,
        data_address: &ReplicatedDataAddress,
        targets: &BTreeSet<XorName>,
    ) {
        for target in targets {
            let entry = self.to_be_transmitted.entry(*target);
            info!("Storing {data_address:?} in replicator for node {target}");
            match entry {
                Entry::Occupied(mut present_entries) => {
                    let addresses = present_entries.get_mut();
                    info!("We already need to provide {addresses:?} for node {target}");
                    addresses.push(*data_address);
                }
                Entry::Vacant(e) => {
                    let _ = e.insert(vec![*data_address]);
                }
            }
        }
    }

    /// Remove data address from replicator in case the target already has the data
    pub(crate) fn stop_replication_for(
        &mut self,
        data_address: ReplicatedDataAddress,
        target: &XorName,
    ) {
        if let Some(data_collection) = self.to_be_transmitted.get_mut(target) {
            if let Some(idx) = data_collection
                .iter()
                .position(|address| address == &data_address)
            {
                let _ = data_collection.remove(idx);
                info!("Successfully cleared replicator entry for {target}");
            } else {
                warn!("Given address {data_address:?} not on replication list for {target}");
            }
        } else {
            warn!("Given node: {target} not on replicator");
        }
    }

    /// Tracks and completes replication for the given data_address and target node.
    // Returns:
    // Some(false) if we still need to hold the data after handing out a replica
    // Some(true) if we can delete the data since we have handed out to all replicas
    // None if we do not handle the data at all
    pub(crate) fn finish_replication_for(
        &mut self,
        data_address: ReplicatedDataAddress,
        target: XorName,
    ) -> Option<bool> {
        let data_collection = self.to_be_transmitted.get_mut(&target)?;

        // Checking for data address in list
        if let Some(idx) = data_collection
            .iter()
            .position(|address| address == &data_address)
        {
            // Removing from data_collection
            let _ = data_collection.remove(idx);

            // Clean up if there is not data left for the target
            if data_collection.is_empty() {
                let _ = self.to_be_transmitted.remove(&target);
            }

            // Aggregating all data addresses
            let mut all_addresses = vec![];
            for addresses in self.to_be_transmitted.values() {
                all_addresses.extend(addresses);
            }

            // Check if we still need to hold the data
            // i.e. if we have completed replication for this data
            if !all_addresses.contains(&data_address) {
                // We can now safely delete it from our storage
                // as we have given away all the replicas
                return Some(true);
            }

            // We still need to hold the data
            return Some(false);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::node::core::DataReplicator;
    use crate::types::{ChunkAddress, ReplicatedDataAddress};
    use crate::DEFAULT_DATA_COPY_COUNT;
    use itertools::Itertools;
    use std::collections::btree_map::Entry;
    use std::collections::{BTreeMap, BTreeSet};
    use xor_name::XorName;

    #[test]
    fn data_replicator_basics() {
        let mut replicator = DataReplicator::new();

        // Create 20 adults with both prefixes

        // Adults with Prefix 0
        let mut adults = (0..10)
            .map(|_| XorName::random().with_bit(0, false))
            .collect::<Vec<XorName>>();

        // Adults with Prefix 1
        let adults1 = (0..10)
            .map(|_| XorName::random().with_bit(0, true))
            .collect::<Vec<XorName>>();

        // Combine the adults
        adults.extend(adults1);

        let mut selected_adults: BTreeMap<XorName, Vec<ReplicatedDataAddress>> = BTreeMap::new();

        // Generate 20 random DataAddresses
        let data_addresses = (0..20)
            .map(|_| ReplicatedDataAddress::Chunk(ChunkAddress(XorName::random())))
            .collect::<Vec<ReplicatedDataAddress>>();

        // Replicate every data at its closest prefix-ed nodes
        for data_address in data_addresses {
            // Pick DEFAULT_DATA_COPY_COUNT adults closest to the address
            let targets = adults
                .iter()
                .sorted_by(|lhs, rhs| data_address.name().cmp_distance(lhs, rhs))
                .take(DEFAULT_DATA_COPY_COUNT)
                .cloned()
                .collect::<BTreeSet<XorName>>();

            // Replicate at the chosen adults
            replicator.start_replication_for(&data_address, &targets);

            // Record chosen adults and the data they should be holding to verify later
            for target in targets {
                let entry = selected_adults.entry(target);
                match entry {
                    Entry::Occupied(mut present_entries) => {
                        let addresses = present_entries.get_mut();
                        addresses.push(data_address);
                    }
                    Entry::Vacant(e) => {
                        let _ = e.insert(vec![data_address]);
                    }
                }
            }
        }

        // Get each data for replication
        for (target, data_addresses) in selected_adults {
            for data_address in data_addresses {
                let _ = replicator.finish_replication_for(data_address, target);
            }
        }

        // Assert that we've emptied the replicator entries
        assert_eq!(replicator.to_be_transmitted.len(), 0)
    }
}
