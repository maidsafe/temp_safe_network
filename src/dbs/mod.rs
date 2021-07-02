// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod data_store;
mod encoding;
mod errors;
mod event_store;

pub use data_store::*;
pub use errors::{Error, Result};
pub use event_store::EventStore;
use pickledb::{PickleDb, PickleDbDumpPolicy};
use std::{fs, path::Path};

///
pub fn new_auto_dump_db<D: AsRef<Path>, N: AsRef<Path>>(db_dir: D, db_name: N) -> Result<PickleDb> {
    let db_path = db_dir.as_ref().join(db_name);
    match PickleDb::load_bin(db_path.clone(), PickleDbDumpPolicy::AutoDump) {
        Ok(db) => Ok(db),
        Err(_) => {
            fs::create_dir_all(db_dir)?;
            let mut db = PickleDb::new_bin(db_path.clone(), PickleDbDumpPolicy::AutoDump);

            // dump is needed to actually write the db to disk.
            db.dump()?;

            PickleDb::load_bin(db_path, PickleDbDumpPolicy::AutoDump).map_err(Error::PickleDb)
        }
    }
}
