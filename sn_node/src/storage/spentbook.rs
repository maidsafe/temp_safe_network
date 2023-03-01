// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{filepath_to_name, list_files_in, prefix_tree_path, Error, Result};

use crate::{storage::StorageLevel, UsedSpace};

use sn_interface::types::{log_markers::LogMarker, Spend, SpendAddress, SpendShare};

use bincode::{deserialize, serialize};
use dashmap::{mapref::entry::Entry, DashMap};
use std::{
    collections::BTreeMap,
    io::ErrorKind,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{
    fs::{self, create_dir_all, metadata, remove_file, File},
    io::{self, AsyncWriteExt},
};
use xor_name::XorName;

const SPENT_BOOK_DIR_NAME: &str = "spentbook";

#[derive(Debug, Clone)]
pub(crate) struct Spentbook {
    store_path: PathBuf,
    used_space: UsedSpace,
    pending_spends: Arc<DashMap<XorName, BTreeMap<XorName, SpendShare>>>,
}

impl Spentbook {
    pub(crate) fn new(path: &Path, used_space: UsedSpace) -> Self {
        Self {
            store_path: path.join(SPENT_BOOK_DIR_NAME),
            used_space,
            pending_spends: Arc::new(DashMap::new()),
        }
    }

    pub(crate) async fn get(&self, addr: &SpendAddress) -> Result<Spend> {
        let file_path = self.addr_to_filepath(addr)?;
        match fs::read(file_path).await {
            Ok(bytes) => {
                let spend: Spend = deserialize(&bytes)?;
                if spend.dst_address() == *addr {
                    trace!(
                        "Got spend {:?} from disk.. tx hash is {:?}",
                        spend.id(),
                        spend.proof().content.transaction_hash
                    );
                    Ok(spend)
                } else {
                    warn!(
                        "Unexpected address of spend: {:?}. Should be {addr:?}.",
                        spend.dst_address()
                    );
                    // This can happen if the content read is empty, or incomplete,
                    // possibly due to an issue with the OS synchronising to disk,
                    // resulting in a mismatch with recreated address of the Chunk.
                    Err(Error::SpendNotFound(*addr.name()))
                }
            }
            Err(io_error @ io::Error { .. }) if io_error.kind() == ErrorKind::NotFound => {
                Err(Error::SpendNotFound(*addr.name()))
            }
            Err(other) => Err(other.into()),
        }
    }

    /// Stores a share at an individual location for that share.
    pub(crate) async fn store(&self, spend: &SpendShare) -> Result<StorageLevel> {
        let id = spend.dbc_id_xorname();
        let tx_share_id = spend.tx_share_id();

        if self.get(&spend.dst_address()).await.is_ok() {
            let _ = self.pending_spends.remove(&id);
            trace!("Already had an aggregated spend of {id:?}. Dropping tx share {tx_share_id:?}.",);
            return Ok(StorageLevel::NoChange);
        }

        trace!(
            "Storing tx share {tx_share_id:?} of spend {id:?}.. (there are {} pending spends)",
            self.pending_spends.len()
        );
        trace!(
            "Storing tx share {tx_share_id:?}, tx hash is: {:?}",
            spend.proof_share().content.transaction_hash
        );

        match self.pending_spends.entry(id) {
            Entry::Vacant(entry) => {
                let _ = entry.insert(BTreeMap::from([(tx_share_id, spend.clone())]));
                trace!(
                    "There was no pending spend for {id:?}. Tx share {tx_share_id:?} was inserted."
                );
            }
            Entry::Occupied(mut entry) => {
                let set = entry.get_mut();
                let _ = set.insert(tx_share_id, spend.clone());
                trace!(
                    "Inserted tx share {tx_share_id:?} to existing pending spend {id:?}. Number of shares: {}.",
                    set.len() - 1
                );
            }
        }

        trace!(
            "{:?} {tx_share_id:?} of spend {id:?}",
            LogMarker::RecordedSpendShare
        );

        if let Some(spend) = self.get_spend(&id) {
            self.write_to_disk(&spend).await
        } else {
            Ok(StorageLevel::NoChange)
        }
    }

    pub(crate) async fn write_to_disk(&self, spend: &Spend) -> Result<StorageLevel> {
        let addr = &spend.dst_address();
        let filepath = self.addr_to_filepath(addr)?;

        if filepath.exists() {
            info!("{}: Spend already exists, not storing: {:?}", self, addr);
            // Nothing more to do here
            return Ok(StorageLevel::NoChange);
        }

        let spend_bytes = serialize(spend)?;

        // Cheap extra security check for space (prone to race conditions)
        // just so we don't go too much overboard.
        // Should not be triggered as data should not be sent to full nodes.
        if !self.used_space.can_add(spend_bytes.len()) {
            return Err(Error::NotEnoughSpace);
        }

        // Store the spend on disk
        trace!("{:?} {addr:?}", LogMarker::StoringSpend);
        trace!(
            "Writing spend {:?} to disk.. tx hash is: {:?}",
            spend.id(),
            spend.proof().content.transaction_hash
        );
        if let Some(dirs) = filepath.parent() {
            create_dir_all(dirs).await?;
        }

        let mut file = File::create(filepath).await?;

        file.write_all(&spend_bytes).await?;
        // Let's sync up OS data to disk to reduce the chances of
        // concurrent reading failing by reading an empty/incomplete file
        file.sync_data().await?;

        let storage_level = self.used_space.increase(spend_bytes.len());
        trace!("{:?} {addr:?}", LogMarker::StoredSpend);

        Ok(storage_level)
    }

    pub(crate) fn addrs(&self) -> Vec<SpendAddress> {
        list_files_in(&self.store_path)
            .iter()
            .filter_map(|filepath| filepath_to_name(filepath).ok())
            .map(SpendAddress::new)
            .collect()
    }

    pub(crate) async fn remove(&self, addr: &SpendAddress) -> Result<()> {
        debug!("Removing spend, {:?}", addr);
        let filepath = self.addr_to_filepath(addr)?;
        let meta = metadata(filepath.clone()).await?;
        remove_file(filepath).await?;
        self.used_space.decrease(meta.len() as usize);
        Ok(())
    }

    // returns a spend if enough shares exist
    fn get_spend(&self, id: &XorName) -> Option<Spend> {
        self.pending_spends
            .get(id)
            .map(|r| r.value().clone())
            .and_then(|share_map| {
                let (key, tx_hash) = share_map.values().last().map(|s| {
                    let content = s.proof_share().content.clone();
                    (content.public_key, content.transaction_hash)
                })?;
                let shares = share_map
                    .values()
                    .map(|v| v.proof_share().clone())
                    .collect();
                sn_dbc::SpentProof::try_from_proof_shares(key, tx_hash, &shares).ok()
            })
            .map(Spend::new)
    }

    fn addr_to_filepath(&self, addr: &SpendAddress) -> Result<PathBuf> {
        let xorname = *addr.name();
        let path = prefix_tree_path(&self.store_path, xorname);
        let filename = hex::encode(xorname);
        Ok(path.join(filename))
    }
}

impl std::fmt::Display for Spentbook {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "Spentbook")
    }
}
