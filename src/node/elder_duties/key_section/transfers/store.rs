// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{to_db_key::from_db_key, utils, utils::Init, Error, Result, ToDbKey};
use log::trace;
use pickledb::PickleDb;
use sn_data_types::{PublicKey, ReplicaEvent};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};
use xor_name::XorName;

const TRANSFERS_DIR_NAME: &str = "transfers";
const DB_EXTENSION: &str = ".db";

/// Disk storage for transfers.
pub struct TransferStore {
    id: XorName,
    db: PickleDb,
}

impl TransferStore {
    pub fn new(id: XorName, root_dir: &PathBuf, init_mode: Init) -> Result<Self> {
        let db_dir = root_dir.join(Path::new(TRANSFERS_DIR_NAME));
        let db_name = format!("{}{}", id.to_db_key(), DB_EXTENSION);
        Ok(Self {
            id,
            db: utils::new_db(db_dir.as_path(), db_name, init_mode)?,
        })
    }

    pub fn id(&self) -> XorName {
        self.id
    }

    pub fn all_stream_keys(&self) -> Option<Vec<PublicKey>> {
        let keys = self
            .db
            .get_all()
            .iter()
            .filter_map(|key| from_db_key(key))
            .collect();

        Some(keys)
    }

    pub fn history(&self) -> Option<Vec<ReplicaEvent>> {
        trace!("Getting History from node store");

        // let name = &self.id.to_db_key();

        // // check list exists. If not, pickle panics
        // if !self.db.lexists(name) {
        //     return None;
        // }

        let list: Vec<ReplicaEvent> = self
            .db
            .liter("name")
            .filter_map(|c| c.get_item::<ReplicaEvent>())
            .collect();
        Some(list)
    }

    pub fn try_load(&self) -> Result<Vec<ReplicaEvent>> {
        // Only the order within the streams is important, not between streams.
        let keys = self.db.get_all();
        let events: Vec<ReplicaEvent> = keys
            .iter()
            //.filter(|key| self.db.lexists(&key)) 
            // not all keys are necessarily lists..,
            // in which case we would get an exception at liter below
            // but in current impl, they all are, so no need to filter, yet.
            .map(|key| {
                self.db
                    .liter(&key)
                    .filter_map(|c| c.get_item::<ReplicaEvent>())
                    .collect::<Vec<ReplicaEvent>>()
            })
            .flatten()
            .collect();
        Ok(events)
    }

    pub fn drop(&mut self, streams: &BTreeSet<PublicKey>) -> Result<()> {
        for stream in streams {
            let _ = self.db.lrem_list(&stream.to_db_key());
        }
        Ok(())
    }

    pub fn init(&mut self, events: Vec<ReplicaEvent>) -> Result<()> {
        for event in events {
            self.try_append(event)?;
        }
        Ok(())
    }

    pub fn try_append(&mut self, event: ReplicaEvent) -> Result<()> {
        match event {
            ReplicaEvent::KnownGroupAdded(_e) => unimplemented!("to be deprecated"),
            ReplicaEvent::TransferPropagated(e) => {
                let key = &e.recipient().to_db_key();
                if !self.db.lexists(key) {
                    // Creates if not exists. A stream always starts with a credit.
                    match self.db.lcreate(key) {
                        Ok(_) => (),
                        Err(error) => return Err(Error::PickleDb(error)),
                    };
                }
                match self.db.ladd(key, &ReplicaEvent::TransferPropagated(e)) {
                    Some(_) => Ok(()),
                    None => Err(Error::NetworkData("Failed to write event to db.".into())),
                }
            }
            ReplicaEvent::TransferValidated(e) => {
                let id = e.sender();
                match self
                    .db
                    .ladd(&id.to_db_key(), &ReplicaEvent::TransferValidated(e))
                {
                    Some(_) => Ok(()),
                    None => Err(Error::NetworkData("Failed to write event to db.".into())), // A stream always starts with a credit, so not existing when debiting is simply invalid.
                }
            }
            ReplicaEvent::TransferRegistered(e) => {
                let id = e.sender();
                match self
                    .db
                    .ladd(&id.to_db_key(), &ReplicaEvent::TransferRegistered(e))
                {
                    Some(_) => Ok(()),
                    None => Err(Error::NetworkData("Failed to write event to db.".into())), // A stream always starts with a credit, so not existing when debiting is simply invalid.
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Result;
    use bls::SecretKey;
    use sn_data_types::{PublicKey, TransferPropagated};
    use sn_transfers::get_genesis;
    use tempdir::TempDir;

    #[test]
    fn history() -> Result<()> {
        let id = xor_name::XorName::random();
        let tmp_dir = TempDir::new("root")?;
        let root_dir = tmp_dir.into_path();
        let mut store = TransferStore::new(id, &root_dir, Init::New)?;
        let wallet_id = get_random_pk();
        let credit_proof = get_genesis(10, wallet_id)?;
        store.try_append(ReplicaEvent::TransferPropagated(TransferPropagated {
            credit_proof,
            crediting_replica_keys: get_random_pk(),
            crediting_replica_sig: dummy_sig(),
        }))?;
        if let Some(history) = store.history() {
            assert_eq!(history.len(), 1);
        } else {
            panic!();
        }
        Ok(())
    }

    #[test]
    fn all_stream_keys() -> Result<()> {
        let id = xor_name::XorName::random();
        let tmp_dir = TempDir::new("root")?;
        let root_dir = tmp_dir.into_path();
        let mut store = TransferStore::new(id, &root_dir, Init::New)?;
        let wallet_id = get_random_pk();
        let credit_proof = get_genesis(10, wallet_id)?;
        store.try_append(ReplicaEvent::TransferPropagated(TransferPropagated {
            credit_proof,
            crediting_replica_keys: get_random_pk(),
            crediting_replica_sig: dummy_sig(),
        }))?;
        if let Some(list) = store.all_stream_keys() {
            assert_eq!(list.len(), 1);
        } else {
            panic!();
        }
        Ok(())
    }

    fn get_random_pk() -> PublicKey {
        PublicKey::from(SecretKey::random().public_key())
    }

    use bls::SecretKeyShare;
    use sn_data_types::SignatureShare;
    fn dummy_sig() -> SignatureShare {
        let dummy_shares = SecretKeyShare::default();
        let dummy_sig = dummy_shares.sign("DUMMY MSG");
        SignatureShare {
            index: 0,
            share: dummy_sig,
        }
    }
}
