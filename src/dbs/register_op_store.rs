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
use sled::{Db, Tree};
use xor_name::XorName;

/// Disk storage for Registers.
#[derive(Clone, Debug)]
pub(crate) struct RegisterOpStore {
    tree: Tree,
    db_name: String,
}

impl RegisterOpStore {
    pub(crate) fn new(id: XorName, db: Db) -> Result<Self> {
        let db_name = id.to_db_key()?;
        let tree = db.open_tree(&db_name)?;
        Ok(Self { tree, db_name })
    }

    /// Get all events stored in db
    pub(crate) fn get_all(&self) -> Result<Vec<RegisterCmd>> {
        let iter = self.tree.iter();

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
        let key = &self.tree.len().to_string();
        if self.tree.get(key)?.is_some() {
            return Err(Error::InvalidOperation(format!(
                "Key exists: {}. Event: {:?}",
                key, event
            )));
        }

        let event = serialise(&event)?;
        let _ = self.tree.insert(key, event).map_err(Error::Sled)?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::RegisterOpStore;
    use crate::messaging::data::{RegisterCmd, RegisterWrite};
    use crate::messaging::DataSigned;
    use crate::node::Result;

    use crate::node::Error;
    use crate::types::{
        register::{PublicPermissions, PublicPolicy, Register, User},
        Keypair,
    };
    use rand::rngs::OsRng;
    use std::collections::BTreeMap;
    use std::path::Path;
    use tempfile::tempdir;
    use xor_name::XorName;

    #[tokio::test(flavor = "multi_thread")]
    async fn history_of_register() -> Result<()> {
        let id = xor_name::XorName::random();
        let tmp_dir = tempdir()?;
        let db_dir = tmp_dir.into_path().join(Path::new(&"db".to_string()));
        let db = sled::open(db_dir).map_err(|error| {
            trace!("Sled Error: {:?}", error);
            Error::Sled(error)
        })?;
        let mut store = RegisterOpStore::new(id, db)?;

        let authority_keypair1 = Keypair::new_ed25519(&mut OsRng);
        let pk = authority_keypair1.public_key();

        let register_name: XorName = rand::random();
        let register_tag = 43_000u64;

        let mut permissions = BTreeMap::default();
        let user_perms = PublicPermissions::new(true);
        let _ = permissions.insert(User::Key(pk), user_perms);

        let replica1 = Register::new_public(
            pk,
            register_name,
            register_tag,
            Some(PublicPolicy {
                owner: pk,
                permissions,
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
