// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{utils, vault::Init, Result, ToDbKey};
use log::{trace, warn};
use pickledb::PickleDb;
use safe_nd::{
    AppPermissions, AppPublicId, ClientPublicId, Error as NdError, PublicKey, Result as NdResult,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    path::Path,
};

type AccountAsTuple = (BTreeMap<PublicKey, AppPermissions>, u64);

const ACCOUNTS_DB_NAME: &str = "accounts.db";

#[derive(Default, Serialize, Deserialize, Debug)]
pub(super) struct Account {
    apps: HashMap<PublicKey, AppPermissions>,
    version: u64,
}

impl Account {
    fn into_tuple(self) -> AccountAsTuple {
        (self.apps.into_iter().collect(), self.version)
    }
}

pub(super) struct AccountsDb {
    db: PickleDb,
}

impl AccountsDb {
    pub fn new<R: AsRef<Path>>(root_dir: R, init_mode: Init) -> Result<Self> {
        Ok(Self {
            db: utils::new_db(root_dir, ACCOUNTS_DB_NAME, init_mode)?,
        })
    }

    pub fn app_permissions(&self, app_public_id: &AppPublicId) -> Option<AppPermissions> {
        self.db
            .get(&app_public_id.owner().to_db_key())
            .and_then(|account: Account| account.apps.get(app_public_id.public_key()).cloned())
    }

    /// If the specified account doesn't exist, a default `AccountAsTuple` is returned.
    pub fn list_auth_keys_and_version(&self, client_id: &ClientPublicId) -> AccountAsTuple {
        let db_key = client_id.to_db_key();
        self.db
            .get::<Account>(&db_key)
            .map(Account::into_tuple)
            .unwrap_or_default()
    }

    /// Inserts `key` and `permissions` into the specified account, creating the account if it
    /// doesn't already exist.
    pub fn ins_auth_key(
        &mut self,
        client_id: &ClientPublicId,
        key: PublicKey,
        new_version: u64,
        permissions: AppPermissions,
    ) -> NdResult<()> {
        let db_key = client_id.to_db_key();
        let mut account = self.get_account_and_increment_version(&db_key, new_version)?;

        // TODO - should we assert the `key` is an App type?

        let _ = account.apps.insert(key, permissions);
        if let Err(error) = self.db.set(&db_key, &account) {
            warn!("Failed to write Account to DB: {:?}", error);
            return Err(NdError::from("Failed to insert authorised key."));
        }

        Ok(())
    }

    /// Deletes `key` from the specified account.  Returns `NoSuchKey` if the account or key doesn't
    /// exist.
    pub fn del_auth_key(
        &mut self,
        client_id: &ClientPublicId,
        key: PublicKey,
        new_version: u64,
    ) -> NdResult<()> {
        let db_key = client_id.to_db_key();
        let mut account = self.get_account_and_increment_version(&db_key, new_version)?;
        if account.apps.remove(&key).is_none() {
            trace!("Failed to delete non-existent authorised key {}", key);
            return Err(NdError::NoSuchKey);
        }
        if let Err(error) = self.db.set(&db_key, &account) {
            warn!("Failed to write Account to DB: {:?}", error);
            return Err(NdError::from("Failed to insert authorised key."));
        }

        Ok(())
    }

    fn get_account_and_increment_version(
        &self,
        db_key: &str,
        new_version: u64,
    ) -> NdResult<Account> {
        let mut account = self.db.get::<Account>(&db_key).unwrap_or_default();

        if account.version + 1 != new_version {
            trace!(
                "Failed to mutate authorised key.  Current version: {}  New version: {}",
                account.version,
                new_version
            );
            return Err(NdError::InvalidSuccessor(account.version));
        }
        account.version = new_version;
        Ok(account)
    }
}
