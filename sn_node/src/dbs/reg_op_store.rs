// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{Error, Result};

use crate::dbs::FileStore;
use sn_interface::types::{RegisterAddress, RegisterCmd, RegisterCmdId};
use std::collections::btree_map::BTreeMap;
use std::fmt::Debug;
use std::path::PathBuf;

pub(crate) type RegisterLog = BTreeMap<RegisterCmdId, RegisterCmd>;

/// Disk storage for logging RegisterCmds.
#[derive(Clone, Debug)]
pub(crate) struct RegOpStore {
    tree: RegisterLog,
    path: PathBuf,
}

impl RegOpStore {
    /// Create a new event store
    pub(crate) async fn new(addr: &RegisterAddress, db: FileStore) -> Result<Self> {
        let (tree, path) = db.open_log_from_disk(addr).await?;
        Ok(Self { tree, path })
    }

    /// Get all events stored
    pub(crate) fn get_all(&self) -> Vec<RegisterCmd> {
        self.tree.iter().map(|(_, cmd)| cmd).cloned().collect()
    }

    /// Append a new entry and write to disk
    pub(crate) async fn append(&mut self, event: RegisterCmd, file_store: FileStore) -> Result<()> {
        let reg_id = event.register_operation_id()?;
        if self.tree.get(&reg_id).is_some() {
            return Err(Error::DataExists);
        }

        let _old_entry = self.tree.insert(reg_id, event);

        file_store.write_log_to_disk(&self.tree, &self.path).await?;

        Ok(())
    }
}
