// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::utils;
use crate::node::{to_db_key::ToDbKey, Error, Result};
use pickledb::PickleDb;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    fmt::Debug,
    marker::PhantomData,
    path::{Path, PathBuf},
};
use xor_name::XorName;

const DB_EXTENSION: &str = ".db";

/// Disk storage for transfers.
pub struct EventStore<TEvent: Debug + Serialize + DeserializeOwned> {
    db: PickleDb,
    db_path: PathBuf,
    _phantom: PhantomData<TEvent>,
}

pub struct DeletableStore {
    db_path: PathBuf,
}

impl DeletableStore {
    pub fn delete(&self) -> Result<()> {
        std::fs::remove_file(self.db_path.as_path()).map_err(Error::Io)
    }
}

impl<'a, TEvent: Debug + Serialize + DeserializeOwned> EventStore<TEvent>
where
    TEvent: 'a,
{
    pub fn new(id: XorName, root_dir: &Path, type_name: String) -> Result<Self> {
        let db_dir = root_dir.join(Path::new(&type_name));
        let db_name = format!("{}{}", id.to_db_key()?, DB_EXTENSION);
        let db_path = db_dir.join(db_name.clone());
        Ok(Self {
            db: utils::new_auto_dump_db(db_dir.as_path(), db_name)?,
            db_path,
            _phantom: PhantomData::default(),
        })
    }

    pub fn as_deletable(&self) -> DeletableStore {
        DeletableStore {
            db_path: self.db_path.clone(),
        }
    }

    ///
    pub fn get_all(&self) -> Vec<TEvent> {
        let keys = self.db.get_all();

        let mut events: Vec<(usize, TEvent)> = keys
            .iter()
            .filter_map(|key| {
                let value = self.db.get::<TEvent>(key);
                let key = key.parse::<usize>();
                match value {
                    Some(v) => match key {
                        Ok(k) => Some((k, v)),
                        _ => None,
                    },
                    None => None,
                }
            })
            .collect();

        events.sort_by(|(key_a, _), (key_b, _)| key_a.partial_cmp(key_b).unwrap());

        let events: Vec<TEvent> = events.into_iter().map(|(_, val)| val).collect();

        events
    }

    ///
    pub fn append(&mut self, event: TEvent) -> Result<()> {
        let key = &self.db.total_keys().to_string();
        if self.db.exists(key) {
            return Err(Error::Logic(format!(
                "Key exists: {}. Event: {:?}",
                key, event
            )));
        }
        self.db.set(key, &event).map_err(Error::PickleDb)
    }
}

#[cfg(test)]
mod test {
    use super::EventStore;
    use crate::node::Result;
    use crate::types::Token;
    use tempdir::TempDir;

    #[test]
    fn history() -> Result<()> {
        let id = xor_name::XorName::random();
        let tmp_dir = TempDir::new("root")?;
        let root_dir = tmp_dir.into_path();
        let mut store = EventStore::new(id, &root_dir, "String".to_string())?;

        store.append(Token::from_nano(10))?;

        let events = store.get_all();
        assert_eq!(events.len(), 1);

        match events.get(0) {
            Some(token) => assert_eq!(token.as_nano(), 10),
            None => unreachable!(),
        }

        Ok(())
    }
}
