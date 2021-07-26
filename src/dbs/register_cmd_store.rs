// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{Error, Result, ToDbKey};
use crate::{
    messaging::data::RegisterCmd,
    types::utils::{deserialise, serialise},
};
use sled::Db;
use std::path::{Path, PathBuf};
use tokio::fs;
use xor_name::XorName;

const DB_EXTENSION: &str = ".db";

/// Disk storage for transfers.
pub(crate) struct RegisterCmdStore {
    db: Db,
    db_path: PathBuf,
}

pub(crate) struct DeletableStore {
    db_path: PathBuf,
}

impl DeletableStore {
    pub(crate) async fn delete(&self) -> Result<()> {
        fs::remove_file(self.db_path.as_path())
            .await
            .map_err(Error::Io)
    }
}

impl RegisterCmdStore {
    pub(crate) async fn new(id: XorName, db_dir: &Path) -> Result<Self> {
        let db_name = format!("{}{}", id.to_db_key()?, DB_EXTENSION);
        let db_path = db_dir.join(db_name.clone());
        Ok(Self {
            db: sled::open(db_path.clone())?,
            db_path,
        })
    }

    pub(crate) fn as_deletable(&self) -> DeletableStore {
        DeletableStore {
            db_path: self.db_path.clone(),
        }
    }

    /// Get all events stored in db
    pub(crate) fn get_all(&self) -> Result<Vec<RegisterCmd>> {
        let iter = self.db.iter();

        let mut events = vec![];
        for (_, res) in iter.enumerate() {
            let (key, val) = res?;
            let db_key = String::from_utf8(key.to_vec())
                .map_err(|_| Error::CouldNotParseDbKey(key.to_vec()))?;

            let value: RegisterCmd = deserialise(&val)?;
            events.push((db_key, value))
        }

        events.sort_by(|(key_a, _), (key_b, _)| key_a.partial_cmp(key_b).unwrap());

        let events: Vec<RegisterCmd> = events.into_iter().map(|(_, val)| val).collect();

        Ok(events)
    }

    /// add a new register cmd
    pub(crate) fn append(&mut self, event: RegisterCmd) -> Result<()> {
        let key = &self.db.len().to_string();
        if let Some(_) = self.db.get(key)? {
            return Err(Error::InvalidOperation(format!(
                "Key exists: {}. Event: {:?}",
                key, event
            )));
        }

        let event = serialise(&event)?;
        let _ = self.db.insert(key, event).map_err(Error::Sled)?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::RegisterCmdStore;
    use crate::messaging::data::{RegisterCmd, RegisterWrite};
    use crate::messaging::DataSigned;
    use crate::node::Result;

    use crate::types::{
        register::{PublicPermissions, PublicPolicy, Register, User},
        Keypair,
    };
    use rand::rngs::OsRng;
    use std::collections::BTreeMap;
    use std::path::Path;
    use tempfile::tempdir;
    use xor_name::XorName;

    #[tokio::test]
    async fn history_of_register() -> Result<()> {
        let id = xor_name::XorName::random();
        let tmp_dir = tempdir()?;
        let db_dir = tmp_dir.into_path().join(Path::new(&"Token".to_string()));

        let mut store = RegisterCmdStore::new(id, db_dir.as_path()).await?;

        let authority_keypair1 = Keypair::new_ed25519(&mut OsRng);
        let pk = authority_keypair1.public_key();

        let register_name: XorName = rand::random();
        let register_tag = 43_000u64;

        let mut perms = BTreeMap::default();
        let user_perms = PublicPermissions::new(true);
        let _ = perms.insert(User::Key(pk), user_perms);

        let replica1 = Register::new_public(
            pk,
            register_name,
            register_tag,
            Some(PublicPolicy {
                owner: pk,
                permissions: perms.clone(),
            }),
        );

        let write = RegisterWrite::New(replica1);

        let client_sig = DataSigned {
            public_key: pk,
            signature: authority_keypair1.sign(b""),
        };

        let cmd = RegisterCmd { write, client_sig };

        store.append(cmd.clone())?;

        let events = store.get_all()?;
        assert_eq!(events.len(), 1);

        match events.get(0) {
            Some(found_cmd) => assert_eq!(found_cmd, &cmd),
            None => unreachable!(),
        }

        Ok(())
    }
}
