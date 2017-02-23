// Copyright 2017 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement, version 1.0.  This, along with the
// Licenses can be found in the root directory of this project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.
//
// Please review the Licences for the specific language governing permissions and limitations
// relating to use of the SAFE Network Software.

use routing::AccountInfo;
use routing::ClientError;
use rust_sodium::crypto::sign;
use std::collections::BTreeSet;

// 500 units, max 100MB for immutable_data (1MB per chunk)
#[cfg(not(feature = "use-mock-crust"))]
pub const DEFAULT_ACCOUNT_SIZE: u64 = 500;
#[cfg(feature = "use-mock-crust")]
pub const DEFAULT_ACCOUNT_SIZE: u64 = 100;

#[derive(RustcEncodable, RustcDecodable, PartialEq, Eq, Debug, Clone)]
pub struct Account {
    pub info: AccountInfo,
    pub auth_keys: BTreeSet<sign::PublicKey>,
    pub version: u64,
}

impl Account {
    pub fn increment_mutation_counter(&mut self) -> Result<(), ClientError> {
        if self.info.mutations_available < 1 {
            return Err(ClientError::LowBalance);
        }

        self.info.mutations_done += 1;
        self.info.mutations_available -= 1;

        Ok(())
    }

    pub fn decrement_mutation_counter(&mut self) -> Result<(), ClientError> {
        if self.info.mutations_done < 1 {
            return Err(ClientError::InvalidOperation);
        }

        self.info.mutations_done -= 1;
        self.info.mutations_available += 1;

        Ok(())
    }
}

impl Default for Account {
    fn default() -> Self {
        Account {
            info: AccountInfo {
                mutations_available: DEFAULT_ACCOUNT_SIZE,
                mutations_done: 0,
            },
            auth_keys: BTreeSet::new(),
            version: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Account, DEFAULT_ACCOUNT_SIZE};

    #[test]
    fn normal_updates() {
        let mut account = Account::default();

        assert_eq!(0, account.info.mutations_done);
        assert_eq!(DEFAULT_ACCOUNT_SIZE, account.info.mutations_available);
        for _ in 0..DEFAULT_ACCOUNT_SIZE {
            assert!(account.increment_mutation_counter().is_ok());
        }
        assert_eq!(DEFAULT_ACCOUNT_SIZE, account.info.mutations_done);
        assert_eq!(0, account.info.mutations_available);

        for _ in 0..DEFAULT_ACCOUNT_SIZE {
            assert!(account.decrement_mutation_counter().is_ok());
        }
        assert_eq!(0, account.info.mutations_done);
        assert_eq!(DEFAULT_ACCOUNT_SIZE, account.info.mutations_available);
    }

    #[test]
    fn error_updates() {
        let mut account = Account::default();

        assert_eq!(0, account.info.mutations_done);
        assert_eq!(DEFAULT_ACCOUNT_SIZE, account.info.mutations_available);
        for _ in 0..DEFAULT_ACCOUNT_SIZE {
            assert!(account.increment_mutation_counter().is_ok());
        }
        assert_eq!(DEFAULT_ACCOUNT_SIZE, account.info.mutations_done);
        assert_eq!(0, account.info.mutations_available);
        assert!(account.increment_mutation_counter().is_err());
        assert_eq!(DEFAULT_ACCOUNT_SIZE, account.info.mutations_done);
        assert_eq!(0, account.info.mutations_available);
    }
}
