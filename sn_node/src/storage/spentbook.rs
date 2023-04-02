// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{filepath_to_name, list_files_in, prefix_tree_path, Error, Result};

use crate::{storage::StorageLevel, UsedSpace};

use sn_interface::types::{log_markers::LogMarker, DbcSpendInfo, Spend, SpendAddress, SpendShare};

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
    /// DbcId name to DbcSpendInfo (all attempted spends of this dbc).
    pending_spends: Arc<DashMap<XorName, DbcSpendInfo>>,
}

impl Spentbook {
    pub(crate) fn new(path: &Path, used_space: UsedSpace) -> Self {
        Self {
            store_path: path.join(SPENT_BOOK_DIR_NAME),
            used_space,
            pending_spends: Arc::new(DashMap::new()),
        }
    }

    /// Stores a share at an individual location for that share.
    pub(crate) async fn store(&self, spend: &SpendShare) -> Result<StorageLevel> {
        let dbc_id_name = spend.dbc_id_xorname();
        let tx_share_id = spend.tx_share_id();

        let mut spend_info = match self.get(&spend.dst_address()).await {
            Ok(info) => info
                .first()
                .cloned()
                .ok_or(Error::SpendNotFound(dbc_id_name))?, // should never be an empty vec..
            Err(_) => DbcSpendInfo {
                dbc_id: *spend.dbc_id(),
                txs: BTreeMap::new(),
                tx_spend_map: BTreeMap::new(),
            },
        };

        trace!(
            "Storing tx share {tx_share_id:?} of dbc {dbc_id_name:?}.. (there are {} pending spends)",
            self.pending_spends.len()
        );
        trace!(
            "Storing tx share {tx_share_id:?}, tx hash is: {:?}",
            spend.proof_share().content.transaction_hash
        );

        match self.pending_spends.entry(dbc_id_name) {
            Entry::Vacant(entry) => {
                match spend_info.txs.get_mut(&spend.tx_id()) {
                    Some(txs) => {
                        let _ = txs.insert(tx_share_id, spend.clone());
                    }
                    None => {
                        let _ = spend_info.txs.insert(
                            spend.tx_id(),
                            BTreeMap::from([(tx_share_id, spend.clone())]),
                        );
                    }
                };
                // now we have it in mem cache
                let _ = entry.insert(spend_info);
                trace!(
                    "There was no pending spend for dbc {dbc_id_name:?}. Tx share {tx_share_id:?} was inserted."
                );
            }
            Entry::Occupied(mut entry) => {
                let spend_info = entry.get_mut();
                match spend_info.txs.get_mut(&spend.tx_id()) {
                    Some(tx_shares) => {
                        let _ = tx_shares.insert(tx_share_id, spend.clone());
                        trace!(
                            "Inserted tx share {tx_share_id:?} to existing dbc spend info {dbc_id_name:?}, and existing tx: {}. Number of shares: {}.",
                            spend.tx_id(),
                            tx_shares.len() - 1
                        );
                    }
                    None => {
                        let _ = spend_info.txs.insert(
                            spend.tx_id(),
                            BTreeMap::from([(tx_share_id, spend.clone())]),
                        );
                        trace!(
                            "Inserted the first tx share ({tx_share_id:?}) to existing dbc spend info {dbc_id_name:?}, and new tx: {}.",
                            spend.tx_id(),
                        );
                    }
                };
            }
        }

        trace!(
            "{:?} {tx_share_id:?} of spend of dbc {dbc_id_name:?}",
            LogMarker::RecordedSpendShare
        );

        self.try_set_spend(&dbc_id_name).await
    }

    pub(crate) async fn write_to_disk(&self, spend_info: &DbcSpendInfo) -> Result<StorageLevel> {
        let addr = &spend_info.dst_address();
        let filepath = self.addr_to_filepath(addr)?;

        // if filepath.exists() {
        //     info!("{}: Spend already exists, not storing: {:?}", self, addr);
        //     // Nothing more to do here
        //     return Ok(StorageLevel::NoChange);
        // }

        let spend_bytes = serialize(spend_info)?;

        // Cheap extra security check for space (prone to race conditions)
        // just so we don't go too much overboard.
        // Should not be triggered as data should not be sent to full nodes.
        if !self.used_space.can_add(spend_bytes.len()) {
            return Err(Error::NotEnoughSpace);
        }

        // Store the spend on disk
        trace!("{:?} {addr:?}", LogMarker::StoringSpend);
        trace!(
            "Writing spend info of dbc {:?} to disk..",
            spend_info.dbc_id,
        );
        if let Some(dirs) = filepath.parent() {
            create_dir_all(dirs).await?;
        }

        let mut file = File::create(filepath).await?;

        file.write_all(&spend_bytes).await?;
        // Sync up OS data to disk to reduce the chances of
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

    pub(crate) async fn get(&self, addr: &SpendAddress) -> Result<Vec<DbcSpendInfo>> {
        let file_path = self.addr_to_filepath(addr)?;
        match fs::read(file_path).await {
            Ok(bytes) => {
                let spend_info: DbcSpendInfo = deserialize(&bytes)?;
                if spend_info.dst_address() == *addr {
                    trace!("Got DbcSpendInfo {:?} from disk.", spend_info.dbc_id,);
                    Ok(vec![spend_info])
                } else {
                    warn!(
                        "Unexpected address of DbcSpendInfo: {:?}. Should be {addr:?}.",
                        spend_info.dst_address()
                    );
                    // This can happen if the content read is empty, or incomplete,
                    // possibly due to an issue with the OS synchronising to disk,
                    // resulting in a mismatch with recreated address of the spend.
                    Err(Error::SpendNotFound(*addr.name()))
                }
            }
            Err(io_error @ io::Error { .. }) if io_error.kind() == ErrorKind::NotFound => {
                Err(Error::SpendNotFound(*addr.name()))
            }
            Err(other) => Err(other.into()),
        }
    }

    // returns a spend if enough shares exist
    async fn try_set_spend(&self, id: &XorName) -> Result<StorageLevel> {
        let _ = self.pending_spends.get_mut(id).map(|mut r| {
            let spend_info = r.value_mut();

            let _ = spend_info
                .txs
                .values() // take the longest tx set
                .max_by(|map_a, map_b| map_a.len().cmp(&map_b.len()))
                .map(|share_map| {
                    let content = share_map
                        .values()
                        .last()
                        .map(|s| s.proof_share().content.clone());
                    content
                        .map(|content| (content.public_key, content.transaction_hash))
                        .map(|(key, tx_hash)| {
                            let shares = share_map
                                .values()
                                .map(|v| v.proof_share().clone())
                                .collect();
                            let spend =
                                sn_dbc::SpentProof::try_from_proof_shares(key, tx_hash, &shares)
                                    .ok()
                                    .map(Spend::new);
                            if let Some(spend) = spend {
                                let index = spend_info.tx_spend_map.len();
                                if let std::collections::btree_map::Entry::Vacant(e) =
                                    spend_info.tx_spend_map.entry(spend.tx_id())
                                {
                                    let _ = e.insert((index, spend));
                                }
                            }
                        })
                });
        });

        let spend_info = self
            .pending_spends
            .get(id)
            .map(|r| r.value().clone())
            .ok_or(Error::SpendNotFound(*id))?;

        self.write_to_disk(&spend_info).await
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
