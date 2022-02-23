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

    // Keep track of nodes and the replicas we need to provide them
    pub(crate) fn add_to_transmitter(
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

    // Remove data address from replicator in case the target already has the data
    pub(crate) fn remove_from_transmitter(
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

    // Returns:
    // Some(true) if we still need to hold the data after handing out a replica
    // Some(false) if we can delete the data since we have handed out to all replicas
    // None if we do not handle the data at all
    pub(crate) fn get_for_replication(
        &mut self,
        data_address: ReplicatedDataAddress,
        target: XorName,
    ) -> Option<bool> {
        let data_collection = self.to_be_transmitted.get_mut(&target)?;

        if let Some(idx) = data_collection
            .iter()
            .position(|address| address == &data_address)
        {
            let _ = data_collection.remove(idx);

            // Clean up if there is not data left for the target
            if data_collection.is_empty() {
                let _ = self.to_be_transmitted.remove(&target);
            }

            let mut all_addresses = vec![];
            for addresses in self.to_be_transmitted.values() {
                all_addresses.extend(addresses);
            }

            // Check if we still need to hold the data
            // i.e. if we have completed replication for this data
            if !all_addresses.contains(&data_address) {
                // We can now safely delete it from our storage
                // as we have given away all the replicas
                return Some(false);
            }
            return Some(true);
        }
        None
    }
}
