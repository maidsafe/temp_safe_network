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

use routing::{AccountInfo, MessageId};
use rust_sodium::crypto::sign;
use std::collections::BTreeSet;

/// Default available number of operations per account.
#[cfg(not(feature = "use-mock-crust"))]
pub const DEFAULT_MAX_OPS_COUNT: u64 = 500;
/// Default available number of mutations per account.
#[cfg(feature = "use-mock-crust")]
pub const DEFAULT_MAX_OPS_COUNT: u64 = 100;

#[derive(Deserialize, Serialize, PartialEq, Eq, Debug, Clone)]
pub struct Account {
    /// Message ids of data operations performed by this account.
    pub data_ops_msg_ids: BTreeSet<MessageId>,
    /// Number of keys operations performed by this account.
    pub keys_ops_count: u64,
    /// App authentication keys.
    pub keys: BTreeSet<sign::PublicKey>,
}

impl Account {
    // TODO: Change the `AccountInfo` struct in routing.
    pub fn balance(&self) -> AccountInfo {
        let done = self.data_ops_msg_ids.len() as u64 + self.keys_ops_count;
        let available = DEFAULT_MAX_OPS_COUNT.saturating_sub(done);

        AccountInfo {
            mutations_done: done,
            mutations_available: available,
        }
    }

    pub fn has_balance(&self) -> bool {
        self.data_ops_msg_ids.len() as u64 + self.keys_ops_count < DEFAULT_MAX_OPS_COUNT
    }
}

impl Default for Account {
    fn default() -> Self {
        Account {
            data_ops_msg_ids: BTreeSet::new(),
            keys_ops_count: 0,
            keys: BTreeSet::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Account, DEFAULT_MAX_OPS_COUNT};
    use routing::MessageId;

    #[test]
    fn balance() {
        let mut account = Account::default();
        assert!(account.has_balance());

        account.keys_ops_count = DEFAULT_MAX_OPS_COUNT - 1;
        assert!(account.has_balance());

        let _ = account.data_ops_msg_ids.insert(MessageId::zero());
        assert!(!account.has_balance());
    }
}
