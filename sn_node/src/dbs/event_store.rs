// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{Error, Result};

use crate::dbs::FileStore;
use sn_interface::types::{RegisterAddress, RegisterCmd};
use std::collections::btree_map::BTreeMap;
use std::fmt::Debug;
use std::path::PathBuf;

/// Disk storage for logging RegisterCmds.
#[derive(Clone, Debug)]
pub(crate) struct RegOpStore {
    tree: BTreeMap<String, RegisterCmd>,
    path: PathBuf,
}

impl RegOpStore {
    /// Create a new event store
    pub(crate) async fn new(addr: &RegisterAddress, db: FileStore) -> Result<Self> {
        let (tree, path) = db.open_log(addr).await?;
        Ok(Self { tree, path })
    }

    /// Get all events stored in db
    pub(crate) fn get_all(&self) -> Result<Vec<RegisterCmd>> {
        let iter = self.tree.iter();

        let mut events = vec![];
        for (_, res) in iter.enumerate() {
            let (db_key, val) = res;

            events.push((db_key, val))
        }

        events.sort_by(|(key_a, _), (key_b, _)| key_a.partial_cmp(key_b).unwrap());

        let events: Vec<RegisterCmd> = events.into_iter().map(|(_, val)| val).cloned().collect();

        Ok(events)
    }

    /// append a new entry
    pub(crate) async fn append(&mut self, event: RegisterCmd, file_store: FileStore) -> Result<()> {
        let key = &self.tree.len().to_string();
        if self.tree.get(key).is_some() {
            return Err(Error::DataExists);
        }

        let _old_entry = self.tree.insert(key.clone(), event);

        file_store
            .write_to_log(self.tree.clone(), &self.path)
            .await?;

        Ok(())
    }
}

// #[cfg(test)]
// mod test {
//     use super::EventStore;
//     use crate::node::{Error, Result};
//     use sn_interface::types::Token;
//
//     use std::path::Path;
//     use tempfile::tempdir;
//     use xor_name::XorName;
//
//     #[tokio::test]
//     async fn history() -> Result<()> {
//         let id: XorName = xor_name::rand::random();
//         let tmp_dir = tempdir()?;
//         let db_dir = tmp_dir.path().join(Path::new(&"Token"));
//         let db = sled::open(db_dir).map_err(|error| {
//             trace!("Sled Error: {:?}", error);
//             Error::Sled(error)
//         })?;
//         let store = EventStore::new(&id, db)?;
//
//         store.append(Token::from_nano(10))?;
//
//         let events = store.get_all()?;
//         assert_eq!(events.len(), 1);
//
//         match events.get(0) {
//             Some(token) => assert_eq!(token.as_nano(), 10),
//             None => unreachable!(),
//         }
//
//         Ok(())
//     }
// }
