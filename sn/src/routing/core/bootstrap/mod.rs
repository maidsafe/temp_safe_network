// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod join;
mod relocate;

pub(crate) use join::join_network;
pub(crate) use relocate::JoiningAsRelocated;

use crate::prefix_map::NetworkPrefixMap;
#[cfg(not(test))]
use tokio::{fs::File, io::AsyncReadExt};

#[cfg(not(test))]
// Reads PrefixMap from '~/.safe/prefix_map' if present.
async fn read_prefix_map_from_disk() -> Option<NetworkPrefixMap> {
    let mut prefix_map_dir = dirs_next::home_dir()?;
    prefix_map_dir.push(".safe");
    prefix_map_dir.push("prefix_map");

    // Read NetworkPrefixMap from disk if present
    let prefix_map: Option<NetworkPrefixMap> =
        if let Ok(mut prefix_map_file) = File::open(prefix_map_dir).await {
            let mut prefix_map_contents = vec![];
            let _ = prefix_map_file
                .read_to_end(&mut prefix_map_contents)
                .await
                .ok()?;

            rmp_serde::from_slice(&prefix_map_contents).ok()
        } else {
            None
        };

    if prefix_map.is_some() {
        info!("Read PrefixMap from disc successfully");
    }

    prefix_map
}

#[cfg(test)]
async fn read_prefix_map_from_disk() -> Option<NetworkPrefixMap> {
    None
}
